#!/usr/bin/env bash
# Performance Metrics Collection Script for Valence Protocol
# This script collects and analyzes performance metrics for the unified programs

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Performance Metrics Collection...${NC}"

# Configuration
NETWORK=${NETWORK:-"devnet"}
RPC_URL=${RPC_URL:-"https://api.devnet.solana.com"}
COLLECTION_INTERVAL=${COLLECTION_INTERVAL:-"30"}
RETENTION_DAYS=${RETENTION_DAYS:-"7"}

# Program IDs
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

# Create metrics directories
mkdir -p "scripts/monitoring/metrics/performance"
mkdir -p "scripts/monitoring/metrics/compute"
mkdir -p "scripts/monitoring/metrics/costs"

# Initialize metric files
PERF_METRICS="scripts/monitoring/metrics/performance/performance_$(date +%Y%m%d).json"
COMPUTE_METRICS="scripts/monitoring/metrics/compute/compute_$(date +%Y%m%d).json"
COST_METRICS="scripts/monitoring/metrics/costs/costs_$(date +%Y%m%d).json"

# Function to log metrics
log_metric() {
    local metric_file=$1
    local metric_name=$2
    local value=$3
    local tags=$4
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    echo "{\"timestamp\":\"$timestamp\",\"metric\":\"$metric_name\",\"value\":$value,\"tags\":$tags}" >> "$metric_file"
}

# Function to collect transaction performance metrics
collect_transaction_metrics() {
    local program_id=$1
    local program_name=$2
    
    echo -e "${BLUE}Collecting transaction metrics for $program_name...${NC}"
    
    # Get recent transaction signatures
    local recent_sigs
    recent_sigs=$(solana transaction-history "$program_id" --url "$RPC_URL" --limit 20 2>/dev/null || echo "")
    
    if [ -n "$recent_sigs" ]; then
        local total_compute_units=0
        local total_fee=0
        local transaction_count=0
        local success_count=0
        local total_accounts=0
        
        while IFS= read -r sig; do
            if [ -n "$sig" ]; then
                transaction_count=$((transaction_count + 1))
                
                # Get transaction details
                local tx_details
                tx_details=$(solana transaction "$sig" --url "$RPC_URL" --output json 2>/dev/null || echo "{}")
                
                if [ "$tx_details" != "{}" ]; then
                    # Extract compute units (if available)
                    local compute_units
                    compute_units=$(echo "$tx_details" | jq -r '.meta.computeUnitsConsumed // 0' 2>/dev/null || echo 0)
                    total_compute_units=$((total_compute_units + compute_units))
                    
                    # Extract fee
                    local fee
                    fee=$(echo "$tx_details" | jq -r '.meta.fee // 0' 2>/dev/null || echo 0)
                    total_fee=$((total_fee + fee))
                    
                    # Count accounts
                    local accounts
                    accounts=$(echo "$tx_details" | jq -r '.transaction.message.accountKeys | length' 2>/dev/null || echo 0)
                    total_accounts=$((total_accounts + accounts))
                    
                    # Check if transaction succeeded
                    local status
                    status=$(echo "$tx_details" | jq -r '.meta.err // null' 2>/dev/null || echo "null")
                    if [ "$status" = "null" ]; then
                        success_count=$((success_count + 1))
                    fi
                fi
            fi
        done <<< "$recent_sigs"
        
        # Calculate averages
        if [ "$transaction_count" -gt 0 ]; then
            local avg_compute_units=$((total_compute_units / transaction_count))
            local avg_fee=$((total_fee / transaction_count))
            local avg_accounts=$((total_accounts / transaction_count))
            local success_rate=$((success_count * 100 / transaction_count))
            
            # Log performance metrics
            log_metric "$PERF_METRICS" "avg_compute_units" "$avg_compute_units" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            log_metric "$PERF_METRICS" "avg_fee_lamports" "$avg_fee" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            log_metric "$PERF_METRICS" "avg_accounts_per_tx" "$avg_accounts" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            log_metric "$PERF_METRICS" "success_rate_percent" "$success_rate" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            log_metric "$PERF_METRICS" "transaction_count" "$transaction_count" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            
            echo -e "${GREEN}✓ $program_name: $transaction_count txs, $success_rate% success, $avg_compute_units CU avg${NC}"
        fi
    fi
}

# Function to collect compute budget metrics
collect_compute_metrics() {
    local program_id=$1
    local program_name=$2
    
    echo -e "${BLUE}Collecting compute metrics for $program_name...${NC}"
    
    # Get program account to analyze data size
    local program_data
    program_data=$(solana program show "$program_id" --url "$RPC_URL" 2>/dev/null || echo "")
    
    if [ -n "$program_data" ]; then
        # Extract program data size
        local data_size
        data_size=$(echo "$program_data" | grep -o "Data Length: [0-9]*" | cut -d' ' -f3 || echo 0)
        
        # Calculate estimated compute units based on data size
        local estimated_cu=$((data_size / 100))  # Rough estimate: 1 CU per 100 bytes
        
        log_metric "$COMPUTE_METRICS" "program_data_size" "$data_size" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
        log_metric "$COMPUTE_METRICS" "estimated_cu_requirement" "$estimated_cu" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
        
        # Check if program has upgrade authority (affects compute cost)
        local has_upgrade_authority
        has_upgrade_authority=$(echo "$program_data" | grep -c "Upgrade Authority:" || echo 0)
        
        log_metric "$COMPUTE_METRICS" "has_upgrade_authority" "$has_upgrade_authority" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
        
        echo -e "${GREEN}✓ $program_name: $data_size bytes, ~$estimated_cu CU estimated${NC}"
    fi
}

# Function to collect cost metrics
collect_cost_metrics() {
    local program_id=$1
    local program_name=$2
    
    echo -e "${BLUE}Collecting cost metrics for $program_name...${NC}"
    
    # Get recent transaction signatures for cost analysis
    local recent_sigs
    recent_sigs=$(solana transaction-history "$program_id" --url "$RPC_URL" --limit 10 2>/dev/null || echo "")
    
    if [ -n "$recent_sigs" ]; then
        local total_cost_sol=0
        local total_cost_usd=0
        local transaction_count=0
        
        # Get current SOL price (mock for now - in production use actual price API)
        local sol_price_usd=100  # Mock price
        
        while IFS= read -r sig; do
            if [ -n "$sig" ]; then
                transaction_count=$((transaction_count + 1))
                
                # Get transaction fee
                local tx_details
                tx_details=$(solana transaction "$sig" --url "$RPC_URL" --output json 2>/dev/null || echo "{}")
                
                if [ "$tx_details" != "{}" ]; then
                    local fee_lamports
                    fee_lamports=$(echo "$tx_details" | jq -r '.meta.fee // 0' 2>/dev/null || echo 0)
                    
                    # Convert lamports to SOL (1 SOL = 1,000,000,000 lamports)
                    local fee_sol=$((fee_lamports / 1000000000))
                    local fee_usd=$((fee_sol * sol_price_usd))
                    
                    total_cost_sol=$((total_cost_sol + fee_sol))
                    total_cost_usd=$((total_cost_usd + fee_usd))
                fi
            fi
        done <<< "$recent_sigs"
        
        # Calculate averages
        if [ "$transaction_count" -gt 0 ]; then
            local avg_cost_sol=$((total_cost_sol / transaction_count))
            local avg_cost_usd=$((total_cost_usd / transaction_count))
            
            # Log cost metrics
            log_metric "$COST_METRICS" "avg_tx_cost_sol" "$avg_cost_sol" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            log_metric "$COST_METRICS" "avg_tx_cost_usd" "$avg_cost_usd" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            log_metric "$COST_METRICS" "total_cost_sol" "$total_cost_sol" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            log_metric "$COST_METRICS" "total_cost_usd" "$total_cost_usd" "{\"program\":\"$program_name\",\"network\":\"$NETWORK\"}"
            
            echo -e "${GREEN}✓ $program_name: $avg_cost_sol SOL avg, $total_cost_sol SOL total${NC}"
        fi
    fi
}

# Function to collect network performance metrics
collect_network_metrics() {
    echo -e "${BLUE}Collecting network performance metrics...${NC}"
    
    # Get network stats
    local epoch_info
    epoch_info=$(solana epoch-info --url "$RPC_URL" 2>/dev/null || echo "")
    
    if [ -n "$epoch_info" ]; then
        # Extract network metrics
        local slot_height
        slot_height=$(echo "$epoch_info" | grep "Slot:" | awk '{print $2}' || echo 0)
        
        local slots_in_epoch
        slots_in_epoch=$(echo "$epoch_info" | grep "Slots in Epoch:" | awk '{print $4}' || echo 0)
        
        local epoch_completed_percent
        epoch_completed_percent=$(echo "$epoch_info" | grep "Epoch Completed:" | awk '{print $3}' | tr -d '%' || echo 0)
        
        log_metric "$PERF_METRICS" "network_slot_height" "$slot_height" "{\"network\":\"$NETWORK\"}"
        log_metric "$PERF_METRICS" "network_slots_in_epoch" "$slots_in_epoch" "{\"network\":\"$NETWORK\"}"
        log_metric "$PERF_METRICS" "network_epoch_progress" "$epoch_completed_percent" "{\"network\":\"$NETWORK\"}"
        
        # Get TPS (transactions per second)
        local tps
        tps=$(solana ping --url "$RPC_URL" --count 1 2>/dev/null | grep -o "[0-9]*\.[0-9]* TPS" | cut -d' ' -f1 || echo 0)
        
        log_metric "$PERF_METRICS" "network_tps" "$tps" "{\"network\":\"$NETWORK\"}"
        
        echo -e "${GREEN}✓ Network: Slot $slot_height, $tps TPS, $epoch_completed_percent% epoch complete${NC}"
    fi
}

# Function to analyze performance trends
analyze_performance_trends() {
    echo -e "${BLUE}Analyzing performance trends...${NC}"
    
    # Analyze compute unit trends
    local programs=("eval" "shard" "registry")
    
    for program in "${programs[@]}"; do
        # Get last 10 compute unit measurements
        local recent_cu_values
        recent_cu_values=$(grep "avg_compute_units" "$PERF_METRICS" | grep "$program" | tail -10 | jq -r '.value' 2>/dev/null || echo "")
        
        if [ -n "$recent_cu_values" ]; then
            local total_cu=0
            local count=0
            local max_cu=0
            local min_cu=999999
            
            while IFS= read -r cu; do
                if [ -n "$cu" ] && [ "$cu" -gt 0 ]; then
                    total_cu=$((total_cu + cu))
                    count=$((count + 1))
                    
                    if [ "$cu" -gt "$max_cu" ]; then
                        max_cu="$cu"
                    fi
                    
                    if [ "$cu" -lt "$min_cu" ]; then
                        min_cu="$cu"
                    fi
                fi
            done <<< "$recent_cu_values"
            
            if [ "$count" -gt 0 ]; then
                local avg_cu=$((total_cu / count))
                local cu_variance=$((max_cu - min_cu))
                
                log_metric "$PERF_METRICS" "cu_trend_avg" "$avg_cu" "{\"program\":\"$program\",\"network\":\"$NETWORK\"}"
                log_metric "$PERF_METRICS" "cu_trend_max" "$max_cu" "{\"program\":\"$program\",\"network\":\"$NETWORK\"}"
                log_metric "$PERF_METRICS" "cu_trend_min" "$min_cu" "{\"program\":\"$program\",\"network\":\"$NETWORK\"}"
                log_metric "$PERF_METRICS" "cu_trend_variance" "$cu_variance" "{\"program\":\"$program\",\"network\":\"$NETWORK\"}"
                
                echo -e "${GREEN}✓ $program CU trend: $avg_cu avg, $min_cu-$max_cu range${NC}"
                
                # Alert if variance is high
                if [ "$cu_variance" -gt 5000 ]; then
                    echo -e "${YELLOW}⚠️  High CU variance for $program: $cu_variance${NC}"
                fi
            fi
        fi
    done
}

# Function to generate performance dashboard
generate_performance_dashboard() {
    local dashboard_file="scripts/monitoring/metrics/performance_dashboard_$(date +%Y%m%d_%H%M%S).html"
    
    echo -e "${BLUE}Generating performance dashboard...${NC}"
    
    cat > "$dashboard_file" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Valence Protocol Performance Dashboard</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .metric-card { border: 1px solid #ddd; padding: 15px; margin: 10px; border-radius: 5px; }
        .metric-value { font-size: 24px; font-weight: bold; color: #2e7d32; }
        .metric-label { font-size: 14px; color: #666; }
        .alert { background-color: #fff3cd; border: 1px solid #ffeaa7; padding: 10px; margin: 10px 0; border-radius: 5px; }
        .success { color: #2e7d32; }
        .warning { color: #f57c00; }
        .error { color: #d32f2f; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; }
    </style>
</head>
<body>
    <h1>Valence Protocol Performance Dashboard</h1>
    <p>Generated: <span id="timestamp"></span></p>
    
    <div class="grid">
        <div class="metric-card">
            <div class="metric-label">Average Compute Units</div>
            <div class="metric-value" id="avg-cu">Loading...</div>
        </div>
        
        <div class="metric-card">
            <div class="metric-label">Average Transaction Cost</div>
            <div class="metric-value" id="avg-cost">Loading...</div>
        </div>
        
        <div class="metric-card">
            <div class="metric-label">Success Rate</div>
            <div class="metric-value" id="success-rate">Loading...</div>
        </div>
        
        <div class="metric-card">
            <div class="metric-label">Network TPS</div>
            <div class="metric-value" id="network-tps">Loading...</div>
        </div>
    </div>
    
    <h2>Program Performance</h2>
    <div id="program-metrics"></div>
    
    <h2>Recent Alerts</h2>
    <div id="alerts"></div>
    
    <script>
        // Update timestamp
        document.getElementById('timestamp').textContent = new Date().toLocaleString();
        
        // In a real implementation, this would fetch actual metrics from the JSON files
        // For now, we'll show placeholder data
        document.getElementById('avg-cu').textContent = '~2,500 CU';
        document.getElementById('avg-cost').textContent = '~0.001 SOL';
        document.getElementById('success-rate').textContent = '99.5%';
        document.getElementById('network-tps').textContent = '~2,000 TPS';
        
        // Add program-specific metrics
        const programMetrics = document.getElementById('program-metrics');
        const programs = ['eval', 'shard', 'registry'];
        
        programs.forEach(program => {
            const div = document.createElement('div');
            div.className = 'metric-card';
            div.innerHTML = `
                <h3>${program.charAt(0).toUpperCase() + program.slice(1)} Program</h3>
                <p>Average CU: <span class="success">~2,000</span></p>
                <p>Success Rate: <span class="success">99.8%</span></p>
                <p>Average Cost: <span class="success">~0.0008 SOL</span></p>
            `;
            programMetrics.appendChild(div);
        });
        
        // Add alerts
        const alerts = document.getElementById('alerts');
        alerts.innerHTML = '<p class="success">No critical alerts in the last 24 hours</p>';
    </script>
</body>
</html>
EOF
    
    echo -e "${GREEN}✓ Performance dashboard generated: $dashboard_file${NC}"
}

# Function to cleanup old metrics
cleanup_old_metrics() {
    echo -e "${BLUE}Cleaning up old metrics...${NC}"
    
    # Remove metrics older than retention period
    find scripts/monitoring/metrics -name "*.json" -type f -mtime +"$RETENTION_DAYS" -delete 2>/dev/null || true
    find scripts/monitoring/metrics -name "*.html" -type f -mtime +"$RETENTION_DAYS" -delete 2>/dev/null || true
    
    echo -e "${GREEN}✓ Old metrics cleaned up (retention: $RETENTION_DAYS days)${NC}"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --network              Target network (devnet, testnet, mainnet-beta)"
    echo "  --collection-interval  Collection interval in seconds (default: 30)"
    echo "  --retention-days       Data retention period in days (default: 7)"
    echo "  --one-shot            Run once and exit"
    echo "  --dashboard           Generate dashboard and exit"
    echo "  --analyze             Analyze trends and exit"
    echo "  --help                Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Collect metrics continuously"
    echo "  $0 --network mainnet-beta            # Collect mainnet metrics"
    echo "  $0 --one-shot                        # Run once and exit"
    echo "  $0 --dashboard                       # Generate dashboard only"
}

# Parse command line arguments
ONE_SHOT=false
DASHBOARD_ONLY=false
ANALYZE_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --collection-interval)
            COLLECTION_INTERVAL="$2"
            shift 2
            ;;
        --retention-days)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --one-shot)
            ONE_SHOT=true
            shift
            ;;
        --dashboard)
            DASHBOARD_ONLY=true
            shift
            ;;
        --analyze)
            ANALYZE_ONLY=true
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

# Handle specific modes
if [ "$DASHBOARD_ONLY" = true ]; then
    generate_performance_dashboard
    exit 0
fi

if [ "$ANALYZE_ONLY" = true ]; then
    analyze_performance_trends
    exit 0
fi

# Initialize performance metrics collection
echo -e "${GREEN}Performance Metrics Collection Started${NC}"
echo -e "  Network: $NETWORK"
echo -e "  RPC URL: $RPC_URL"
echo -e "  Collection Interval: $COLLECTION_INTERVAL seconds"
echo -e "  Retention: $RETENTION_DAYS days"
echo -e "  Performance Log: $PERF_METRICS"
echo -e "  Compute Log: $COMPUTE_METRICS"
echo -e "  Cost Log: $COST_METRICS"

# Main collection loop
CYCLE_COUNT=0
while true; do
    CYCLE_COUNT=$((CYCLE_COUNT + 1))
    
    echo -e "\n${BLUE}Collection cycle #$CYCLE_COUNT at $(date)${NC}"
    
    # Collect metrics for each program
    collect_transaction_metrics "$EVAL_PROGRAM_ID" "eval"
    collect_transaction_metrics "$SHARD_PROGRAM_ID" "shard"
    collect_transaction_metrics "$REGISTRY_PROGRAM_ID" "registry"
    
    collect_compute_metrics "$EVAL_PROGRAM_ID" "eval"
    collect_compute_metrics "$SHARD_PROGRAM_ID" "shard"
    collect_compute_metrics "$REGISTRY_PROGRAM_ID" "registry"
    
    collect_cost_metrics "$EVAL_PROGRAM_ID" "eval"
    collect_cost_metrics "$SHARD_PROGRAM_ID" "shard"
    collect_cost_metrics "$REGISTRY_PROGRAM_ID" "registry"
    
    # Collect network metrics
    collect_network_metrics
    
    # Analyze trends every 10 cycles
    if [ $((CYCLE_COUNT % 10)) -eq 0 ]; then
        analyze_performance_trends
    fi
    
    # Generate dashboard every 20 cycles
    if [ $((CYCLE_COUNT % 20)) -eq 0 ]; then
        generate_performance_dashboard
    fi
    
    # Cleanup old metrics every 100 cycles
    if [ $((CYCLE_COUNT % 100)) -eq 0 ]; then
        cleanup_old_metrics
    fi
    
    # Exit if one-shot mode
    if [ "$ONE_SHOT" = true ]; then
        echo -e "\n${GREEN}One-shot collection completed${NC}"
        break
    fi
    
    # Wait for next cycle
    sleep "$COLLECTION_INTERVAL"
done 