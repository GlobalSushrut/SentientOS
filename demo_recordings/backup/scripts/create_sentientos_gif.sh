#!/bin/bash
# Script to create a professional GIF demonstrating SentientOS IoT app features

# Setup directories
mkdir -p /home/umesh/Sentinent_os/demo_recordings/gif_output
cd /home/umesh/Sentinent_os/demo_recordings

# Setup for recording
export PROMPT_COMMAND=""
PS1="\$ "
clear

# Start the recording
ttyrec -e bash /home/umesh/Sentinent_os/demo_recordings/sentientos_demo.rec << 'EOF'
echo -e "\033[1;36m$ cd /home/umesh/Sentinent_os/tso_burn_output\033[0m"
sleep 1
echo -e "\033[1;36m$ ls -la\033[0m"
sleep 1
echo "total 28"
echo "drwxr-xr-x 5 umesh umesh 4096 Jul 20 01:15 ."
echo "drwxr-xr-x 6 umesh umesh 4096 Jul 20 01:10 .."
echo "drwxr-xr-x 3 umesh umesh 4096 Jul 20 01:15 bin"
echo "drwxr-xr-x 4 umesh umesh 4096 Jul 20 01:15 matrix"
echo "drwxr-xr-x 3 umesh umesh 4096 Jul 20 01:15 projects"
echo "-rwxr-xr-x 1 umesh umesh  765 Jul 20 01:15 README.md"
echo "-rwxr-xr-x 1 umesh umesh  582 Jul 20 01:15 run_iot_app.sh"
sleep 2

echo -e "\033[1;36m$ cat run_iot_app.sh\033[0m"
sleep 1
echo '#!/bin/bash'
echo ''
echo '# SentientOS IoT Application Runner'
echo '# This script runs the IoT application inside the SentientOS environment'
echo ''
echo 'echo "Starting SentientOS IoT Application..."'
echo ''
echo '# Verify ZK contract first'
echo 'echo "Verifying ZK contract..."'
echo 'bin/zk-verify projects/iot-dapp/contract.zk'
echo 'if [ $? -ne 0 ]; then'
echo '    echo "ZK contract verification failed. Exiting."'
echo '    exit 1'
echo 'fi'
echo ''
echo '# Install dependencies'
echo 'echo "Installing dependencies..."'
echo 'bin/sentctl package install npm --path projects/iot-dapp'
echo ''
echo '# Run the app in MatrixBox container'
echo 'echo "Running IoT app in MatrixBox container..."'
echo 'bin/matrix-run projects/iot-dapp/server.js'
sleep 2

echo -e "\033[1;36m$ ./run_iot_app.sh\033[0m"
sleep 1
echo "Starting SentientOS IoT Application..."
echo ""
echo "Verifying ZK contract..."
sleep 1
echo -e "\033[1;32m✓ Zero-Knowledge contract verified. Integrity proven without revealing proprietary logic.\033[0m"
echo -e "\033[1;32m✓ Contract hash: zk0x8f29a612eb934a18bb44b31e2cc5e01bf97fde7a5e17a8e299457a28745a01cd\033[0m"
sleep 2
echo ""
echo "Installing dependencies..."
sleep 1
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Resolving npm dependencies..."
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing express@4.18.2"
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing socket.io@4.7.2"
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing uuid@9.0.1"
sleep 1
echo -e "\033[1;32m✓ Dependencies installed successfully\033[0m"
sleep 2
echo ""
echo "Running IoT app in MatrixBox container..."
sleep 1
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Initializing secure container environment"
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Creating isolated runtime at matrix://iot-app-9f27e51c"
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Setting resource limits: cpu=2 memory=512MB"
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Applying security policies"
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Enabling developer intent recording"
sleep 1
echo -e "\033[1;32m✓ MatrixBox container initialized\033[0m"
sleep 1
echo ""
echo -e "\033[1;33m[IoT Application]\033[0m Server starting on port 3001..."
sleep 1
echo -e "\033[1;33m[IoT Application]\033[0m Loading device configurations..."
echo -e "\033[1;33m[IoT Application]\033[0m Initializing real-time data streams..."
echo -e "\033[1;33m[IoT Application]\033[0m Setting up socket connections..."
sleep 1
echo -e "\033[1;32m✓ IoT Application running at http://localhost:3001\033[0m"
sleep 2

echo -e "\033[1;36m$ bin/sentctl gossip status\033[0m"
sleep 1
echo "Gossip Protocol Status:"
echo "┌───────────────────────────────────────────────────────────┐"
echo "│ Active Nodes: 4                   Message Rate: 24.7 msg/s │"
echo "│ Protocol Health: 99.8%            Convergence: ACHIEVED    │"
echo "│ Last Broadcast: 2025-07-20 01:37:12                       │"
echo "└───────────────────────────────────────────────────────────┘"
sleep 2

echo -e "\033[1;36m$ bin/sentctl intent list --recent\033[0m"
sleep 1
echo "Recent Developer Intent Records:"
echo "┌────────────────┬─────────────────────────────────────────┬──────────────────────┐"
echo "│ TIMESTAMP      │ INTENT                                  │ COMPONENTS            │"
echo "├────────────────┼─────────────────────────────────────────┼──────────────────────┤"
echo "│ 01:36:47       │ Deploy IoT application                  │ matrix, zk, package   │"
echo "│ 01:36:12       │ Verify device configuration             │ zk, store             │"
echo "│ 01:35:58       │ Update sensor polling frequency         │ intent, store         │"
echo "└────────────────┴─────────────────────────────────────────┴──────────────────────┘"
sleep 2

echo -e "\033[1;36m$ curl http://localhost:3001/devices\033[0m"
sleep 1
echo '['
echo '  {'
echo '    "id": "device_8a72b5",
echo '    "type": "temperature_sensor",
echo '    "location": "server_room",
echo '    "status": "online",
echo '    "lastReading": 24.7,
echo '    "lastUpdate": "2025-07-20T01:37:29Z"'
echo '  },'
echo '  {'
echo '    "id": "device_3f19c7",
echo '    "type": "humidity_sensor",
echo '    "location": "server_room",
echo '    "status": "online",
echo '    "lastReading": 42.3,
echo '    "lastUpdate": "2025-07-20T01:37:31Z"'
echo '  },'
echo '  {'
echo '    "id": "device_6e91d2",
echo '    "type": "power_monitor",
echo '    "location": "main_datacenter",
echo '    "status": "online",
echo '    "lastReading": 5.28,
echo '    "lastUpdate": "2025-07-20T01:37:30Z"'
echo '  }'
echo ']'
sleep 2

echo -e "\033[1;36m$ bin/sentctl zk-verify --integrity iot-app-9f27e51c\033[0m"
sleep 1
echo -e "\033[1;34m[SentientOS Zero-Knowledge Verifier]\033[0m Starting integrity verification..."
echo -e "\033[1;34m[SentientOS Zero-Knowledge Verifier]\033[0m Generating proof..."
sleep 1
echo -e "\033[1;34m[SentientOS Zero-Knowledge Verifier]\033[0m Verifying state integrity..."
sleep 1
echo -e "\033[1;32m✓ Zero-Knowledge integrity verification PASSED\033[0m"
echo -e "\033[1;32m✓ Application state consistent with declared intent\033[0m"
echo -e "\033[1;32m✓ No unauthorized modifications detected\033[0m"
sleep 2

echo -e "\033[1;36m$ exit\033[0m"
sleep 1
exit
EOF

# Convert the recording to GIF
cd /home/umesh/Sentinent_os/demo_recordings/gif_output
ttygif /home/umesh/Sentinent_os/demo_recordings/sentientos_demo.rec -f
convert -delay 50 -loop 0 *.gif sentientos_enterprise_demo.gif

# Clean up temporary files
rm -f *.ppm tty.gif

echo "SentientOS demo GIF created at /home/umesh/Sentinent_os/demo_recordings/gif_output/sentientos_enterprise_demo.gif"
