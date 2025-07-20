#!/bin/bash
# Demo script for SentientOS MatrixBox Containers feature
clear
echo -e "\033[1;36m$ cd /home/umesh/Sentinent_os\033[0m"
sleep 1
echo -e "\033[1;36m$ bin/matrix-inspect\033[0m"
sleep 2
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Listing active containers:"
echo "┌────────────────────┬──────────────┬─────────────┬─────────────────┐"
echo "│ CONTAINER ID       │ APPLICATION  │ STATUS      │ CREATED         │"
echo "├────────────────────┼──────────────┼─────────────┼─────────────────┤"
echo "│ matrix://app-fb32c │ payment-api  │ RUNNING     │ 12 minutes ago  │"
echo "│ matrix://app-7e91a │ auth-service │ RUNNING     │ 25 minutes ago  │"
echo "└────────────────────┴──────────────┴─────────────┴─────────────────┘"
sleep 3

echo -e "\033[1;36m$ bin/matrix-run projects/data-processor/server.js\033[0m"
sleep 2
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Creating secure container environment..."
sleep 1
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Scanning code for vulnerabilities..."
sleep 1.5
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Setting up isolated runtime..."
sleep 1
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Applying resource limits: cpu=2 memory=512MB"
sleep 1
echo -e "\033[1;35m[MatrixBox Container Engine]\033[0m Creating network security boundaries..."
sleep 1
echo -e "\033[1;32m✓ MatrixBox container initialized: matrix://app-c4d71\033[0m"
sleep 1
echo -e "\033[1;33m[Data Processor]\033[0m Starting server on port 3002..."
sleep 1
echo -e "\033[1;33m[Data Processor]\033[0m Connected to data sources"
sleep 1
echo -e "\033[1;32m✓ Application running in MatrixBox container\033[0m"
sleep 2

echo -e "\033[1;36m$ bin/matrix-stats matrix://app-c4d71\033[0m"
sleep 2
echo -e "\033[1;35m[MatrixBox Stats]\033[0m Container: matrix://app-c4d71 (data-processor)"
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│ CPU: 12.4%                          Memory: 187.5 MB / 512MB │"
echo "│ Network: 1.27 MB/s                  Disk: 4.8 MB/s           │"
echo "│ Uptime: 00:01:42                    Status: HEALTHY          │"
echo "└─────────────────────────────────────────────────────────────┘"
sleep 3
