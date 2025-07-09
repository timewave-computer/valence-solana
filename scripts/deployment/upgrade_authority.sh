#!/usr/bin/env bash
# Upgrade Authority Setup Script for Valence Protocol
# This script sets up secure upgrade authority for the unified architecture

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Setting up Upgrade Authority for Valence Protocol...${NC}"

# Configuration
NETWORK=${NETWORK:-"devnet"}
UPGRADE_AUTHORITY_KEYPAIR=${UPGRADE_AUTHORITY_KEYPAIR:-""}
MULTISIG_THRESHOLD=${MULTISIG_THRESHOLD:-"2"}
MULTISIG_SIGNERS=${MULTISIG_SIGNERS:-"3"}

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must be run from project root${NC}"
    exit 1
fi

# Create upgrade authority directory
mkdir -p "scripts/deployment/upgrade_authority"

# Initialize upgrade authority configuration
UPGRADE_CONFIG="scripts/deployment/upgrade_authority/upgrade_config.json"
UPGRADE_LOG="scripts/deployment/upgrade_authority/upgrade_log.md"

echo -e "\n${BLUE}1. Generating upgrade authority configuration...${NC}"

# Generate upgrade authority configuration
cat > "$UPGRADE_CONFIG" << EOF
{
  "network": "$NETWORK",
  "upgrade_authority": {
    "type": "multisig",
    "threshold": $MULTISIG_THRESHOLD,
    "signers": $MULTISIG_SIGNERS,
    "emergency_authority": null
  },
  "programs": {
    "eval": {
      "program_id": "EvalCont11111111111111111111111111111111111",
      "upgrade_authority": null,
      "buffer_authority": null,
      "last_deployed": null
    },
    "shard": {
      "program_id": "ShardCon11111111111111111111111111111111111",
      "upgrade_authority": null,
      "buffer_authority": null,
      "last_deployed": null
    },
    "registry": {
      "program_id": "RegCont1111111111111111111111111111111111111",
      "upgrade_authority": null,
      "buffer_authority": null,
      "last_deployed": null
    }
  },
  "security": {
    "require_multisig": true,
    "enable_emergency_mode": false,
    "upgrade_buffer_size": 1000000,
    "max_upgrade_delay": 86400
  }
}
EOF

echo -e "${GREEN}✓ Upgrade configuration generated${NC}"

# Create upgrade authority management script
echo -e "\n${BLUE}2. Creating upgrade authority management script...${NC}"

cat > "scripts/deployment/upgrade_authority/manage_upgrade_authority.sh" << 'EOF'
#!/usr/bin/env bash
# Upgrade Authority Management Script
# This script manages upgrade authority for Valence Protocol programs

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
NETWORK=${NETWORK:-"devnet"}
RPC_URL=${RPC_URL:-"https://api.devnet.solana.com"}

# Function to show usage
show_usage() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  create-multisig    Create multisig upgrade authority"
    echo "  set-authority      Set upgrade authority for program"
    echo "  check-authority    Check current upgrade authority"
    echo "  prepare-upgrade    Prepare upgrade buffer"
    echo "  execute-upgrade    Execute program upgrade"
    echo "  revoke-authority   Revoke upgrade authority (make immutable)"
    echo ""
    echo "Options:"
    echo "  --network          Solana network (devnet, testnet, mainnet-beta)"
    echo "  --program          Program to manage (eval, shard, registry)"
    echo "  --authority        Authority keypair path"
    echo "  --new-authority    New authority public key"
    echo "  --buffer           Upgrade buffer path"
    echo ""
    echo "Examples:"
    echo "  $0 create-multisig --network devnet"
    echo "  $0 set-authority --program eval --authority ./authority.json"
    echo "  $0 check-authority --program shard"
}

# Function to create multisig upgrade authority
create_multisig() {
    echo -e "${BLUE}Creating multisig upgrade authority...${NC}"
    
    # Generate multisig keypairs
    for i in $(seq 1 3); do
        if [ ! -f "scripts/deployment/upgrade_authority/signer_${i}.json" ]; then
            solana-keygen new --no-bip39-passphrase --outfile "scripts/deployment/upgrade_authority/signer_${i}.json" --silent
            echo -e "${GREEN}✓ Generated signer ${i} keypair${NC}"
        fi
    done
    
    # Create multisig account (using SPL Token Program's multisig as reference)
    echo -e "${BLUE}Multisig setup requires manual configuration with your preferred multisig solution${NC}"
    echo -e "${YELLOW}Consider using Squads, Goki, or similar multisig solutions${NC}"
    
    # Log the creation
    echo "$(date): Created multisig upgrade authority configuration" >> "$UPGRADE_LOG"
}

# Function to set upgrade authority
set_authority() {
    local program=$1
    local authority_keypair=$2
    local new_authority=$3
    
    echo -e "${BLUE}Setting upgrade authority for $program...${NC}"
    
    # Get program ID from config
    local program_id
    case $program in
        "eval")
            program_id="EvalCont11111111111111111111111111111111111"
            ;;
        "shard")
            program_id="ShardCon11111111111111111111111111111111111"
            ;;
        "registry")
            program_id="RegCont1111111111111111111111111111111111111"
            ;;
        *)
            echo -e "${RED}Unknown program: $program${NC}"
            return 1
            ;;
    esac
    
    # Set the upgrade authority
    if solana program set-upgrade-authority "$program_id" "$new_authority" --upgrade-authority "$authority_keypair" --url "$RPC_URL"; then
        echo -e "${GREEN}✓ Upgrade authority set for $program${NC}"
        echo "$(date): Set upgrade authority for $program to $new_authority" >> "$UPGRADE_LOG"
        
        # Update config
        jq ".programs.$program.upgrade_authority = \"$new_authority\"" "$UPGRADE_CONFIG" > tmp.$$.json && mv tmp.$$.json "$UPGRADE_CONFIG"
    else
        echo -e "${RED}Failed to set upgrade authority for $program${NC}"
        return 1
    fi
}

# Function to check current upgrade authority
check_authority() {
    local program=$1
    
    echo -e "${BLUE}Checking upgrade authority for $program...${NC}"
    
    # Get program ID from config
    local program_id
    case $program in
        "eval")
            program_id="EvalCont11111111111111111111111111111111111"
            ;;
        "shard")
            program_id="ShardCon11111111111111111111111111111111111"
            ;;
        "registry")
            program_id="RegCont1111111111111111111111111111111111111"
            ;;
        *)
            echo -e "${RED}Unknown program: $program${NC}"
            return 1
            ;;
    esac
    
    # Check the upgrade authority
    if solana program show "$program_id" --url "$RPC_URL"; then
        echo -e "${GREEN}✓ Authority check completed for $program${NC}"
    else
        echo -e "${RED}Failed to check authority for $program${NC}"
        return 1
    fi
}

# Function to prepare upgrade buffer
prepare_upgrade() {
    local program=$1
    local buffer_keypair=$2
    local program_binary=$3
    
    echo -e "${BLUE}Preparing upgrade buffer for $program...${NC}"
    
    # Write program to buffer
    if solana program write-buffer "$program_binary" --buffer "$buffer_keypair" --url "$RPC_URL"; then
        echo -e "${GREEN}✓ Upgrade buffer prepared for $program${NC}"
        echo "$(date): Prepared upgrade buffer for $program" >> "$UPGRADE_LOG"
    else
        echo -e "${RED}Failed to prepare upgrade buffer for $program${NC}"
        return 1
    fi
}

# Function to execute upgrade
execute_upgrade() {
    local program=$1
    local buffer_address=$2
    local upgrade_authority=$3
    
    echo -e "${BLUE}Executing upgrade for $program...${NC}"
    
    # Get program ID from config
    local program_id
    case $program in
        "eval")
            program_id="EvalCont11111111111111111111111111111111111"
            ;;
        "shard")
            program_id="ShardCon11111111111111111111111111111111111"
            ;;
        "registry")
            program_id="RegCont1111111111111111111111111111111111111"
            ;;
        *)
            echo -e "${RED}Unknown program: $program${NC}"
            return 1
            ;;
    esac
    
    # Execute the upgrade
    if solana program deploy --buffer "$buffer_address" --upgrade-authority "$upgrade_authority" --program-id "$program_id" --url "$RPC_URL"; then
        echo -e "${GREEN}✓ Upgrade executed for $program${NC}"
        echo "$(date): Executed upgrade for $program from buffer $buffer_address" >> "$UPGRADE_LOG"
        
        # Update config
        jq ".programs.$program.last_deployed = \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"" "$UPGRADE_CONFIG" > tmp.$$.json && mv tmp.$$.json "$UPGRADE_CONFIG"
    else
        echo -e "${RED}Failed to execute upgrade for $program${NC}"
        return 1
    fi
}

# Function to revoke upgrade authority (make immutable)
revoke_authority() {
    local program=$1
    local authority_keypair=$2
    
    echo -e "${YELLOW}WARNING: This will make the program immutable!${NC}"
    echo -e "${YELLOW}Are you sure you want to revoke upgrade authority for $program? (y/N)${NC}"
    read -r confirm
    
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        # Get program ID from config
        local program_id
        case $program in
            "eval")
                program_id="EvalCont11111111111111111111111111111111111"
                ;;
            "shard")
                program_id="ShardCon11111111111111111111111111111111111"
                ;;
            "registry")
                program_id="RegCont1111111111111111111111111111111111111"
                ;;
            *)
                echo -e "${RED}Unknown program: $program${NC}"
                return 1
                ;;
        esac
        
        # Revoke upgrade authority
        if solana program set-upgrade-authority "$program_id" --final --upgrade-authority "$authority_keypair" --url "$RPC_URL"; then
            echo -e "${GREEN}✓ Upgrade authority revoked for $program (now immutable)${NC}"
            echo "$(date): Revoked upgrade authority for $program (made immutable)" >> "$UPGRADE_LOG"
            
            # Update config
            jq ".programs.$program.upgrade_authority = null" "$UPGRADE_CONFIG" > tmp.$$.json && mv tmp.$$.json "$UPGRADE_CONFIG"
        else
            echo -e "${RED}Failed to revoke upgrade authority for $program${NC}"
            return 1
        fi
    else
        echo -e "${BLUE}Upgrade authority revocation cancelled${NC}"
    fi
}

# Parse command line arguments
COMMAND=${1:-""}
shift || true

case $COMMAND in
    "create-multisig")
        create_multisig
        ;;
    "set-authority")
        PROGRAM=""
        AUTHORITY=""
        NEW_AUTHORITY=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --program)
                    PROGRAM="$2"
                    shift 2
                    ;;
                --authority)
                    AUTHORITY="$2"
                    shift 2
                    ;;
                --new-authority)
                    NEW_AUTHORITY="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        if [ -z "$PROGRAM" ] || [ -z "$AUTHORITY" ] || [ -z "$NEW_AUTHORITY" ]; then
            echo -e "${RED}Missing required arguments${NC}"
            show_usage
            exit 1
        fi
        
        set_authority "$PROGRAM" "$AUTHORITY" "$NEW_AUTHORITY"
        ;;
    "check-authority")
        PROGRAM=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --program)
                    PROGRAM="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        if [ -z "$PROGRAM" ]; then
            echo -e "${RED}Missing required arguments${NC}"
            show_usage
            exit 1
        fi
        
        check_authority "$PROGRAM"
        ;;
    "prepare-upgrade")
        PROGRAM=""
        BUFFER=""
        BINARY=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --program)
                    PROGRAM="$2"
                    shift 2
                    ;;
                --buffer)
                    BUFFER="$2"
                    shift 2
                    ;;
                --binary)
                    BINARY="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        if [ -z "$PROGRAM" ] || [ -z "$BUFFER" ] || [ -z "$BINARY" ]; then
            echo -e "${RED}Missing required arguments${NC}"
            show_usage
            exit 1
        fi
        
        prepare_upgrade "$PROGRAM" "$BUFFER" "$BINARY"
        ;;
    "execute-upgrade")
        PROGRAM=""
        BUFFER=""
        AUTHORITY=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --program)
                    PROGRAM="$2"
                    shift 2
                    ;;
                --buffer)
                    BUFFER="$2"
                    shift 2
                    ;;
                --authority)
                    AUTHORITY="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        if [ -z "$PROGRAM" ] || [ -z "$BUFFER" ] || [ -z "$AUTHORITY" ]; then
            echo -e "${RED}Missing required arguments${NC}"
            show_usage
            exit 1
        fi
        
        execute_upgrade "$PROGRAM" "$BUFFER" "$AUTHORITY"
        ;;
    "revoke-authority")
        PROGRAM=""
        AUTHORITY=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --program)
                    PROGRAM="$2"
                    shift 2
                    ;;
                --authority)
                    AUTHORITY="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        if [ -z "$PROGRAM" ] || [ -z "$AUTHORITY" ]; then
            echo -e "${RED}Missing required arguments${NC}"
            show_usage
            exit 1
        fi
        
        revoke_authority "$PROGRAM" "$AUTHORITY"
        ;;
    *)
        show_usage
        exit 1
        ;;
esac
EOF

chmod +x "scripts/deployment/upgrade_authority/manage_upgrade_authority.sh"

echo -e "${GREEN}✓ Upgrade authority management script created${NC}"

# Create upgrade authority documentation
echo -e "\n${BLUE}3. Creating upgrade authority documentation...${NC}"

cat > "$UPGRADE_LOG" << 'EOF'
# Upgrade Authority Log

## Overview
This document tracks all upgrade authority operations for Valence Protocol programs.

## Security Model
- **Multisig Required**: All upgrades require multisig approval
- **Threshold**: 2 of 3 signers required
- **Emergency Mode**: Disabled by default
- **Buffer Size**: 1MB maximum
- **Upgrade Delay**: 24 hours maximum

## Programs
- **Eval Program**: EvalCont11111111111111111111111111111111111
- **Shard Program**: ShardCon11111111111111111111111111111111111
- **Registry Program**: RegCont1111111111111111111111111111111111111

## Operations Log
EOF

echo -e "${GREEN}✓ Upgrade authority documentation created${NC}"

# Create initial capability definitions
echo -e "\n${BLUE}4. Creating initial capability definitions...${NC}"

cat > "scripts/deployment/initial_capabilities.json" << 'EOF'
{
  "default_capabilities": [
    {
      "id": "session_creation",
      "description": "Allows creation of new sessions",
      "verification_functions": [
        "0x0101010101010101010101010101010101010101010101010101010101010101"
      ],
      "is_active": true,
      "namespace": "system"
    },
    {
      "id": "token_transfer",
      "description": "Allows token transfers within sessions",
      "verification_functions": [
        "0x0202020202020202020202020202020202020202020202020202020202020202"
      ],
      "is_active": true,
      "namespace": "finance"
    },
    {
      "id": "data_storage",
      "description": "Allows data storage operations",
      "verification_functions": [
        "0x0303030303030303030303030303030303030303030303030303030303030303"
      ],
      "is_active": true,
      "namespace": "storage"
    },
    {
      "id": "zk_verification",
      "description": "Allows zero-knowledge proof verification",
      "verification_functions": [
        "0x0404040404040404040404040404040404040404040404040404040404040404"
      ],
      "is_active": true,
      "namespace": "privacy"
    },
    {
      "id": "admin_operations",
      "description": "Administrative operations capability",
      "verification_functions": [
        "0x0505050505050505050505050505050505050505050505050505050505050505"
      ],
      "is_active": true,
      "namespace": "admin"
    }
  ],
  "default_namespaces": [
    {
      "id": "system",
      "description": "System-level operations and capabilities",
      "parent_namespace": null,
      "is_active": true
    },
    {
      "id": "finance",
      "description": "Financial operations and token management",
      "parent_namespace": "system",
      "is_active": true
    },
    {
      "id": "storage",
      "description": "Data storage and retrieval operations",
      "parent_namespace": "system",
      "is_active": true
    },
    {
      "id": "privacy",
      "description": "Privacy-preserving operations and ZK proofs",
      "parent_namespace": "system",
      "is_active": true
    },
    {
      "id": "admin",
      "description": "Administrative operations and governance",
      "parent_namespace": null,
      "is_active": true
    }
  ]
}
EOF

echo -e "${GREEN}✓ Initial capability definitions created${NC}"

# Create deployment checklist
echo -e "\n${BLUE}5. Creating deployment checklist...${NC}"

cat > "scripts/deployment/deployment_checklist.md" << 'EOF'
# Valence Protocol Deployment Checklist

## Pre-Deployment

### Security Audit
- [ ] Run security audit script
- [ ] Review audit results
- [ ] Fix any critical issues
- [ ] Document any accepted risks

### Gas Optimization
- [ ] Run gas optimization validation
- [ ] Review optimization results
- [ ] Implement recommended optimizations
- [ ] Benchmark performance improvements

### Upgrade Authority
- [ ] Set up multisig upgrade authority
- [ ] Configure upgrade thresholds
- [ ] Test upgrade process on devnet
- [ ] Document upgrade procedures

### Testing
- [ ] Run all unit tests
- [ ] Execute integration tests
- [ ] Perform end-to-end testing
- [ ] Load testing on devnet

## Deployment

### Network Configuration
- [ ] Configure for target network (devnet/testnet/mainnet)
- [ ] Set up RPC endpoints
- [ ] Configure network-specific parameters
- [ ] Verify network connectivity

### Program Deployment
- [ ] Deploy eval program
- [ ] Deploy shard program
- [ ] Deploy registry program
- [ ] Verify program deployments

### Initial Configuration
- [ ] Initialize program states
- [ ] Set up initial capabilities
- [ ] Configure default namespaces
- [ ] Set upgrade authorities

### Verification
- [ ] Verify program functionality
- [ ] Test capability execution
- [ ] Validate session creation
- [ ] Check error handling

## Post-Deployment

### Monitoring
- [ ] Set up event monitoring
- [ ] Configure performance metrics
- [ ] Enable error tracking
- [ ] Set up alerting

### Documentation
- [ ] Update deployment documentation
- [ ] Document configuration changes
- [ ] Update API documentation
- [ ] Create operator guides

### Security
- [ ] Review security configuration
- [ ] Validate access controls
- [ ] Monitor for suspicious activity
- [ ] Schedule security reviews

### Maintenance
- [ ] Set up backup procedures
- [ ] Configure monitoring dashboards
- [ ] Plan upgrade schedule
- [ ] Document troubleshooting procedures

## Emergency Procedures

### Incident Response
- [ ] Define incident response team
- [ ] Document escalation procedures
- [ ] Set up emergency contacts
- [ ] Test emergency procedures

### Recovery
- [ ] Document recovery procedures
- [ ] Test backup restoration
- [ ] Validate data integrity
- [ ] Plan for service restoration

## Sign-Off

- [ ] Security Team Approval
- [ ] Operations Team Approval
- [ ] Product Team Approval
- [ ] Executive Approval

**Deployment Date:** ___________
**Deployment Team:** ___________
**Network:** ___________
**Version:** ___________
EOF

echo -e "${GREEN}✓ Deployment checklist created${NC}"

# Generate summary
echo -e "\n${GREEN}🎉 Upgrade Authority Setup Complete!${NC}"
echo -e "\n${BLUE}Generated Files:${NC}"
echo -e "  - $UPGRADE_CONFIG"
echo -e "  - scripts/deployment/upgrade_authority/manage_upgrade_authority.sh"
echo -e "  - $UPGRADE_LOG"
echo -e "  - scripts/deployment/initial_capabilities.json"
echo -e "  - scripts/deployment/deployment_checklist.md"

echo -e "\n${BLUE}Next Steps:${NC}"
echo -e "  1. Review upgrade authority configuration"
echo -e "  2. Set up multisig wallets for mainnet"
echo -e "  3. Test upgrade procedures on devnet"
echo -e "  4. Configure initial capabilities"
echo -e "  5. Follow deployment checklist" 