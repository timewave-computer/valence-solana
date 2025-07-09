#!/usr/bin/env bash
# Security audit script for Valence Protocol unified architecture
# This script validates security best practices and performs automated security checks

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Valence Protocol Security Audit...${NC}"

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must be run from project root${NC}"
    exit 1
fi

# Initialize audit results
AUDIT_RESULTS="scripts/deployment/audit_results.md"
mkdir -p "$(dirname "$AUDIT_RESULTS")"

cat > "$AUDIT_RESULTS" << 'EOF'
# Valence Protocol Security Audit Results

## Overview
This document contains the results of the automated security audit for Valence Protocol's unified architecture.

## Audit Date
EOF

date >> "$AUDIT_RESULTS"

echo -e "\n## Security Checks\n" >> "$AUDIT_RESULTS"

# Function to log audit results
log_audit() {
    local status=$1
    local check=$2
    local details=$3
    
    echo -e "### $check\n" >> "$AUDIT_RESULTS"
    echo -e "**Status:** $status\n" >> "$AUDIT_RESULTS"
    echo -e "$details\n" >> "$AUDIT_RESULTS"
    
    if [ "$status" = "✅ PASS" ]; then
        echo -e "${GREEN}✓ $check${NC}"
    elif [ "$status" = "⚠️ WARNING" ]; then
        echo -e "${YELLOW}⚠ $check${NC}"
    else
        echo -e "${RED}✗ $check${NC}"
    fi
}

# 1. Check for integer overflow vulnerabilities
echo -e "\n${YELLOW}1. Checking for integer overflow vulnerabilities...${NC}"
OVERFLOW_ISSUES=$(grep -r "add\|sub\|mul\|div" programs/ --include="*.rs" | grep -v "saturating_add\|saturating_sub\|saturating_mul\|checked_add\|checked_sub\|checked_mul" | wc -l)
if [ "$OVERFLOW_ISSUES" -eq 0 ]; then
    log_audit "✅ PASS" "Integer Overflow Protection" "All arithmetic operations use safe methods (saturating_* or checked_*)"
else
    log_audit "❌ FAIL" "Integer Overflow Protection" "Found $OVERFLOW_ISSUES potential unsafe arithmetic operations"
fi

# 2. Check for proper PDA validation
echo -e "\n${YELLOW}2. Checking PDA validation...${NC}"
PDA_CHECKS=$(grep -r "seeds\|bump" programs/ --include="*.rs" | grep -c "constraint\|has_one")
if [ "$PDA_CHECKS" -gt 0 ]; then
    log_audit "✅ PASS" "PDA Validation" "Found $PDA_CHECKS PDA validation constraints"
else
    log_audit "❌ FAIL" "PDA Validation" "No PDA validation constraints found"
fi

# 3. Check for proper authorization patterns
echo -e "\n${YELLOW}3. Checking authorization patterns...${NC}"
AUTH_CHECKS=$(grep -r "require_auth\|constraint.*authority" programs/ --include="*.rs" | wc -l)
if [ "$AUTH_CHECKS" -gt 0 ]; then
    log_audit "✅ PASS" "Authorization Patterns" "Found $AUTH_CHECKS authorization checks"
else
    log_audit "❌ FAIL" "Authorization Patterns" "No authorization patterns found"
fi

# 4. Check for proper account validation
echo -e "\n${YELLOW}4. Checking account validation...${NC}"
ACCOUNT_VALIDATION=$(grep -r "require_valid\|validate_state" programs/ --include="*.rs" | wc -l)
if [ "$ACCOUNT_VALIDATION" -gt 0 ]; then
    log_audit "✅ PASS" "Account Validation" "Found $ACCOUNT_VALIDATION account validation checks"
else
    log_audit "❌ FAIL" "Account Validation" "No account validation patterns found"
fi

# 5. Check for proper error handling
echo -e "\n${YELLOW}5. Checking error handling...${NC}"
ERROR_HANDLING=$(grep -r "ValenceError\|Result<" programs/ --include="*.rs" | wc -l)
if [ "$ERROR_HANDLING" -gt 10 ]; then
    log_audit "✅ PASS" "Error Handling" "Found $ERROR_HANDLING error handling patterns"
else
    log_audit "⚠️ WARNING" "Error Handling" "Found $ERROR_HANDLING error handling patterns (may need more)"
fi

# 6. Check for dangerous operations
echo -e "\n${YELLOW}6. Checking for dangerous operations...${NC}"
DANGEROUS_OPS=$(grep -r "unchecked\|unwrap\|panic\|todo\|unimplemented" programs/ --include="*.rs" | wc -l)
if [ "$DANGEROUS_OPS" -eq 0 ]; then
    log_audit "✅ PASS" "Dangerous Operations" "No dangerous operations found"
else
    log_audit "❌ FAIL" "Dangerous Operations" "Found $DANGEROUS_OPS potentially dangerous operations"
fi

# 7. Check for proper state management
echo -e "\n${YELLOW}7. Checking state management...${NC}"
STATE_MANAGEMENT=$(grep -r "ProgramState\|RegistryEntryBase\|LifecycleManaged" programs/ --include="*.rs" | wc -l)
if [ "$STATE_MANAGEMENT" -gt 0 ]; then
    log_audit "✅ PASS" "State Management" "Found $STATE_MANAGEMENT unified state management patterns"
else
    log_audit "❌ FAIL" "State Management" "No unified state management patterns found"
fi

# 8. Check for proper compute budget management
echo -e "\n${YELLOW}8. Checking compute budget management...${NC}"
COMPUTE_BUDGET=$(grep -r "compute_budget\|CU_LIMIT" programs/ --include="*.rs" | wc -l)
if [ "$COMPUTE_BUDGET" -gt 0 ]; then
    log_audit "✅ PASS" "Compute Budget Management" "Found $COMPUTE_BUDGET compute budget management patterns"
else
    log_audit "⚠️ WARNING" "Compute Budget Management" "No compute budget management found"
fi

# 9. Check for proper account sizing
echo -e "\n${YELLOW}9. Checking account sizing...${NC}"
ACCOUNT_SIZING=$(grep -r "SIZE\|get_space" programs/ --include="*.rs" | wc -l)
if [ "$ACCOUNT_SIZING" -gt 0 ]; then
    log_audit "✅ PASS" "Account Sizing" "Found $ACCOUNT_SIZING account sizing patterns"
else
    log_audit "❌ FAIL" "Account Sizing" "No account sizing patterns found"
fi

# 10. Check for proper testing coverage
echo -e "\n${YELLOW}10. Checking testing coverage...${NC}"
TEST_COVERAGE=$(find tests/ -name "*.rs" | wc -l)
if [ "$TEST_COVERAGE" -gt 3 ]; then
    log_audit "✅ PASS" "Testing Coverage" "Found $TEST_COVERAGE test files"
else
    log_audit "⚠️ WARNING" "Testing Coverage" "Found $TEST_COVERAGE test files (may need more)"
fi

# Run clippy for additional security checks
echo -e "\n${YELLOW}11. Running Clippy security checks...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings -D clippy::all > /dev/null 2>&1; then
    log_audit "✅ PASS" "Clippy Security Checks" "All clippy checks passed"
else
    log_audit "❌ FAIL" "Clippy Security Checks" "Clippy found potential issues"
fi

# Generate final summary
echo -e "\n## Summary\n" >> "$AUDIT_RESULTS"
PASS_COUNT=$(grep -c "✅ PASS" "$AUDIT_RESULTS" || echo 0)
WARNING_COUNT=$(grep -c "⚠️ WARNING" "$AUDIT_RESULTS" || echo 0)
FAIL_COUNT=$(grep -c "❌ FAIL" "$AUDIT_RESULTS" || echo 0)

echo -e "- **Passed:** $PASS_COUNT checks" >> "$AUDIT_RESULTS"
echo -e "- **Warnings:** $WARNING_COUNT checks" >> "$AUDIT_RESULTS"
echo -e "- **Failed:** $FAIL_COUNT checks\n" >> "$AUDIT_RESULTS"

if [ "$FAIL_COUNT" -eq 0 ]; then
    echo -e "**Overall Status:** ✅ READY FOR DEPLOYMENT" >> "$AUDIT_RESULTS"
    echo -e "\n${GREEN}🎉 Security audit completed successfully!${NC}"
else
    echo -e "**Overall Status:** ❌ REQUIRES FIXES BEFORE DEPLOYMENT" >> "$AUDIT_RESULTS"
    echo -e "\n${RED}⚠️ Security audit found issues that need to be addressed${NC}"
fi

echo -e "\n${GREEN}Audit results saved to: $AUDIT_RESULTS${NC}" 