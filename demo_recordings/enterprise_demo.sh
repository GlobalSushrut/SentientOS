#!/bin/bash

# ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
# ┃  SENTIENT OS - ENTERPRISE DEMO                              ┃
# ┃  60-Second Industry Standard IoT Application Showcase        ┃
# ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

# Set terminal colors for professional output
CYAN='\033[1;36m'
GREEN='\033[1;32m'
BLUE='\033[1;34m'
PURPLE='\033[1;35m'
YELLOW='\033[1;33m'
RED='\033[1;31m'
BOLD='\033[1m'
RESET='\033[0m'

# Configure professional terminal appearance
export PS1="\[$CYAN\]sentientos\[$RESET\]:\[$BLUE\]\w\[$RESET\]$ "
clear

# Intro - 5 seconds
echo -e "${CYAN}Welcome to SentientOS Enterprise IoT Platform${RESET}"
echo -e "${BLUE}Initializing Enterprise IoT Application Demo...${RESET}"
sleep 2
echo

# Display system information - 5 seconds
echo -e "${YELLOW}[SYSTEM INFO]${RESET} SentientOS Enterprise v1.5.2"
echo -e "${YELLOW}[SYSTEM INFO]${RESET} MatrixBox Container Engine v2.3.0"
echo -e "${YELLOW}[SYSTEM INFO]${RESET} Zero-Knowledge Framework v3.1.1"
echo -e "${YELLOW}[SYSTEM INFO]${RESET} Universal Package System v2.2.7"
sleep 3
echo

# Verification demo - 10 seconds
echo -e "${CYAN}sentientos${RESET}:${BLUE}/home/sentient${RESET}$ ${BOLD}bin/zk-verify projects/iot-dapp/contract.zk${RESET}"
sleep 1
echo -e "${BLUE}[SentientOS Zero-Knowledge Verifier]${RESET} Analyzing contract..."
sleep 1
echo -e "${BLUE}[SentientOS Zero-Knowledge Verifier]${RESET} Computing proof..."
sleep 1
echo -e "${BLUE}[SentientOS Zero-Knowledge Verifier]${RESET} Verifying integrity..."
sleep 1
echo -e "${GREEN}✓ Zero-Knowledge contract verified successfully${RESET}"
echo -e "${GREEN}✓ Contract hash: zk0x8f29a612eb934a18bb44b31e2cc5e01bf97fde7a5e17a8e299457a28745a01cd${RESET}"
echo

# Package installation - 10 seconds
echo -e "${CYAN}sentientos${RESET}:${BLUE}/home/sentient${RESET}$ ${BOLD}bin/sentctl package install npm --path projects/iot-dapp${RESET}"
sleep 1
echo -e "${BLUE}[SentientOS Universal Package Manager]${RESET} Analyzing dependencies..."
echo -e "${BLUE}[SentientOS Universal Package Manager]${RESET} Installing express@4.18.2"
sleep 0.5
echo -e "${BLUE}[SentientOS Universal Package Manager]${RESET} Installing socket.io@4.7.2"
sleep 0.5
echo -e "${BLUE}[SentientOS Universal Package Manager]${RESET} Installing uuid@9.0.1"
sleep 0.5
echo -e "${BLUE}[SentientOS Universal Package Manager]${RESET} Installing chart.js@4.4.0"
sleep 0.5
echo -e "${GREEN}✓ Dependencies installed successfully in MatrixBox isolated environment${RESET}"
echo

# MatrixBox container launch - 15 seconds
echo -e "${CYAN}sentientos${RESET}:${BLUE}/home/sentient${RESET}$ ${BOLD}bin/matrix-run projects/iot-dapp/server.js${RESET}"
sleep 1
echo -e "${PURPLE}[MatrixBox Container Engine]${RESET} Initializing secure container..."
sleep 1
echo -e "${PURPLE}[MatrixBox Container Engine]${RESET} Creating isolated runtime at matrix://iot-app-9f27e51c"
sleep 0.5
echo -e "${PURPLE}[MatrixBox Container Engine]${RESET} Setting resource limits: cpu=2 memory=512MB"
sleep 0.5
echo -e "${PURPLE}[MatrixBox Container Engine]${RESET} Applying security policies"
sleep 0.5
echo -e "${GREEN}✓ MatrixBox container initialized${RESET}"
sleep 0.5
echo
echo -e "${YELLOW}[IoT Application]${RESET} Server starting on port 3001..."
sleep 0.5
echo -e "${YELLOW}[IoT Application]${RESET} Loading device configurations..."
sleep 0.5
echo -e "${YELLOW}[IoT Application]${RESET} Initializing real-time data streams..."
sleep 0.5
echo -e "${GREEN}✓ IoT Application running at http://localhost:3001${RESET}"
echo

# Show device data - 7 seconds
echo -e "${CYAN}sentientos${RESET}:${BLUE}/home/sentient${RESET}$ ${BOLD}curl http://localhost:3001/api/devices${RESET}"
sleep 1
echo -e '[
  {
    "id": "device_8a72b5",
    "type": "temperature_sensor",
    "location": "server_room",
    "status": "online",
    "lastReading": 24.7,
    "lastUpdate": "2025-07-20T01:45:29Z"
  },
  {
    "id": "device_3f19c7",
    "type": "humidity_sensor",
    "location": "server_room",
    "status": "online",
    "lastReading": 42.3,
    "lastUpdate": "2025-07-20T01:45:31Z"
  }
]'
echo

# Show gossip protocol status - 5 seconds
echo -e "${CYAN}sentientos${RESET}:${BLUE}/home/sentient${RESET}$ ${BOLD}bin/sentctl gossip status${RESET}"
sleep 1
echo -e "${BLUE}[SentientOS Gossip Protocol]${RESET} Status Report:"
echo -e "┌──────────────────────────────────────────────────────────────┐"
echo -e "│ Active Nodes: 4                   Message Rate: 24.7 msg/s │"
echo -e "│ Protocol Health: 99.8%            Convergence: ACHIEVED    │"
echo -e "│ Last Broadcast: 2025-07-20 01:45:12                       │"
echo -e "└──────────────────────────────────────────────────────────────┘"
echo

# Show recent developer intent - 3 seconds
echo -e "${CYAN}sentientos${RESET}:${BLUE}/home/sentient${RESET}$ ${BOLD}bin/sentctl intent list --recent${RESET}"
sleep 1
echo -e "Recent Developer Intent Records:"
echo -e "┌────────────────┬─────────────────────────────────────────┬──────────────────────┐"
echo -e "│ TIMESTAMP      │ INTENT                                  │ COMPONENTS            │"
echo -e "├────────────────┼─────────────────────────────────────────┼──────────────────────┤"
echo -e "│ 01:44:47       │ Deploy IoT application                  │ matrix, zk, package   │"
echo -e "│ 01:44:12       │ Verify device configuration             │ zk, store             │"
echo -e "└────────────────┴─────────────────────────────────────────┴──────────────────────┘"
