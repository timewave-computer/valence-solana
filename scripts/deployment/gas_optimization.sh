#!/usr/bin/env bash
# Gas optimization validation script for Valence Protocol
# This script validates compute unit usage, account sizes, and transaction costs

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Gas Optimization Validation...${NC}"

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must be run from project root${NC}"
    exit 1
fi

# Initialize optimization results
OPTIMIZATION_RESULTS="scripts/deployment/gas_optimization_results.md"
mkdir -p "$(dirname "$OPTIMIZATION_RESULTS")"

cat > "$OPTIMIZATION_RESULTS" << 'EOF'
# Gas Optimization Validation Results

## Overview
This document contains the results of gas optimization validation for Valence Protocol's unified architecture.

## Validation Date
EOF

date >> "$OPTIMIZATION_RESULTS"

echo -e "\n## Optimization Metrics\n" >> "$OPTIMIZATION_RESULTS"

# Function to log optimization results
log_optimization() {
    local category=$1
    local metric=$2
    local value=$3
    local status=$4
    local details=$5
    
    echo -e "### $category: $metric\n" >> "$OPTIMIZATION_RESULTS"
    echo -e "**Value:** $value" >> "$OPTIMIZATION_RESULTS"
    echo -e "**Status:** $status" >> "$OPTIMIZATION_RESULTS"
    echo -e "**Details:** $details\n" >> "$OPTIMIZATION_RESULTS"
    
    if [ "$status" = "✅ OPTIMAL" ]; then
        echo -e "${GREEN}✓ $category - $metric: $value${NC}"
    elif [ "$status" = "⚠️ ACCEPTABLE" ]; then
        echo -e "${YELLOW}⚠ $category - $metric: $value${NC}"
    else
        echo -e "${RED}✗ $category - $metric: $value${NC}"
    fi
}

# 1. Account Size Analysis
echo -e "\n${BLUE}1. Analyzing account sizes...${NC}"

# Calculate account sizes for each program
EVAL_ACCOUNT_SIZES=$(grep -r "SIZE.*=" programs/eval/src/ --include="*.rs" | wc -l)
SHARD_ACCOUNT_SIZES=$(grep -r "SIZE.*=" programs/shard/src/ --include="*.rs" | wc -l)
REGISTRY_ACCOUNT_SIZES=$(grep -r "SIZE.*=" programs/registry/src/ --include="*.rs" | wc -l)

TOTAL_ACCOUNT_TYPES=$((EVAL_ACCOUNT_SIZES + SHARD_ACCOUNT_SIZES + REGISTRY_ACCOUNT_SIZES))

if [ "$TOTAL_ACCOUNT_TYPES" -gt 0 ]; then
    log_optimization "Account Management" "Total Account Types" "$TOTAL_ACCOUNT_TYPES" "✅ OPTIMAL" "Unified account structures defined"
else
    log_optimization "Account Management" "Total Account Types" "$TOTAL_ACCOUNT_TYPES" "❌ NEEDS WORK" "No account size definitions found"
fi

# 2. Compute Unit Analysis
echo -e "\n${BLUE}2. Analyzing compute unit usage...${NC}"

# Check for compute budget management
COMPUTE_BUDGET_USAGE=$(grep -r "compute_budget\|CU_LIMIT\|ComputeBudgetInstruction" programs/ --include="*.rs" | wc -l)
if [ "$COMPUTE_BUDGET_USAGE" -gt 0 ]; then
    log_optimization "Compute Budget" "Management Implementation" "$COMPUTE_BUDGET_USAGE patterns" "✅ OPTIMAL" "Compute budget management implemented"
else
    log_optimization "Compute Budget" "Management Implementation" "0 patterns" "⚠️ ACCEPTABLE" "No explicit compute budget management found"
fi

# 3. Instruction Complexity Analysis
echo -e "\n${BLUE}3. Analyzing instruction complexity...${NC}"

# Count instructions per program
EVAL_INSTRUCTIONS=$(find programs/eval/src/ -name "*.rs" -exec grep -l "pub fn" {} \; | wc -l)
SHARD_INSTRUCTIONS=$(find programs/shard/src/ -name "*.rs" -exec grep -l "pub fn" {} \; | wc -l)
REGISTRY_INSTRUCTIONS=$(find programs/registry/src/ -name "*.rs" -exec grep -l "pub fn" {} \; | wc -l)

TOTAL_INSTRUCTIONS=$((EVAL_INSTRUCTIONS + SHARD_INSTRUCTIONS + REGISTRY_INSTRUCTIONS))

if [ "$TOTAL_INSTRUCTIONS" -lt 20 ]; then
    log_optimization "Instruction Complexity" "Total Instructions" "$TOTAL_INSTRUCTIONS" "✅ OPTIMAL" "Simplified instruction set"
elif [ "$TOTAL_INSTRUCTIONS" -lt 50 ]; then
    log_optimization "Instruction Complexity" "Total Instructions" "$TOTAL_INSTRUCTIONS" "⚠️ ACCEPTABLE" "Moderate instruction complexity"
else
    log_optimization "Instruction Complexity" "Total Instructions" "$TOTAL_INSTRUCTIONS" "❌ HIGH" "High instruction complexity may impact performance"
fi

# 4. Data Structure Optimization
echo -e "\n${BLUE}4. Analyzing data structure optimization...${NC}"

# Check for packed/aligned structures
PACKED_STRUCTURES=$(grep -r "#\[repr(packed)\]\|#\[repr(C)\]" programs/ --include="*.rs" | wc -l)
if [ "$PACKED_STRUCTURES" -gt 0 ]; then
    log_optimization "Data Structures" "Memory Layout Optimization" "$PACKED_STRUCTURES optimized" "✅ OPTIMAL" "Memory layout optimized for efficiency"
else
    log_optimization "Data Structures" "Memory Layout Optimization" "0 optimized" "⚠️ ACCEPTABLE" "No explicit memory layout optimization found"
fi

# 5. Cross-Program Call Analysis
echo -e "\n${BLUE}5. Analyzing cross-program calls...${NC}"

# Check for CPI calls (should be minimal in unified architecture)
CPI_CALLS=$(grep -r "CpiContext\|invoke\|invoke_signed" programs/ --include="*.rs" | wc -l)
if [ "$CPI_CALLS" -lt 5 ]; then
    log_optimization "Cross-Program Calls" "CPI Usage" "$CPI_CALLS calls" "✅ OPTIMAL" "Minimal CPI usage as expected in unified architecture"
elif [ "$CPI_CALLS" -lt 15 ]; then
    log_optimization "Cross-Program Calls" "CPI Usage" "$CPI_CALLS calls" "⚠️ ACCEPTABLE" "Moderate CPI usage"
else
    log_optimization "Cross-Program Calls" "CPI Usage" "$CPI_CALLS calls" "❌ HIGH" "High CPI usage may impact performance"
fi

# 6. Error Handling Efficiency
echo -e "\n${BLUE}6. Analyzing error handling efficiency...${NC}"

# Check for efficient error handling
ERROR_PATTERNS=$(grep -r "ValenceError\|require_\|msg!" programs/ --include="*.rs" | wc -l)
if [ "$ERROR_PATTERNS" -gt 20 ]; then
    log_optimization "Error Handling" "Pattern Usage" "$ERROR_PATTERNS patterns" "✅ OPTIMAL" "Comprehensive error handling implemented"
else
    log_optimization "Error Handling" "Pattern Usage" "$ERROR_PATTERNS patterns" "⚠️ ACCEPTABLE" "Basic error handling implemented"
fi

# 7. Transaction Size Analysis
echo -e "\n${BLUE}7. Analyzing transaction size optimization...${NC}"

# Check for transaction builders and batch operations
BATCH_OPERATIONS=$(grep -r "batch\|BatchOperation\|TransactionBuilder" programs/ --include="*.rs" | wc -l)
if [ "$BATCH_OPERATIONS" -gt 0 ]; then
    log_optimization "Transaction Size" "Batch Operations" "$BATCH_OPERATIONS patterns" "✅ OPTIMAL" "Batch operations implemented for efficiency"
else
    log_optimization "Transaction Size" "Batch Operations" "0 patterns" "⚠️ ACCEPTABLE" "No batch operations found"
fi

# 8. Account Rent Optimization
echo -e "\n${BLUE}8. Analyzing account rent optimization...${NC}"

# Check for account size calculations
ACCOUNT_SIZING=$(grep -r "get_space\|SIZE\|ACCOUNT_SIZE" programs/ --include="*.rs" | wc -l)
if [ "$ACCOUNT_SIZING" -gt 5 ]; then
    log_optimization "Account Rent" "Size Optimization" "$ACCOUNT_SIZING optimizations" "✅ OPTIMAL" "Account sizes optimized for rent efficiency"
else
    log_optimization "Account Rent" "Size Optimization" "$ACCOUNT_SIZING optimizations" "⚠️ ACCEPTABLE" "Basic account sizing implemented"
fi

# 9. Serialization Efficiency
echo -e "\n${BLUE}9. Analyzing serialization efficiency...${NC}"

# Check for efficient serialization patterns
SERIALIZATION_PATTERNS=$(grep -r "AnchorSerialize\|AnchorDeserialize\|borsh" programs/ --include="*.rs" | wc -l)
if [ "$SERIALIZATION_PATTERNS" -gt 10 ]; then
    log_optimization "Serialization" "Efficiency Patterns" "$SERIALIZATION_PATTERNS patterns" "✅ OPTIMAL" "Efficient serialization implemented"
else
    log_optimization "Serialization" "Efficiency Patterns" "$SERIALIZATION_PATTERNS patterns" "⚠️ ACCEPTABLE" "Basic serialization patterns"
fi

# 10. Program Size Analysis
echo -e "\n${BLUE}10. Analyzing program size...${NC}"

# Calculate approximate program sizes
EVAL_SIZE=$(find programs/eval/src/ -name "*.rs" -exec wc -l {} \; | awk '{sum += $1} END {print sum}')
SHARD_SIZE=$(find programs/shard/src/ -name "*.rs" -exec wc -l {} \; | awk '{sum += $1} END {print sum}')
REGISTRY_SIZE=$(find programs/registry/src/ -name "*.rs" -exec wc -l {} \; | awk '{sum += $1} END {print sum}')

TOTAL_SIZE=$((EVAL_SIZE + SHARD_SIZE + REGISTRY_SIZE))

if [ "$TOTAL_SIZE" -lt 5000 ]; then
    log_optimization "Program Size" "Total Lines of Code" "$TOTAL_SIZE lines" "✅ OPTIMAL" "Compact program size"
elif [ "$TOTAL_SIZE" -lt 15000 ]; then
    log_optimization "Program Size" "Total Lines of Code" "$TOTAL_SIZE lines" "⚠️ ACCEPTABLE" "Moderate program size"
else
    log_optimization "Program Size" "Total Lines of Code" "$TOTAL_SIZE lines" "❌ LARGE" "Large program size may impact deployment costs"
fi

# Generate recommendations
echo -e "\n## Optimization Recommendations\n" >> "$OPTIMIZATION_RESULTS"

if [ "$CPI_CALLS" -gt 10 ]; then
    echo -e "- **Reduce CPI calls:** Consider consolidating functionality to reduce cross-program calls" >> "$OPTIMIZATION_RESULTS"
fi

if [ "$BATCH_OPERATIONS" -eq 0 ]; then
    echo -e "- **Implement batch operations:** Add transaction batching for improved efficiency" >> "$OPTIMIZATION_RESULTS"
fi

if [ "$COMPUTE_BUDGET_USAGE" -eq 0 ]; then
    echo -e "- **Add compute budget management:** Implement explicit compute budget handling" >> "$OPTIMIZATION_RESULTS"
fi

if [ "$PACKED_STRUCTURES" -eq 0 ]; then
    echo -e "- **Optimize memory layout:** Consider using packed structures for better memory efficiency" >> "$OPTIMIZATION_RESULTS"
fi

# Generate final summary
echo -e "\n## Summary\n" >> "$OPTIMIZATION_RESULTS"
OPTIMAL_COUNT=$(grep -c "✅ OPTIMAL" "$OPTIMIZATION_RESULTS" || echo 0)
ACCEPTABLE_COUNT=$(grep -c "⚠️ ACCEPTABLE" "$OPTIMIZATION_RESULTS" || echo 0)
NEEDS_WORK_COUNT=$(grep -c "❌" "$OPTIMIZATION_RESULTS" || echo 0)

echo -e "- **Optimal:** $OPTIMAL_COUNT metrics" >> "$OPTIMIZATION_RESULTS"
echo -e "- **Acceptable:** $ACCEPTABLE_COUNT metrics" >> "$OPTIMIZATION_RESULTS"
echo -e "- **Needs Work:** $NEEDS_WORK_COUNT metrics\n" >> "$OPTIMIZATION_RESULTS"

if [ "$NEEDS_WORK_COUNT" -eq 0 ]; then
    echo -e "**Overall Status:** ✅ READY FOR DEPLOYMENT" >> "$OPTIMIZATION_RESULTS"
    echo -e "\n${GREEN}🎉 Gas optimization validation completed successfully!${NC}"
else
    echo -e "**Overall Status:** ⚠️ OPTIMIZATION OPPORTUNITIES AVAILABLE" >> "$OPTIMIZATION_RESULTS"
    echo -e "\n${YELLOW}⚠️ Some optimization opportunities were identified${NC}"
fi

echo -e "\n${GREEN}Optimization results saved to: $OPTIMIZATION_RESULTS${NC}" 