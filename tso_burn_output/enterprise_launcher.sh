#!/bin/bash

# SentientOS Enterprise Launcher
# This script orchestrates the enterprise-grade deployment process

echo "=============================================="
echo "  SentientOS Enterprise Deployment System     "
echo "=============================================="
echo ""

# Execute each stage of the deployment process
echo "[LAUNCHER] Starting enterprise deployment sequence"

# Set important environment variables
export SENTIENT_ROOT="/home/umesh/Sentinent_os"
export SENTIENT_TSO_DIR="$SENTIENT_ROOT/tso_burn_output"
export SENTIENT_APP_DIR="$SENTIENT_TSO_DIR/projects/iot-dapp"
export SENTIENT_ENTERPRISE=1
export SENTIENT_DEPLOYMENT_ID=$(date +%s)

# Execute the stages in sequence
bash "$SENTIENT_TSO_DIR/stages/01_prepare_environment.sh"
bash "$SENTIENT_TSO_DIR/stages/02_verify_contracts.sh"
bash "$SENTIENT_TSO_DIR/stages/03_install_dependencies.sh"
bash "$SENTIENT_TSO_DIR/stages/04_prepare_container.sh"
bash "$SENTIENT_TSO_DIR/stages/05_deploy_application.sh"

echo ""
echo "[LAUNCHER] Enterprise deployment sequence completed"
echo "=============================================="
