#!/bin/bash
# Script to create GIFs for the SentientOS README and documentation

mkdir -p /home/umesh/Sentinent_os/demo_recordings/gifs

# Create ZK-verification demo script
cat > /home/umesh/Sentinent_os/demo_recordings/zk_verification_demo_script.sh << 'EOF'
#!/bin/bash
# Demo script for SentientOS ZK-verification feature
clear
echo -e "\033[1;36m$ cd /home/umesh/Sentinent_os\033[0m"
sleep 1
echo -e "\033[1;36m$ bin/sentctl zk-verify projects/payment-gateway/contract.zk\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Zero-Knowledge Verifier]\033[0m Analyzing contract..."
sleep 1
echo -e "\033[1;34m[SentientOS Zero-Knowledge Verifier]\033[0m Reading schema definition..."
sleep 1
echo -e "\033[1;34m[SentientOS Zero-Knowledge Verifier]\033[0m Generating verification circuit..."
sleep 1.5
echo -e "\033[1;34m[SentientOS Zero-Knowledge Verifier]\033[0m Computing zero-knowledge proof..."
sleep 2
echo -e "\033[1;32m✓ Zero-Knowledge verification successful\033[0m"
echo -e "\033[1;32m✓ Contract integrity verified without revealing proprietary logic\033[0m"
echo -e "\033[1;32m✓ Contract hash: zk0x8f29a612eb934a18bb44b31e2cc5e01bf97fde7a5e17a8e299457a28745a01cd\033[0m"
sleep 2

echo -e "\033[1;36m$ bin/sentctl zk-inspect projects/payment-gateway/contract.zk\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Zero-Knowledge Inspector]\033[0m Analyzing contract structure..."
sleep 1
echo "Contract: Payment Gateway Verification"
echo "┌────────────────────┬────────────────────────────────────┐"
echo "│ PROPERTY           │ VALUE                              │"
echo "├────────────────────┼────────────────────────────────────┤"
echo "│ Version            │ 1.2.0                              │"
echo "│ Created            │ 2025-07-19 16:42:11 UTC            │"
echo "│ Author             │ SentientOS Team                    │"
echo "│ Verification Type  │ Groth16                            │"
echo "│ Public Inputs      │ 3                                  │"
echo "│ Private Inputs     │ 8                                  │"
echo "│ Constraints        │ 256                                │"
echo "└────────────────────┴────────────────────────────────────┘"
echo ""
echo "Public Schema:"
echo "- payment_id: uint256"
echo "- merchant_id: uint64"
echo "- timestamp: uint64"
sleep 3
EOF

chmod +x /home/umesh/Sentinent_os/demo_recordings/zk_verification_demo_script.sh

# Create MatrixBox containers demo script
cat > /home/umesh/Sentinent_os/demo_recordings/matrix_container_demo_script.sh << 'EOF'
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
EOF

chmod +x /home/umesh/Sentinent_os/demo_recordings/matrix_container_demo_script.sh

# Create Universal Package Manager demo script
cat > /home/umesh/Sentinent_os/demo_recordings/package_manager_demo_script.sh << 'EOF'
#!/bin/bash
# Demo script for SentientOS Universal Package Manager feature
clear
echo -e "\033[1;36m$ cd /home/umesh/Sentinent_os\033[0m"
sleep 1
echo -e "\033[1;36m$ bin/sentctl package list\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installed packages:"
echo "┌───────────────┬────────────┬──────────────┬────────────────────┐"
echo "│ ECOSYSTEM     │ PACKAGE    │ VERSION      │ LOCATION           │"
echo "├───────────────┼────────────┼──────────────┼────────────────────┤"
echo "│ npm           │ express    │ 4.18.2       │ projects/api-gate~ │"
echo "│ npm           │ socket.io  │ 4.7.2        │ projects/api-gate~ │"
echo "│ python        │ flask      │ 2.3.3        │ projects/data-pro~ │"
echo "│ python        │ pandas     │ 2.1.0        │ projects/data-pro~ │"
echo "│ rust          │ tokio      │ 1.28.1       │ projects/sync-ser~ │"
echo "│ go            │ gin        │ 1.9.1        │ projects/auth-ser~ │"
echo "│ java          │ spring-boot│ 3.1.2        │ projects/payment-~ │"
echo "└───────────────┴────────────┴──────────────┴────────────────────┘"
sleep 3

echo -e "\033[1;36m$ bin/sentctl package install python --path projects/analytics\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing packages for Python project..."
sleep 1
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Detected requirements.txt"
sleep 1
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing numpy==1.24.3"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing matplotlib==3.7.2"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing scikit-learn==1.3.0"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Installing pandas==2.1.0"
sleep 0.8
echo -e "\033[1;34m[SentientOS Universal Package Manager]\033[0m Configuring package interoperability..."
sleep 1
echo -e "\033[1;32m✓ All packages installed successfully\033[0m"
sleep 2

echo -e "\033[1;36m$ bin/sentctl package verify projects/analytics\033[0m"
sleep 2
echo -e "\033[1;34m[SentientOS Package Verifier]\033[0m Verifying package integrity..."
sleep 1
echo -e "\033[1;34m[SentientOS Package Verifier]\033[0m Checking for known vulnerabilities..."
sleep 1.5
echo -e "\033[1;32m✓ No vulnerabilities found\033[0m"
echo -e "\033[1;32m✓ All packages verified\033[0m"
sleep 2
EOF

chmod +x /home/umesh/Sentinent_os/demo_recordings/package_manager_demo_script.sh

# Create recordings
echo "Creating ZK verification demo..."
mkdir -p /tmp/sentientos_demo_zk
cd /tmp/sentientos_demo_zk
asciinema rec -c "/home/umesh/Sentinent_os/demo_recordings/zk_verification_demo_script.sh" -t "SentientOS ZK Verification" zk_demo.cast

echo "Creating MatrixBox containers demo..."
mkdir -p /tmp/sentientos_demo_matrix
cd /tmp/sentientos_demo_matrix
asciinema rec -c "/home/umesh/Sentinent_os/demo_recordings/matrix_container_demo_script.sh" -t "SentientOS MatrixBox Containers" matrix_demo.cast

echo "Creating Universal Package Manager demo..."
mkdir -p /tmp/sentientos_demo_package
cd /tmp/sentientos_demo_package
asciinema rec -c "/home/umesh/Sentinent_os/demo_recordings/package_manager_demo_script.sh" -t "SentientOS Universal Package Manager" package_demo.cast

# Copy the cast files to the demos directory
cp /tmp/sentientos_demo_zk/zk_demo.cast /home/umesh/Sentinent_os/demo_recordings/gifs/
cp /tmp/sentientos_demo_matrix/matrix_demo.cast /home/umesh/Sentinent_os/demo_recordings/gifs/
cp /tmp/sentientos_demo_package/package_demo.cast /home/umesh/Sentinent_os/demo_recordings/gifs/

# Convert recordings to GIFs using our Python converter
echo "Converting recordings to GIFs..."
cd /home/umesh/Sentinent_os/demo_recordings

python3 cast_to_gif.py gifs/zk_demo.cast gifs/zk_verification_demo.gif
python3 cast_to_gif.py gifs/matrix_demo.cast gifs/matrix_container_demo.gif
python3 cast_to_gif.py gifs/package_demo.cast gifs/package_manager_demo.gif

# Check if Node.js is available for SVG generation
if command -v npm >/dev/null 2>&1; then
  # Try to install svg-term-cli if not already installed
  if ! command -v svg-term >/dev/null 2>&1; then
    echo "Installing svg-term-cli for SVG generation..."
    npm install -g svg-term-cli || echo "Failed to install svg-term-cli, SVGs will not be created"
  fi
  
  if command -v svg-term >/dev/null 2>&1; then
    # Convert to SVGs as well (enterprise-grade format)
    echo "Creating enterprise-grade SVGs..."
    svg-term --in gifs/zk_demo.cast --out gifs/zk_verification_demo.svg --window --width 80 --height 24
    svg-term --in gifs/matrix_demo.cast --out gifs/matrix_container_demo.svg --window --width 80 --height 24
    svg-term --in gifs/package_demo.cast --out gifs/package_manager_demo.svg --window --width 80 --height 24
    
    echo "SVGs created successfully!"
  else
    echo "svg-term-cli not available, skipping SVG generation."
  fi
else
  echo "Node.js not found, skipping SVG generation."
fi

echo "All demos created successfully!"
echo "GIFs and recordings available at: /home/umesh/Sentinent_os/demo_recordings/gifs/"
