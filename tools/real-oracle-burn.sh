#!/bin/bash
# Real SentientOS Oracle Burn Test - TSO Mode
# This implements an actual functional Oracle burn that works like a real OS

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}   SentientOS Real Oracle Burn      ${NC}"
echo -e "${BLUE}====================================${NC}"
echo -e "${YELLOW}Starting real OS components at $(date)${NC}\n"

# Setup directories for real OS components
mkdir -p ../.runtime
mkdir -p ../.lock/{binary.zk,zk.trace,zk.remind,zk.rollup}
mkdir -p ../.auth
mkdir -p ../.heal/{container,boot,trigger}
mkdir -p ../.gossip/{peers,pull,verify}
mkdir -p ../.intent/{sessions,replay,timeline}
mkdir -p ../.panic
mkdir -p ../.zero/{cli,auth,trace}
mkdir -p ../.unsecure/{wasm,legacy}
mkdir -p ../.container
mkdir -p ../.tree ../.boot
mkdir -p ../burn_results

# Set path to binaries
SENTIENTOS_BIN="../target/release/sentientos"
SENTCTL_BIN="../target/release/sentctl"

# Initialize real OS components
echo -e "${BLUE}Initializing SentientOS core components...${NC}"

# Create simple ZK contract for testing
cat > ../.lock/zk.trace/system.zky << EOF
name: "system_trace_contract"
version: "1.0"
permissions:
  - filesystem.read
  - network.localhost
verification:
  input:
    - name: "trace_hash"
      type: "string"
  output:
    - name: "verified"
      type: "boolean"
  constraints:
    - "verified == true"
EOF

# Create a peer hash file
cat > ../.gossip/peers/peer1.json << EOF
{
  "peer_id": "peer1",
  "endpoint": "localhost:8877",
  "pubkey": "046a2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f",
  "last_seen": $(date +%s)
}
EOF

# Create a test wasm container
echo -e "${BLUE}Setting up MatrixBox container...${NC}"
mkdir -p ../.container/test_app
cat > ../.container/test_app/meta.yaml << EOF
name: test_app
version: 1.0.0
runtime: wasm
entry: main.wasm
EOF

# Function to run real component tests
run_real_test() {
  local component=$1
  local command=$2
  
  echo -e "${YELLOW}Testing real component: ${component}${NC}"
  echo -e "${YELLOW}Command: ${command}${NC}"
  
  local start_time=$(date +%s.%N)
  eval "$command" > ../burn_results/$component.log 2>&1
  local status=$?
  local end_time=$(date +%s.%N)
  local duration=$(echo "$end_time - $start_time" | bc)
  
  if [ $status -eq 0 ]; then
    echo -e "${GREEN}✓ ${component} passed in ${duration}s${NC}"
    echo "{\"status\": \"passed\", \"duration\": $duration}" > ../burn_results/$component.json
    return 0
  else
    echo -e "${RED}✗ ${component} failed after ${duration}s${NC}"
    echo "{\"status\": \"failed\", \"duration\": $duration}" > ../burn_results/$component.json
    return 1
  fi
}

# Run the actual tests with real components
echo -e "\n${BLUE}Running real Oracle burn tests...${NC}"

# 1. System bootstrap test (initialize the core runtime)
run_real_test "bootstrap" "$SENTIENTOS_BIN init --tso-mode"

# 2. ZK verification test (verify ZK contracts)
run_real_test "zk_verification" "$SENTCTL_BIN zk-verify --contract=../.lock/zk.trace/system.zky"

# 3. MatrixBox container test
run_real_test "container" "$SENTCTL_BIN matrixbox ls"

# 4. Gossip protocol test with real peer synchronization
run_real_test "gossip" "$SENTCTL_BIN gossip verify-trace"

# 5. Panic recovery system test
run_real_test "panic_recovery" "$SENTCTL_BIN panic recover --simulate"

# 6. Intent recording and replay
run_real_test "intent_replay" "$SENTCTL_BIN intent record --duration=2 && $SENTCTL_BIN replay last"

# 7. Package manager test
run_real_test "store" "$SENTCTL_BIN store install test-package"

# 8. Performance benchmark
run_real_test "performance" "$SENTCTL_BIN benchmark --full"

# Generate final report
echo -e "\n${BLUE}Generating comprehensive burn report...${NC}"
jq -s '{ 
  "test_run": "'$(date +%s)'",
  "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
  "system": "SentientOS",
  "mode": "TSO Oracle Burn",
  "results": {
    "bootstrap": .[0],
    "zk_verification": .[1],
    "container": .[2],
    "gossip": .[3],
    "panic_recovery": .[4],
    "intent_replay": .[5],
    "store": .[6],
    "performance": .[7]
  },
  "system_info": {
    "kernel": "'$(uname -r)'",
    "hostname": "'$(hostname)'",
    "memory_total": "'$(free -m | awk '/^Mem:/{print $2}')' MB"
  }
}' ../burn_results/*.json > ../burn_results/real-oracle-burn-results.json

# Calculate overall results
passed=$(grep -c "\"status\": \"passed\"" ../burn_results/*.json)
total=8
pass_percentage=$((passed * 100 / total))

echo -e "\n${BLUE}====================================${NC}"
echo -e "${BLUE}   Real Oracle Burn Results         ${NC}"
echo -e "${BLUE}====================================${NC}"

echo -e "Tests passed: $passed/$total ($pass_percentage%)"

if [ $passed -eq $total ]; then
  echo -e "${GREEN}REAL ORACLE BURN TEST SUCCESSFUL!${NC}"
  echo -e "${GREEN}→ SentientOS is ready for production deployment${NC}"
else
  echo -e "${RED}REAL ORACLE BURN TEST INCOMPLETE${NC}"
  echo -e "${RED}→ Check individual component logs in burn_results directory${NC}"
fi

echo -e "\n${BLUE}Detailed report saved to burn_results/real-oracle-burn-results.json${NC}"
