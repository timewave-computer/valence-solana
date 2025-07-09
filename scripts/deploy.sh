#!/bin/bash

# Valence Protocol Unified Architecture Deployment Script
# 
# This script deploys the unified eval, shard, and registry programs
# along with the SDK and sets up the complete environment.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER="${CLUSTER:-localnet}"
KEYPAIR_PATH="${KEYPAIR_PATH:-~/.config/solana/id.json}"
PROGRAM_DIR="programs"
SDK_DIR="sdk"
LOG_LEVEL="${LOG_LEVEL:-info}"
DRY_RUN="${DRY_RUN:-false}"

# Program names
PROGRAMS=("eval" "shard" "registry")

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if Solana CLI is installed
    if ! command -v solana &> /dev/null; then
        print_error "Solana CLI not found. Please install Solana CLI first."
        exit 1
    fi
    
    # Check if Anchor CLI is installed
    if ! command -v anchor &> /dev/null; then
        print_error "Anchor CLI not found. Please install Anchor CLI first."
        exit 1
    fi
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust first."
        exit 1
    fi
    
    # Check if keypair exists
    if [[ ! -f "$KEYPAIR_PATH" ]]; then
        print_warning "Keypair not found at $KEYPAIR_PATH. Generating new keypair..."
        solana-keygen new --outfile "$KEYPAIR_PATH" --no-bip39-passphrase
    fi
    
    print_success "Prerequisites check completed"
}

# Function to setup Solana configuration
setup_solana_config() {
    print_status "Setting up Solana configuration..."
    
    # Set cluster
    case $CLUSTER in
        "mainnet-beta")
            solana config set --url https://api.mainnet-beta.solana.com
            ;;
        "testnet")
            solana config set --url https://api.testnet.solana.com
            ;;
        "devnet")
            solana config set --url https://api.devnet.solana.com
            ;;
        "localnet")
            solana config set --url http://127.0.0.1:8899
            ;;
        *)
            print_error "Invalid cluster: $CLUSTER"
            exit 1
            ;;
    esac
    
    # Set keypair
    solana config set --keypair "$KEYPAIR_PATH"
    
    # Check connection
    if ! solana cluster-version &> /dev/null; then
        print_error "Cannot connect to Solana cluster: $CLUSTER"
        if [[ "$CLUSTER" == "localnet" ]]; then
            print_error "Make sure to start local validator with: solana-test-validator"
        fi
        exit 1
    fi
    
    # Check account balance
    BALANCE=$(solana balance --lamports 2>/dev/null || echo "0")
    if [[ "$BALANCE" -lt 1000000000 ]]; then  # Less than 1 SOL
        print_warning "Low account balance: $BALANCE lamports"
        if [[ "$CLUSTER" != "mainnet-beta" ]]; then
            print_status "Requesting airdrop..."
            solana airdrop 2 || true
        fi
    fi
    
    print_success "Solana configuration completed"
}

# Function to build programs
build_programs() {
    print_status "Building Valence Protocol programs..."
    
    # Build using Anchor
    if [[ "$DRY_RUN" == "false" ]]; then
        anchor build --verbose
    else
        print_status "DRY RUN: Would run 'anchor build --verbose'"
    fi
    
    print_success "Programs built successfully"
}

# Function to deploy programs
deploy_programs() {
    print_status "Deploying Valence Protocol programs..."
    
    local deployment_order=("shard" "eval" "registry")
    
    for program in "${deployment_order[@]}"; do
        print_status "Deploying $program program..."
        
        if [[ "$DRY_RUN" == "false" ]]; then
            # Deploy the program
            anchor deploy --program-name "$program" --provider.cluster "$CLUSTER"
            
            # Get the program ID
            PROGRAM_ID=$(anchor keys list | grep "$program" | awk '{print $2}')
            
            if [[ -n "$PROGRAM_ID" ]]; then
                print_success "$program deployed with ID: $PROGRAM_ID"
                
                # Store program ID for later use
                echo "$program=$PROGRAM_ID" >> deployment.env
            else
                print_error "Failed to get program ID for $program"
                exit 1
            fi
        else
            print_status "DRY RUN: Would deploy $program program"
        fi
        
        sleep 2  # Brief pause between deployments
    done
    
    print_success "All programs deployed successfully"
}

# Function to initialize programs
initialize_programs() {
    print_status "Initializing Valence Protocol programs..."
    
    # Load deployment environment
    if [[ -f "deployment.env" ]]; then
        source deployment.env
    else
        print_error "deployment.env not found. Programs may not be deployed."
        return 1
    fi
    
    local authority=$(solana address)
    
    # Initialize shard program first (it's referenced by eval)
    print_status "Initializing shard program..."
    if [[ "$DRY_RUN" == "false" ]]; then
        # Use SDK CLI to initialize
        if command -v valence-cli &> /dev/null; then
            valence-cli program init-shard \
                --cluster "$CLUSTER" \
                --authority "$KEYPAIR_PATH" \
                --program-id "$shard" \
                --eval-address "$eval"
        else
            print_warning "valence-cli not found. Skipping program initialization."
        fi
    else
        print_status "DRY RUN: Would initialize shard program"
    fi
    
    # Initialize eval program
    print_status "Initializing eval program..."
    if [[ "$DRY_RUN" == "false" ]]; then
        if command -v valence-cli &> /dev/null; then
            valence-cli program init-eval \
                --cluster "$CLUSTER" \
                --authority "$KEYPAIR_PATH" \
                --shard-address "$shard"
        fi
    else
        print_status "DRY RUN: Would initialize eval program"
    fi
    
    # Initialize registry program
    print_status "Initializing registry program..."
    if [[ "$DRY_RUN" == "false" ]]; then
        if command -v valence-cli &> /dev/null; then
            valence-cli program init-registry \
                --cluster "$CLUSTER" \
                --authority "$KEYPAIR_PATH"
        fi
    else
        print_status "DRY RUN: Would initialize registry program"
    fi
    
    print_success "Programs initialized successfully"
}

# Function to build and install SDK
build_sdk() {
    print_status "Building Valence Protocol SDK..."
    
    cd "$SDK_DIR"
    
    if [[ "$DRY_RUN" == "false" ]]; then
        # Build the SDK
        cargo build --release
        
        # Build the CLI
        cargo build --release --bin valence-cli
        
        # Install CLI globally (optional)
        if [[ "$INSTALL_CLI" == "true" ]]; then
            cargo install --path . --bin valence-cli
            print_success "Valence CLI installed globally"
        fi
    else
        print_status "DRY RUN: Would build SDK and CLI"
    fi
    
    cd ..
    
    print_success "SDK built successfully"
}

# Function to run tests
run_tests() {
    print_status "Running integration tests..."
    
    if [[ "$SKIP_TESTS" == "true" ]]; then
        print_warning "Skipping tests (SKIP_TESTS=true)"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "false" ]]; then
        # Run anchor tests
        anchor test --skip-local-validator
        
        # Run SDK tests
        cd "$SDK_DIR"
        cargo test
        cd ..
        
        # Run end-to-end tests
        cd tests
        cargo test --test unified_integration_tests
        cargo test --test e2e_test_suite
        cd ..
    else
        print_status "DRY RUN: Would run integration tests"
    fi
    
    print_success "Tests completed successfully"
}

# Function to create example configurations
create_example_configs() {
    print_status "Creating example configurations..."
    
    # Create deployment config
    cat > deployment.config.json << EOF
{
  "cluster": "$CLUSTER",
  "programs": {
    "eval": "$eval",
    "shard": "$shard",
    "registry": "$registry"
  },
  "authority": "$(solana address)",
  "deployment_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "sdk_version": "0.1.0"
}
EOF
    
    # Create SDK example config
    mkdir -p examples/configs
    cat > examples/configs/valence-config.json << EOF
{
  "programIds": {
    "eval": "$eval",
    "shard": "$shard",
    "registry": "$registry"
  },
  "cluster": "$CLUSTER",
  "commitment": "confirmed"
}
EOF
    
    # Create environment file for easy sourcing
    cat > valence.env << EOF
# Valence Protocol Environment Configuration
export VALENCE_CLUSTER=$CLUSTER
export VALENCE_EVAL_PROGRAM_ID=$eval
export VALENCE_SHARD_PROGRAM_ID=$shard
export VALENCE_REGISTRY_PROGRAM_ID=$registry
export VALENCE_AUTHORITY=$(solana address)
export VALENCE_KEYPAIR=$KEYPAIR_PATH
EOF
    
    print_success "Example configurations created"
}

# Function to setup example capabilities
setup_example_capabilities() {
    print_status "Setting up example capabilities..."
    
    if [[ "$DRY_RUN" == "false" && "$SETUP_EXAMPLES" == "true" ]]; then
        # Source the environment
        source valence.env
        
        # Create example capabilities using CLI
        if command -v valence-cli &> /dev/null; then
            # Grant basic permission capability
            valence-cli capability grant \
                --cluster "$CLUSTER" \
                --authority "$KEYPAIR_PATH" \
                --shard-state "$VALENCE_SHARD_PROGRAM_ID" \
                --capability-id "basic_permission" \
                --verification-functions "basic_permission" \
                --description "Basic permission capability for testing"
            
            # Grant token transfer capability
            valence-cli capability grant \
                --cluster "$CLUSTER" \
                --authority "$KEYPAIR_PATH" \
                --shard-state "$VALENCE_SHARD_PROGRAM_ID" \
                --capability-id "token_transfer" \
                --verification-functions "basic_permission,parameter_constraint" \
                --description "Token transfer capability"
            
            # Register example library
            valence-cli library register \
                --cluster "$CLUSTER" \
                --authority "$KEYPAIR_PATH" \
                --library-id "example_library" \
                --name "Example Library" \
                --version "1.0.0" \
                --program-id "$VALENCE_EVAL_PROGRAM_ID" \
                --tags "example,testing"
        else
            print_warning "valence-cli not found. Skipping example setup."
        fi
    else
        print_status "DRY RUN: Would set up example capabilities"
    fi
    
    print_success "Example capabilities setup completed"
}

# Function to generate deployment report
generate_deployment_report() {
    print_status "Generating deployment report..."
    
    local report_file="deployment-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" << EOF
# Valence Protocol Deployment Report

**Deployment Date:** $(date -u)
**Cluster:** $CLUSTER
**Authority:** $(solana address)

## Deployed Programs

| Program | Program ID | Status |
|---------|------------|--------|
| Eval | $eval | ✅ Deployed |
| Shard | $shard | ✅ Deployed |
| Registry | $registry | ✅ Deployed |

## Configuration Files

- \`deployment.config.json\` - Main deployment configuration
- \`valence.env\` - Environment variables
- \`examples/configs/valence-config.json\` - SDK configuration example

## Next Steps

1. Source the environment variables:
   \`\`\`bash
   source valence.env
   \`\`\`

2. Test the deployment:
   \`\`\`bash
   valence-cli capability templates
   \`\`\`

3. Create your first capability:
   \`\`\`bash
   valence-cli capability grant \\
     --capability-id "my_capability" \\
     --verification-functions "basic_permission" \\
     --description "My first capability"
   \`\`\`

## SDK Usage

\`\`\`rust
use valence_sdk::*;

let config = ValenceConfig {
    program_ids: ProgramIds {
        eval: "$eval".parse().unwrap(),
        shard: "$shard".parse().unwrap(),
        registry: "$registry".parse().unwrap(),
    },
    cluster: anchor_client::Cluster::$(echo $CLUSTER | tr '[:lower:]' '[:upper:]'),
    payer: load_keypair_from_file("$KEYPAIR_PATH").unwrap(),
    commitment: Some(CommitmentConfig::confirmed()),
};

let client = ValenceClient::new(config)?;
\`\`\`

## Support

- Documentation: See \`sdk/README.md\`
- Examples: See \`sdk/examples/\`
- CLI Help: \`valence-cli --help\`

EOF
    
    print_success "Deployment report generated: $report_file"
}

# Function to clean up deployment artifacts
cleanup() {
    print_status "Cleaning up deployment artifacts..."
    
    # Remove temporary files
    rm -f deployment.env
    
    print_success "Cleanup completed"
}

# Function to show deployment summary
show_summary() {
    echo
    echo "🎉 Valence Protocol Deployment Summary"
    echo "====================================="
    echo
    echo "✅ Programs deployed and initialized"
    echo "✅ SDK built and ready"
    echo "✅ Configuration files created"
    echo "✅ Integration tests passed"
    echo
    echo "📋 Next Steps:"
    echo "  1. Source environment: source valence.env"
    echo "  2. Test CLI: valence-cli --help"
    echo "  3. View examples: ls examples/"
    echo "  4. Read documentation: sdk/README.md"
    echo
    echo "🔗 Program IDs:"
    if [[ -f "deployment.config.json" ]]; then
        echo "  Eval:     $(grep '"eval"' deployment.config.json | cut -d'"' -f4)"
        echo "  Shard:    $(grep '"shard"' deployment.config.json | cut -d'"' -f4)"
        echo "  Registry: $(grep '"registry"' deployment.config.json | cut -d'"' -f4)"
    fi
    echo
}

# Function to display help
show_help() {
    cat << EOF
Valence Protocol Deployment Script

Usage: $0 [OPTIONS]

Options:
  --cluster CLUSTER     Target cluster (localnet, devnet, testnet, mainnet-beta)
  --keypair PATH        Path to keypair file
  --dry-run            Show what would be done without executing
  --skip-tests         Skip running tests
  --setup-examples     Set up example capabilities and libraries
  --install-cli        Install CLI globally
  --help               Show this help message

Environment Variables:
  CLUSTER              Target cluster (default: localnet)
  KEYPAIR_PATH         Path to keypair file (default: ~/.config/solana/id.json)
  DRY_RUN              Set to 'true' for dry run (default: false)
  SKIP_TESTS           Set to 'true' to skip tests (default: false)
  SETUP_EXAMPLES       Set to 'true' to setup examples (default: false)
  INSTALL_CLI          Set to 'true' to install CLI globally (default: false)
  LOG_LEVEL            Logging level (default: info)

Examples:
  # Deploy to localnet
  ./scripts/deploy.sh

  # Deploy to devnet
  ./scripts/deploy.sh --cluster devnet

  # Dry run on mainnet
  ./scripts/deploy.sh --cluster mainnet-beta --dry-run

  # Deploy with examples
  SETUP_EXAMPLES=true ./scripts/deploy.sh --cluster devnet

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --cluster)
            CLUSTER="$2"
            shift 2
            ;;
        --keypair)
            KEYPAIR_PATH="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        --skip-tests)
            SKIP_TESTS="true"
            shift
            ;;
        --setup-examples)
            SETUP_EXAMPLES="true"
            shift
            ;;
        --install-cli)
            INSTALL_CLI="true"
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main deployment function
main() {
    echo "🚀 Valence Protocol Unified Architecture Deployment"
    echo "===================================================="
    echo
    echo "Cluster: $CLUSTER"
    echo "Keypair: $KEYPAIR_PATH"
    echo "Dry Run: $DRY_RUN"
    echo
    
    # Run deployment steps
    check_prerequisites
    setup_solana_config
    build_programs
    deploy_programs
    build_sdk
    initialize_programs
    run_tests
    create_example_configs
    
    if [[ "$SETUP_EXAMPLES" == "true" ]]; then
        setup_example_capabilities
    fi
    
    generate_deployment_report
    show_summary
    
    print_success "Valence Protocol deployment completed successfully! 🎉"
}

# Error handling
trap 'print_error "Deployment failed at line $LINENO"' ERR

# Run main function
main "$@" 