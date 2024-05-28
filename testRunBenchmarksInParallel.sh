#!/bin/bash

# Base command to run
CMD_1="/usr/bin/python3 /home/ubuntu/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ubuntu/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-us-east-1-multiple-nic-test \
  --region us-east-1 \
  --throughput 100.0 \
  --files-dir /home/ubuntu/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ubuntu/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-1.run.json"
CMD_2="/usr/bin/python3 /home/ubuntu/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ubuntu/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-us-east-1-multiple-nic-test \
  --region us-east-1 \
  --throughput 100.0 \
  --files-dir /home/ubuntu/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ubuntu/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-2.run.json"
CMD_3="/usr/bin/python3 /home/ubuntu/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ubuntu/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-us-east-1-multiple-nic-test \
  --region us-east-1 \
  --throughput 100.0 \
  --files-dir /home/ubuntu/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ubuntu/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-3.run.json"
CMD_4="/usr/bin/python3 /home/ubuntu/work/aws-crt-s3-benchmarks/scripts/run-benchmarks.py \
  --runner-cmd /home/ubuntu/work/aws-crt-s3-benchmarks/build/c/install/bin/s3-benchrunner-c \
  --s3-client crt-c \
  --bucket waqar-us-east-1-multiple-nic-test \
  --region us-east-1 \
  --throughput 100.0 \
  --files-dir /home/ubuntu/work/aws-crt-s3-benchmarks/files \
  --workloads /home/ubuntu/work/aws-crt-s3-benchmarks/workloads/download-100GiB-1x-ram-4.run.json"




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
WAQAR_NETWORK_DEVICE_NAME="1" WAQAR_NUMA_NODE="1" $CMD_1 &
$CMD_2 &
#WAQAR_NETWORK_DEVICE_NAME="ens96" $CMD_3 &
#WAQAR_NETWORK_DEVICE_NAME="ens128" $CMD_4 &

# $CMD_1 &
# $CMD_2 &
# $CMD_3 &
# $CMD_4 &

# Wait for all background processes to finish
wait

# End time
end_time=$(date +%s)

# Calculate and print the overall time
elapsed_time=$((end_time - start_time))
echo "Overall time: $elapsed_time seconds"
