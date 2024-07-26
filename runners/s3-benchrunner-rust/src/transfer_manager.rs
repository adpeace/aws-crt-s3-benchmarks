use std::{sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use aws_config::{self, BehaviorVersion, Region};
use aws_s3_transfer_manager::{
    download::Downloader, io::InputStream, types::{ConcurrencySetting, PartSize}, upload::{UploadRequest, Uploader}
};
use aws_sdk_s3::operation::get_object::builders::GetObjectInputBuilder;
use aws_sdk_s3::operation::put_object::builders::PutObjectInputBuilder;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;
use std::path::PathBuf;

use crate::{
    BenchmarkConfig, Result, RunBenchmark, RunnerError, TaskAction, TaskConfig, PART_SIZE,
};

/// Benchmark runner using aws-s3-transfer-manager
#[derive(Clone)]
pub struct TransferManagerRunner {
    handle: Arc<Handle>,
}

struct Handle {
    config: BenchmarkConfig,
    downloader: Downloader,
    uploader: Uploader,
}

impl TransferManagerRunner {
    pub async fn new(config: BenchmarkConfig) -> TransferManagerRunner {
        let sdk_config: aws_config::SdkConfig = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .load()
            .await;

        // Blugh, the user shouldn't need to manually configure concurrency like this.
        let total_concurrency = calculate_concurrency(config.target_throughput_gigabits_per_sec);
        let num_objects = config.workload.tasks.len();
        let concurrency_per_object = (total_concurrency / num_objects).max(1);

        let downloader = Downloader::builder()
            .sdk_config(sdk_config.clone())
            .part_size(PartSize::Target(PART_SIZE))
            .concurrency(ConcurrencySetting::Explicit(concurrency_per_object))
            .build();

        let uploader = Uploader::builder()
            .sdk_config(sdk_config.clone())
            .part_size(PartSize::Target(PART_SIZE))
            .concurrency(ConcurrencySetting::Explicit(concurrency_per_object))
            .build();

        TransferManagerRunner {
            handle: Arc::new(Handle { config, downloader, uploader}),
        }
    }

    async fn run_task(self, task_i: usize) -> Result<()> {
        let task_config = &self.config().workload.tasks[task_i];

        if self.config().workload.checksum.is_some() {
            return Err(RunnerError::SkipBenchmark(
                "checksums not yet implemented".to_string()
            ));
        }

        match task_config.action {
            TaskAction::Download => self.download(task_config).await,
            TaskAction::Upload => self.upload(task_config).await,
        }.map_err(|err| RunnerError::Fail(err.into()))
    }

    async fn download(&self, task_config: &TaskConfig) -> Result<()> {
        let key = &task_config.key;

        let input = GetObjectInputBuilder::default()
            .bucket(&self.config().bucket)
            .key(key);

        let mut download_handle = self
            .handle
            .downloader
            .download(input.into())
            .await
            .with_context(|| format!("failed starting download: {key}"))?;

        // if files_on_disk: open file for writing
        let mut dest_file = if self.config().workload.files_on_disk {
            let file = fs::File::create(key)
                .await
                .with_context(|| format!("failed creating file: {key}"))?;
            Some(file)
        } else {
            None
        };

        let mut total_size = 0u64;
        while let Some(chunk_result) = download_handle.body.next().await {
            let chunk =
                chunk_result.with_context(|| format!("failed downloading next chunk of: {key}"))?;

            for segment in chunk.into_segments() {
                // if files_on_disk: write to file
                if let Some(dest_file) = &mut dest_file {
                    dest_file
                        .write_all(&segment)
                        .await
                        .with_context(|| format!("failed writing file: {key}"))?;
                }

                total_size += segment.len() as u64;
            }
        }

        assert_eq!(total_size, task_config.size);

        Ok(())
    }
    async fn upload(&self, task_config: &TaskConfig) -> Result<()> {
        let key = &task_config.key;
        // TODO: Why not use PutObjectInputBuilder vs UploadRequestBuilder
        //TODO: Error handling? How to just convert the ? to proper error?
        //
//        let mut input = PutObjectInputBuilder::default()
//            .bucket(&self.config().bucket)
//            .key(key)
//            .build()
//            .map_err(|err| err.into())?;
//        let mut upload_handle = self.handle.uploader.upload(input.into()).await.with_context(|| format!("Failed starting upload: {key}"))?;
//
         
        let uploader = self.handle.uploader.clone();
        let path: PathBuf = key.into();
        let stream = InputStream::from_path(path).map_err(|err| RunnerError::Fail(err.into()))?;

        let request = UploadRequest::builder()
            .bucket(&self.config().bucket)
            .key(key)
            .body(stream)
            .build()
            .with_context(|| format!("failed to create the request: {key}"))?;

        let handle = uploader.upload(request).await.with_context(|| format!("test"))?;
        let _resp = handle.join().await.with_context(|| format!("test"))?;

        Ok(())
    }
}

#[async_trait]
impl RunBenchmark for TransferManagerRunner {
    async fn run(&self) -> Result<()> {
        // Spawn concurrent tasks for all uploads/downloads.
        // We want the benchmark to fail fast if anything goes wrong,
        // so we're using a JoinSet.
        let mut task_set: JoinSet<std::result::Result<(), RunnerError>> = JoinSet::new();
        for i in 0..self.config().workload.tasks.len() {
            task_set.spawn(self.clone().run_task(i));
        }

        while let Some(join_result) = task_set.join_next().await {
            let task_result = join_result.unwrap();
            task_result?;
        }

        Ok(())
    }

    fn config(&self) -> &BenchmarkConfig {
        &self.handle.config
    }
}

/// Calculate the best concurrency, given target throughput.
/// This is based on aws-c-s3's math for determining max-http-connections, circa July 2024:
/// https://github.com/awslabs/aws-c-s3/blob/94e3342c12833c519900516edd2e85c78dc1639d/source/s3_client.c#L57-L69
/// These are magic numbers work well for large-object workloads.
fn calculate_concurrency(target_throughput_gigabits_per_sec: f64) -> usize {
    let concurrency = target_throughput_gigabits_per_sec * 2.5;
    (concurrency as usize).max(10)
}

