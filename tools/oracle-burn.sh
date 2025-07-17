#!/bin/bash
# SentientOS Oracle Burn Test Suite
# This script runs comprehensive stress tests on SentientOS

set -e

# Set path to SentientOS binaries
SENTIENTOS_BIN="../target/release/sentientos"
if [ ! -f "$SENTIENTOS_BIN" ]; then
    echo "Error: SentientOS binary not found at $SENTIENTOS_BIN"
    echo "Please build SentientOS first using ./build.sh"
    exit 1
fi

# Configuration
REPORT_FILE="burn-report.json"
ZK_CONTRACTS_DIR="../src/zk/contracts"
TEST_DATA_DIR="./test_data"
LOG_DIR="./burn_logs"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create directories
mkdir -p $LOG_DIR
mkdir -p $TEST_DATA_DIR

# Print header
echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}   SentientOS Oracle Burn Test      ${NC}"
echo -e "${BLUE}====================================${NC}"
echo -e "${YELLOW}Starting test at $(date)${NC}\n"

# Initialize report structure
cat > $REPORT_FILE << EOF
{
  "test_run": "$(date +%s)",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "system": "SentientOS",
  "results": {
    "bootstrap": {},
    "zk_verification": {},
    "container": {},
    "gossip": {},
    "panic_recovery": {},
    "intent_replay": {},
    "store": {},
    "performance": {}
  }
}
EOF

# Helper function for updating report
update_report() {
  local section=$1
  local key=$2
  local value=$3
  
  # Use jq to update the report file
  tmp=$(mktemp)
  jq ".results.$section.$key = $value" $REPORT_FILE > "$tmp" && mv "$tmp" $REPORT_FILE
}

# Helper function for test sections
run_test_section() {
  local name=$1
  local cmd=$2
  
  echo -e "\n${YELLOW}Running test section: ${name}${NC}"
  echo -e "${YELLOW}------------------------------------${NC}"
  
  # Execute test command with timing
  run_test_command() {
    local command="$1"
    local start_time=$(date +%s.%N)
    eval "$command"
    local status=$?
    local end_time=$(date +%s.%N)
    local duration=$(echo "$end_time - $start_time" | bc)
    echo "duration:$duration,status:$status"
    return $status
  }
  
  local result=$(run_test_command "$cmd")
  local duration=$(echo "$result" | cut -d',' -f1 | cut -d':' -f2)
  local status=$(echo "$result" | cut -d',' -f2 | cut -d':' -f2)
  
  if [ $status -eq 0 ]; then
    echo -e "${GREEN}✓ ${name} passed in ${duration}s${NC}"
    update_report "$name" "status" "\"passed\""
    update_report "$name" "duration" "$duration"
    return 0
  else
    duration=$(echo "$end_time - $start_time" | bc)
    
    echo -e "${RED}✗ ${name} failed after ${duration}s${NC}"
    update_report "$name" "status" "\"failed\""
    update_report "$name" "duration" "$duration"
    update_report "$name" "error_log" "\"$LOG_DIR/${name}.log\""
    return 1
  fi
}

# Generate test ZK contracts and proof data
generate_test_data() {
  echo -e "\n${BLUE}Generating test data...${NC}"
  mkdir -p $TEST_DATA_DIR/zk_contracts/
  
  # Generate ZK-YAML test contracts
  cat > $TEST_DATA_DIR/zk_contracts/test1.zky << EOF
name: "test_contract"
version: "1.0"
author: "SentientOS Test Suite"
permissions:
  - filesystem.read
  - network.none
  - container.spawn: false
verification:
  input:
    - name: "value"
      type: "u64"
  output:
    - name: "result"
      type: "u64"
  constraints:
    - "result >= value"
    - "result <= value * 2"
EOF

  # Generate more test data files
  for i in {1..10}; do
    cat > $TEST_DATA_DIR/zk_contracts/test$i.zky << EOF
name: "test_contract_$i"
version: "1.0"
author: "SentientOS Test Suite"
permissions:
  - filesystem.read
  - network.local
  - container.spawn: $([[ $((i % 2)) == 0 ]] && echo "true" || echo "false")
verification:
  input:
    - name: "value"
      type: "u64"
  output:
    - name: "result"
      type: "u64"
  constraints:
    - "result == value * $i"
EOF
  done
  
  echo -e "${GREEN}Test data generated successfully${NC}"
}

# 1. Bootstrap Testing
bootstrap_test() {
  echo "Testing SentientOS bootstrap process..."
  
  # Attempt to initialize the system
  ../target/release/sentientos init
  
  # Verify core directories exist
  for dir in .zk .gossip .intent .panic .matrixbox .boot .store; do
    if [ ! -d "../$dir" ]; then
      echo "Error: Directory $dir not found after bootstrap"
      return 1
    fi
  done
  
  # Verify services are running
  ../target/release/sentctl status | grep "OK" > /dev/null
  
  return 0
}

# 2. ZK Verification Stress Test
zk_verification_test() {
  echo "Running ZK verification stress test..."
  
  # Process each test contract
  success_count=0
  total=10
  
  for i in {1..10}; do
    echo "Testing contract $i with multiple proof generations..."
    
    # Generate and verify proofs multiple times
    for j in {1..100}; do
      input_value=$((RANDOM % 1000))
      expected_output=$((input_value * i))
      
      if ! ../target/release/sentctl zk generate --contract "$TEST_DATA_DIR/zk_contracts/test$i.zky" \
           --input "{\"value\": $input_value}" --output "{\"result\": $expected_output}" > /dev/null; then
        echo "Failed to generate proof for contract $i with input $input_value"
        continue
      fi
      
      # Verify the generated proof
      if ../target/release/sentctl zk verify --last > /dev/null; then
        success_count=$((success_count + 1))
      fi
    done
  done
  
  # Calculate success rate
  success_rate=$(echo "scale=2; $success_count / 1000 * 100" | bc)
  echo "ZK verification success rate: ${success_rate}%"
  
  update_report "zk_verification" "success_rate" "$success_rate"
  
  # Test passes if success rate is at least 95%
  [ $(echo "$success_rate >= 95" | bc) -eq 1 ]
  return $?
}

# 3. Container Stress Test
container_stress_test() {
  echo "Running MatrixBox container stress test..."
  
  # Create test container
  cat > $TEST_DATA_DIR/test_container.yaml << EOF
name: "test_container"
version: "1.0"
author: "Burn Test"
permissions:
  filesystem: ["read"]
  network: ["local"]
EOF
  
  # Copy sample WASM file
  cp ../examples/hello.wasm $TEST_DATA_DIR/main.wasm
  
  # Create container
  ../target/release/sentctl matrixbox create --path $TEST_DATA_DIR --name "burn-test-container"
  
  # Launch containers in parallel
  success_count=0
  for i in {1..50}; do
    echo "Launching container instance $i..."
    ../target/release/sentctl tso-run $TEST_DATA_DIR/burn-test-container.tso &
    pid=$!
    
    # Wait briefly then check if running
    sleep 0.5
    if ../target/release/sentctl matrixbox ls | grep "burn-test-container" > /dev/null; then
      success_count=$((success_count + 1))
    fi
    
    # Stop after brief run
    sleep 1
    kill $pid 2>/dev/null || true
  done
  
  # Calculate success rate
  success_rate=$(echo "scale=2; $success_count / 50 * 100" | bc)
  echo "Container launch success rate: ${success_rate}%"
  
  update_report "container" "success_rate" "$success_rate"
  
  # Test passes if success rate is at least 90%
  [ $(echo "$success_rate >= 90" | bc) -eq 1 ]
  return $?
}

# 4. Gossip Protocol Test
gossip_test() {
  echo "Testing gossip synchronization protocol..."
  
  # Enable gossip sync
  ../target/release/sentctl gossip enable
  
  # Generate random trace data
  for i in {1..5}; do
    # Create simulated peer data
    mkdir -p "../.gossip/peers/peer$i/traces"
    
    # Generate some random trace files
    for j in {1..10}; do
      openssl rand -base64 1000 > "../.gossip/peers/peer$i/traces/trace_$j.dat"
    done
    
    # Create hash files
    sha256sum "../.gossip/peers/peer$i/traces/"* > "../.gossip/peers/peer$i/hashes.txt"
  done
  
  # Try to verify traces
  ../target/release/sentctl gossip verify-trace
  
  # Check if verification logged success
  grep "verified" "../.gossip/log/sync.log" > /dev/null
  
  return $?
}

# 5. Panic Recovery Test
panic_recovery_test() {
  echo "Testing panic recovery system..."
  
  # Create a snapshot before intentional panic
  ../target/release/sentctl heal snapshot --reason "pre-panic-test"
  
  # Record initial state hash
  initial_hash=$(find ../.heal/snapshots -type f -name "*.json" | xargs sha256sum | cut -d' ' -f1)
  
  # Simulate a panic
  ../target/release/sentctl panic --reason "burn-test-panic" --simulate
  
  # Verify panic was recorded
  if ! [ -f "../.panic/status.json" ]; then
    echo "Panic status file not found"
    return 1
  fi
  
  # Recover from panic
  ../target/release/sentctl panic recover
  
  # Verify recovery restored the system
  recovered_hash=$(find ../.heal/active -type f -name "*.json" | xargs sha256sum | cut -d' ' -f1)
  
  # Compare hashes
  if [ "$initial_hash" == "$recovered_hash" ]; then
    echo "Recovery successful, state restored"
    return 0
  else
    echo "Recovery failed, state mismatch"
    return 1
  fi
}

# 6. Intent Replay Test
intent_replay_test() {
  echo "Testing developer intent recording and replay..."
  
  # Start intent recording
  ../target/release/sentctl intent record
  
  # Perform a series of actions
  ../target/release/sentctl zk-verify
  ../target/release/sentctl matrixbox ls
  ../target/release/sentctl store list
  
  # Stop recording
  ../target/release/sentctl intent stop
  
  # Get the recorded session ID
  session_id=$(ls -1 ../.intent/sessions/ | tail -1)
  
  # Try to replay the session
  ../target/release/sentctl intent replay --session "$session_id"
  
  # Check if replay was successful
  grep "replay completed" "../.intent/log/replay.log" > /dev/null
  
  return $?
}

# 7. Store Package Test
store_test() {
  echo "Testing ZK-Store package management..."
  
  # Create a test package index
  mkdir -p "../.store"
  cat > "../.store/index.json" << EOF
{
  "last_updated": $(date +%s),
  "packages": {
    "test-package": {
      "name": "test-package",
      "version": "1.0.0",
      "description": "Test package for burn testing",
      "author": "Oracle Burn Test",
      "license": "MIT",
      "dependencies": [],
      "url": "file://$TEST_DATA_DIR/test-package.tso",
      "hash": "0000000000000000000000000000000000000000000000000000000000000000",
      "signature": "",
      "zk_contract": null,
      "size": 1024
    }
  }
}
EOF

  # Create test package
  mkdir -p "$TEST_DATA_DIR/test-package"
  echo "console.log('Hello from test package');" > "$TEST_DATA_DIR/test-package/main.js"
  
  # Create package manifest
  cat > "$TEST_DATA_DIR/test-package/meta.yaml" << EOF
name: "test-package"
version: "1.0.0"
author: "Oracle Burn Test"
license: "MIT"
EOF

  # Try to install and then remove package
  ../target/release/sentctl store install --name "test-package" && \
  ../target/release/sentctl store remove --name "test-package"
  
  return $?
}

# 8. Performance Test
performance_test() {
  echo "Running performance benchmarks..."
  
  # Measure system boot time
  boot_start=$(date +%s.%N)
  ../target/release/sentientos boot --benchmark
  boot_end=$(date +%s.%N)
  boot_time=$(echo "$boot_end - $boot_start" | bc)
  
  # Measure ZK proof generation time
  zk_start=$(date +%s.%N)
  ../target/release/sentctl zk generate --contract "$TEST_DATA_DIR/zk_contracts/test1.zky" \
    --input "{\"value\": 42}" --output "{\"result\": 42}" > /dev/null
  zk_end=$(date +%s.%N)
  zk_time=$(echo "$zk_end - $zk_start" | bc)
  
  # Measure container startup time
  container_start=$(date +%s.%N)
  ../target/release/sentctl tso-run $TEST_DATA_DIR/burn-test-container.tso > /dev/null
  container_end=$(date +%s.%N)
  container_time=$(echo "$container_end - $container_start" | bc)
  
  # Update performance metrics in report
  update_report "performance" "boot_time_seconds" "$boot_time"
  update_report "performance" "zk_proof_generation_seconds" "$zk_time"
  update_report "performance" "container_startup_seconds" "$container_time"
  
  # Memory usage
  mem_usage=$(ps -o rss= -p $(pgrep -f sentientos) | awk '{sum+=$1} END {print sum/1024}')
  update_report "performance" "memory_usage_mb" "$mem_usage"
  
  # Pass if boot time under target threshold
  [ $(echo "$boot_time < 5" | bc) -eq 1 ]
  return $?
}

# Main test execution
main() {
  # Generate test data first
  generate_test_data
  
  # Run all test sections
  run_test_section "bootstrap" bootstrap_test
  bootstrap_result=$?
  
  run_test_section "zk_verification" zk_verification_test
  zk_result=$?
  
  run_test_section "container" container_stress_test
  container_result=$?
  
  run_test_section "gossip" gossip_test
  gossip_result=$?
  
  run_test_section "panic_recovery" panic_recovery_test
  panic_result=$?
  
  run_test_section "intent_replay" intent_replay_test
  intent_result=$?
  
  run_test_section "store" store_test
  store_result=$?
  
  run_test_section "performance" performance_test
  performance_result=$?
  
  # Summarize results
  echo -e "\n${BLUE}====================================${NC}"
  echo -e "${BLUE}   Oracle Burn Test Results         ${NC}"
  echo -e "${BLUE}====================================${NC}"
  
  total_tests=8
  passed_tests=0
  
  for result in $bootstrap_result $zk_result $container_result $gossip_result \
                $panic_result $intent_result $store_result $performance_result; do
    if [ $result -eq 0 ]; then
      passed_tests=$((passed_tests + 1))
    fi
  done
  
  pass_percentage=$((passed_tests * 100 / total_tests))
  
  echo -e "Tests passed: ${passed_tests}/${total_tests} (${pass_percentage}%)"
  
  # Update overall results
  tmp=$(mktemp)
  jq ".overall_result = \"$passed_tests/$total_tests tests passed\"" $REPORT_FILE > "$tmp" && mv "$tmp" $REPORT_FILE
  jq ".pass_percentage = $pass_percentage" $REPORT_FILE > "$tmp" && mv "$tmp" $REPORT_FILE
  
  if [ $pass_percentage -ge 95 ]; then
    echo -e "${GREEN}ORACLE BURN TEST PASSED${NC}"
    jq ".status = \"passed\"" $REPORT_FILE > "$tmp" && mv "$tmp" $REPORT_FILE
    return 0
  else
    echo -e "${RED}ORACLE BURN TEST FAILED${NC}"
    jq ".status = \"failed\"" $REPORT_FILE > "$tmp" && mv "$tmp" $REPORT_FILE
    return 1
  fi
}

# Parse command line arguments
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
  echo "Usage: $0 [OPTIONS]"
  echo ""
  echo "Options:"
  echo "  --full                 Run full test suite (default)"
  echo "  --zk-stress            Run only ZK verification stress test"
  echo "  --container-stress     Run only container stress test"
  echo "  --sync-stress          Run only sync stress test"
  echo "  --panic-recovery       Run only panic recovery test"
  echo "  --help, -h             Show this help message"
  exit 0
elif [ "$1" = "--zk-stress" ]; then
  generate_test_data
  run_test_section "zk_verification" zk_verification_test
  exit $?
elif [ "$1" = "--container-stress" ]; then
  generate_test_data
  run_test_section "container" container_stress_test
  exit $?
elif [ "$1" = "--sync-stress" ]; then
  run_test_section "gossip" gossip_test
  exit $?
elif [ "$1" = "--panic-recovery" ]; then
  run_test_section "panic_recovery" panic_recovery_test
  exit $?
else
  # Run full test suite
  main
  exit $?
fi
