// ================================
// Protocol Scoping Example
// ================================

use anchor_lang::prelude::*;
use valence_kernel::*;

// ================================
// Scenario: Preventing State Contamination
// ================================

/// Safe lending protocol session creation
pub fn create_safe_lending_session(
    owner: Pubkey,
    lending_protocol: Pubkey,
    lending_state: Pubkey,
    collateral_state: Pubkey,
) -> Result<Vec<StateBinding>> {
    // Create bindings with explicit protocol ownership
    let bindings = vec![
        StateBinding {
            state: lending_state,
            owning_protocol: lending_protocol,
            permissions: StatePermission::combine(&[
                StatePermission::Read,
                StatePermission::Write,
                StatePermission::Execute,
            ]),
            constraints: StateConstraints {
                max_operations: Some(100),
                access_window: None,
                required_guard: None,
            },
        },
        StateBinding {
            state: collateral_state,
            owning_protocol: lending_protocol,
            permissions: StatePermission::combine(&[
                StatePermission::Read,
                StatePermission::Write,
            ]),
            constraints: StateConstraints {
                max_operations: Some(50),
                access_window: None,
                required_guard: None,
            },
        },
    ];
    
    Ok(bindings)
}

/// Malicious attempt to bind unrelated state
pub fn malicious_nft_binding_attempt(
    lending_protocol: Pubkey,
    malicious_nft_protocol: Pubkey,
    lending_state: Pubkey,
    malicious_nft_state: Pubkey,
) -> Result<Vec<StateBinding>> {
    // Attacker tries to bind NFT state to lending session
    let bindings = vec![
        StateBinding {
            state: lending_state,
            owning_protocol: lending_protocol,
            permissions: StatePermission::combine(&[
                StatePermission::Read,
                StatePermission::Write,
                StatePermission::Execute,
            ]),
            constraints: StateConstraints::default(),
        },
        // This malicious binding would be rejected
        StateBinding {
            state: malicious_nft_state,
            owning_protocol: malicious_nft_protocol, // Different protocol!
            permissions: StatePermission::combine(&[
                StatePermission::Read,
                StatePermission::Write,
                StatePermission::CrossProtocol, // Even with cross-protocol flag
            ]),
            constraints: StateConstraints::default(),
        },
    ];
    
    Ok(bindings)
}

// ================================
// Guard Implementation with Protocol Awareness
// ================================

/// Protocol-aware guard that validates operations
pub fn protocol_scoped_guard(
    session: &Session,
    operation: &[u8],
    requesting_protocol: &Pubkey,
) -> Result<bool> {
    // Extract operation details
    let op = decode_operation(operation)?;
    
    // Check if requesting protocol matches session's primary protocol
    if requesting_protocol != &session.primary_protocol {
        msg!("Cross-protocol operation detected");
        
        // For cross-protocol operations, check explicit permissions
        for state_idx in op.accessed_states {
            if !session.can_protocol_access_state(
                requesting_protocol,
                state_idx,
                StatePermission::CrossProtocol
            ) {
                msg!("Protocol {} denied access to state {}", 
                    requesting_protocol, state_idx);
                return Ok(false);
            }
        }
    }
    
    // Validate each state access
    for (state_idx, required_perm) in op.state_permissions {
        if !session.can_protocol_access_state(
            requesting_protocol,
            state_idx,
            required_perm
        ) {
            msg!("Insufficient permissions for state {}", state_idx);
            return Ok(false);
        }
    }
    
    Ok(true)
}

// ================================
// Execution with Protocol Boundaries
// ================================

/// Execute lending operation with protocol validation
pub fn execute_lending_operation(
    session: &mut Session,
    lending_protocol: &Pubkey,
    operation: LendingOperation,
) -> Result<()> {
    // Get states accessible by lending protocol
    let accessible_states = session.get_protocol_states(lending_protocol);
    
    match operation {
        LendingOperation::Borrow { amount, .. } => {
            // Can only access lending protocol's states
            for (idx, state, perms) in &accessible_states {
                if !StatePermission::has_permission(*perms, StatePermission::Write) {
                    return Err(ValenceError::InsufficientPermissions.into());
                }
            }
            
            // Execute borrow logic only on permitted states
            msg!("Executing borrow on {} accessible states", accessible_states.len());
        }
        LendingOperation::Liquidate { .. } => {
            // Liquidation might need cross-protocol access
            // But only if explicitly permitted
            msg!("Liquidation requires special permissions");
        }
    }
    
    Ok(())
}

/// Malicious NFT protocol tries to access lending state
pub fn malicious_nft_access_attempt(
    session: &Session,
    nft_protocol: &Pubkey,
    target_state_index: usize,
) -> Result<()> {
    // This will fail - NFT protocol cannot access lending states
    if session.can_protocol_access_state(
        nft_protocol,
        target_state_index,
        StatePermission::Read
    ) {
        msg!("SECURITY BREACH: NFT protocol accessed lending state!");
        return Err(ValenceError::Unauthorized.into());
    }
    
    msg!("Access denied - protocol boundaries enforced");
    Ok(())
}

// ================================
// Cross-Protocol Composition (Safe)
// ================================

/// Safe cross-protocol composition with explicit permissions
pub fn create_defi_aggregator_session(
    owner: Pubkey,
    aggregator_protocol: Pubkey,
    lending_protocol: Pubkey,
    amm_protocol: Pubkey,
) -> Result<Vec<StateBinding>> {
    let bindings = vec![
        // Aggregator's own state
        StateBinding {
            state: Pubkey::new_unique(), // aggregator state
            owning_protocol: aggregator_protocol,
            permissions: StatePermission::combine(&[
                StatePermission::Read,
                StatePermission::Write,
                StatePermission::Execute,
            ]),
            constraints: StateConstraints::default(),
        },
        // Read-only access to lending state
        StateBinding {
            state: Pubkey::new_unique(), // lending pool state
            owning_protocol: lending_protocol,
            permissions: StatePermission::Read, // Read-only!
            constraints: StateConstraints {
                max_operations: Some(10),
                access_window: None,
                required_guard: Some(*b"AGGREGATOR_READ_GUARD___________"),
            },
        },
        // Read-only access to AMM state
        StateBinding {
            state: Pubkey::new_unique(), // AMM pool state
            owning_protocol: amm_protocol,
            permissions: StatePermission::Read, // Read-only!
            constraints: StateConstraints {
                max_operations: Some(10),
                access_window: None,
                required_guard: Some(*b"AGGREGATOR_READ_GUARD___________"),
            },
        },
    ];
    
    Ok(bindings)
}

// ================================
// Supporting Types
// ================================

#[derive(Debug)]
struct DecodedOperation {
    accessed_states: Vec<usize>,
    state_permissions: Vec<(usize, StatePermission)>,
}

fn decode_operation(operation: &[u8]) -> Result<DecodedOperation> {
    // Decode operation format
    Ok(DecodedOperation {
        accessed_states: vec![0, 1],
        state_permissions: vec![
            (0, StatePermission::Read),
            (1, StatePermission::Write),
        ],
    })
}

#[derive(Debug)]
enum LendingOperation {
    Borrow { amount: u64, collateral: u64 },
    Liquidate { position: Pubkey },
}

// ================================
// Benefits of Protocol Scoping
// ================================

/*
1. **State Isolation**: Each protocol's states are isolated by default
2. **Explicit Cross-Protocol Access**: Must be explicitly granted
3. **Permission Granularity**: Read/Write/Execute/Delegate per state
4. **Protocol Validation**: Every operation validated against protocol boundaries
5. **Audit Trail**: Clear record of which protocols access which states

This prevents:
- Malicious state contamination
- Unexpected cross-protocol interactions
- Dependency confusion attacks
- State namespace collisions
*/