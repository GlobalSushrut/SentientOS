#!/bin/bash

# Direct SentientOS command execution script

echo "Running IoT application in real SentientOS environment"

# 1. First verify the ZK contract
echo "Step 1: Verifying ZK contract"
./bin/zk-verify iot-dapp

# 2. Use our universal package manager to install dependencies
echo "Step 2: Installing dependencies with universal package manager"
./bin/sentctl package install express socket.io chart.js uuid --ecosystem npm

# 3. Run the IoT app in a MatrixBox container
echo "Step 3: Running IoT application in MatrixBox container"
./bin/matrix-run iot-dapp

# This script executes the real SentientOS commands directly
# rather than simulating or wrapping them
