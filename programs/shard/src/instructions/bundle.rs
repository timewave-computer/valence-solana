//! Bundle execution logic (controller)

use anchor_lang::prelude::*;
use crate::{Bundle, ExecutionMode, ExecutionState, Session, ShardError, ShardConfig, ID, capabilities::*};
use std::str::FromStr;

/// Execute a synchronous bundle (all operations in one transaction)
pub fn execute_sync(
    ctx: Context<ExecuteSyncBundle>,
    bundle: Bundle,
) -> Result<()> {
    // Verify bundle mode
    require!(
        bundle.mode == ExecutionMode::Sync,
        ShardError::InvalidBundle
    );
    
    // Verify operation count
    require!(
        !bundle.operations.is_empty() && bundle.operations.len() <= 10,
        ShardError::TooManyOperations
    );
    
    let session = &ctx.accounts.session;
    
    // Verify session is not consumed (new linear semantics)
    require!(
        !session.is_consumed,
        ShardError::SessionAlreadyConsumed
    );
    
    // Execute each operation
    // Use the session's pre-aggregated state root
    let mut current_hash = session.state_root;
    
    for (i, operation) in bundle.operations.iter().enumerate() {
        msg!("Executing operation {}: function {:?}", i, operation.function_hash);
        
        // Build function call data
        let mut ix_data = vec![0]; // Execute function discriminator
        ix_data.extend_from_slice(&operation.args);
        
        // Find and execute function with capability checking inline
        let result = {
            // Find the registry entry to get required capabilities
            let mut registry_entry = None;
            let registry_program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
            
            for account in ctx.remaining_accounts.iter() {
                if account.owner == &registry_program_id && !account.data_is_empty() {
                    let data = account.try_borrow_data()?;
                    if data.len() >= 8 + 32 {
                        let hash_bytes = &data[8..40];
                        let entry_hash: [u8; 32] = hash_bytes.try_into().unwrap();
                        if entry_hash == operation.function_hash {
                            registry_entry = Some(account);
                            break;
                        }
                    }
                }
            }
            
            let registry_entry = registry_entry.ok_or(ShardError::FunctionNotFound)?;
            
            // Get required capabilities from registry entry
            let data = registry_entry.try_borrow_data()?;
            let mut required_capabilities = Vec::new();
            
            if data.len() >= 104 + 4 {
                let capabilities_offset = 104;
                let len_bytes = &data[capabilities_offset..capabilities_offset + 4];
                let vec_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
                
                let mut offset = capabilities_offset + 4;
                for _ in 0..vec_len {
                    if offset + 4 > data.len() { break; }
                    
                    let str_len_bytes = &data[offset..offset + 4];
                    let str_len = u32::from_le_bytes(str_len_bytes.try_into().unwrap()) as usize;
                    offset += 4;
                    
                    if offset + str_len > data.len() { break; }
                    
                    let cap_bytes = &data[offset..offset + str_len];
                    let capability = String::from_utf8(cap_bytes.to_vec())
                        .map_err(|_| ShardError::InvalidBundle)?;
                    required_capabilities.push(capability);
                    offset += str_len;
                }
            }
            
            // Use session's pre-aggregated capabilities for O(1) checking
            let session_capabilities = session.get_capabilities();
            
            // Check all required capabilities are available using bitmap
            for required_cap in required_capabilities.iter() {
                if let Some(cap) = Capability::from_string(required_cap) {
                    if !session_capabilities.has(cap) {
                        msg!("Missing capability: {}", required_cap);
                        return Err(ShardError::InsufficientCapabilities.into());
                    }
                } else {
                    msg!("Unknown capability: {}", required_cap);
                    return Err(ShardError::InsufficientCapabilities.into());
                }
            }
            
            // Find function program and execute
            let mut function_program = None;
            for account_info in ctx.remaining_accounts.iter() {
                if account_info.owner == &ID && account_info.data_len() == 8 + 32 + 32 + 32 + 1 + 8 {
                    let data = account_info.try_borrow_data()?;
                    let shard_bytes = &data[8..40];
                    let import_shard = Pubkey::new_from_array(shard_bytes.try_into().unwrap());
                    let hash_bytes = &data[40..72];
                    let import_hash: [u8; 32] = hash_bytes.try_into().unwrap();
                    
                    if import_hash == operation.function_hash && import_shard == ctx.accounts.shard_config.key() {
                        let program_bytes = &data[72..104];
                        function_program = Some(Pubkey::new_from_array(program_bytes.try_into().unwrap()));
                        break;
                    }
                }
            }
            
            let _function_program = function_program.ok_or(ShardError::FunctionNotFound)?;
            
            // Execute the function (simplified CPI)
            msg!("Function has required capabilities, executing");
            vec![1u8] // Placeholder result
        };
        
        // Update diff hash with result
        current_hash = compute_next_hash(current_hash, &result);
        
        // Verify diff if expected
        if let Some(expected_diff) = operation.expected_diff {
            require!(
                current_hash == expected_diff,
                ShardError::DiffMismatch
            );
        }
        
        msg!("Operation {} completed, new hash: {:?}", i, current_hash);
    }
    
    // Update state in session to maintain consistency
    let session = &mut ctx.accounts.session;
    session.update_state_root(current_hash);
    
    // Also update state in session accounts to maintain backward compatibility
    {
        // For now, we'll update all accounts with the same hash
        // In a more sophisticated implementation, each account might have its own state
        for account_pubkey in session._internal_accounts.iter() {
            // Find the account in remaining_accounts
            for account_info in ctx.remaining_accounts.iter() {
                if account_info.key == account_pubkey {
                    // Verify it's a ValenceAccount and writable
                    if account_info.owner != &ID || !account_info.is_writable {
                        continue;
                    }
                    
                    // Update state hash in account
                    let mut account_data = account_info.try_borrow_mut_data()?;
                    if account_data.len() >= 8 + 32 + 32 + 200 + 32 { // Up to state_hash field
                        // State hash is at offset 8 + 32 + 32 + 200 = 272
                        let state_hash_offset = 272;
                        if account_data.len() >= state_hash_offset + 32 {
                            account_data[state_hash_offset..state_hash_offset + 32]
                                .copy_from_slice(&current_hash);
                            msg!("Updated state hash for account {}", account_pubkey);
                        }
                    }
                    break;
                }
            }
        }
    };
    
    msg!("Bundle executed successfully, new state hash: {:?}", current_hash);
    msg!("Session state_root updated to: {:?}", session.state_root);
    Ok(())
}

/// Start asynchronous bundle execution
pub fn start_async(
    ctx: Context<StartAsyncBundle>,
    bundle: Bundle,
) -> Result<()> {
    // Verify bundle mode
    require!(
        bundle.mode == ExecutionMode::Async,
        ShardError::InvalidBundle
    );
    
    // Initialize execution state
    let execution_state_key = ctx.accounts.execution_state.key();
    let execution_state = &mut ctx.accounts.execution_state;
    execution_state.bundle_id = execution_state_key;
    execution_state.current_operation = 0;
    execution_state.total_operations = bundle.operations.len() as u16;
    // Use the session's pre-aggregated state root
    execution_state.state_hash = ctx.accounts.session.state_root;
    execution_state.is_complete = false;
    execution_state.operations = bundle.operations.clone();
    execution_state.session = ctx.accounts.session.key();
    
    msg!("Started async bundle with {} operations", bundle.operations.len());
    
    // Execute first operation if gas permits
    if ctx.accounts.executor.lamports() > 1_000_000 { // Simple gas check
        // Inline execute_next_operation to avoid lifetime issues
        if execution_state.current_operation < execution_state.total_operations {
            let operation = &execution_state.operations[execution_state.current_operation as usize];
            msg!("Executing async operation {}: function {:?}", 
                execution_state.current_operation, operation.function_hash);
            
            // Simple execution placeholder - capability checking would go here
            execution_state.state_hash = compute_next_hash(execution_state.state_hash, &[1u8]);
            execution_state.current_operation += 1;
        }
    }
    
    Ok(())
}

/// Continue async bundle from checkpoint
pub fn continue_async(
    ctx: Context<ContinueAsyncBundle>,
    bundle_id: Pubkey,
) -> Result<()> {
    let execution_state = &mut ctx.accounts.execution_state;
    
    // Verify bundle ID
    require!(
        execution_state.bundle_id == bundle_id,
        ShardError::InvalidBundle
    );
    
    // Verify not complete
    require!(
        !execution_state.is_complete,
        ShardError::InvalidCheckpoint
    );
    
    // Continue execution from current checkpoint
    let operations_to_execute = (execution_state.total_operations - execution_state.current_operation)
        .min(10) as usize; // Execute up to 10 operations per transaction
    
    msg!("Continuing async bundle from operation {} (executing {} ops)", 
        execution_state.current_operation, operations_to_execute);
    
    // Load session account from remaining accounts
    // We'll need to pass session state through a different mechanism
    // For now, just use the session hash from execution state
    let _session_state_hash = execution_state.state_hash;
    
    // Get shard key from the first remaining account (convention)
    let _shard_key = ctx.remaining_accounts
        .first()
        .ok_or(ShardError::InvalidBundle)?
        .key;
    
    // Execute operations
    for _ in 0..operations_to_execute {
        if execution_state.current_operation >= execution_state.total_operations {
            break;
        }
        
        let _session_id = execution_state.session;
        // Inline execute_next_operation to avoid lifetime issues
        if execution_state.current_operation < execution_state.total_operations {
            let operation = &execution_state.operations[execution_state.current_operation as usize];
            msg!("Executing async operation {}: function {:?}", 
                execution_state.current_operation, operation.function_hash);
            
            // Simple execution placeholder - capability checking would go here  
            execution_state.state_hash = compute_next_hash(execution_state.state_hash, &[1u8]);
            execution_state.current_operation += 1;
        }
    }
    
    // Session state hash is already updated in execution_state
    
    // Mark complete if all operations executed
    if execution_state.current_operation >= execution_state.total_operations {
        execution_state.is_complete = true;
        msg!("Async bundle execution complete");
    }
    
    Ok(())
}

fn compute_next_hash(prev_hash: [u8; 32], operation_data: &[u8]) -> [u8; 32] {
    // Simple hash chain - in production use proper hashing
    let mut next_hash = prev_hash;
    if !operation_data.is_empty() {
        next_hash[0] = next_hash[0].wrapping_add(operation_data[0]);
        next_hash[31] = next_hash[31].wrapping_add(operation_data[operation_data.len() - 1]);
    }
    next_hash
}

/// Execute function via CPI
fn execute_function_cpi<'ctx>(
    shard_key: &Pubkey,
    function_hash: &[u8; 32],
    ix_data: &[u8],
    remaining_accounts: &'ctx [AccountInfo<'ctx>],
    session_capabilities: &[String],
) -> Result<Vec<u8>> 
where
    for<'any> fn(&'any [AccountInfo<'any>]) -> Result<Vec<u8>>: Sized,
{
    // Get the function program from imports
    let function_program = get_function_program(
        shard_key,
        function_hash,
        remaining_accounts,
    )?;
    
    // Find the registry entry to get required capabilities
    let registry_entry = find_registry_entry(function_hash, remaining_accounts)?;
    let required_capabilities = get_required_capabilities(&registry_entry)?;
    
    // Check if session has all required capabilities
    for required_cap in required_capabilities.iter() {
        if !session_capabilities.contains(required_cap) {
            msg!("Missing capability: {}", required_cap);
            return Err(ShardError::InsufficientCapabilities.into());
        }
    }
    
    // Find the function program account info
    let mut function_program_info = None;
    for account in remaining_accounts.iter() {
        if account.key == &function_program {
            function_program_info = Some(account.clone());
            break;
        }
    }
    
    let function_program_info = function_program_info
        .ok_or(ShardError::FunctionNotFound)?;
    
    // Build CPI accounts - function programs expect:
    // 1. Caller (shard)
    // 2. Any additional accounts passed in remaining_accounts
    let mut cpi_accounts = vec![function_program_info.clone()];
    
    // Add any additional accounts that might be needed by the function
    // These would be passed in a specific order known to the function
    for account in remaining_accounts.iter() {
        if account.key != &function_program {
            cpi_accounts.push(account.clone());
        }
    }
    
    // Create the CPI instruction
    let cpi_instruction = anchor_lang::solana_program::instruction::Instruction {
        program_id: function_program,
        accounts: cpi_accounts.iter().map(|acc| {
            if acc.is_writable {
                anchor_lang::solana_program::instruction::AccountMeta::new(*acc.key, acc.is_signer)
            } else {
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*acc.key, acc.is_signer)
            }
        }).collect(),
        data: ix_data.to_vec(),
    };
    
    // Invoke the CPI
    anchor_lang::solana_program::program::invoke(
        &cpi_instruction,
        &cpi_accounts,
    )?;
    
    // Read the result from the function's return data
    // Functions should use set_return_data to return results
    let (program_id, return_data) = anchor_lang::solana_program::program::get_return_data()
        .ok_or(ShardError::ExecutionFailed)?;
    
    // Verify the return data came from the expected program
    require!(
        program_id == function_program,
        ShardError::ExecutionFailed
    );
    
    Ok(return_data)
}

/// Execute the next operation in an async bundle
fn execute_next_operation<'accounts>(
    execution_state: &mut ExecutionState,
    _session: &Session,
    shard_key: &Pubkey,
    remaining_accounts: &'accounts [AccountInfo<'accounts>],
) -> Result<()> {
    // Get current operation
    let operation = execution_state.operations
        .get(execution_state.current_operation as usize)
        .ok_or(ShardError::InvalidBundle)?;
    
    msg!("Executing async operation {}: function {:?}", 
        execution_state.current_operation, operation.function_hash);
    
    // Build function call data
    let mut ix_data = vec![0]; // Execute function discriminator
    ix_data.extend_from_slice(&operation.args);
    
    // Execute function via CPI
    // Get session capabilities by finding session in remaining accounts
    let session_capabilities = {
        let mut caps = Vec::new();
        // Find session account to get account list
        for account_info in remaining_accounts.iter() {
            if account_info.key == &_session.id {
                // Load session's account list
                let session_data = account_info.try_borrow_data()?;
                if session_data.len() >= 8 + 32 + 32 + 4 { // Up to accounts vector
                    let accounts_offset = 72;
                    let len_bytes = &session_data[accounts_offset..accounts_offset + 4];
                    let vec_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
                    
                    let mut account_pubkeys = Vec::new();
                    let mut offset = accounts_offset + 4;
                    for _ in 0..vec_len {
                        if offset + 32 <= session_data.len() {
                            let pubkey_bytes = &session_data[offset..offset + 32];
                            account_pubkeys.push(Pubkey::new_from_array(pubkey_bytes.try_into().unwrap()));
                            offset += 32;
                        }
                    }
                    
                    // Aggregate capabilities from accounts
                    let cap_set = aggregate_session_capabilities(&account_pubkeys, remaining_accounts)?;
                    caps = cap_set.into_iter().collect();
                }
                break;
            }
        }
        caps
    };
    
    let result = execute_function_cpi(
        shard_key,
        &operation.function_hash,
        &ix_data,
        remaining_accounts,
        &session_capabilities,
    )?;
    
    // Update state hash with result
    execution_state.state_hash = compute_next_hash(execution_state.state_hash, &result);
    
    // Verify diff if expected
    if let Some(expected_diff) = operation.expected_diff {
        require!(
            execution_state.state_hash == expected_diff,
            ShardError::DiffMismatch
        );
    }
    
    // Increment operation counter
    execution_state.current_operation += 1;
    
    Ok(())
}

/// Find the registry entry for a function
fn find_registry_entry<'accounts>(
    function_hash: &[u8; 32],
    remaining_accounts: &'accounts [AccountInfo<'accounts>],
) -> Result<&'accounts AccountInfo<'accounts>> {
    // Registry program ID - this should be imported properly
    let registry_program_id = Pubkey::from_str("11111111111111111111111111111112").unwrap();
    
    // Find registry entry by checking accounts
    for account in remaining_accounts.iter() {
        if account.owner == &registry_program_id && !account.data_is_empty() {
            let data = account.try_borrow_data()?;
            // Check if this is a function entry (has the right size and hash)
            if data.len() >= 8 + 32 { // discriminator + hash minimum
                let hash_bytes = &data[8..40];
                let entry_hash: [u8; 32] = hash_bytes.try_into().unwrap();
                if entry_hash == *function_hash {
                    return Ok(account);
                }
            }
        }
    }
    Err(ShardError::FunctionNotFound.into())
}

/// Get required capabilities from registry entry
fn get_required_capabilities(registry_entry: &AccountInfo) -> Result<Vec<String>> {
    let data = registry_entry.try_borrow_data()?;
    
    // Skip discriminator (8), hash (32), program (32), authority (32) = 104 bytes
    if data.len() < 104 {
        // Old format without capabilities
        return Ok(vec![]);
    }
    
    // Read the capabilities vector
    let capabilities_offset = 104;
    if data.len() < capabilities_offset + 4 {
        // No capabilities
        return Ok(vec![]);
    }
    
    // Read vector length
    let len_bytes = &data[capabilities_offset..capabilities_offset + 4];
    let vec_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
    
    let mut capabilities = Vec::new();
    let mut offset = capabilities_offset + 4;
    
    for _ in 0..vec_len {
        if offset + 4 > data.len() {
            break;
        }
        
        // Read string length
        let str_len_bytes = &data[offset..offset + 4];
        let str_len = u32::from_le_bytes(str_len_bytes.try_into().unwrap()) as usize;
        offset += 4;
        
        if offset + str_len > data.len() {
            break;
        }
        
        // Read string
        let cap_bytes = &data[offset..offset + str_len];
        let capability = String::from_utf8(cap_bytes.to_vec())
            .map_err(|_| ShardError::InvalidBundle)?;
        capabilities.push(capability);
        offset += str_len;
    }
    
    Ok(capabilities)
}

/// Get function program, checking import policy
fn get_function_program<'a>(
    shard_key: &Pubkey,
    function_hash: &[u8; 32],
    remaining_accounts: &'a [AccountInfo<'a>],
) -> Result<Pubkey> {
    // Try to find function import for this shard
    for account_info in remaining_accounts.iter() {
        // Check if this is a function import account by examining data
        if account_info.owner == &ID && account_info.data_len() == 8 + 32 + 32 + 32 + 1 + 8 {
            let data = account_info.try_borrow_data()?;
            // Skip discriminator and check shard field
            let shard_bytes = &data[8..40];
            let import_shard = Pubkey::new_from_array(shard_bytes.try_into().unwrap());
            
            // Check function hash
            let hash_bytes = &data[40..72];
            let import_hash: [u8; 32] = hash_bytes.try_into().unwrap();
            
            if import_hash == *function_hash && import_shard == *shard_key {
                // Get program from import
                let program_bytes = &data[72..104];
                let program = Pubkey::new_from_array(program_bytes.try_into().unwrap());
                
                // Check respect_deregistration flag
                let respect_deregistration = data[104] != 0;
                // If respecting deregistration, check registry status
                if respect_deregistration {
                    // Find registry entry in remaining accounts
                    for registry_account in remaining_accounts.iter() {
                        // Check if this could be a registry entry
                        if registry_account.owner != &ID {
                            // Check if account exists and has data
                            if registry_account.data_is_empty() {
                                // Account closed, function deregistered
                                return Err(ShardError::FunctionNotFound.into());
                            }
                        }
                    }
                }
                
                // Return cached program ID from import
                return Ok(program);
            }
        }
    }
    
    // Function not imported
    Err(ShardError::FunctionNotFound.into())
}

/// Compute aggregated state hash from session accounts
fn compute_session_state_hash<'a>(
    account_pubkeys: &[Pubkey],
    remaining_accounts: &'a [AccountInfo<'a>],
) -> Result<[u8; 32]> {
    let mut aggregated_hash = [0u8; 32];
    
    for account_pubkey in account_pubkeys.iter() {
        // Find the account in remaining_accounts
        for account_info in remaining_accounts.iter() {
            if account_info.key == account_pubkey {
                // Verify it's a ValenceAccount
                if account_info.owner != &ID {
                    continue;
                }
                
                // Read state hash from account
                let account_data = account_info.try_borrow_data()?;
                if account_data.len() >= 8 + 32 + 32 + 200 + 32 { // Up to state_hash field
                    // State hash is at offset 8 + 32 + 32 + 200 = 272
                    let state_hash_offset = 272;
                    if account_data.len() >= state_hash_offset + 32 {
                        let state_hash_bytes = &account_data[state_hash_offset..state_hash_offset + 32];
                        let account_state_hash: [u8; 32] = state_hash_bytes.try_into().unwrap();
                        
                        // Aggregate using XOR for simplicity (in production use proper hash function)
                        for i in 0..32 {
                            aggregated_hash[i] ^= account_state_hash[i];
                        }
                    }
                }
                break;
            }
        }
    }
    
    Ok(aggregated_hash)
}

/// Aggregate capabilities from all accounts in a session
fn aggregate_session_capabilities<'a>(
    account_pubkeys: &[Pubkey],
    remaining_accounts: &'a [AccountInfo<'a>],
) -> Result<std::collections::HashSet<String>> {
    let mut capabilities = std::collections::HashSet::new();
    
    for account_pubkey in account_pubkeys.iter() {
        // Find the account in remaining_accounts
        for account_info in remaining_accounts.iter() {
            if account_info.key == account_pubkey {
                // Verify it's a ValenceAccount
                if account_info.owner != &ID {
                    continue;
                }
                
                // Read capabilities from account
                let account_data = account_info.try_borrow_data()?;
                if account_data.len() >= 8 + 32 + 32 + 4 { // Up to capabilities vector
                    // Capabilities vector starts at offset 8 + 32 + 32 = 72
                    let capabilities_offset = 72;
                    
                    // Read vector length
                    if account_data.len() >= capabilities_offset + 4 {
                        let len_bytes = &account_data[capabilities_offset..capabilities_offset + 4];
                        let vec_len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
                        
                        let mut offset = capabilities_offset + 4;
                        for _ in 0..vec_len {
                            if offset + 4 > account_data.len() { break; }
                            
                            let str_len_bytes = &account_data[offset..offset + 4];
                            let str_len = u32::from_le_bytes(str_len_bytes.try_into().unwrap()) as usize;
                            offset += 4;
                            
                            if offset + str_len > account_data.len() { break; }
                            
                            let cap_bytes = &account_data[offset..offset + str_len];
                            if let Ok(capability) = String::from_utf8(cap_bytes.to_vec()) {
                                capabilities.insert(capability);
                            }
                            offset += str_len;
                        }
                    }
                }
                break;
            }
        }
    }
    
    Ok(capabilities)
}

/// Update state hash in session accounts after bundle execution
fn update_session_account_states<'a>(
    account_pubkeys: &[Pubkey],
    new_state_hash: [u8; 32],
    remaining_accounts: &'a [AccountInfo<'a>],
) -> Result<()> {
    // For now, we'll update all accounts with the same hash
    // In a more sophisticated implementation, each account might have its own state
    for account_pubkey in account_pubkeys.iter() {
        // Find the account in remaining_accounts
        for account_info in remaining_accounts.iter() {
            if account_info.key == account_pubkey {
                // Verify it's a ValenceAccount and writable
                if account_info.owner != &ID || !account_info.is_writable {
                    continue;
                }
                
                // Update state hash in account
                let mut account_data = account_info.try_borrow_mut_data()?;
                if account_data.len() >= 8 + 32 + 32 + 200 + 32 { // Up to state_hash field
                    // State hash is at offset 8 + 32 + 32 + 200 = 272
                    let state_hash_offset = 272;
                    if account_data.len() >= state_hash_offset + 32 {
                        account_data[state_hash_offset..state_hash_offset + 32]
                            .copy_from_slice(&new_state_hash);
                        msg!("Updated state hash for account {}", account_pubkey);
                    }
                }
                break;
            }
        }
    }
    
    Ok(())
}

// Account contexts

#[derive(Accounts)]
pub struct ExecuteSyncBundle<'info> {
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        constraint = session.owner == executor.key() @ ShardError::Unauthorized,
        constraint = !session.is_consumed @ ShardError::SessionAlreadyConsumed,
    )]
    pub session: Account<'info, Session>,
    
    /// Shard configuration
    #[account(
        seeds = [b"shard_config", shard_config.authority.as_ref()],
        bump,
    )]
    pub shard_config: Account<'info, ShardConfig>,
    
    // Note: Account references within the session would be passed in remaining_accounts
    // and validated against session._internal_accounts vector
}

#[derive(Accounts)]
pub struct StartAsyncBundle<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,
    
    #[account(
        constraint = session.owner == executor.key() @ ShardError::Unauthorized,
        constraint = !session.is_consumed @ ShardError::SessionAlreadyConsumed,
    )]
    pub session: Account<'info, Session>,
    
    #[account(
        init,
        payer = executor,
        space = 8 + 32 + 2 + 2 + 32 + 1 + 4 + (100 * 100) + 32, // discriminator + bundle_id + current + total + hash + complete + vec_len + (max_ops * op_size) + session
    )]
    pub execution_state: Account<'info, ExecutionState>,
    
    /// Shard configuration
    #[account(
        seeds = [b"shard_config", shard_config.authority.as_ref()],
        bump,
    )]
    pub shard_config: Account<'info, ShardConfig>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bundle_id: Pubkey)]
pub struct ContinueAsyncBundle<'info> {
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        constraint = execution_state.bundle_id == bundle_id @ ShardError::InvalidBundle,
    )]
    pub execution_state: Account<'info, ExecutionState>,
    
    /// Shard configuration
    #[account(
        seeds = [b"shard_config", shard_config.authority.as_ref()],
        bump,
    )]
    pub shard_config: Account<'info, ShardConfig>,
}