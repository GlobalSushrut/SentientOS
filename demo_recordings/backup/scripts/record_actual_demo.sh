#!/bin/bash

# Enterprise-grade recording script for SentientOS IoT App Demo
# Records the actual IoT app running in the SentientOS environment

# Set up paths
IOT_APP_PATH="/home/umesh/Sentinent_os/tso_burn_output"
RECORDING_PATH="/home/umesh/Sentinent_os/demo_recordings"

# Create recordings directory if it doesn't exist
mkdir -p "$RECORDING_PATH"

# Generate timestamp for unique recording name
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RECORDING_FILE="$RECORDING_PATH/sentientos_actual_demo_${TIMESTAMP}.cast"

echo "======================================================================"
echo "  SentientOS IoT App Demo Recording"
echo "  Enterprise-Grade Recording Tool"
echo "======================================================================"
echo 
echo "Recording will be saved to: $RECORDING_FILE"
echo "Starting recording in 3 seconds..."
sleep 3

# Record the actual IoT app running
asciinema rec "$RECORDING_FILE" --command "cd $IOT_APP_PATH && ./run_iot_app.sh && sleep 5 && bin/sentctl gossip status && sleep 3 && bin/sentctl intent list --recent && sleep 3"

echo
echo "Recording completed and saved to: $RECORDING_FILE"
echo

# Convert recording to GIF
echo "Converting recording to GIF using enterprise-grade converter..."
cd "$RECORDING_PATH"
python3 cast_to_gif.py "$RECORDING_FILE" "sentientos_enterprise_demo_${TIMESTAMP}.gif"

echo
echo "Enterprise-grade GIF created at: $RECORDING_PATH/sentientos_enterprise_demo_${TIMESTAMP}.gif"
echo "======================================================================"
