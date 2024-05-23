#!/bin/bash

# Base command to run
CMD_1="/usr/bin/python3 /home/ec2-user/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ec2-user/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-aws-c-s3-test-bucket \
  --region us-west-2 \
  --throughput 100.0 \
  --files-dir /home/ec2-user/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ec2-user/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-1.run.json"
CMD_2="/usr/bin/python3 /home/ec2-user/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ec2-user/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-aws-c-s3-test-bucket \
  --region us-west-2 \
  --throughput 100.0 \
  --files-dir /home/ec2-user/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ec2-user/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-2.run.json"
CMD_3="/usr/bin/python3 /home/ec2-user/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ec2-user/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-aws-c-s3-test-bucket \
  --region us-west-2 \
  --throughput 100.0 \
  --files-dir /home/ec2-user/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ec2-user/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-3.run.json"
CMD_4="/usr/bin/python3 /home/ec2-user/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ec2-user/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-aws-c-s3-test-bucket \
  --region us-west-2 \
  --throughput 100.0 \
  --files-dir /home/ec2-user/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ec2-user/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-4.run.json"




# Function to handle script termination
cleanup() {
  echo "Cleaning up background processes..."
  pkill -P $$
  exit 1
}

# Set trap to catch termination signals (SIGINT, SIGTERM) and run cleanup function
trap cleanup SIGINT SIGTERM


# Start time
start_time=$(date +%s)

# Loop over workloads and run each command in parallel
BIND_ADDR="172.31.83.94" LD_PRELOAD=/home/ec2-user/work/bindhack/bind.so $CMD_1 &
BIND_ADDR="172.31.82.67" LD_PRELOAD=/home/ec2-user/work/bindhack/bind.so $CMD_2 &
BIND_ADDR="172.31.83.94" LD_PRELOAD=/home/ec2-user/work/bindhack/bind.so $CMD_3 &
BIND_ADDR="172.31.82.67" LD_PRELOAD=/home/ec2-user/work/bindhack/bind.so $CMD_4 &

# Wait for all background processes to finish
wait

# End time
end_time=$(date +%s)

# Calculate and print the overall time
elapsed_time=$((end_time - start_time))
echo "Overall time: $elapsed_time seconds"
