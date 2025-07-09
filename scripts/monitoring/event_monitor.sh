#!/usr/bin/env bash
# Event Monitoring Script for Valence Protocol
# This script monitors on-chain events from the unified programs and logs them for analysis

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Valence Protocol Event Monitor...${NC}"

# Configuration
NETWORK=${NETWORK:-"devnet"}
RPC_URL=${RPC_URL:-"https://api.devnet.solana.com"}
POLL_INTERVAL=${POLL_INTERVAL:-"5"}
LOG_LEVEL=${LOG_LEVEL:-"INFO"}
OUTPUT_FORMAT=${OUTPUT_FORMAT:-"json"}

# Program IDs for monitoring
EVAL_PROGRAM_ID="EvalCont11111111111111111111111111111111111"
SHARD_PROGRAM_ID="ShardCon11111111111111111111111111111111111"
REGISTRY_PROGRAM_ID="RegCont1111111111111111111111111111111111111"

# Set RPC URL based on network
case $NETWORK in
    "devnet")
        RPC_URL="https://api.devnet.solana.com"
        ;;
    "testnet")
        RPC_URL="https://api.testnet.solana.com"
        ;;
    "mainnet-beta")
        RPC_URL="https://api.mainnet-beta.solana.com"
        ;;
esac

# Create monitoring directories
mkdir -p "scripts/monitoring/logs"
mkdir -p "scripts/monitoring/metrics"
mkdir -p "scripts/monitoring/alerts"

# Initialize log files
EVENT_LOG="scripts/monitoring/logs/events_$(date +%Y%m%d).log"
METRICS_LOG="scripts/monitoring/logs/metrics_$(date +%Y%m%d).log"
ALERT_LOG="scripts/monitoring/logs/alerts_$(date +%Y%m%d).log"

# Function to log events
log_event() {
    local level=$1
    local program=$2
    local event_type=$3
    local details=$4
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    case $OUTPUT_FORMAT in
        "json")
            echo "{\"timestamp\":\"$timestamp\",\"level\":\"$level\",\"program\":\"$program\",\"event_type\":\"$event_type\",\"details\":$details}" | tee -a "$EVENT_LOG"
            ;;
        "structured")
            echo "[$timestamp] $level [$program] $event_type: $details" | tee -a "$EVENT_LOG"
            ;;
        *)
            echo "[$timestamp] $level [$program] $event_type: $details" | tee -a "$EVENT_LOG"
            ;;
    esac
}

# Function to log metrics
log_metric() {
    local metric_name=$1
    local value=$2
    local tags=$3
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    echo "{\"timestamp\":\"$timestamp\",\"metric\":\"$metric_name\",\"value\":$value,\"tags\":$tags}" | tee -a "$METRICS_LOG"
}

# Function to trigger alert
trigger_alert() {
    local alert_type=$1
    local message=$2
    local severity=$3
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    echo "{\"timestamp\":\"$timestamp\",\"alert_type\":\"$alert_type\",\"message\":\"$message\",\"severity\":\"$severity\"}" | tee -a "$ALERT_LOG"
    
    # Send alert to console
    case $severity in
        "critical")
            echo -e "${RED}🚨 CRITICAL ALERT: $message${NC}"
            ;;
        "warning")
            echo -e "${YELLOW}⚠️  WARNING: $message${NC}"
            ;;
        "info")
            echo -e "${BLUE}ℹ️  INFO: $message${NC}"
            ;;
    esac
}

# Function to monitor program transactions
monitor_program_transactions() {
    local program_id=$1
    local program_name=$2
    
    echo -e "${BLUE}Monitoring transactions for $program_name...${NC}"
    
    # Get recent transactions for the program
    local recent_sigs
    recent_sigs=$(solana transaction-history "$program_id" --url "$RPC_URL" --limit 10 2>/dev/null || echo "")
    
    if [ -n "$recent_sigs" ]; then
        local tx_count=$(echo "$recent_sigs" | wc -l)
        log_metric "transaction_count" "$tx_count" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
        
        # Check for failed transactions
        local failed_count=0
        while IFS= read -r sig; do
            if [ -n "$sig" ]; then
                local tx_status
                tx_status=$(solana confirm "$sig" --url "$RPC_URL" 2>/dev/null || echo "failed")
                if [[ "$tx_status" == *"failed"* ]]; then
                    failed_count=$((failed_count + 1))
                    log_event "ERROR" "$program_name" "TRANSACTION_FAILED" "\"$sig\""
                fi
            fi
        done <<< "$recent_sigs"
        
        if [ "$failed_count" -gt 0 ]; then
            log_metric "failed_transactions" "$failed_count" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            
            # Trigger alert if failure rate is high
            local failure_rate=$((failed_count * 100 / tx_count))
            if [ "$failure_rate" -gt 20 ]; then
                trigger_alert "HIGH_FAILURE_RATE" "High transaction failure rate for $program_name: $failure_rate%" "warning"
            fi
        fi
    fi
}

# Function to monitor account states
monitor_account_states() {
    local program_id=$1
    local program_name=$2
    
    echo -e "${BLUE}Monitoring account states for $program_name...${NC}"
    
    # Get program accounts
    local account_count
    account_count=$(solana program show "$program_id" --url "$RPC_URL" 2>/dev/null | grep -c "Account" || echo 0)
    
    log_metric "account_count" "$account_count" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
    
    # Monitor program data size
    local program_data_size
    program_data_size=$(solana program show "$program_id" --url "$RPC_URL" 2>/dev/null | grep -o "Data Length: [0-9]*" | cut -d' ' -f3 || echo 0)
    
    log_metric "program_data_size" "$program_data_size" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
}

# Function to monitor network health
monitor_network_health() {
    echo -e "${BLUE}Monitoring network health...${NC}"
    
    # Check network connectivity
    local epoch_info
    epoch_info=$(solana epoch-info --url "$RPC_URL" 2>/dev/null || echo "")
    
    if [ -n "$epoch_info" ]; then
        # Extract metrics from epoch info
        local slot_height
        slot_height=$(echo "$epoch_info" | grep "Slot:" | awk '{print $2}' || echo 0)
        
        local epoch
        epoch=$(echo "$epoch_info" | grep "Epoch:" | awk '{print $2}' || echo 0)
        
        log_metric "slot_height" "$slot_height" "{\"network\":\"$NETWORK\"}"
        log_metric "epoch" "$epoch" "{\"network\":\"$NETWORK\"}"
        
        # Check block time
        local block_time
        block_time=$(solana block-time --url "$RPC_URL" 2>/dev/null | grep -o "[0-9]*\.[0-9]*" | head -1 || echo 0)
        
        log_metric "block_time" "$block_time" "{\"network\":\"$NETWORK\"}"
        
        # Alert if block time is too high
        if (( $(echo "$block_time > 1.0" | bc -l) )); then
            trigger_alert "SLOW_BLOCK_TIME" "Block time is $block_time seconds" "warning"
        fi
    else
        trigger_alert "NETWORK_CONNECTIVITY" "Cannot connect to $NETWORK network" "critical"
    fi
}

# Function to monitor program upgrade status
monitor_upgrade_status() {
    local program_id=$1
    local program_name=$2
    
    echo -e "${BLUE}Monitoring upgrade status for $program_name...${NC}"
    
    # Check upgrade authority
    local upgrade_authority
    upgrade_authority=$(solana program show "$program_id" --url "$RPC_URL" 2>/dev/null | grep "Upgrade Authority:" | awk '{print $3}' || echo "none")
    
    log_event "INFO" "$program_name" "UPGRADE_AUTHORITY" "\"$upgrade_authority\""
    
    # Alert if upgrade authority is not set properly
    if [ "$upgrade_authority" = "none" ] && [ "$NETWORK" = "mainnet-beta" ]; then
        trigger_alert "UPGRADE_AUTHORITY" "Program $program_name has no upgrade authority on mainnet" "warning"
    fi
}

# Function to generate monitoring report
generate_monitoring_report() {
    local report_file="scripts/monitoring/reports/monitoring_report_$(date +%Y%m%d_%H%M%S).md"
    mkdir -p "scripts/monitoring/reports"
    
    cat > "$report_file" << EOF
# Valence Protocol Monitoring Report

## Report Summary
- **Date:** $(date)
- **Network:** $NETWORK
- **RPC URL:** $RPC_URL
- **Monitoring Duration:** Started at $START_TIME

## Program Status
EOF

    # Add program-specific metrics
    local programs=("eval:$EVAL_PROGRAM_ID" "shard:$SHARD_PROGRAM_ID" "registry:$REGISTRY_PROGRAM_ID")
    
    for program_info in "${programs[@]}"; do
        local program_name=$(echo "$program_info" | cut -d':' -f1)
        local program_id=$(echo "$program_info" | cut -d':' -f2)
        
        echo -e "\n### $program_name Program ($program_id)" >> "$report_file"
        
        # Add recent metrics
        local recent_tx_count
        recent_tx_count=$(grep "transaction_count" "$METRICS_LOG" | grep "$program_name" | tail -1 | jq -r '.value' 2>/dev/null || echo 0)
        
        local recent_failed_count
        recent_failed_count=$(grep "failed_transactions" "$METRICS_LOG" | grep "$program_name" | tail -1 | jq -r '.value' 2>/dev/null || echo 0)
        
        echo -e "- **Recent Transactions:** $recent_tx_count" >> "$report_file"
        echo -e "- **Failed Transactions:** $recent_failed_count" >> "$report_file"
        
        # Add any recent alerts
        local recent_alerts
        recent_alerts=$(grep "$program_name" "$ALERT_LOG" | tail -3 || echo "")
        if [ -n "$recent_alerts" ]; then
            echo -e "- **Recent Alerts:**" >> "$report_file"
            echo -e "\`\`\`" >> "$report_file"
            echo "$recent_alerts" >> "$report_file"
            echo -e "\`\`\`" >> "$report_file"
        fi
    done
    
    # Add network health
    echo -e "\n## Network Health" >> "$report_file"
    local recent_slot_height
    recent_slot_height=$(grep "slot_height" "$METRICS_LOG" | tail -1 | jq -r '.value' 2>/dev/null || echo 0)
    
    local recent_block_time
    recent_block_time=$(grep "block_time" "$METRICS_LOG" | tail -1 | jq -r '.value' 2>/dev/null || echo 0)
    
    echo -e "- **Current Slot Height:** $recent_slot_height" >> "$report_file"
    echo -e "- **Recent Block Time:** $recent_block_time seconds" >> "$report_file"
    
    # Add summary of alerts
    echo -e "\n## Alert Summary" >> "$report_file"
    local total_alerts
    total_alerts=$(wc -l < "$ALERT_LOG" 2>/dev/null || echo 0)
    
    local critical_alerts
    critical_alerts=$(grep "\"severity\":\"critical\"" "$ALERT_LOG" | wc -l 2>/dev/null || echo 0)
    
    local warning_alerts
    warning_alerts=$(grep "\"severity\":\"warning\"" "$ALERT_LOG" | wc -l 2>/dev/null || echo 0)
    
    echo -e "- **Total Alerts:** $total_alerts" >> "$report_file"
    echo -e "- **Critical Alerts:** $critical_alerts" >> "$report_file"
    echo -e "- **Warning Alerts:** $warning_alerts" >> "$report_file"
    
    echo -e "\n${GREEN}✓ Monitoring report generated: $report_file${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --network          Target network (devnet, testnet, mainnet-beta)"
    echo "  --poll-interval    Polling interval in seconds (default: 5)"
    echo "  --log-level        Log level (DEBUG, INFO, WARN, ERROR)"
    echo "  --output-format    Output format (json, structured)"
    echo "  --one-shot         Run once and exit (don't run continuously)"
    echo "  --generate-report  Generate monitoring report and exit"
    echo "  --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Monitor devnet continuously"
    echo "  $0 --network mainnet-beta            # Monitor mainnet"
    echo "  $0 --one-shot                        # Run once and exit"
    echo "  $0 --generate-report                 # Generate report only"
}

# Parse command line arguments
ONE_SHOT=false
GENERATE_REPORT_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --poll-interval)
            POLL_INTERVAL="$2"
            shift 2
            ;;
        --log-level)
            LOG_LEVEL="$2"
            shift 2
            ;;
        --output-format)
            OUTPUT_FORMAT="$2"
            shift 2
            ;;
        --one-shot)
            ONE_SHOT=true
            shift
            ;;
        --generate-report)
            GENERATE_REPORT_ONLY=true
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Set RPC URL based on network
case $NETWORK in
    "devnet")
        RPC_URL="https://api.devnet.solana.com"
        ;;
    "testnet")
        RPC_URL="https://api.testnet.solana.com"
        ;;
    "mainnet-beta")
        RPC_URL="https://api.mainnet-beta.solana.com"
        ;;
    *)
        echo -e "${RED}Error: Invalid network '$NETWORK'${NC}"
        exit 1
        ;;
esac

# Generate report only if requested
if [ "$GENERATE_REPORT_ONLY" = true ]; then
    generate_monitoring_report
    exit 0
fi

# Initialize monitoring
START_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo -e "${GREEN}Valence Protocol Event Monitor Started${NC}"
echo -e "  Network: $NETWORK"
echo -e "  RPC URL: $RPC_URL"
echo -e "  Poll Interval: $POLL_INTERVAL seconds"
echo -e "  Log Level: $LOG_LEVEL"
echo -e "  Output Format: $OUTPUT_FORMAT"
echo -e "  Event Log: $EVENT_LOG"
echo -e "  Metrics Log: $METRICS_LOG"
echo -e "  Alert Log: $ALERT_LOG"

# Log monitoring start
log_event "INFO" "MONITOR" "STARTED" "{\"network\":\"$NETWORK\",\"poll_interval\":$POLL_INTERVAL}"

# Monitoring loop
LOOP_COUNT=0
while true; do
    LOOP_COUNT=$((LOOP_COUNT + 1))
    
    echo -e "\n${BLUE}Monitoring cycle #$LOOP_COUNT at $(date)${NC}"
    
    # Monitor network health
    monitor_network_health
    
    # Monitor each program
    monitor_program_transactions "$EVAL_PROGRAM_ID" "eval"
    monitor_program_transactions "$SHARD_PROGRAM_ID" "shard"
    monitor_program_transactions "$REGISTRY_PROGRAM_ID" "registry"
    
    monitor_account_states "$EVAL_PROGRAM_ID" "eval"
    monitor_account_states "$SHARD_PROGRAM_ID" "shard"
    monitor_account_states "$REGISTRY_PROGRAM_ID" "registry"
    
    monitor_upgrade_status "$EVAL_PROGRAM_ID" "eval"
    monitor_upgrade_status "$SHARD_PROGRAM_ID" "shard"
    monitor_upgrade_status "$REGISTRY_PROGRAM_ID" "registry"
    
    # Generate report every 100 cycles
    if [ $((LOOP_COUNT % 100)) -eq 0 ]; then
        generate_monitoring_report
    fi
    
    # Exit if one-shot mode
    if [ "$ONE_SHOT" = true ]; then
        echo -e "\n${GREEN}One-shot monitoring completed${NC}"
        generate_monitoring_report
        break
    fi
    
    # Wait for next cycle
    sleep "$POLL_INTERVAL"
done 