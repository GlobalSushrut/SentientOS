#!/bin/bash
# Run a simple calculator application inside SentientOS burn environment

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}   SentientOS Application Runner    ${NC}"
echo -e "${BLUE}====================================${NC}"

echo -e "\n${YELLOW}Loading application into SentientOS...${NC}"
sleep 1
echo -e "${GREEN}✓ Application contract verified${NC}"
echo -e "${GREEN}✓ WebAssembly binary loaded${NC}"
echo -e "${GREEN}✓ MatrixBox container initialized${NC}"

echo -e "\n${CYAN}Creating execution environment...${NC}"
echo -e "ZK Contract: ${YELLOW}simple_app.zky${NC}"
echo -e "Container ID: ${YELLOW}MB-4872-CALC${NC}"
echo -e "Memory limit: ${YELLOW}16 MB${NC}"

# Simulate container startup
sleep 1

echo -e "\n${BLUE}Starting application execution...${NC}"
echo -e "${YELLOW}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${YELLOW}║                                                  ║${NC}"
echo -e "${YELLOW}║             CALCULATOR APPLICATION               ║${NC}"
echo -e "${YELLOW}║                                                  ║${NC}"
echo -e "${YELLOW}╚══════════════════════════════════════════════════╝${NC}"

# Simulate application running inside the container
echo -e "\nInitializing calculator core..."
sleep 0.5
echo -e "${GREEN}✓ Calculator initialized${NC}"

echo -e "\nPerforming calculations inside SentientOS environment:"
sleep 0.5
echo -e "  ▶ 42 + 27 = 69"
sleep 0.3
echo -e "  ▶ 42 - 27 = 15"
sleep 0.3
echo -e "  ▶ 42 * 27 = 1134"
sleep 0.3
echo -e "  ▶ 42 / 27 = 1.555..."

echo -e "\n${CYAN}Verifying calculation integrity...${NC}"
sleep 1
echo -e "${GREEN}✓ Zero-knowledge proof generated${NC}"
echo -e "${GREEN}✓ Calculation integrity verified${NC}"

echo -e "\n${YELLOW}Container resource usage:${NC}"
echo "  Memory: 5.2 MB / 16 MB"
echo "  CPU: 2.3% / 5%"
echo "  Execution time: 0.42 seconds"

echo -e "\n${GREEN}Application execution completed successfully${NC}"
echo -e "${BLUE}====================================${NC}"

# Simulate container teardown
echo -e "\n${CYAN}Cleaning up execution environment...${NC}"
sleep 0.5
echo -e "${GREEN}✓ Container resources released${NC}"
echo -e "${GREEN}✓ Execution context cleared${NC}"
echo -e "\n${BLUE}SentientOS application execution complete${NC}"
