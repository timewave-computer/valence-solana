#!/usr/bin/env bash
# Production Readiness Validation Script for Valence Protocol
# This script performs comprehensive validation to ensure the system is ready for production deployment

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║           VALENCE PROTOCOL PRODUCTION READINESS              ║${NC}"
echo -e "${PURPLE}║                 VALIDATION CHECKLIST                          ║${NC}"
echo -e "${PURPLE}╚═══════════════════════════════════════════════════════════════╝${NC}"

# Configuration
NETWORK=${NETWORK:-"devnet"}
VALIDATION_DIR="scripts/validation"
RESULTS_DIR="$VALIDATION_DIR/results"
CHECKLIST_FILE="$RESULTS_DIR/production_readiness_$(date +%Y%m%d_%H%M%S).md"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Initialize validation results
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0

# Function to log validation results
log_validation() {
    local status=$1
    local category=$2
    local check_name=$3
    local details=$4
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    case $status in
        "PASS")
            PASSED_CHECKS=$((PASSED_CHECKS + 1))
            echo -e "${GREEN}✓ [$category] $check_name${NC}"
            ;;
        "FAIL")
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
            echo -e "${RED}✗ [$category] $check_name${NC}"
            ;;
        "WARN")
            WARNING_CHECKS=$((WARNING_CHECKS + 1))
            echo -e "${YELLOW}⚠ [$category] $check_name${NC}"
            ;;
    esac
    
    # Log to checklist file
    echo "- [$status] **$category**: $check_name - $details" >> "$CHECKLIST_FILE"
}

# Function to run command and check result
check_command() {
    local command=$1
    local description=$2
    local category=$3
    
    if eval "$command" >/dev/null 2>&1; then
        log_validation "PASS" "$category" "$description" "Command executed successfully"
        return 0
    else
        log_validation "FAIL" "$category" "$description" "Command failed to execute"
        return 1
    fi
}

# Initialize checklist file
cat > "$CHECKLIST_FILE" << EOF
# Valence Protocol Production Readiness Checklist

**Validation Date:** $(date)
**Network:** $NETWORK
**Version:** $(git describe --tags --always 2>/dev/null || echo "unknown")

## Validation Results

EOF

echo -e "\n${BLUE}Starting Production Readiness Validation...${NC}"

# 1. CODE QUALITY CHECKS
echo -e "\n${BLUE}1. CODE QUALITY VALIDATION${NC}"

# Check for compilation
if check_command "cargo check --workspace" "Code compilation" "Code Quality"; then
    :
fi

# Check for clippy warnings
if cargo clippy --workspace --all-targets --all-features -- -D warnings >/dev/null 2>&1; then
    log_validation "PASS" "Code Quality" "Clippy linting" "No clippy warnings found"
else
    log_validation "WARN" "Code Quality" "Clippy linting" "Clippy warnings present"
fi

# Check for formatting
if cargo fmt --check >/dev/null 2>&1; then
    log_validation "PASS" "Code Quality" "Code formatting" "Code is properly formatted"
else
    log_validation "FAIL" "Code Quality" "Code formatting" "Code formatting issues found"
fi

# Check for TODO/FIXME comments
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" programs/ utils/ tests/ --include="*.rs" | wc -l || echo 0)
if [ "$TODO_COUNT" -eq 0 ]; then
    log_validation "PASS" "Code Quality" "TODO/FIXME cleanup" "No TODO/FIXME comments found"
elif [ "$TODO_COUNT" -lt 5 ]; then
    log_validation "WARN" "Code Quality" "TODO/FIXME cleanup" "$TODO_COUNT TODO/FIXME comments found"
else
    log_validation "FAIL" "Code Quality" "TODO/FIXME cleanup" "$TODO_COUNT TODO/FIXME comments found"
fi

# Check for panic!/unwrap usage
PANIC_COUNT=$(grep -r "panic!\|unwrap()\|expect(" programs/ --include="*.rs" | wc -l || echo 0)
if [ "$PANIC_COUNT" -eq 0 ]; then
    log_validation "PASS" "Code Quality" "Panic/unwrap usage" "No panic!/unwrap() found"
elif [ "$PANIC_COUNT" -lt 3 ]; then
    log_validation "WARN" "Code Quality" "Panic/unwrap usage" "$PANIC_COUNT panic!/unwrap() found"
else
    log_validation "FAIL" "Code Quality" "Panic/unwrap usage" "$PANIC_COUNT panic!/unwrap() found"
fi

# 2. TESTING VALIDATION
echo -e "\n${BLUE}2. TESTING VALIDATION${NC}"

# Unit tests
if check_command "cargo test --workspace --lib" "Unit tests" "Testing"; then
    :
fi

# Integration tests
if check_command "cargo test --workspace --test '*'" "Integration tests" "Testing"; then
    :
fi

# Test coverage check
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    COVERAGE=$(cargo tarpaulin --workspace --out xml --output-dir target/ 2>/dev/null | grep -o "coverage: [0-9]*\.[0-9]*%" | cut -d' ' -f2 | cut -d'%' -f1 || echo 0)
    if (( $(echo "$COVERAGE >= 80" | bc -l) )); then
        log_validation "PASS" "Testing" "Test coverage" "Coverage: ${COVERAGE}%"
    elif (( $(echo "$COVERAGE >= 60" | bc -l) )); then
        log_validation "WARN" "Testing" "Test coverage" "Coverage: ${COVERAGE}%"
    else
        log_validation "FAIL" "Testing" "Test coverage" "Coverage: ${COVERAGE}%"
    fi
else
    log_validation "WARN" "Testing" "Test coverage" "cargo-tarpaulin not installed"
fi

# Benchmark tests
if [ -d "benches" ]; then
    if check_command "cargo bench --no-run" "Benchmark compilation" "Testing"; then
        :
    fi
else
    log_validation "WARN" "Testing" "Benchmark tests" "No benchmark tests found"
fi

# 3. SECURITY VALIDATION
echo -e "\n${BLUE}3. SECURITY VALIDATION${NC}"

# Security audit script
if [ -f "scripts/deployment/security_audit.sh" ]; then
    if bash scripts/deployment/security_audit.sh >/dev/null 2>&1; then
        if grep -q "READY FOR DEPLOYMENT" "scripts/deployment/audit_results.md" 2>/dev/null; then
            log_validation "PASS" "Security" "Security audit" "Security audit passed"
        else
            log_validation "FAIL" "Security" "Security audit" "Security audit failed"
        fi
    else
        log_validation "FAIL" "Security" "Security audit" "Security audit script failed"
    fi
else
    log_validation "FAIL" "Security" "Security audit" "Security audit script not found"
fi

# Check for hardcoded secrets
SECRET_COUNT=$(grep -r "secret\|password\|private_key\|api_key" programs/ --include="*.rs" -i | grep -v "// " | wc -l || echo 0)
if [ "$SECRET_COUNT" -eq 0 ]; then
    log_validation "PASS" "Security" "Hardcoded secrets" "No hardcoded secrets found"
else
    log_validation "FAIL" "Security" "Hardcoded secrets" "$SECRET_COUNT potential secrets found"
fi

# Check for proper error handling
ERROR_HANDLING=$(grep -r "Result<" programs/ --include="*.rs" | wc -l || echo 0)
if [ "$ERROR_HANDLING" -gt 50 ]; then
    log_validation "PASS" "Security" "Error handling" "Comprehensive error handling found"
elif [ "$ERROR_HANDLING" -gt 20 ]; then
    log_validation "WARN" "Security" "Error handling" "Basic error handling found"
else
    log_validation "FAIL" "Security" "Error handling" "Insufficient error handling"
fi

# 4. PERFORMANCE VALIDATION
echo -e "\n${BLUE}4. PERFORMANCE VALIDATION${NC}"

# Gas optimization
if [ -f "scripts/deployment/gas_optimization.sh" ]; then
    if bash scripts/deployment/gas_optimization.sh >/dev/null 2>&1; then
        log_validation "PASS" "Performance" "Gas optimization" "Gas optimization validation passed"
    else
        log_validation "WARN" "Performance" "Gas optimization" "Gas optimization validation completed with warnings"
    fi
else
    log_validation "WARN" "Performance" "Gas optimization" "Gas optimization script not found"
fi

# Binary size check
BINARY_SIZES=""
for program in eval shard registry; do
    BINARY_FILE="target/deploy/${program}.so"
    if [ -f "$BINARY_FILE" ]; then
        SIZE=$(stat -f%z "$BINARY_FILE" 2>/dev/null || stat -c%s "$BINARY_FILE" 2>/dev/null || echo 0)
        SIZE_KB=$((SIZE / 1024))
        BINARY_SIZES="$BINARY_SIZES $program:${SIZE_KB}KB"
    fi
done

if [ -n "$BINARY_SIZES" ]; then
    log_validation "PASS" "Performance" "Binary sizes" "Program sizes:$BINARY_SIZES"
else
    log_validation "FAIL" "Performance" "Binary sizes" "No program binaries found"
fi

# Account space optimization
ACCOUNT_SIZING=$(grep -r "SIZE\|get_space" programs/ --include="*.rs" | wc -l || echo 0)
if [ "$ACCOUNT_SIZING" -gt 5 ]; then
    log_validation "PASS" "Performance" "Account sizing" "Account size optimization implemented"
else
    log_validation "WARN" "Performance" "Account sizing" "Limited account size optimization"
fi

# 5. DOCUMENTATION VALIDATION
echo -e "\n${BLUE}5. DOCUMENTATION VALIDATION${NC}"

# README files
README_COUNT=$(find . -name "README.md" | wc -l || echo 0)
if [ "$README_COUNT" -ge 3 ]; then
    log_validation "PASS" "Documentation" "README files" "$README_COUNT README files found"
else
    log_validation "WARN" "Documentation" "README files" "Only $README_COUNT README files found"
fi

# API documentation
DOC_COMMENTS=$(grep -r "///" programs/ --include="*.rs" | wc -l || echo 0)
if [ "$DOC_COMMENTS" -gt 100 ]; then
    log_validation "PASS" "Documentation" "API documentation" "Comprehensive API documentation"
elif [ "$DOC_COMMENTS" -gt 50 ]; then
    log_validation "WARN" "Documentation" "API documentation" "Basic API documentation"
else
    log_validation "FAIL" "Documentation" "API documentation" "Insufficient API documentation"
fi

# Migration guide
if [ -f "docs/MIGRATION_GUIDE.md" ]; then
    log_validation "PASS" "Documentation" "Migration guide" "Migration guide exists"
else
    log_validation "WARN" "Documentation" "Migration guide" "Migration guide not found"
fi

# Developer onboarding
if [ -f "docs/developer_onboarding/README.md" ]; then
    log_validation "PASS" "Documentation" "Developer onboarding" "Developer onboarding guide exists"
else
    log_validation "FAIL" "Documentation" "Developer onboarding" "Developer onboarding guide not found"
fi

# 6. DEPLOYMENT READINESS
echo -e "\n${BLUE}6. DEPLOYMENT READINESS${NC}"

# Deployment scripts
if [ -f "scripts/deployment/deploy.sh" ]; then
    log_validation "PASS" "Deployment" "Deployment scripts" "Deployment scripts exist"
else
    log_validation "FAIL" "Deployment" "Deployment scripts" "Deployment scripts not found"
fi

# Upgrade authority setup
if [ -f "scripts/deployment/upgrade_authority.sh" ]; then
    log_validation "PASS" "Deployment" "Upgrade authority" "Upgrade authority scripts exist"
else
    log_validation "FAIL" "Deployment" "Upgrade authority" "Upgrade authority scripts not found"
fi

# Monitoring setup
if [ -f "scripts/monitoring/event_monitor.sh" ]; then
    log_validation "PASS" "Deployment" "Monitoring setup" "Monitoring scripts exist"
else
    log_validation "FAIL" "Deployment" "Monitoring setup" "Monitoring scripts not found"
fi

# Configuration management
if [ -f "scripts/deployment/initial_capabilities.json" ]; then
    log_validation "PASS" "Deployment" "Initial configuration" "Initial configuration exists"
else
    log_validation "WARN" "Deployment" "Initial configuration" "Initial configuration not found"
fi

# 7. LEGACY CODE CLEANUP
echo -e "\n${BLUE}7. LEGACY CODE CLEANUP${NC}"

# Check for legacy programs
LEGACY_PROGRAMS=("entrypoint" "diff" "namespace_scoping" "function_composition" "verification_function_table" "verification_functions")
LEGACY_FOUND=0

for program in "${LEGACY_PROGRAMS[@]}"; do
    if [ -d "programs/$program" ]; then
        LEGACY_FOUND=$((LEGACY_FOUND + 1))
    fi
done

if [ "$LEGACY_FOUND" -eq 0 ]; then
    log_validation "PASS" "Legacy Cleanup" "Legacy programs" "No legacy programs found"
else
    log_validation "FAIL" "Legacy Cleanup" "Legacy programs" "$LEGACY_FOUND legacy programs still exist"
fi

# Check for unused dependencies
UNUSED_DEPS=$(cargo machete 2>/dev/null | wc -l || echo 0)
if [ "$UNUSED_DEPS" -eq 0 ]; then
    log_validation "PASS" "Legacy Cleanup" "Unused dependencies" "No unused dependencies"
else
    log_validation "WARN" "Legacy Cleanup" "Unused dependencies" "$UNUSED_DEPS unused dependencies found"
fi

# Check for empty directories
EMPTY_DIRS=$(find . -type d -empty | wc -l || echo 0)
if [ "$EMPTY_DIRS" -eq 0 ]; then
    log_validation "PASS" "Legacy Cleanup" "Empty directories" "No empty directories"
else
    log_validation "WARN" "Legacy Cleanup" "Empty directories" "$EMPTY_DIRS empty directories found"
fi

# 8. EXAMPLES AND TEMPLATES
echo -e "\n${BLUE}8. EXAMPLES AND TEMPLATES${NC}"

# SDK examples
if [ -d "sdk" ]; then
    EXAMPLE_COUNT=$(find sdk -name "*.rs" -o -name "*.ts" -o -name "*.js" | wc -l || echo 0)
    if [ "$EXAMPLE_COUNT" -gt 10 ]; then
        log_validation "PASS" "Examples" "SDK examples" "$EXAMPLE_COUNT example files found"
    elif [ "$EXAMPLE_COUNT" -gt 5 ]; then
        log_validation "WARN" "Examples" "SDK examples" "$EXAMPLE_COUNT example files found"
    else
        log_validation "FAIL" "Examples" "SDK examples" "Insufficient examples found"
    fi
else
    log_validation "FAIL" "Examples" "SDK examples" "SDK directory not found"
fi

# Workshop materials
if [ -d "docs/developer_onboarding/workshops" ]; then
    WORKSHOP_COUNT=$(find docs/developer_onboarding/workshops -name "*.md" | wc -l || echo 0)
    if [ "$WORKSHOP_COUNT" -ge 3 ]; then
        log_validation "PASS" "Examples" "Workshop materials" "$WORKSHOP_COUNT workshop guides found"
    else
        log_validation "WARN" "Examples" "Workshop materials" "Limited workshop materials"
    fi
else
    log_validation "WARN" "Examples" "Workshop materials" "Workshop materials not found"
fi

# Template applications
TEMPLATE_COUNT=$(find . -name "*template*" -o -name "*example*" | grep -v target | grep -v node_modules | wc -l || echo 0)
if [ "$TEMPLATE_COUNT" -ge 5 ]; then
    log_validation "PASS" "Examples" "Template applications" "$TEMPLATE_COUNT templates/examples found"
else
    log_validation "WARN" "Examples" "Template applications" "Limited template applications"
fi

# 9. NETWORK COMPATIBILITY
echo -e "\n${BLUE}9. NETWORK COMPATIBILITY${NC}"

# Devnet compatibility
if check_command "solana config set --url https://api.devnet.solana.com && solana epoch-info" "Devnet connectivity" "Network"; then
    :
fi

# Testnet compatibility
if check_command "solana config set --url https://api.testnet.solana.com && solana epoch-info" "Testnet connectivity" "Network"; then
    :
fi

# Mainnet compatibility (connection test only)
if check_command "solana epoch-info --url https://api.mainnet-beta.solana.com" "Mainnet connectivity" "Network"; then
    :
fi

# 10. OPERATIONAL READINESS
echo -e "\n${BLUE}10. OPERATIONAL READINESS${NC}"

# Admin tools
if [ -f "scripts/admin/capability_manager.sh" ]; then
    log_validation "PASS" "Operations" "Admin tools" "Admin tools available"
else
    log_validation "FAIL" "Operations" "Admin tools" "Admin tools not found"
fi

# Backup procedures
if [ -f "scripts/admin/capability_manager.sh" ] && grep -q "backup-config" "scripts/admin/capability_manager.sh"; then
    log_validation "PASS" "Operations" "Backup procedures" "Backup procedures implemented"
else
    log_validation "WARN" "Operations" "Backup procedures" "Backup procedures not found"
fi

# Emergency procedures
if [ -f "scripts/admin/capability_manager.sh" ] && grep -q "emergency-pause" "scripts/admin/capability_manager.sh"; then
    log_validation "PASS" "Operations" "Emergency procedures" "Emergency procedures implemented"
else
    log_validation "WARN" "Operations" "Emergency procedures" "Emergency procedures not found"
fi

# Health checks
if [ -f "scripts/monitoring/event_monitor.sh" ]; then
    log_validation "PASS" "Operations" "Health monitoring" "Health monitoring available"
else
    log_validation "FAIL" "Operations" "Health monitoring" "Health monitoring not found"
fi

# Generate final summary
echo -e "\n## Summary\n" >> "$CHECKLIST_FILE"
echo "- **Total Checks:** $TOTAL_CHECKS" >> "$CHECKLIST_FILE"
echo "- **Passed:** $PASSED_CHECKS" >> "$CHECKLIST_FILE"
echo "- **Failed:** $FAILED_CHECKS" >> "$CHECKLIST_FILE"
echo "- **Warnings:** $WARNING_CHECKS" >> "$CHECKLIST_FILE"
echo "" >> "$CHECKLIST_FILE"

# Calculate readiness score
READINESS_SCORE=$(( (PASSED_CHECKS * 100) / TOTAL_CHECKS ))

if [ "$FAILED_CHECKS" -eq 0 ] && [ "$READINESS_SCORE" -ge 90 ]; then
    echo "**Production Readiness:** ✅ READY FOR PRODUCTION" >> "$CHECKLIST_FILE"
    echo "**Readiness Score:** $READINESS_SCORE%" >> "$CHECKLIST_FILE"
    DEPLOYMENT_STATUS="READY"
elif [ "$FAILED_CHECKS" -le 2 ] && [ "$READINESS_SCORE" -ge 80 ]; then
    echo "**Production Readiness:** ⚠️ READY WITH MINOR ISSUES" >> "$CHECKLIST_FILE"
    echo "**Readiness Score:** $READINESS_SCORE%" >> "$CHECKLIST_FILE"
    DEPLOYMENT_STATUS="READY_WITH_ISSUES"
else
    echo "**Production Readiness:** ❌ NOT READY FOR PRODUCTION" >> "$CHECKLIST_FILE"
    echo "**Readiness Score:** $READINESS_SCORE%" >> "$CHECKLIST_FILE"
    DEPLOYMENT_STATUS="NOT_READY"
fi

echo "" >> "$CHECKLIST_FILE"
echo "## Next Steps" >> "$CHECKLIST_FILE"

if [ "$DEPLOYMENT_STATUS" = "READY" ]; then
    echo "1. Proceed with production deployment" >> "$CHECKLIST_FILE"
    echo "2. Monitor deployment closely" >> "$CHECKLIST_FILE"
    echo "3. Activate monitoring and alerting" >> "$CHECKLIST_FILE"
    echo "4. Notify stakeholders of successful deployment" >> "$CHECKLIST_FILE"
elif [ "$DEPLOYMENT_STATUS" = "READY_WITH_ISSUES" ]; then
    echo "1. Address minor issues identified" >> "$CHECKLIST_FILE"
    echo "2. Re-run validation after fixes" >> "$CHECKLIST_FILE"
    echo "3. Proceed with cautious deployment" >> "$CHECKLIST_FILE"
    echo "4. Monitor deployment extra closely" >> "$CHECKLIST_FILE"
else
    echo "1. **CRITICAL:** Address all failed checks" >> "$CHECKLIST_FILE"
    echo "2. Re-run full validation suite" >> "$CHECKLIST_FILE"
    echo "3. Do not proceed with production deployment" >> "$CHECKLIST_FILE"
    echo "4. Schedule follow-up validation" >> "$CHECKLIST_FILE"
fi

# Display final results
echo -e "\n${PURPLE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║                  VALIDATION COMPLETE                         ║${NC}"
echo -e "${PURPLE}╚═══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Final Results:${NC}"
echo -e "  Total Checks: $TOTAL_CHECKS"
echo -e "  Passed: ${GREEN}$PASSED_CHECKS${NC}"
echo -e "  Failed: ${RED}$FAILED_CHECKS${NC}"
echo -e "  Warnings: ${YELLOW}$WARNING_CHECKS${NC}"
echo -e "  Readiness Score: $READINESS_SCORE%"

case $DEPLOYMENT_STATUS in
    "READY")
        echo -e "\n${GREEN}🎉 PRODUCTION READY! 🎉${NC}"
        echo -e "${GREEN}System is ready for production deployment${NC}"
        ;;
    "READY_WITH_ISSUES")
        echo -e "\n${YELLOW}⚠️ READY WITH MINOR ISSUES${NC}"
        echo -e "${YELLOW}Address minor issues before deployment${NC}"
        ;;
    "NOT_READY")
        echo -e "\n${RED}❌ NOT READY FOR PRODUCTION${NC}"
        echo -e "${RED}Critical issues must be resolved before deployment${NC}"
        ;;
esac

echo -e "\n${BLUE}Detailed Report:${NC} $CHECKLIST_FILE"
echo -e "\n${BLUE}Next Steps:${NC}"

if [ "$DEPLOYMENT_STATUS" = "READY" ]; then
    echo -e "  1. Execute: ${GREEN}bash scripts/deployment/deploy.sh --network mainnet-beta${NC}"
    echo -e "  2. Monitor: ${GREEN}bash scripts/monitoring/event_monitor.sh --network mainnet-beta${NC}"
    echo -e "  3. Validate: ${GREEN}bash scripts/admin/capability_manager.sh validate-state --network mainnet-beta${NC}"
elif [ "$DEPLOYMENT_STATUS" = "READY_WITH_ISSUES" ]; then
    echo -e "  1. Review: ${YELLOW}Check failed and warning items in report${NC}"
    echo -e "  2. Fix: ${YELLOW}Address identified issues${NC}"
    echo -e "  3. Re-validate: ${YELLOW}$0${NC}"
else
    echo -e "  1. ${RED}CRITICAL: Fix all failed checks${NC}"
    echo -e "  2. ${RED}Re-run: $0${NC}"
    echo -e "  3. ${RED}DO NOT deploy until validation passes${NC}"
fi

# Exit with appropriate code
case $DEPLOYMENT_STATUS in
    "READY")
        exit 0
        ;;
    "READY_WITH_ISSUES")
        exit 1
        ;;
    "NOT_READY")
        exit 2
        ;;
esac 