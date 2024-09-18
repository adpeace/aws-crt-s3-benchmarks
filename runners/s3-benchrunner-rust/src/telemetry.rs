use anyhow::Context;
use opentelemetry::trace::{TraceContextExt, Tracer, TracerProvider};
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::{metrics::SdkMeterProvider, runtime, trace::Config, Resource};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::Result;

pub struct Telemetry {
    meter_provider: SdkMeterProvider,
    logger_provider: LoggerProvider,
}

impl Telemetry {
    pub fn new() -> Result<Self> {
        // Code adapted from:
        // https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp/src/main.rs

        let resource = Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "s3-benchrunner-rust",
        )]);

        let endpoint = "http://localhost:4317";

        // Initialize tracer
        let tracer_provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_trace_config(Config::default().with_resource(resource.clone()))
            .install_batch(runtime::Tokio)
            .with_context(|| format!("Tracer telemetry init failed"))?;

        global::set_tracer_provider(tracer_provider);

        // Initialize meter (metrics)
        let meter_provider = opentelemetry_otlp::new_pipeline()
            .metrics(runtime::Tokio)
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_export_config(ExportConfig {
                        endpoint: endpoint.to_string(),
                        ..ExportConfig::default()
                    }),
            )
            .with_resource(resource.clone())
            .build()
            .with_context(|| format!("Metrics telemetry init failed"))?;

        global::set_meter_provider(meter_provider.clone());

        // Initialize logger
        let logger_provider = opentelemetry_otlp::new_pipeline()
            .logging()
            .with_resource(resource)
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .install_batch(runtime::Tokio)
            .with_context(|| format!("Logger telemetry init failed"))?;

        // Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
        let layer = OpenTelemetryTracingBridge::new(&logger_provider);

        // Add a tracing filter to filter events from crates used by opentelemetry-otlp.
        // The filter levels are set as follows:
        // - Allow `info` level and above by default.
        // - Restrict `hyper`, `tonic`, and `reqwest` to `error` level logs only.
        // This ensures events generated from these crates within the OTLP Exporter are not looped back,
        // thus preventing infinite event generation.
        // Note: This will also drop events from these crates used outside the OTLP Exporter.
        // For more details, see: https://github.com/open-telemetry/opentelemetry-rust/issues/761
        let filter = EnvFilter::new("info")
            .add_directive("hyper=error".parse().unwrap())
            .add_directive("tonic=error".parse().unwrap())
            .add_directive("reqwest=error".parse().unwrap());

        tracing_subscriber::registry()
            .with(filter)
            .with(layer)
            .init();

        // ???
        let tracer = global::tracer_provider().tracer_builder("basic").build();

        tracer.in_span("Main operation", |cx| {
            let span = cx.span();
            span.add_event(
                "Nice operation!".to_string(),
                vec![KeyValue::new("bogons", 100)],
            );

            tracing::error!("graebm error");

            tracer.in_span("Sub operation...", |cx| {
                let span = cx.span();
                span.set_attribute(KeyValue::new("another.key", "yes"));
                span.add_event("Sub span event", vec![]);
            });
        });

        Ok(Telemetry {
            meter_provider,
            logger_provider,
        })
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();

        if let Err(e) = self.meter_provider.shutdown() {
            eprintln!("Metrics telemetry shutdown failed: {e}");
        }

        if let Err(e) = self.logger_provider.shutdown() {
            eprintln!("Logger telemetry shutdown failed: {e}");
        }
    }
}
