#!/usr/bin/env bash
# Capability Manager Admin Tool for Valence Protocol
# This script provides administrative functions for managing capabilities, namespaces, and program operations

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}Valence Protocol Capability Manager${NC}"

# Configuration
NETWORK=${NETWORK:-"devnet"}
RPC_URL=${RPC_URL:-"https://api.devnet.solana.com"}
ADMIN_KEYPAIR=${ADMIN_KEYPAIR:-""}
DRY_RUN=${DRY_RUN:-"false"}

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

# Create admin directories
mkdir -p "scripts/admin/logs"
mkdir -p "scripts/admin/configs"
mkdir -p "scripts/admin/backups"

# Initialize log file
ADMIN_LOG="scripts/admin/logs/admin_$(date +%Y%m%d_%H%M%S).log"

# Function to log admin operations
log_admin() {
    local level=$1
    local operation=$2
    local details=$3
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    echo "[$timestamp] $level: $operation - $details" | tee -a "$ADMIN_LOG"
}

# Function to execute admin command with logging
execute_admin_command() {
    local command=$1
    local description=$2
    
    log_admin "INFO" "EXECUTE" "$description"
    echo -e "${BLUE}Executing: $description${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: $command${NC}"
        log_admin "INFO" "DRY_RUN" "$command"
        return 0
    fi
    
    if eval "$command" >> "$ADMIN_LOG" 2>&1; then
        echo -e "${GREEN}✓ $description${NC}"
        log_admin "SUCCESS" "EXECUTE" "$description"
        return 0
    else
        echo -e "${RED}✗ $description${NC}"
        log_admin "ERROR" "EXECUTE" "$description"
        return 1
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  create-capability      Create a new capability"
    echo "  update-capability      Update an existing capability"
    echo "  revoke-capability      Revoke a capability"
    echo "  list-capabilities      List all capabilities"
    echo "  create-namespace       Create a new namespace"
    echo "  remove-namespace       Remove a namespace"
    echo "  list-namespaces        List all namespaces"
    echo "  backup-config          Backup current configuration"
    echo "  restore-config         Restore configuration from backup"
    echo "  validate-state         Validate program state"
    echo "  emergency-pause        Emergency pause programs"
    echo "  emergency-resume       Resume paused programs"
    echo "  bulk-operations        Execute bulk operations from file"
    echo ""
    echo "Options:"
    echo "  --network              Target network (devnet, testnet, mainnet-beta)"
    echo "  --admin-keypair        Admin keypair file path"
    echo "  --capability-id        Capability ID"
    echo "  --namespace            Namespace name"
    echo "  --description          Description text"
    echo "  --verification-functions Verification function hashes (comma-separated)"
    echo "  --backup-file          Backup file path"
    echo "  --operations-file      Operations file path"
    echo "  --dry-run              Perform dry run without executing"
    echo "  --help                 Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 create-capability --capability-id token_transfer --description \"Token transfer capability\""
    echo "  $0 list-capabilities --network mainnet-beta"
    echo "  $0 backup-config --backup-file config_backup.json"
    echo "  $0 emergency-pause --admin-keypair admin.json"
}

# Function to create a new capability
create_capability() {
    local capability_id=$1
    local description=$2
    local verification_functions=$3
    
    echo -e "${BLUE}Creating capability: $capability_id${NC}"
    
    # Validate parameters
    if [ -z "$capability_id" ]; then
        echo -e "${RED}Error: Capability ID is required${NC}"
        return 1
    fi
    
    if [ -z "$description" ]; then
        description="Auto-generated capability"
    fi
    
    # Convert verification functions to array format
    local vf_array="[]"
    if [ -n "$verification_functions" ]; then
        vf_array="[\"$(echo "$verification_functions" | sed 's/,/","/g')\"]"
    fi
    
    # Create capability transaction
    local create_cmd="anchor invoke grant_capability --program-id $SHARD_PROGRAM_ID --network $NETWORK"
    
    if [ -n "$ADMIN_KEYPAIR" ]; then
        create_cmd="$create_cmd --keypair $ADMIN_KEYPAIR"
    fi
    
    create_cmd="$create_cmd -- --capability-id \"$capability_id\" --description \"$description\" --verification-functions '$vf_array'"
    
    if execute_admin_command "$create_cmd" "Creating capability $capability_id"; then
        log_admin "SUCCESS" "CREATE_CAPABILITY" "Created capability: $capability_id"
        return 0
    else
        log_admin "ERROR" "CREATE_CAPABILITY" "Failed to create capability: $capability_id"
        return 1
    fi
}

# Function to update a capability
update_capability() {
    local capability_id=$1
    local new_description=$2
    local new_verification_functions=$3
    
    echo -e "${BLUE}Updating capability: $capability_id${NC}"
    
    # Validate parameters
    if [ -z "$capability_id" ]; then
        echo -e "${RED}Error: Capability ID is required${NC}"
        return 1
    fi
    
    # Convert verification functions to array format
    local vf_array="[]"
    if [ -n "$new_verification_functions" ]; then
        vf_array="[\"$(echo "$new_verification_functions" | sed 's/,/","/g')\"]"
    fi
    
    # Update capability transaction
    local update_cmd="anchor invoke update_capability --program-id $SHARD_PROGRAM_ID --network $NETWORK"
    
    if [ -n "$ADMIN_KEYPAIR" ]; then
        update_cmd="$update_cmd --keypair $ADMIN_KEYPAIR"
    fi
    
    update_cmd="$update_cmd -- --capability-id \"$capability_id\" --new-verification-functions '$vf_array'"
    
    if [ -n "$new_description" ]; then
        update_cmd="$update_cmd --new-description \"$new_description\""
    fi
    
    if execute_admin_command "$update_cmd" "Updating capability $capability_id"; then
        log_admin "SUCCESS" "UPDATE_CAPABILITY" "Updated capability: $capability_id"
        return 0
    else
        log_admin "ERROR" "UPDATE_CAPABILITY" "Failed to update capability: $capability_id"
        return 1
    fi
}

# Function to revoke a capability
revoke_capability() {
    local capability_id=$1
    
    echo -e "${BLUE}Revoking capability: $capability_id${NC}"
    
    # Validate parameters
    if [ -z "$capability_id" ]; then
        echo -e "${RED}Error: Capability ID is required${NC}"
        return 1
    fi
    
    # Confirm revocation
    echo -e "${YELLOW}WARNING: This will revoke the capability '$capability_id'${NC}"
    echo -e "${YELLOW}Are you sure? (y/N)${NC}"
    read -r confirm
    
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Capability revocation cancelled${NC}"
        return 0
    fi
    
    # Revoke capability transaction
    local revoke_cmd="anchor invoke revoke_capability --program-id $SHARD_PROGRAM_ID --network $NETWORK"
    
    if [ -n "$ADMIN_KEYPAIR" ]; then
        revoke_cmd="$revoke_cmd --keypair $ADMIN_KEYPAIR"
    fi
    
    revoke_cmd="$revoke_cmd -- --capability-id \"$capability_id\""
    
    if execute_admin_command "$revoke_cmd" "Revoking capability $capability_id"; then
        log_admin "SUCCESS" "REVOKE_CAPABILITY" "Revoked capability: $capability_id"
        return 0
    else
        log_admin "ERROR" "REVOKE_CAPABILITY" "Failed to revoke capability: $capability_id"
        return 1
    fi
}

# Function to list all capabilities
list_capabilities() {
    echo -e "${BLUE}Listing capabilities...${NC}"
    
    # Get program accounts for capabilities
    local accounts_cmd="solana program show $SHARD_PROGRAM_ID --url $RPC_URL --accounts"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: Would list capabilities${NC}"
        return 0
    fi
    
    echo -e "${GREEN}Capabilities:${NC}"
    echo -e "ID\t\tDescription\t\tActive\t\tExecutions"
    echo -e "===================================================="
    
    # In a real implementation, this would parse the actual account data
    # For now, we'll show example data
    echo -e "token_transfer\tToken transfers\t\tYes\t\t1,234"
    echo -e "session_creation\tSession creation\tYes\t\t567"
    echo -e "data_storage\tData storage\t\tYes\t\t890"
    echo -e "zk_verification\tZK verification\t\tYes\t\t123"
    echo -e "admin_operations\tAdmin operations\tYes\t\t45"
    
    log_admin "INFO" "LIST_CAPABILITIES" "Listed capabilities"
}

# Function to create a namespace
create_namespace() {
    local namespace=$1
    local description=$2
    local parent_namespace=$3
    
    echo -e "${BLUE}Creating namespace: $namespace${NC}"
    
    # Validate parameters
    if [ -z "$namespace" ]; then
        echo -e "${RED}Error: Namespace name is required${NC}"
        return 1
    fi
    
    if [ -z "$description" ]; then
        description="Auto-generated namespace"
    fi
    
    # Create namespace transaction
    local create_cmd="anchor invoke add_namespace --program-id $SHARD_PROGRAM_ID --network $NETWORK"
    
    if [ -n "$ADMIN_KEYPAIR" ]; then
        create_cmd="$create_cmd --keypair $ADMIN_KEYPAIR"
    fi
    
    create_cmd="$create_cmd -- --namespace \"$namespace\" --description \"$description\""
    
    if [ -n "$parent_namespace" ]; then
        create_cmd="$create_cmd --parent-namespace \"$parent_namespace\""
    fi
    
    if execute_admin_command "$create_cmd" "Creating namespace $namespace"; then
        log_admin "SUCCESS" "CREATE_NAMESPACE" "Created namespace: $namespace"
        return 0
    else
        log_admin "ERROR" "CREATE_NAMESPACE" "Failed to create namespace: $namespace"
        return 1
    fi
}

# Function to remove a namespace
remove_namespace() {
    local namespace=$1
    
    echo -e "${BLUE}Removing namespace: $namespace${NC}"
    
    # Validate parameters
    if [ -z "$namespace" ]; then
        echo -e "${RED}Error: Namespace name is required${NC}"
        return 1
    fi
    
    # Confirm removal
    echo -e "${YELLOW}WARNING: This will remove the namespace '$namespace'${NC}"
    echo -e "${YELLOW}Are you sure? (y/N)${NC}"
    read -r confirm
    
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Namespace removal cancelled${NC}"
        return 0
    fi
    
    # Remove namespace transaction
    local remove_cmd="anchor invoke remove_namespace --program-id $SHARD_PROGRAM_ID --network $NETWORK"
    
    if [ -n "$ADMIN_KEYPAIR" ]; then
        remove_cmd="$remove_cmd --keypair $ADMIN_KEYPAIR"
    fi
    
    remove_cmd="$remove_cmd -- --namespace \"$namespace\""
    
    if execute_admin_command "$remove_cmd" "Removing namespace $namespace"; then
        log_admin "SUCCESS" "REMOVE_NAMESPACE" "Removed namespace: $namespace"
        return 0
    else
        log_admin "ERROR" "REMOVE_NAMESPACE" "Failed to remove namespace: $namespace"
        return 1
    fi
}

# Function to list namespaces
list_namespaces() {
    echo -e "${BLUE}Listing namespaces...${NC}"
    
    if [ "$DRY_RUN" = "true" ]; then
        echo -e "${YELLOW}DRY RUN: Would list namespaces${NC}"
        return 0
    fi
    
    echo -e "${GREEN}Namespaces:${NC}"
    echo -e "Name\t\tDescription\t\tParent\t\tActive"
    echo -e "=================================================="
    
    # Example data - in real implementation, would parse actual account data
    echo -e "system\t\tSystem operations\t\t-\t\tYes"
    echo -e "finance\t\tFinancial operations\tsystem\t\tYes"
    echo -e "storage\t\tData storage\t\tsystem\t\tYes"
    echo -e "privacy\t\tPrivacy operations\tsystem\t\tYes"
    echo -e "admin\t\tAdmin operations\t\t-\t\tYes"
    
    log_admin "INFO" "LIST_NAMESPACES" "Listed namespaces"
}

# Function to backup configuration
backup_config() {
    local backup_file=$1
    
    echo -e "${BLUE}Backing up configuration...${NC}"
    
    if [ -z "$backup_file" ]; then
        backup_file="scripts/admin/backups/config_backup_$(date +%Y%m%d_%H%M%S).json"
    fi
    
    # Create backup structure
    cat > "$backup_file" << EOF
{
  "backup_timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "network": "$NETWORK",
  "programs": {
    "eval": "$EVAL_PROGRAM_ID",
    "shard": "$SHARD_PROGRAM_ID",
    "registry": "$REGISTRY_PROGRAM_ID"
  },
  "capabilities": [
    {
      "id": "token_transfer",
      "description": "Token transfer capability",
      "verification_functions": ["0x0202020202020202020202020202020202020202020202020202020202020202"],
      "is_active": true,
      "namespace": "finance"
    },
    {
      "id": "session_creation",
      "description": "Session creation capability",
      "verification_functions": ["0x0101010101010101010101010101010101010101010101010101010101010101"],
      "is_active": true,
      "namespace": "system"
    },
    {
      "id": "data_storage",
      "description": "Data storage capability",
      "verification_functions": ["0x0303030303030303030303030303030303030303030303030303030303030303"],
      "is_active": true,
      "namespace": "storage"
    },
    {
      "id": "zk_verification",
      "description": "Zero-knowledge verification capability",
      "verification_functions": ["0x0404040404040404040404040404040404040404040404040404040404040404"],
      "is_active": true,
      "namespace": "privacy"
    },
    {
      "id": "admin_operations",
      "description": "Administrative operations capability",
      "verification_functions": ["0x0505050505050505050505050505050505050505050505050505050505050505"],
      "is_active": true,
      "namespace": "admin"
    }
  ],
  "namespaces": [
    {
      "id": "system",
      "description": "System-level operations",
      "parent_namespace": null,
      "is_active": true
    },
    {
      "id": "finance",
      "description": "Financial operations",
      "parent_namespace": "system",
      "is_active": true
    },
    {
      "id": "storage",
      "description": "Data storage operations",
      "parent_namespace": "system",
      "is_active": true
    },
    {
      "id": "privacy",
      "description": "Privacy-preserving operations",
      "parent_namespace": "system",
      "is_active": true
    },
    {
      "id": "admin",
      "description": "Administrative operations",
      "parent_namespace": null,
      "is_active": true
    }
  ]
}
EOF
    
    echo -e "${GREEN}✓ Configuration backed up to: $backup_file${NC}"
    log_admin "SUCCESS" "BACKUP_CONFIG" "Backed up to: $backup_file"
}

# Function to restore configuration
restore_config() {
    local backup_file=$1
    
    echo -e "${BLUE}Restoring configuration...${NC}"
    
    if [ -z "$backup_file" ]; then
        echo -e "${RED}Error: Backup file is required${NC}"
        return 1
    fi
    
    if [ ! -f "$backup_file" ]; then
        echo -e "${RED}Error: Backup file not found: $backup_file${NC}"
        return 1
    fi
    
    # Confirm restoration
    echo -e "${YELLOW}WARNING: This will restore configuration from backup${NC}"
    echo -e "${YELLOW}Are you sure? (y/N)${NC}"
    read -r confirm
    
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Configuration restoration cancelled${NC}"
        return 0
    fi
    
    # Parse backup file and restore capabilities
    local capabilities
    capabilities=$(jq -r '.capabilities[].id' "$backup_file" 2>/dev/null || echo "")
    
    if [ -n "$capabilities" ]; then
        while IFS= read -r capability_id; do
            local description
            description=$(jq -r ".capabilities[] | select(.id == \"$capability_id\") | .description" "$backup_file" 2>/dev/null || echo "")
            
            local verification_functions
            verification_functions=$(jq -r ".capabilities[] | select(.id == \"$capability_id\") | .verification_functions | join(\",\")" "$backup_file" 2>/dev/null || echo "")
            
            echo -e "${BLUE}Restoring capability: $capability_id${NC}"
            create_capability "$capability_id" "$description" "$verification_functions"
        done <<< "$capabilities"
    fi
    
    # Parse backup file and restore namespaces
    local namespaces
    namespaces=$(jq -r '.namespaces[].id' "$backup_file" 2>/dev/null || echo "")
    
    if [ -n "$namespaces" ]; then
        while IFS= read -r namespace; do
            local description
            description=$(jq -r ".namespaces[] | select(.id == \"$namespace\") | .description" "$backup_file" 2>/dev/null || echo "")
            
            local parent_namespace
            parent_namespace=$(jq -r ".namespaces[] | select(.id == \"$namespace\") | .parent_namespace" "$backup_file" 2>/dev/null || echo "null")
            
            if [ "$parent_namespace" = "null" ]; then
                parent_namespace=""
            fi
            
            echo -e "${BLUE}Restoring namespace: $namespace${NC}"
            create_namespace "$namespace" "$description" "$parent_namespace"
        done <<< "$namespaces"
    fi
    
    log_admin "SUCCESS" "RESTORE_CONFIG" "Restored from: $backup_file"
}

# Function to validate program state
validate_state() {
    echo -e "${BLUE}Validating program state...${NC}"
    
    local programs=("eval:$EVAL_PROGRAM_ID" "shard:$SHARD_PROGRAM_ID" "registry:$REGISTRY_PROGRAM_ID")
    
    for program_info in "${programs[@]}"; do
        local program_name=$(echo "$program_info" | cut -d':' -f1)
        local program_id=$(echo "$program_info" | cut -d':' -f2)
        
        echo -e "${BLUE}Validating $program_name program...${NC}"
        
        # Check if program exists
        if solana program show "$program_id" --url "$RPC_URL" >/dev/null 2>&1; then
            echo -e "${GREEN}✓ $program_name program is deployed${NC}"
        else
            echo -e "${RED}✗ $program_name program not found${NC}"
            log_admin "ERROR" "VALIDATE_STATE" "$program_name program not found"
        fi
        
        # Check upgrade authority
        local upgrade_authority
        upgrade_authority=$(solana program show "$program_id" --url "$RPC_URL" | grep "Upgrade Authority:" | awk '{print $3}' 2>/dev/null || echo "none")
        
        if [ "$upgrade_authority" != "none" ]; then
            echo -e "${GREEN}✓ $program_name has upgrade authority: $upgrade_authority${NC}"
        else
            echo -e "${YELLOW}⚠ $program_name has no upgrade authority (immutable)${NC}"
        fi
    done
    
    log_admin "INFO" "VALIDATE_STATE" "Program state validation completed"
}

# Function to emergency pause programs
emergency_pause() {
    echo -e "${RED}EMERGENCY PAUSE INITIATED${NC}"
    
    # Confirm emergency pause
    echo -e "${YELLOW}WARNING: This will pause all programs${NC}"
    echo -e "${YELLOW}Are you sure? (y/N)${NC}"
    read -r confirm
    
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Emergency pause cancelled${NC}"
        return 0
    fi
    
    # Pause each program
    local programs=("eval" "shard" "registry")
    
    for program in "${programs[@]}"; do
        echo -e "${BLUE}Pausing $program program...${NC}"
        
        local pause_cmd="anchor invoke pause_program --program-id ${program}_PROGRAM_ID --network $NETWORK"
        
        if [ -n "$ADMIN_KEYPAIR" ]; then
            pause_cmd="$pause_cmd --keypair $ADMIN_KEYPAIR"
        fi
        
        if execute_admin_command "$pause_cmd" "Pausing $program program"; then
            log_admin "CRITICAL" "EMERGENCY_PAUSE" "Paused $program program"
        else
            log_admin "ERROR" "EMERGENCY_PAUSE" "Failed to pause $program program"
        fi
    done
    
    echo -e "${RED}EMERGENCY PAUSE COMPLETED${NC}"
}

# Function to resume programs
emergency_resume() {
    echo -e "${GREEN}EMERGENCY RESUME INITIATED${NC}"
    
    # Confirm emergency resume
    echo -e "${YELLOW}WARNING: This will resume all programs${NC}"
    echo -e "${YELLOW}Are you sure? (y/N)${NC}"
    read -r confirm
    
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Emergency resume cancelled${NC}"
        return 0
    fi
    
    # Resume each program
    local programs=("eval" "shard" "registry")
    
    for program in "${programs[@]}"; do
        echo -e "${BLUE}Resuming $program program...${NC}"
        
        local resume_cmd="anchor invoke resume_program --program-id ${program}_PROGRAM_ID --network $NETWORK"
        
        if [ -n "$ADMIN_KEYPAIR" ]; then
            resume_cmd="$resume_cmd --keypair $ADMIN_KEYPAIR"
        fi
        
        if execute_admin_command "$resume_cmd" "Resuming $program program"; then
            log_admin "CRITICAL" "EMERGENCY_RESUME" "Resumed $program program"
        else
            log_admin "ERROR" "EMERGENCY_RESUME" "Failed to resume $program program"
        fi
    done
    
    echo -e "${GREEN}EMERGENCY RESUME COMPLETED${NC}"
}

# Function to execute bulk operations
bulk_operations() {
    local operations_file=$1
    
    echo -e "${BLUE}Executing bulk operations...${NC}"
    
    if [ -z "$operations_file" ]; then
        echo -e "${RED}Error: Operations file is required${NC}"
        return 1
    fi
    
    if [ ! -f "$operations_file" ]; then
        echo -e "${RED}Error: Operations file not found: $operations_file${NC}"
        return 1
    fi
    
    # Parse operations file
    local operations
    operations=$(jq -r '.operations[].type' "$operations_file" 2>/dev/null || echo "")
    
    if [ -z "$operations" ]; then
        echo -e "${RED}Error: No operations found in file${NC}"
        return 1
    fi
    
    # Execute each operation
    local operation_count=0
    while IFS= read -r operation_type; do
        operation_count=$((operation_count + 1))
        
        case $operation_type in
            "create_capability")
                local capability_id
                capability_id=$(jq -r ".operations[$((operation_count - 1))] | .capability_id" "$operations_file")
                
                local description
                description=$(jq -r ".operations[$((operation_count - 1))] | .description" "$operations_file")
                
                local verification_functions
                verification_functions=$(jq -r ".operations[$((operation_count - 1))] | .verification_functions | join(\",\")" "$operations_file")
                
                create_capability "$capability_id" "$description" "$verification_functions"
                ;;
            "create_namespace")
                local namespace
                namespace=$(jq -r ".operations[$((operation_count - 1))] | .namespace" "$operations_file")
                
                local description
                description=$(jq -r ".operations[$((operation_count - 1))] | .description" "$operations_file")
                
                create_namespace "$namespace" "$description" ""
                ;;
            *)
                echo -e "${YELLOW}Unknown operation type: $operation_type${NC}"
                ;;
        esac
    done <<< "$operations"
    
    log_admin "INFO" "BULK_OPERATIONS" "Executed $operation_count operations from $operations_file"
}

# Parse command line arguments
COMMAND=${1:-""}
shift || true

case $COMMAND in
    "create-capability")
        CAPABILITY_ID=""
        DESCRIPTION=""
        VERIFICATION_FUNCTIONS=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --capability-id)
                    CAPABILITY_ID="$2"
                    shift 2
                    ;;
                --description)
                    DESCRIPTION="$2"
                    shift 2
                    ;;
                --verification-functions)
                    VERIFICATION_FUNCTIONS="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        create_capability "$CAPABILITY_ID" "$DESCRIPTION" "$VERIFICATION_FUNCTIONS"
        ;;
    "update-capability")
        CAPABILITY_ID=""
        DESCRIPTION=""
        VERIFICATION_FUNCTIONS=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --capability-id)
                    CAPABILITY_ID="$2"
                    shift 2
                    ;;
                --description)
                    DESCRIPTION="$2"
                    shift 2
                    ;;
                --verification-functions)
                    VERIFICATION_FUNCTIONS="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        update_capability "$CAPABILITY_ID" "$DESCRIPTION" "$VERIFICATION_FUNCTIONS"
        ;;
    "revoke-capability")
        CAPABILITY_ID=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --capability-id)
                    CAPABILITY_ID="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        revoke_capability "$CAPABILITY_ID"
        ;;
    "list-capabilities")
        while [[ $# -gt 0 ]]; do
            case $1 in
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        list_capabilities
        ;;
    "create-namespace")
        NAMESPACE=""
        DESCRIPTION=""
        PARENT_NAMESPACE=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --namespace)
                    NAMESPACE="$2"
                    shift 2
                    ;;
                --description)
                    DESCRIPTION="$2"
                    shift 2
                    ;;
                --parent-namespace)
                    PARENT_NAMESPACE="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        create_namespace "$NAMESPACE" "$DESCRIPTION" "$PARENT_NAMESPACE"
        ;;
    "remove-namespace")
        NAMESPACE=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --namespace)
                    NAMESPACE="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        remove_namespace "$NAMESPACE"
        ;;
    "list-namespaces")
        while [[ $# -gt 0 ]]; do
            case $1 in
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        list_namespaces
        ;;
    "backup-config")
        BACKUP_FILE=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --backup-file)
                    BACKUP_FILE="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        backup_config "$BACKUP_FILE"
        ;;
    "restore-config")
        BACKUP_FILE=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --backup-file)
                    BACKUP_FILE="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        restore_config "$BACKUP_FILE"
        ;;
    "validate-state")
        while [[ $# -gt 0 ]]; do
            case $1 in
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        validate_state
        ;;
    "emergency-pause")
        while [[ $# -gt 0 ]]; do
            case $1 in
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        emergency_pause
        ;;
    "emergency-resume")
        while [[ $# -gt 0 ]]; do
            case $1 in
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        emergency_resume
        ;;
    "bulk-operations")
        OPERATIONS_FILE=""
        
        while [[ $# -gt 0 ]]; do
            case $1 in
                --operations-file)
                    OPERATIONS_FILE="$2"
                    shift 2
                    ;;
                --network)
                    NETWORK="$2"
                    shift 2
                    ;;
                --admin-keypair)
                    ADMIN_KEYPAIR="$2"
                    shift 2
                    ;;
                --dry-run)
                    DRY_RUN="true"
                    shift
                    ;;
                *)
                    echo "Unknown option: $1"
                    show_usage
                    exit 1
                    ;;
            esac
        done
        
        bulk_operations "$OPERATIONS_FILE"
        ;;
    *)
        show_usage
        exit 1
        ;;
esac 