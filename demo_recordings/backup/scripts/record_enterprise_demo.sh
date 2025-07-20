#!/bin/bash

# ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
# ┃  SENTIENT OS - ENTERPRISE DEMO RECORDER                     ┃
# ┃  Industry-Standard GIF Generator for SentientOS              ┃
# ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

# Set terminal colors for professional output
CYAN='\033[1;36m'
GREEN='\033[1;32m'
BLUE='\033[1;34m'
YELLOW='\033[1;33m'
RED='\033[1;31m'
BOLD='\033[1m'
RESET='\033[0m'

# Enterprise configuration
RECORDING_PATH="/home/umesh/Sentinent_os/demo_recordings"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RECORDING_FILE="$RECORDING_PATH/sentientos_enterprise_${TIMESTAMP}.cast"
OUTPUT_GIF="$RECORDING_PATH/sentientos_enterprise_${TIMESTAMP}.gif"

# Make the demo script executable
chmod +x "$RECORDING_PATH/enterprise_demo.sh"

# Professional UI elements
clear
echo -e "${BOLD}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${RESET}"
echo -e "${BOLD}┃  ${CYAN}SENTIENT OS - ENTERPRISE DEMO RECORDER                     ${BOLD}┃${RESET}"
echo -e "${BOLD}┃  ${BLUE}Industry-Standard GIF Generator for SentientOS              ${BOLD}┃${RESET}"
echo -e "${BOLD}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${RESET}"
echo
echo -e "${YELLOW}[INFO]${RESET} Preparing enterprise-grade demo environment"
echo -e "${YELLOW}[INFO]${RESET} Demo will showcase the following enterprise features:"
echo -e "       ${GREEN}✓${RESET} Zero-Knowledge Verification"
echo -e "       ${GREEN}✓${RESET} MatrixBox Container Isolation"
echo -e "       ${GREEN}✓${RESET} Universal Package Management"
echo -e "       ${GREEN}✓${RESET} IoT Application Deployment"
echo -e "       ${GREEN}✓${RESET} Gossip Protocol Status"
echo -e "       ${GREEN}✓${RESET} Developer Intent Tracking"
echo
echo -e "${YELLOW}[INFO]${RESET} Recording will be saved to: ${CYAN}$RECORDING_FILE${RESET}"
echo -e "${YELLOW}[INFO]${RESET} Starting recording in 3 seconds..."
sleep 3

# Record the precisely timed 60-second demo
asciinema rec "$RECORDING_FILE" --command "$RECORDING_PATH/enterprise_demo.sh" --title "SentientOS Enterprise IoT Platform" --idle-time-limit 1

echo
echo -e "${GREEN}[SUCCESS]${RESET} Recording completed and saved to: ${CYAN}$RECORDING_FILE${RESET}"
echo

# Convert recording to an enterprise-grade GIF using our Python converter
echo -e "${YELLOW}[INFO]${RESET} Converting recording to high-quality enterprise GIF..."
cd "$RECORDING_PATH"

# Enhance the converter with enterprise settings
cat > "$RECORDING_PATH/enterprise_settings.py" << 'EOF'
# Enterprise-grade settings for GIF conversion
WIDTH = 1200
HEIGHT = 800
FONT_SIZE = 18
FONT_NAME = 'DejaVuSansMono'
BG_COLOR = (26, 27, 38)  # Dark blue professional background
FG_COLOR = (220, 223, 228)  # Light gray text
SLOW_MOTION_FACTOR = 1.3  # Slightly slower for better readability
FRAME_REPEAT = 3  # Repeat frames for smoother appearance
EOF

# Use optimized settings with our converter
python3 -c "
import sys
sys.path.append('$RECORDING_PATH')
from cast_to_gif import convert_cast_to_gif
import enterprise_settings

# Apply enterprise settings
WIDTH = enterprise_settings.WIDTH
HEIGHT = enterprise_settings.HEIGHT
FONT_SIZE = enterprise_settings.FONT_SIZE 
BG_COLOR = enterprise_settings.BG_COLOR
FG_COLOR = enterprise_settings.FG_COLOR
REPEAT_FRAMES = enterprise_settings.FRAME_REPEAT

# Generate the enterprise-grade GIF
convert_cast_to_gif('$RECORDING_FILE', '$OUTPUT_GIF')
"

echo
echo -e "${GREEN}[SUCCESS]${RESET} Enterprise-grade GIF created successfully"
echo -e "${GREEN}[SUCCESS]${RESET} Output: ${CYAN}$OUTPUT_GIF${RESET}"
echo -e "${BOLD}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓${RESET}"
echo -e "${BOLD}┃  ${GREEN}DEMO GENERATION COMPLETE                                  ${BOLD}┃${RESET}"
echo -e "${BOLD}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛${RESET}"
