//! Simplified Session API - Clean developer interface

use anchor_lang::prelude::*;
use crate::{Session, ShardError, capabilities::*, ValenceAccount, SimpleBundle};

/// Create a session with direct capability specification
/// This is the new simplified API that hides account complexity
pub fn create_session_v2(
    ctx: Context<CreateSessionV2>,
    capabilities: u64,
    initial_state: Vec<u8>,
    namespace: String,
    nonce: u64,
    metadata: Vec<u8>,
) -> Result<()> {
    // Validate inputs
    require!(
        !namespace.is_empty() && namespace.len() <= 64,
        ShardError::InvalidSessionRequest
    );
    
    require!(
        initial_state.len() <= 32,
        ShardError::InvalidSessionRequest
    );
    
    // Create the session
    let session_key = ctx.accounts.session.key();
    let session = &mut ctx.accounts.session;
    
    // Set basic fields
    session.id = session_key;
    session.owner = ctx.accounts.owner.key();
    session.namespace = namespace;
    session.is_consumed = false;
    session.nonce = nonce;
    session.created_at = Clock::get()?.unix_timestamp;
    session.metadata = metadata;
    
    // Set capabilities directly
    session.capabilities = capabilities;
    
    // Set initial state
    let mut state_root = [0u8; 32];
    if !initial_state.is_empty() {
        let len = initial_state.len().min(32);
        state_root[..len].copy_from_slice(&initial_state[..len]);
    }
    session.state_root = state_root;
    
    // Create a single backing account automatically
    // In a full implementation, this would be done via CPI
    let backing_account = ctx.accounts.backing_account.key();
    session._internal_accounts = vec![backing_account];
    
    // Initialize the backing account
    let account = &mut ctx.accounts.backing_account;
    account.id = backing_account;
    account.owner = ctx.accounts.owner.key();
    
    // Convert capabilities bitmap to string vector for backward compatibility
    let mut cap_strings = Vec::new();
    let caps = Capabilities(capabilities);
    
    // Check each capability bit
    if caps.has(Capability::Read) { cap_strings.push("read".to_string()); }
    if caps.has(Capability::Write) { cap_strings.push("write".to_string()); }
    if caps.has(Capability::Execute) { cap_strings.push("execute".to_string()); }
    if caps.has(Capability::Admin) { cap_strings.push("admin".to_string()); }
    if caps.has(Capability::Transfer) { cap_strings.push("transfer".to_string()); }
    if caps.has(Capability::Mint) { cap_strings.push("mint".to_string()); }
    if caps.has(Capability::Burn) { cap_strings.push("burn".to_string()); }
    if caps.has(Capability::CreateAccount) { cap_strings.push("create_account".to_string()); }
    if caps.has(Capability::CallFunction) { cap_strings.push("call_function".to_string()); }
    
    account.capabilities = cap_strings;
    account.state_hash = state_root;
    account.is_active = true;
    account.created_at = Clock::get()?.unix_timestamp;
    
    msg!("Session created via simplified API - capabilities: {:064b}, state: {:?}", 
         session.capabilities, session.state_root);
    
    Ok(())
}

#[derive(Accounts)]
pub struct CreateSessionV2<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 4 + (10 * 32) + 4 + 64 + 1 + 8 + 8 + 4 + 256 + 8 + 32, // Session size
    )]
    pub session: Account<'info, Session>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 4 + 200 + 32 + 1 + 8, // ValenceAccount size
    )]
    pub backing_account: Account<'info, ValenceAccount>,
    
    pub system_program: Program<'info, System>,
}

/// Execute operations directly on a session
/// This replaces the complex bundle execution with direct session operations
pub fn execute_on_session(
    ctx: Context<ExecuteOnSession>,
    function_hash: [u8; 32],
    args: Vec<u8>,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    
    // Verify session is not consumed
    require!(
        !session.is_consumed,
        ShardError::SessionAlreadyConsumed
    );
    
    // Check capabilities directly on session
    // In a real implementation, we'd look up the function's required capabilities
    // For now, just check if session has execute capability
    require!(
        session.has_capability(Capability::Execute),
        ShardError::InsufficientCapabilities
    );
    
    msg!("Executing function {:?} on session with capabilities {:064b}", 
         function_hash, session.capabilities);
    
    // Execute the function (simplified - would do CPI in real implementation)
    // Update state root based on execution
    let result_hash = anchor_lang::solana_program::hash::hash(&args);
    session.apply_state_diff(&result_hash.to_bytes());
    
    msg!("Execution complete, new state: {:?}", session.state_root);
    
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteOnSession<'info> {
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        constraint = session.owner == executor.key() @ ShardError::Unauthorized,
        constraint = !session.is_consumed @ ShardError::SessionAlreadyConsumed,
    )]
    pub session: Account<'info, Session>,
}

/// Execute a simplified bundle on a session
/// This API works directly with session capabilities without account complexity
pub fn execute_bundle_v2(
    ctx: Context<ExecuteBundleV2>,
    bundle: SimpleBundle,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    
    // Verify session is not consumed
    require!(
        !session.is_consumed,
        ShardError::SessionAlreadyConsumed
    );
    
    // Verify bundle is for this session
    require!(
        bundle.session == session.id,
        ShardError::InvalidBundle
    );
    
    // Get session capabilities once
    let session_caps = Capabilities(session.capabilities);
    
    // Execute each operation
    for (i, operation) in bundle.operations.iter().enumerate() {
        msg!("Executing operation {}: function {:?}", i, operation.function_hash);
        
        // Check if session has required capabilities (O(1) check)
        let required_caps = Capabilities(operation.required_capabilities);
        
        // Check each required capability
        for cap_bit in 0..64 {
            let cap_mask = 1u64 << cap_bit;
            if (required_caps.0 & cap_mask) != 0 {
                // This capability is required
                if (session_caps.0 & cap_mask) == 0 {
                    // Session doesn't have this capability
                    msg!("Missing capability bit {}", cap_bit);
                    return Err(ShardError::InsufficientCapabilities.into());
                }
            }
        }
        
        msg!("Capability check passed, executing function");
        
        // Execute the function (simplified - would do CPI in real implementation)
        let result_hash = anchor_lang::solana_program::hash::hash(&operation.args);
        session.apply_state_diff(&result_hash.to_bytes());
        
        msg!("Operation {} completed, new state: {:?}", i, session.state_root);
    }
    
    msg!("Bundle executed successfully with {} operations", bundle.operations.len());
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteBundleV2<'info> {
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        constraint = session.owner == executor.key() @ ShardError::Unauthorized,
        constraint = !session.is_consumed @ ShardError::SessionAlreadyConsumed,
    )]
    pub session: Account<'info, Session>,
}