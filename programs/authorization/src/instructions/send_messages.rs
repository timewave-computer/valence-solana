use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization, CurrentExecution, ProcessorMessage, Priority, SubroutineType, PermissionType};
use crate::error::AuthorizationError;
// use crate::validation::Validator; // Temporarily disabled
use valence_utils::{ComputeBudgetEstimator, ComputeBudgetManager, ComputeBudgetMonitor, estimate_compute_budget, TransactionSizeOptimizer, CompactSerialize};

pub fn handler(
    ctx: Context<SendMessages>,
    authorization_label: String,
    messages: Vec<ProcessorMessage>,
) -> Result<()> {
    // Estimate compute budget for this operation
    let account_count = 4; // authorization_state, authorization, current_execution, sender
    let cpi_count = 1; // CPI to processor program
    let estimated_compute = estimate_compute_budget!(basic: account_count, cpi_count);
    
    // Validate compute budget requirements
    ComputeBudgetManager::validate_compute_budget(estimated_compute, None)?;
    
    // Log compute budget estimation
    ComputeBudgetMonitor::log_compute_usage(
        "send_messages",
        estimated_compute,
        None,
    );
    
    // Validate transaction size constraints
    let estimated_instruction_size = 32 + authorization_label.len() + 
        messages.iter().map(|m| m.compact_size()).sum::<usize>();
    TransactionSizeOptimizer::validate_transaction_size(
        estimated_instruction_size,
        account_count as usize,
        1, // Single signature
    )?;
    
    // TODO: Re-enable validation once validation module is working
    // Validator::validate_label(&authorization_label)?;
    // Validator::validate_message_batch(&messages)?;
    
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
    
    // Create execution record with new String-based approach
    let execution = &mut ctx.accounts.current_execution;
    execution.id = execution_id;
    execution.authorization_label = authorization_label.clone();
    execution.sender = ctx.accounts.sender.key();
    execution.start_time = current_time;
    execution.bump = ctx.bumps.current_execution;
    
    // Increment active executions counter
    auth.current_executions = auth.current_executions.checked_add(1)
        .ok_or(AuthorizationError::InvalidParameters)?;
    
    // Forward messages to processor program via CPI
    msg!("Sending {} messages to processor with execution ID: {}", 
         messages.len(), execution_id);
         
    // Create the program address that we'll invoke via CPI
    let processor_program_id = state.processor_id;
    
    // Prepare the CPI call
    // Note: In a production implementation, you would need to build a proper CPI call
    // to the processor program with the messages. This is a placeholder for the actual implementation.
    
    // Sample code to perform the CPI:
    // let processor_program = ctx.accounts.processor_program.to_account_info();
    // let cpi_accounts = EnqueueMessages {
    //     processor_state: processor_state.to_account_info(),
    //     execution: execution.to_account_info(),
    //     sender: sender.to_account_info(),
    // };
    // let cpi_program = processor_program;
    // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    // processor::cpi::enqueue_messages(cpi_ctx, execution_id, auth.priority, auth.subroutine_type, messages)?;
    
    // For now we just log that we would send the messages
    msg!("CPI to processor would be performed here with execution ID: {}", execution_id);
    
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
        // Additional constraints for validation
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
    
    // Uncomment for actual processor CPI implementation
    // /// CHECK: This is the processor program that will be invoked via CPI
    // #[account(address = authorization_state.processor_id)]
    // pub processor_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
} 