use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AuthorizationError;
use crate::cache::{AuthorizationCache, helpers};

// Use the same thread-local storage from send_messages.rs
thread_local! {
    static AUTHORIZATION_CACHE: RefCell<AuthorizationCache> = RefCell::new(AuthorizationCache::new());
}

pub fn handler(
    ctx: Context<ReceiveCallback>,
    execution_id: u64,
    result: ExecutionResult,
    executed_count: u32,
    error_data: Option<Vec<u8>>,
) -> Result<()> {
    // Verify execution ID matches
    if ctx.accounts.execution.id != execution_id {
        return Err(error!(AuthorizationError::ExecutionNotFound));
    }
    
    // Get authorization account
    let authorization = &mut ctx.accounts.authorization;
    
    // Ensure we have executions to reduce
    if authorization.current_executions == 0 {
        return Err(error!(AuthorizationError::InvalidExecutionState));
    }
    
    // Update cache with the current authorization for future lookups
    AUTHORIZATION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.add_authorization(&authorization);
    });
    
    // Decrement current executions count
    authorization.current_executions = authorization.current_executions.saturating_sub(1);
    
    // Log execution result
    match result {
        ExecutionResult::Success => {
            msg!(
                "Execution {} for authorization {} completed successfully. {} messages executed.",
                execution_id,
                ctx.accounts.execution.authorization_label,
                executed_count
            );
        }
        ExecutionResult::Failure => {
            msg!(
                "Execution {} for authorization {} failed. {} messages executed before failure.",
                execution_id,
                ctx.accounts.execution.authorization_label,
                executed_count
            );
            
            // Log error data if present
            if let Some(error_data) = error_data {
                if !error_data.is_empty() {
                    msg!("Error data: {:?}", error_data);
                }
            }
        }
    }
    
    // Note: The execution account is closed in the account validation, 
    // and the rent is returned to the original sender
    
    Ok(())
} 