use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization, CurrentExecution, ExecutionResult};
use crate::error::AuthorizationError;

pub fn handler(
    ctx: Context<ReceiveCallback>,
    execution_id: u64,
    result: ExecutionResult,
    executed_count: u32,
    error_data: Option<Vec<u8>>,
) -> Result<()> {
    // Verify callback is from processor program
    if ctx.accounts.processor_program.key() != ctx.accounts.authorization_state.processor_id {
        return Err(AuthorizationError::UnauthorizedCallback.into());
    }
    
    // Verify execution ID matches
    if ctx.accounts.current_execution.id != execution_id {
        return Err(AuthorizationError::ExecutionNotFound.into());
    }
    
    // Lookup authorization
    let auth = &mut ctx.accounts.authorization;
    
    // Decrement active executions counter
    auth.current_executions = auth.current_executions.saturating_sub(1);
    
    // Log result
    match result {
        ExecutionResult::Success => {
            msg!("Execution {} completed successfully. Executed {} messages", 
                execution_id, executed_count);
        },
        ExecutionResult::Failure => {
            msg!("Execution {} failed after executing {} messages", 
                execution_id, executed_count);
            
            if let Some(error) = error_data {
                msg!("Error data: {:?}", error);
            }
        }
    }
    
    // The current_execution account is closed in the account validation
    // which returns the lamports back to the original sender
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(execution_id: u64, result: ExecutionResult, executed_count: u32, error_data: Option<Vec<u8>>)]
pub struct ReceiveCallback<'info> {
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), current_execution.authorization_label.as_bytes()],
        bump = authorization.bump,
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        mut,
        seeds = [b"execution".as_ref(), &current_execution.id.to_le_bytes()],
        bump = current_execution.bump,
        close = sender
    )]
    pub current_execution: Account<'info, CurrentExecution>,
    
    /// CHECK: The processor program - must match the one in authorization state
    pub processor_program: UncheckedAccount<'info>,
    
    /// CHECK: The original sender of the messages, who will receive lamports back
    #[account(mut, address = current_execution.sender)]
    pub sender: UncheckedAccount<'info>,
} 