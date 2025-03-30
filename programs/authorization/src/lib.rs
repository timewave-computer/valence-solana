use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod error;
pub mod state;

use error::AuthorizationError;
use state::*;

#[program]
pub mod authorization {
    use super::*;
    
    /// Initialize the authorization program with the owner, processor program ID, and registry.
    pub fn initialize(
        ctx: Context<Initialize>, 
        processor_id: Pubkey,
        registry_id: Pubkey
    ) -> Result<()> {
        msg!("Initializing authorization program");
        
        let state = &mut ctx.accounts.authorization_state;
        state.owner = ctx.accounts.owner.key();
        state.sub_owners = Vec::new();
        state.processor_program_id = processor_id;
        state.execution_counter = 0;
        state.valence_registry = registry_id;
        state.bump = *ctx.bumps.get("authorization_state").unwrap();
        
        Ok(())
    }
    
    /// Create a new authorization with the given parameters
    pub fn create_authorization(
        ctx: Context<CreateAuthorization>,
        label: String,
        permission_type: PermissionType,
        allowed_users: Option<Vec<Pubkey>>,
        not_before: i64,
        expiration: Option<i64>,
        max_concurrent_executions: u32,
        priority: Priority,
        subroutine_type: SubroutineType,
    ) -> Result<()> {
        // Validate parameters
        if label.is_empty() || label.len() > 32 {
            return Err(AuthorizationError::InvalidParameters.into());
        }

        if let Some(exp) = expiration {
            if exp <= not_before {
                return Err(AuthorizationError::InvalidParameters.into());
            }
        }
        
        let auth = &mut ctx.accounts.authorization;
        
        // Initialize authorization using the new method
        auth.set_label(&label);
        auth.owner = ctx.accounts.owner.key();
        auth.is_active = true;
        auth.permission_type = permission_type;
        auth.allowed_users = allowed_users.unwrap_or_default();
        auth.not_before = not_before;
        auth.expiration = expiration;
        auth.max_concurrent_executions = max_concurrent_executions;
        auth.priority = priority;
        auth.subroutine_type = subroutine_type;
        auth.current_executions = 0;
        auth.bump = *ctx.bumps.get("authorization").unwrap();
        
        msg!("Created new authorization with label: {}", label);
        
        Ok(())
    }
    
    /// Modify an existing authorization
    pub fn modify_authorization(
        ctx: Context<ModifyAuthorization>,
        permission_type: Option<PermissionType>,
        allowed_users: Option<Vec<Pubkey>>,
        not_before: Option<i64>,
        expiration: Option<Option<i64>>,
        max_concurrent_executions: Option<u32>,
        priority: Option<Priority>,
        subroutine_type: Option<SubroutineType>,
    ) -> Result<()> {
        let auth = &mut ctx.accounts.authorization;
        
        // Update fields if provided
        if let Some(ptype) = permission_type {
            auth.permission_type = ptype;
        }
        
        if let Some(users) = allowed_users {
            auth.allowed_users = users;
        }
        
        if let Some(time) = not_before {
            auth.not_before = time;
        }
        
        if let Some(exp) = expiration {
            auth.expiration = exp;
        }
        
        if let Some(max) = max_concurrent_executions {
            auth.max_concurrent_executions = max;
        }
        
        if let Some(pri) = priority {
            auth.priority = pri;
        }
        
        if let Some(sub) = subroutine_type {
            auth.subroutine_type = sub;
        }
        
        // Validate updated state
        if let Some(exp) = auth.expiration {
            if exp <= auth.not_before {
                return Err(AuthorizationError::InvalidParameters.into());
            }
        }
        
        msg!("Modified authorization: {}", auth.get_label());
        
        Ok(())
    }
    
    /// Disable an authorization
    pub fn disable_authorization(ctx: Context<DisableAuthorization>) -> Result<()> {
        ctx.accounts.authorization.is_active = false;
        msg!("Disabled authorization: {}", ctx.accounts.authorization.get_label());
        Ok(())
    }
    
    /// Enable an authorization
    pub fn enable_authorization(ctx: Context<EnableAuthorization>) -> Result<()> {
        ctx.accounts.authorization.is_active = true;
        msg!("Enabled authorization: {}", ctx.accounts.authorization.get_label());
        Ok(())
    }
    
    /// Send messages using an authorization
    pub fn send_messages(
        ctx: Context<SendMessages>,
        authorization_label: String,
        messages: Vec<ProcessorMessage>,
    ) -> Result<()> {
        // Validate parameters
        if messages.is_empty() {
            return Err(AuthorizationError::EmptyMessageBatch.into());
        }
        
        // Validate authorization label length
        if authorization_label.is_empty() || authorization_label.len() > 32 {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
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
        
        // Create execution record with new method
        let execution = &mut ctx.accounts.current_execution;
        execution.id = execution_id;
        execution.set_authorization_label(&authorization_label);
        execution.sender = ctx.accounts.sender.key();
        execution.start_time = current_time;
        execution.bump = *ctx.bumps.get("current_execution").unwrap();
        
        // Increment active executions counter
        auth.current_executions = auth.current_executions.checked_add(1)
            .ok_or(AuthorizationError::InvalidParameters)?;
        
        // Forward messages to processor program via CPI
        msg!("Sending {} messages to processor with execution ID: {}", 
             messages.len(), execution_id);
             
        // For now we just log that we would send the messages
        msg!("CPI to processor would be performed here with execution ID: {}", execution_id);
        
        Ok(())
    }
    
    /// Process callback from the Processor Program
    pub fn receive_callback(
        ctx: Context<ReceiveCallback>,
        execution_id: u64,
        result: ExecutionResult,
        executed_count: u32,
        error_data: Option<Vec<u8>>,
    ) -> Result<()> {
        // Verify callback is from processor program
        if ctx.accounts.processor_program.key() != ctx.accounts.authorization_state.processor_program_id {
            return Err(AuthorizationError::UnauthorizedCallback.into());
        }
        
        // Verify execution ID matches
        if ctx.accounts.current_execution.id != execution_id {
            return Err(AuthorizationError::ExecutionNotFound.into());
        }
        
        // Lookup authorization
        let auth = &mut ctx.accounts.authorization;
        
        // Decrement active executions counter
        auth.current_executions = auth.current_executions.checked_sub(1)
            .unwrap_or(0);
        
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
    
    /// Lookup an authorization by label and return its address
    pub fn lookup_authorization(
        ctx: Context<LookupAuthorization>,
        label: String,
    ) -> Result<Pubkey> {
        // Validate label length
        if label.is_empty() || label.len() > 32 {
            return Err(AuthorizationError::InvalidParameters.into());
        }
        
        // Compute the authorization PDA
        let (auth_pda, _) = Pubkey::find_program_address(
            &[b"authorization".as_ref(), label.as_bytes()],
            ctx.program_id
        );
        
        msg!("Found authorization at: {}", auth_pda);
        
        Ok(auth_pda)
    }
}

// Include all the account validation structs at the end of the file
// This is an alternative to using the instructions module

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<AuthorizationState>() + 32 * 10, // Extra space for sub_owners
        seeds = [b"authorization_state".as_ref()],
        bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(label: String)]
pub struct CreateAuthorization<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<Authorization>() + 32 * 50, // Extra space for allowed_users
        seeds = [b"authorization".as_ref(), label.as_bytes()],
        bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        mut,
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(mut, constraint = owner.key() == authorization_state.owner || authorization_state.sub_owners.contains(&owner.key()))]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModifyAuthorization<'info> {
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization.get_label().as_bytes()],
        bump = authorization.bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(constraint = owner.key() == authorization.owner || authorization_state.sub_owners.contains(&owner.key()))]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct DisableAuthorization<'info> {
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization.get_label().as_bytes()],
        bump = authorization.bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(constraint = owner.key() == authorization.owner || authorization_state.sub_owners.contains(&owner.key()))]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct EnableAuthorization<'info> {
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization.get_label().as_bytes()],
        bump = authorization.bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(constraint = owner.key() == authorization.owner || authorization_state.sub_owners.contains(&owner.key()))]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(authorization_label: String, messages: Vec<ProcessorMessage>)]
pub struct SendMessages<'info> {
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization_label.as_bytes()],
        bump = authorization.bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        mut,
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(
        init,
        payer = sender,
        space = 8 + std::mem::size_of::<CurrentExecution>(),
        seeds = [
            b"execution".as_ref(),
            authorization_state.execution_counter.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub current_execution: Account<'info, CurrentExecution>,
    
    #[account(mut)]
    pub sender: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(execution_id: u64)]
pub struct ReceiveCallback<'info> {
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization.get_label().as_bytes()],
        bump = authorization.bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(
        mut,
        close = sender,
        seeds = [
            b"execution".as_ref(),
            execution_id.to_le_bytes().as_ref()
        ],
        bump = current_execution.bump
    )]
    pub current_execution: Account<'info, CurrentExecution>,
    
    /// CHECK: We verify this account is the processor program in the handler
    pub processor_program: UncheckedAccount<'info>,
    
    /// The original sender to receive lamports when closing the account
    /// CHECK: Address is verified by the constraint
    #[account(mut, address = current_execution.sender)]
    pub sender: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct LookupAuthorization<'info> {
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
} 