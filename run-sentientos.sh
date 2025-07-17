#!/bin/bash
# SentientOS Oracle Burn Runtime Launcher
# This script launches SentientOS in TSO (Oracle) mode as a functional OS layer

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}   SentientOS Oracle Burn Runtime   ${NC}"
echo -e "${BLUE}====================================${NC}"
echo -e "${YELLOW}Booting SentientOS v0.1.0${NC}\n"

# Create core OS directories if they don't exist
mkdir -p .runtime .lock .auth .heal .gossip .intent .panic .zero .unsecure .browser
mkdir -p .container .tree .boot .matrixbox .store .zk .db
mkdir -p bin usr app termux.io contrac.to zk_contracts

# Initialize SentientOS runtime components
echo -e "${BLUE}Initializing core runtime...${NC}"
./target/release/sentientos init --tso-mode

# Set up MatrixBox container environment
echo -e "${BLUE}Starting MatrixBox container runtime...${NC}"
./target/release/sentctl matrixbox start

# Start ZK verification service
echo -e "${BLUE}Initializing ZK verification layer...${NC}"
./target/release/sentctl zk-service start &

# Launch gossip protocol sync daemon
echo -e "${BLUE}Starting gossip protocol daemon...${NC}"
./target/release/sentctl gossip start &

# Start contract service
echo -e "${BLUE}Starting contract service daemon...${NC}"
./target/release/sentctl contract daemon &

# Start system monitoring for panic recovery
echo -e "${BLUE}Starting system monitor...${NC}"
./target/release/sentctl panic watch &

# Launch the terminal UI
echo -e "\n${GREEN}SentientOS booted successfully!${NC}"
echo -e "${GREEN}Starting SentientOS terminal...${NC}\n"

# Launch terminal shell with custom prompt
export PS1="[\[\033[1;32m\]sentient\[\033[0m\]]$ "

# Create shell wrapper function for sentctl commands
cat > .sentshell << 'EOF'
#!/bin/bash
# SentientOS Shell

SENTCTL="./target/release/sentctl"

handle_command() {
  cmd="$1"
  shift
  
  case "$cmd" in
    ls|cat|pwd|cd|mkdir|echo|grep)
      # Pass through basic commands to the underlying shell
      "$cmd" "$@"
      ;;
    matrixbox)
      $SENTCTL matrixbox "$@"
      ;;
    zk)
      $SENTCTL zk "$@"
      ;;
    container)
      $SENTCTL container "$@"
      ;;
    store)
      $SENTCTL store "$@"
      ;;
    contract)
      $SENTCTL contract "$@"
      ;;
    run)
      $SENTCTL run "$@"
      ;;
    status)
      echo "SentientOS Status:"
      echo "  Runtime: Active"
      echo "  ZK Service: Running"
      echo "  MatrixBox Containers: $(ls -1 .container 2>/dev/null | wc -l)"
      echo "  Memory Usage: $(free -m | awk '/^Mem:/{print $3}') MB"
      ;;
    help)
      echo "SentientOS Commands:"
      echo "  matrixbox [ls|run|stop] - Manage containers"
      echo "  zk [verify|generate] - ZK operations" 
      echo "  container [create|list] - Container operations"
      echo "  store [install|list] - Package management"
      echo "  contract [load|verify] - Contract operations"
      echo "  run <app> - Run application"
      echo "  exit - Exit SentientOS"
      ;;
    exit)
      echo "Shutting down SentientOS..."
      $SENTCTL shutdown --clean
      exit 0
      ;;
    *)
      # Try sentctl command
      if $SENTCTL "$cmd" "$@"; then
        return 0
      else
        # If not a sentctl command, try as regular command
        if command -v "$cmd" &>/dev/null; then
          "$cmd" "$@"
        else
          echo "Unknown command: $cmd"
          echo "Type 'help' for available commands"
          return 1
        fi
      fi
      ;;
  esac
}

echo -e "${GREEN}SentientOS Terminal${NC}"
echo -e "${BLUE}Type 'help' for available commands${NC}"
echo -e "${BLUE}Type 'exit' to shutdown SentientOS${NC}\n"

while true; do
  read -e -p "[sentient]$ " cmd_line
  
  # Skip empty commands
  if [ -z "$cmd_line" ]; then
    continue
  fi
  
  # Parse command line
  read -ra cmd_array <<< "$cmd_line"
  handle_command "${cmd_array[@]}"
done
EOF

chmod +x .sentshell

# Run the shell
./sentshell
