use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization, CurrentExecution, ProcessorMessage, PermissionType};
use crate::error::AuthorizationError;
use crate::validation::Validator;

pub fn handler(
    ctx: Context<SendMessages>,
    authorization_label: String,
    messages: Vec<ProcessorMessage>,
) -> Result<()> {
    // Validate input parameters
    Validator::validate_label(&authorization_label)?;
    Validator::validate_message_batch(&messages)?;
    
    let auth = &mut ctx.accounts.authorization;
    let state = &mut ctx.accounts.authorization_state;
    
    // Check authorization status
    if !auth.is_active {
        return Err(AuthorizationError::AuthorizationInactive.into());
    }
    
    // Check timestamps
    let current_time = Clock::get()?.unix_timestamp;
    
    if current_time < auth.not_before {
        return Err(AuthorizationError::AuthorizationNotYetValid.into());
    }
    
    if let Some(exp) = auth.expiration {
        if current_time > exp {
            return Err(AuthorizationError::AuthorizationExpired.into());
        }
    }
    
    // Check permissions
    match auth.permission_type {
        PermissionType::Public => {},
        PermissionType::OwnerOnly => {
            if ctx.accounts.sender.key() != auth.owner {
                return Err(AuthorizationError::UnauthorizedSender.into());
            }
        },
        PermissionType::Allowlist => {
            if !auth.allowed_users.contains(&ctx.accounts.sender.key()) {
                return Err(AuthorizationError::UnauthorizedSender.into());
            }
        }
    }
    
    // Check concurrent executions
    if auth.current_executions >= auth.max_concurrent_executions {
        return Err(AuthorizationError::MaxConcurrentExecutionsReached.into());
    }
    
    // Generate execution ID and increment counter
    let execution_id = state.execution_counter;
    state.execution_counter = state.execution_counter.checked_add(1)
        .ok_or(AuthorizationError::InvalidParameters)?;
    
    // Create execution record
    let execution = &mut ctx.accounts.current_execution;
    execution.id = execution_id;
    execution.authorization_label = authorization_label.clone();
    execution.sender = ctx.accounts.sender.key();
    execution.start_time = current_time;
    execution.bump = ctx.bumps.current_execution;
    
    // Increment active executions counter
    auth.current_executions = auth.current_executions.checked_add(1)
        .ok_or(AuthorizationError::InvalidParameters)?;
    
    // For now, just log the messages (like CosmWasm validation pattern)
    msg!("Authorized {} messages for execution ID: {}", 
         messages.len(), execution_id);
    
    // In a full implementation, messages would be forwarded to processor
    // But following the canonical pattern, authorization just validates and tracks
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(authorization_label: String, messages: Vec<ProcessorMessage>)]
pub struct SendMessages<'info> {
    #[account(
        mut,
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization_label.as_bytes()],
        bump = authorization.bump,
        constraint = authorization_label.len() <= 32 @ AuthorizationError::InvalidParameters,
        constraint = !messages.is_empty() @ AuthorizationError::EmptyMessageBatch,
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        init,
        payer = sender,
        space = CurrentExecution::space(authorization_label.len()),
        seeds = [b"execution".as_ref(), &authorization_state.execution_counter.to_le_bytes()],
        bump
    )]
    pub current_execution: Account<'info, CurrentExecution>,
    
    #[account(mut)]
    pub sender: Signer<'info>,
    
    pub system_program: Program<'info, System>,
} 