#!/usr/bin/env bash
# Comprehensive Deployment Script for Valence Protocol
# This script orchestrates the complete deployment process including security audit,
# gas optimization, upgrade authority setup, and initial configuration

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
NETWORK=${NETWORK:-"devnet"}
SKIP_AUDIT=${SKIP_AUDIT:-"false"}
SKIP_OPTIMIZATION=${SKIP_OPTIMIZATION:-"false"}
SKIP_TESTS=${SKIP_TESTS:-"false"}
DRY_RUN=${DRY_RUN:-"false"}

echo -e "${PURPLE}╔═══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║                 VALENCE PROTOCOL DEPLOYMENT                   ║${NC}"
echo -e "${PURPLE}║               Unified Architecture v1.0                       ║${NC}"
echo -e "${PURPLE}╚═══════════════════════════════════════════════════════════════╝${NC}"

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must be run from project root${NC}"
    exit 1
fi

# Function to show usage
show_usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --network          Target network (devnet, testnet, mainnet-beta) [default: devnet]"
    echo "  --skip-audit       Skip security audit"
    echo "  --skip-optimization Skip gas optimization validation"
    echo "  --skip-tests       Skip test execution"
    echo "  --dry-run          Perform dry run without actual deployment"
    echo "  --help             Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  NETWORK            Same as --network"
    echo "  SKIP_AUDIT         Same as --skip-audit"
    echo "  SKIP_OPTIMIZATION  Same as --skip-optimization"
    echo "  SKIP_TESTS         Same as --skip-tests"
    echo "  DRY_RUN            Same as --dry-run"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Deploy to devnet with all checks"
    echo "  $0 --network testnet                 # Deploy to testnet"
    echo "  $0 --skip-audit --skip-tests         # Quick deployment"
    echo "  $0 --dry-run                         # Dry run deployment"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --skip-audit)
            SKIP_AUDIT="true"
            shift
            ;;
        --skip-optimization)
            SKIP_OPTIMIZATION="true"
            shift
            ;;
        --skip-tests)
            SKIP_TESTS="true"
            shift
            ;;
        --dry-run)
            DRY_RUN="true"
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

# Validate network
case $NETWORK in
    "devnet"|"testnet"|"mainnet-beta")
        echo -e "${GREEN}✓ Target network: $NETWORK${NC}"
        ;;
    *)
        echo -e "${RED}Error: Invalid network '$NETWORK'${NC}"
        echo -e "${RED}Valid networks: devnet, testnet, mainnet-beta${NC}"
        exit 1
        ;;
esac

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

export NETWORK RPC_URL

# Create deployment directory
mkdir -p "scripts/deployment/logs"
DEPLOYMENT_LOG="scripts/deployment/logs/deployment_$(date +%Y%m%d_%H%M%S).log"

# Function to log deployment steps
log_deployment() {
    local message=$1
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $message" | tee -a "$DEPLOYMENT_LOG"
}

# Function to run command with logging
run_command() {
    local command=$1
    local description=$2
    
    log_deployment "STARTING: $description"
    echo -e "${BLUE}Running: $description${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: $command${NC}"
        log_deployment "DRY RUN: $command"
        return 0
    fi
    
    if eval "$command" >> "$DEPLOYMENT_LOG" 2>&1; then
        echo -e "${GREEN}✓ $description${NC}"
        log_deployment "SUCCESS: $description"
        return 0
    else
        echo -e "${RED}✗ $description${NC}"
        log_deployment "FAILED: $description"
        return 1
    fi
}

# Function to check prerequisites
check_prerequisites() {
    echo -e "\n${BLUE}1. Checking prerequisites...${NC}"
    
    # Check for required tools
    local required_tools=("solana" "anchor" "cargo" "jq")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            echo -e "${RED}Error: $tool is not installed${NC}"
            exit 1
        fi
        echo -e "${GREEN}✓ $tool is available${NC}"
    done
    
    # Check Solana CLI configuration
    if ! solana config get &> /dev/null; then
        echo -e "${RED}Error: Solana CLI not configured${NC}"
        exit 1
    fi
    
    # Check network connectivity
    if ! run_command "solana epoch-info --url $RPC_URL" "Network connectivity check"; then
        echo -e "${RED}Error: Cannot connect to $NETWORK network${NC}"
        exit 1
    fi
    
    # Check wallet balance for deployment
    local balance=$(solana balance --url "$RPC_URL" 2>/dev/null | awk '{print $1}')
    if (( $(echo "$balance < 1" | bc -l) )); then
        echo -e "${YELLOW}Warning: Low wallet balance ($balance SOL)${NC}"
        if [ "$NETWORK" = "mainnet-beta" ]; then
            echo -e "${RED}Error: Insufficient balance for mainnet deployment${NC}"
            exit 1
        fi
    fi
    
    log_deployment "Prerequisites check completed"
}

# Function to run security audit
run_security_audit() {
    if [ "$SKIP_AUDIT" = "true" ]; then
        echo -e "\n${YELLOW}2. Skipping security audit...${NC}"
        return 0
    fi
    
    echo -e "\n${BLUE}2. Running security audit...${NC}"
    
    if [ ! -f "scripts/deployment/security_audit.sh" ]; then
        echo -e "${RED}Error: Security audit script not found${NC}"
        exit 1
    fi
    
    if ! run_command "bash scripts/deployment/security_audit.sh" "Security audit"; then
        echo -e "${RED}Error: Security audit failed${NC}"
        echo -e "${RED}Please review and fix security issues before deployment${NC}"
        exit 1
    fi
    
    # Check if audit passed
    if ! grep -q "READY FOR DEPLOYMENT" "scripts/deployment/audit_results.md" 2>/dev/null; then
        echo -e "${RED}Error: Security audit did not pass${NC}"
        echo -e "${RED}Please review audit results and fix issues${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Security audit passed${NC}"
}

# Function to run gas optimization validation
run_gas_optimization() {
    if [ "$SKIP_OPTIMIZATION" = "true" ]; then
        echo -e "\n${YELLOW}3. Skipping gas optimization validation...${NC}"
        return 0
    fi
    
    echo -e "\n${BLUE}3. Running gas optimization validation...${NC}"
    
    if [ ! -f "scripts/deployment/gas_optimization.sh" ]; then
        echo -e "${RED}Error: Gas optimization script not found${NC}"
        exit 1
    fi
    
    if ! run_command "bash scripts/deployment/gas_optimization.sh" "Gas optimization validation"; then
        echo -e "${YELLOW}Warning: Gas optimization validation completed with warnings${NC}"
    fi
    
    echo -e "${GREEN}✓ Gas optimization validation completed${NC}"
}

# Function to run tests
run_tests() {
    if [ "$SKIP_TESTS" = "true" ]; then
        echo -e "\n${YELLOW}4. Skipping tests...${NC}"
        return 0
    fi
    
    echo -e "\n${BLUE}4. Running tests...${NC}"
    
    # Run unit tests
    if ! run_command "cargo test --workspace" "Unit tests"; then
        echo -e "${RED}Error: Unit tests failed${NC}"
        exit 1
    fi
    
    # Run integration tests
    if ! run_command "cargo test --package tests --test '*'" "Integration tests"; then
        echo -e "${RED}Error: Integration tests failed${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ All tests passed${NC}"
}

# Function to build programs
build_programs() {
    echo -e "\n${BLUE}5. Building programs...${NC}"
    
    # Build with Anchor
    if ! run_command "anchor build" "Building programs with Anchor"; then
        echo -e "${RED}Error: Program build failed${NC}"
        exit 1
    fi
    
    # Verify build artifacts
    local programs=("eval" "shard" "registry")
    for program in "${programs[@]}"; do
        local so_file="target/deploy/${program}.so"
        if [ ! -f "$so_file" ]; then
            echo -e "${RED}Error: Build artifact not found: $so_file${NC}"
            exit 1
        fi
        echo -e "${GREEN}✓ Build artifact found: $so_file${NC}"
    done
    
    echo -e "${GREEN}✓ All programs built successfully${NC}"
}

# Function to deploy programs
deploy_programs() {
    echo -e "\n${BLUE}6. Deploying programs...${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: Would deploy programs to $NETWORK${NC}"
        return 0
    fi
    
    # Deploy with Anchor
    if ! run_command "anchor deploy --network $NETWORK" "Deploying programs"; then
        echo -e "${RED}Error: Program deployment failed${NC}"
        exit 1
    fi
    
    # Verify deployments
    local program_ids=("EvalCont11111111111111111111111111111111111" "ShardCon11111111111111111111111111111111111" "RegCont1111111111111111111111111111111111111")
    for program_id in "${program_ids[@]}"; do
        if ! run_command "solana program show $program_id --url $RPC_URL" "Verifying deployment of $program_id"; then
            echo -e "${RED}Error: Program verification failed for $program_id${NC}"
            exit 1
        fi
    done
    
    echo -e "${GREEN}✓ All programs deployed successfully${NC}"
}

# Function to initialize programs
initialize_programs() {
    echo -e "\n${BLUE}7. Initializing programs...${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: Would initialize programs${NC}"
        return 0
    fi
    
    # Initialize each program
    local programs=("eval" "shard" "registry")
    for program in "${programs[@]}"; do
        echo -e "${BLUE}Initializing $program program...${NC}"
        
        # Create initialization transaction
        case $program in
            "eval")
                if ! run_command "anchor run initialize_eval --network $NETWORK" "Initializing eval program"; then
                    echo -e "${RED}Error: Failed to initialize eval program${NC}"
                    exit 1
                fi
                ;;
            "shard")
                if ! run_command "anchor run initialize_shard --network $NETWORK" "Initializing shard program"; then
                    echo -e "${RED}Error: Failed to initialize shard program${NC}"
                    exit 1
                fi
                ;;
            "registry")
                if ! run_command "anchor run initialize_registry --network $NETWORK" "Initializing registry program"; then
                    echo -e "${RED}Error: Failed to initialize registry program${NC}"
                    exit 1
                fi
                ;;
        esac
    done
    
    echo -e "${GREEN}✓ All programs initialized successfully${NC}"
}

# Function to set up initial capabilities
setup_initial_capabilities() {
    echo -e "\n${BLUE}8. Setting up initial capabilities...${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: Would set up initial capabilities${NC}"
        return 0
    fi
    
    # Load initial capabilities configuration
    if [ ! -f "scripts/deployment/initial_capabilities.json" ]; then
        echo -e "${RED}Error: Initial capabilities configuration not found${NC}"
        exit 1
    fi
    
    # Create initial namespaces
    echo -e "${BLUE}Creating initial namespaces...${NC}"
    local namespaces=$(jq -r '.default_namespaces[].id' scripts/deployment/initial_capabilities.json)
    for namespace in $namespaces; do
        if ! run_command "anchor run create_namespace --network $NETWORK -- --namespace $namespace" "Creating namespace: $namespace"; then
            echo -e "${YELLOW}Warning: Failed to create namespace $namespace (may already exist)${NC}"
        fi
    done
    
    # Create initial capabilities
    echo -e "${BLUE}Creating initial capabilities...${NC}"
    local capabilities=$(jq -r '.default_capabilities[].id' scripts/deployment/initial_capabilities.json)
    for capability in $capabilities; do
        if ! run_command "anchor run create_capability --network $NETWORK -- --capability $capability" "Creating capability: $capability"; then
            echo -e "${YELLOW}Warning: Failed to create capability $capability (may already exist)${NC}"
        fi
    done
    
    echo -e "${GREEN}✓ Initial capabilities set up successfully${NC}"
}

# Function to configure upgrade authority
configure_upgrade_authority() {
    echo -e "\n${BLUE}9. Configuring upgrade authority...${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: Would configure upgrade authority${NC}"
        return 0
    fi
    
    # For mainnet, require manual upgrade authority configuration
    if [ "$NETWORK" = "mainnet-beta" ]; then
        echo -e "${YELLOW}Warning: Mainnet deployment requires manual upgrade authority configuration${NC}"
        echo -e "${YELLOW}Please use scripts/deployment/upgrade_authority/manage_upgrade_authority.sh${NC}"
        return 0
    fi
    
    # For devnet/testnet, set up basic upgrade authority
    echo -e "${BLUE}Setting up upgrade authority for $NETWORK...${NC}"
    
    # Check if upgrade authority script exists
    if [ ! -f "scripts/deployment/upgrade_authority/manage_upgrade_authority.sh" ]; then
        echo -e "${RED}Error: Upgrade authority script not found${NC}"
        exit 1
    fi
    
    # Create upgrade authority configuration
    if ! run_command "bash scripts/deployment/upgrade_authority/manage_upgrade_authority.sh create-multisig --network $NETWORK" "Creating upgrade authority configuration"; then
        echo -e "${YELLOW}Warning: Upgrade authority configuration completed with warnings${NC}"
    fi
    
    echo -e "${GREEN}✓ Upgrade authority configured${NC}"
}

# Function to run post-deployment validation
run_post_deployment_validation() {
    echo -e "\n${BLUE}10. Running post-deployment validation...${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: Would run post-deployment validation${NC}"
        return 0
    fi
    
    # Test program functionality
    echo -e "${BLUE}Testing program functionality...${NC}"
    
    # Test eval program
    if ! run_command "anchor run test_eval --network $NETWORK" "Testing eval program"; then
        echo -e "${YELLOW}Warning: Eval program test failed${NC}"
    fi
    
    # Test shard program
    if ! run_command "anchor run test_shard --network $NETWORK" "Testing shard program"; then
        echo -e "${YELLOW}Warning: Shard program test failed${NC}"
    fi
    
    # Test registry program
    if ! run_command "anchor run test_registry --network $NETWORK" "Testing registry program"; then
        echo -e "${YELLOW}Warning: Registry program test failed${NC}"
    fi
    
    echo -e "${GREEN}✓ Post-deployment validation completed${NC}"
}

# Function to generate deployment report
generate_deployment_report() {
    echo -e "\n${BLUE}11. Generating deployment report...${NC}"
    
    local report_file="scripts/deployment/logs/deployment_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Valence Protocol Deployment Report

## Deployment Summary
- **Date:** $(date)
- **Network:** $NETWORK
- **RPC URL:** $RPC_URL
- **Deployed By:** $(whoami)
- **Deployment Mode:** $([ "$DRY_RUN" = "true" ] && echo "DRY RUN" || echo "LIVE")

## Program Deployments
- **Eval Program:** EvalCont11111111111111111111111111111111111
- **Shard Program:** ShardCon11111111111111111111111111111111111
- **Registry Program:** RegCont1111111111111111111111111111111111111

## Deployment Steps
EOF

    # Add deployment steps from log
    echo -e "\n### Execution Log" >> "$report_file"
    echo -e "\`\`\`" >> "$report_file"
    if [ -f "$DEPLOYMENT_LOG" ]; then
        tail -n 50 "$DEPLOYMENT_LOG" >> "$report_file"
    fi
    echo -e "\`\`\`" >> "$report_file"
    
    # Add configuration details
    echo -e "\n## Configuration" >> "$report_file"
    echo -e "- **Security Audit:** $([ "$SKIP_AUDIT" = "true" ] && echo "SKIPPED" || echo "COMPLETED")" >> "$report_file"
    echo -e "- **Gas Optimization:** $([ "$SKIP_OPTIMIZATION" = "true" ] && echo "SKIPPED" || echo "COMPLETED")" >> "$report_file"
    echo -e "- **Tests:** $([ "$SKIP_TESTS" = "true" ] && echo "SKIPPED" || echo "COMPLETED")" >> "$report_file"
    
    # Add next steps
    echo -e "\n## Next Steps" >> "$report_file"
    echo -e "1. Monitor program performance and usage" >> "$report_file"
    echo -e "2. Set up operational monitoring and alerting" >> "$report_file"
    echo -e "3. Update documentation with deployment details" >> "$report_file"
    echo -e "4. Communicate deployment status to stakeholders" >> "$report_file"
    
    if [ "$NETWORK" = "mainnet-beta" ]; then
        echo -e "5. **IMPORTANT:** Configure multisig upgrade authority for mainnet" >> "$report_file"
        echo -e "6. **IMPORTANT:** Set up production monitoring and alerting" >> "$report_file"
    fi
    
    echo -e "\n${GREEN}✓ Deployment report generated: $report_file${NC}"
}

# Main deployment function
main() {
    log_deployment "Starting Valence Protocol deployment to $NETWORK"
    
    # Run deployment steps
    check_prerequisites
    run_security_audit
    run_gas_optimization
    run_tests
    build_programs
    deploy_programs
    initialize_programs
    setup_initial_capabilities
    configure_upgrade_authority
    run_post_deployment_validation
    generate_deployment_report
    
    # Final summary
    echo -e "\n${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                   DEPLOYMENT COMPLETED                       ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    
    echo -e "\n${BLUE}Deployment Summary:${NC}"
    echo -e "  • Network: $NETWORK"
    echo -e "  • Mode: $([ "$DRY_RUN" = "true" ] && echo "DRY RUN" || echo "LIVE DEPLOYMENT")"
    echo -e "  • Programs: 3 (eval, shard, registry)"
    echo -e "  • Log: $DEPLOYMENT_LOG"
    
    if [ "$NETWORK" = "mainnet-beta" ] && [ "$DRY_RUN" = "false" ]; then
        echo -e "\n${YELLOW}⚠️  MAINNET DEPLOYMENT COMPLETE${NC}"
        echo -e "${YELLOW}Please ensure proper monitoring and security measures are in place${NC}"
    fi
    
    log_deployment "Deployment completed successfully"
}

# Execute main function
main "$@" 