#!/bin/bash

# BitCell Network Launcher
# Interactive TUI for managing BitCell nodes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Configuration
DATA_DIR=".bitcell/nodes"
LOG_DIR=".bitcell/logs"
PIDS_FILE=".bitcell/pids.txt"

# Non-interactive mode flag
NON_INTERACTIVE=false
if [[ "$1" == "--non-interactive" ]] || [[ "$1" == "-n" ]]; then
    NON_INTERACTIVE=true
fi

# Initialize directories
mkdir -p "$DATA_DIR" "$LOG_DIR"

# Function to print header
print_header() {
    clear
    echo -e "${PURPLE}${BOLD}"
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                                                            ║"
    echo "║              🌌 BitCell Network Launcher 🌌                ║"
    echo "║                                                            ║"
    echo "║          Cellular Automaton Tournament Blockchain         ║"
    echo "║                                                            ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Function to check if nodes are running
check_running_nodes() {
    if [[ -f "$PIDS_FILE" ]]; then
        local count=0
        while IFS= read -r line; do
            local pid=$(echo "$line" | cut -d: -f1)
            if ps -p "$pid" > /dev/null 2>&1; then
                ((count++))
            fi
        done < "$PIDS_FILE"
        echo "$count"
    else
        echo "0"
    fi
}

# Function to display status
show_status() {
    local running=$(check_running_nodes)
    echo -e "${CYAN}${BOLD}Network Status:${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if [[ "$running" -gt 0 ]]; then
        echo -e "${GREEN}✓ $running node(s) running${NC}"
        echo ""
        if [[ -f "$PIDS_FILE" ]]; then
            while IFS= read -r line; do
                local pid=$(echo "$line" | cut -d: -f1)
                local type=$(echo "$line" | cut -d: -f2)
                local port=$(echo "$line" | cut -d: -f3)
                if ps -p "$pid" > /dev/null 2>&1; then
                    echo -e "  ${GREEN}●${NC} $type (PID: $pid, Port: $port)"
                    echo -e "    Metrics: ${BLUE}http://localhost:$((port+1))/metrics${NC}"
                fi
            done < "$PIDS_FILE"
        fi
    else
        echo -e "${YELLOW}○ No nodes running${NC}"
    fi
    echo ""
}

# Function to build project
build_project() {
    echo -e "${CYAN}Building BitCell...${NC}"
    if cargo build --release -p bitcell-node 2>&1 | grep -E "(Finished|error)"; then
        echo -e "${GREEN}✓ Build successful${NC}"
        return 0
    else
        echo -e "${RED}✗ Build failed${NC}"
        return 1
    fi
}

# Function to start a validator
start_validator() {
    local port=${1:-19000}
    local key_seed=${2:-""}
    local enable_dht=${3:-false}
    
    echo -e "${CYAN}Starting Validator on port $port...${NC}"
    
    local cmd="./target/release/bitcell-node validator --port $port"
    [[ -n "$key_seed" ]] && cmd="$cmd --key-seed $key_seed"
    [[ "$enable_dht" == "true" ]] && cmd="$cmd --enable-dht"
    
    $cmd > "$LOG_DIR/validator_$port.log" 2>&1 &
    local pid=$!
    
    sleep 2
    if ps -p "$pid" > /dev/null 2>&1; then
        echo "$pid:Validator:$port" >> "$PIDS_FILE"
        echo -e "${GREEN}✓ Validator started (PID: $pid)${NC}"
        echo -e "  Logs: ${BLUE}$LOG_DIR/validator_$port.log${NC}"
        echo -e "  Metrics: ${BLUE}http://localhost:$((port+1))/metrics${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to start validator${NC}"
        return 1
    fi
}

# Function to start a miner
start_miner() {
    local port=${1:-19100}
    local key_seed=${2:-""}
    local enable_dht=${3:-false}
    local bootstrap=${4:-""}
    
    echo -e "${CYAN}Starting Miner on port $port...${NC}"
    
    local cmd="./target/release/bitcell-node miner --port $port"
    [[ -n "$key_seed" ]] && cmd="$cmd --key-seed $key_seed"
    [[ "$enable_dht" == "true" ]] && cmd="$cmd --enable-dht"
    [[ -n "$bootstrap" ]] && cmd="$cmd --bootstrap $bootstrap"
    
    $cmd > "$LOG_DIR/miner_$port.log" 2>&1 &
    local pid=$!
    
    sleep 2
    if ps -p "$pid" > /dev/null 2>&1; then
        echo "$pid:Miner:$port" >> "$PIDS_FILE"
        echo -e "${GREEN}✓ Miner started (PID: $pid)${NC}"
        echo -e "  Logs: ${BLUE}$LOG_DIR/miner_$port.log${NC}"
        echo -e "  Metrics: ${BLUE}http://localhost:$((port+1))/metrics${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to start miner${NC}"
        return 1
    fi
}

# Function to start admin dashboard
start_admin() {
    local port=${1:-3000}
    
    echo -e "${CYAN}Starting Admin Dashboard on port $port...${NC}"
    
    cargo run --release -p bitcell-admin > "$LOG_DIR/admin_$port.log" 2>&1 &
    local pid=$!
    
    sleep 3
    if ps -p "$pid" > /dev/null 2>&1; then
        echo "$pid:Admin:$port" >> "$PIDS_FILE"
        echo -e "${GREEN}✓ Admin Dashboard started (PID: $pid)${NC}"
        echo -e "  Dashboard: ${BLUE}http://localhost:$port${NC}"
        echo -e "  Logs: ${BLUE}$LOG_DIR/admin_$port.log${NC}"
        
        # Try to open in browser
        if command -v open &> /dev/null; then
            sleep 2
            open "http://localhost:$port" 2>/dev/null || true
        fi
        return 0
    else
        echo -e "${RED}✗ Failed to start admin dashboard${NC}"
        cat "$LOG_DIR/admin_$port.log"
        return 1
    fi
}

# Function to clean .bitcell directory
clean_data() {
    echo -e "${YELLOW}${BOLD}⚠️  Warning: This will delete all node data, logs, and PIDs${NC}"
    echo ""
    
    if [[ -d ".bitcell" ]]; then
        echo -e "${CYAN}Current .bitcell contents:${NC}"
        du -sh .bitcell/* 2>/dev/null || echo "  (empty)"
        echo ""
    else
        echo -e "${YELLOW}No .bitcell directory found${NC}"
        return 0
    fi
    
    read -p "Are you sure you want to clean .bitcell? (yes/no): " confirm
    
    if [[ "$confirm" == "yes" ]]; then
        echo -e "${CYAN}Cleaning .bitcell directory...${NC}"
        rm -rf .bitcell
        mkdir -p "$DATA_DIR" "$LOG_DIR"
        echo -e "${GREEN}✓ .bitcell directory cleaned${NC}"
    else
        echo -e "${YELLOW}Clean cancelled${NC}"
    fi
}

# Function to stop all nodes
stop_all_nodes() {
    echo -e "${CYAN}Stopping all nodes...${NC}"
    
    if [[ ! -f "$PIDS_FILE" ]]; then
        echo -e "${YELLOW}No nodes to stop${NC}"
        return 0
    fi
    
    local count=0
    while IFS= read -r line; do
        local pid=$(echo "$line" | cut -d: -f1)
        local type=$(echo "$line" | cut -d: -f2)
        if ps -p "$pid" > /dev/null 2>&1; then
            kill "$pid" 2>/dev/null && ((count++))
            echo -e "${GREEN}✓ Stopped $type (PID: $pid)${NC}"
        fi
    done < "$PIDS_FILE"
    
    rm -f "$PIDS_FILE"
    echo -e "${GREEN}✓ Stopped $count node(s)${NC}"
}

# Function to view logs
view_logs() {
    echo -e "${CYAN}Available logs:${NC}"
    local i=1
    local logs=()
    
    for log in "$LOG_DIR"/*.log; do
        if [[ -f "$log" ]]; then
            logs+=("$log")
            echo "  $i) $(basename "$log")"
            ((i++))
        fi
    done
    
    if [[ ${#logs[@]} -eq 0 ]]; then
        echo -e "${YELLOW}No logs available${NC}"
        return
    fi
    
    echo ""
    read -p "Select log to view (1-${#logs[@]}, or 0 to cancel): " choice
    
    if [[ "$choice" -gt 0 ]] && [[ "$choice" -le ${#logs[@]} ]]; then
        local selected_log="${logs[$((choice-1))]}"
        echo -e "${CYAN}Viewing: $(basename "$selected_log")${NC}"
        echo -e "${CYAN}Press 'q' to exit${NC}"
        sleep 1
        tail -f "$selected_log"
    fi
}

# Function to launch test network
launch_test_network() {
    echo -e "${CYAN}${BOLD}Launching Test Network${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Build first
    if ! build_project; then
        echo ""
        read -p "Press Enter to continue..."
        return 1
    fi
    
    echo ""
    echo -e "${CYAN}Configuration:${NC}"
    echo "  • 1 Validator (port 19000)"
    echo "    - DHT: ${GREEN}ENABLED${NC}"
    echo "    - Key seed: 'bootstrap'"
    echo "  • 2 Miners (ports 19100, 19200)"
    echo "    - DHT: ${GREEN}ENABLED${NC}"
    echo "    - Key seeds: 'miner1', 'miner2'"
    echo "    - Bootstrap: /ip4/127.0.0.1/tcp/19000"
    echo ""
    
    if [[ "$NON_INTERACTIVE" == "false" ]]; then
        read -p "Continue? (y/n): " confirm
        [[ "$confirm" != "y" ]] && return 0
    fi
    
    echo ""
    start_validator 19000 "bootstrap" true
    sleep 2
    start_miner 19100 "miner1" true "/ip4/127.0.0.1/tcp/19000"
    sleep 2
    start_miner 19200 "miner2" true "/ip4/127.0.0.1/tcp/19000"
    
    echo ""
    echo -e "${GREEN}${BOLD}✓ Test network launched!${NC}"
    echo ""
    show_status
    
    if [[ "$NON_INTERACTIVE" == "false" ]]; then
        echo ""
        read -p "Press Enter to continue..."
    fi
}

# Main menu
show_menu() {
    print_header
    show_status
    
    echo -e "${CYAN}${BOLD}Main Menu:${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  1) 🚀 Launch Test Network (1 Validator + 2 Miners, DHT enabled)"
    echo "  2) 🌐 Start Web Admin Dashboard"
    echo "  3) ⚡ Start Single Validator"
    echo "  4) ⛏️  Start Single Miner"
    echo "  5) 📊 View Logs"
    echo "  6) 🔨 Build Project"
    echo "  7) 🛑 Stop All Nodes"
    echo "  8) 🧹 Clean .bitcell Directory"
    echo "  9) 🚪 Exit"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# Non-interactive mode
if [[ "$NON_INTERACTIVE" == "true" ]]; then
    print_header
    launch_test_network
    exit 0
fi

# Interactive mode main loop
while true; do
    show_menu
    read -p "Select option (1-9): " choice
    
    case $choice in
        1)
            launch_test_network
            ;;
        2)
            echo ""
            read -p "Port (default 3000): " port
            port=${port:-3000}
            echo ""
            if build_project; then
                echo ""
                start_admin "$port"
            fi
            echo ""
            read -p "Press Enter to continue..."
            ;;
        3)
            echo ""
            read -p "Port (default 19000): " port
            port=${port:-19000}
            read -p "Key seed (optional): " seed
            read -p "Enable DHT? (y/n): " dht
            dht_enabled="false"
            [[ "$dht" == "y" ]] && dht_enabled="true"
            echo ""
            if build_project; then
                echo ""
                start_validator "$port" "$seed" "$dht_enabled"
            fi
            echo ""
            read -p "Press Enter to continue..."
            ;;
        4)
            echo ""
            read -p "Port (default 19100): " port
            port=${port:-19100}
            read -p "Key seed (optional): " seed
            read -p "Enable DHT? (y/n): " dht
            dht_enabled="false"
            bootstrap=""
            if [[ "$dht" == "y" ]]; then
                dht_enabled="true"
                read -p "Bootstrap address (optional): " bootstrap
            fi
            echo ""
            if build_project; then
                echo ""
                start_miner "$port" "$seed" "$dht_enabled" "$bootstrap"
            fi
            echo ""
            read -p "Press Enter to continue..."
            ;;
        5)
            view_logs
            ;;
        6)
            echo ""
            build_project
            echo ""
            read -p "Press Enter to continue..."
            ;;
        7)
            echo ""
            stop_all_nodes
            echo ""
            read -p "Press Enter to continue..."
            ;;
        8)
            echo ""
            clean_data
            echo ""
            read -p "Press Enter to continue..."
            ;;
        9)
            running=$(check_running_nodes)
            if [[ "$running" -gt 0 ]]; then
                echo ""
                read -p "Stop all nodes before exiting? (y/n): " stop
                if [[ "$stop" == "y" ]]; then
                    stop_all_nodes
                fi
            fi
            echo ""
            echo -e "${PURPLE}Goodbye! 👋${NC}"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option${NC}"
            sleep 1
            ;;
    esac
done
