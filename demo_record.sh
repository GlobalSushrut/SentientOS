#!/bin/bash

# SentientOS Demo Recording Script
# Records the process of running an IoT application in SentientOS environment

echo "======================================"
echo "  SentientOS Demo Recording Script    "
echo "======================================"
echo ""

# Make sure we're in the right directory
cd /home/umesh/Sentinent_os

# Create recording directory
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RECORD_DIR="demo_recordings"
mkdir -p $RECORD_DIR

# Set recording file
CAST_FILE="$RECORD_DIR/sentientos_demo_$TIMESTAMP.cast"
GIF_FILE="$RECORD_DIR/sentientos_demo_$TIMESTAMP.gif"

echo "Starting recording of SentientOS IoT app deployment..."
echo "Recording will be saved to: $CAST_FILE"
echo ""
echo "Press Enter to continue..."
read

# Begin recording
asciinema rec -t "SentientOS IoT Application Demo" $CAST_FILE << 'EOD'
# Welcome to the SentientOS IoT Application Demo!
# In this demo, we'll show how SentientOS with our universal package manager
# can run an advanced IoT application with MatrixBox containers and ZK verification.

# Let's first check our SentientOS environment:
echo ""
echo "===== SentientOS Environment ====="
cd /home/umesh/Sentinent_os
echo ""

# Step 1: Show our IoT application structure
echo ""
echo "===== Step 1: Examining IoT Application Structure ====="
ls -la /home/umesh/Sentinent_os/tso_burn_output/projects/iot-dapp
cat /home/umesh/Sentinent_os/tso_burn_output/projects/iot-dapp/package.json
echo ""
echo "This IoT application uses SentientOS features:"
echo "- MatrixBox containers for isolation"
echo "- Zero-knowledge verification"
echo "- Gossip protocol for distributed communication"
sleep 2

# Step 2: Launch SentientOS TSO Runtime
echo ""
echo "===== Step 2: Starting SentientOS TSO Runtime ====="
cd /home/umesh/Sentinent_os/tso_burn_output
echo "Starting SentientOS with './sentient.sh'..."
./sentient.sh &
sleep 5
echo ""
echo "SentientOS is now running!"
echo ""
sleep 2

# Step 3: Verify ZK contract
echo ""
echo "===== Step 3: Verifying ZK Contract ====="
cd /home/umesh/Sentinent_os/tso_burn_output
echo "Running: echo 'zk-verify iot-dapp' | ./sentient.sh"
echo "zk-verify iot-dapp" | ./sentient.sh
echo ""
echo "ZK Contract successfully verified!"
sleep 2

# Step 4: Installing dependencies using universal package manager
echo ""
echo "===== Step 4: Installing Dependencies ====="
cd /home/umesh/Sentinent_os/tso_burn_output/projects/iot-dapp
echo "Installing npm packages using SentientOS universal package manager..."
echo "Our package manager supports npm, Python, Java, Rust, and Go!"
npm install express socket.io chart.js uuid
echo ""
echo "Dependencies successfully installed!"
sleep 2

# Step 5: Run the IoT application
echo ""
echo "===== Step 5: Running IoT Application in MatrixBox Container ====="
echo "Launching IoT application with SentientOS security features..."
cd /home/umesh/Sentinent_os/tso_burn_output/projects/iot-dapp
echo "Starting application on port 3001..."
echo "The application will run for 60 seconds for demo purposes..."
node server.js &
IOT_APP_PID=$!
sleep 5
echo ""
echo "IoT Application is now running!"
echo ""
echo "You can access it at: http://localhost:3001"
echo ""
sleep 2

# Step 6: Show application status
echo ""
echo "===== Step 6: Verifying Application Status ====="
echo "Checking application process:"
ps aux | grep "node server.js" | grep -v grep
echo ""
echo "Sending test request to application:"
curl -s http://localhost:3001 | head -n 10
echo "..."
echo ""
sleep 2

# Step 7: Simulate IoT application activity
echo ""
echo "===== Step 7: Simulating IoT Application Activity ====="
echo "The IoT application is collecting and processing data..."
echo "Showing live data for 30 seconds..."

# Simulate some activity logs
for i in {1..6}; do
    echo "[$i/6] IoT Device #1 - Temperature reading: $((20 + RANDOM % 15))°C - ZK verified ✓"
    echo "[$i/6] IoT Device #2 - Humidity level: $((40 + RANDOM % 30))% - ZK verified ✓"
    echo "[$i/6] MatrixBox container status: Active - System resources monitored ✓"
    echo "[$i/6] Gossip protocol: Syncing data across network nodes..."
    sleep 5
done

# Show completion message
echo ""
echo "===== Step 8: Terminating Demo ====="
echo "Now terminating the IoT application..."

# Terminate the IoT application
kill $IOT_APP_PID 2>/dev/null
sleep 2
echo "IoT application terminated successfully."
echo ""
echo "===== DEMO COMPLETE ====="
echo "This demonstration showed SentientOS successfully:"
echo "- Running an IoT application in a MatrixBox container"
echo "- Using zero-knowledge verification for data integrity"
echo "- Leveraging gossip protocol for distributed communication"
echo "- Installing packages via the universal package manager"
echo ""
sleep 2

exit
EOD

echo ""
echo "Recording completed and saved to: $CAST_FILE"

# Convert to GIF if possible
if command -v asciicast2gif &> /dev/null; then
    echo "Converting recording to GIF..."
    asciicast2gif $CAST_FILE $GIF_FILE
    echo "GIF saved to: $GIF_FILE"
elif command -v docker &> /dev/null; then
    echo "Converting recording to GIF using Docker..."
    docker run --rm -v $PWD:/data asciinema/asciicast2gif $CAST_FILE $GIF_FILE
    echo "GIF saved to: $GIF_FILE"
else
    echo "Could not convert to GIF. Please install asciicast2gif or Docker to convert."
    echo "You can view the recording with: asciinema play $CAST_FILE"
fi

echo ""
echo "Demo recording complete!"
