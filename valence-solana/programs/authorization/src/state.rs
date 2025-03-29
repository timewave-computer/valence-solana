use anchor_lang::prelude::*;

/// Permission type for an authorization
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum PermissionType {
    /// Anyone can use this authorization
    Public,
    /// Only the owner can use this authorization
    OwnerOnly,
    /// Only specified users can use this authorization
    Allowlist,
}

/// Priority level for messages
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum Priority {
    /// Low priority, processed last
    Low,
    /// Medium priority, processed after high
    Medium,
    /// High priority, processed first
    High,
}

/// Subroutine execution type
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum SubroutineType {
    /// Atomic execution - all messages must succeed
    Atomic,
    /// Non-atomic execution - messages can fail individually
    NonAtomic,
}

/// Result of execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum ExecutionResult {
    /// Execution succeeded
    Success,
    /// Execution failed
    Failure,
}

/// Message to be processed
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProcessorMessage {
    /// Program ID to call
    pub program_id: Pubkey,
    /// Instruction data
    pub data: Vec<u8>,
    /// Account metas
    pub accounts: Vec<AccountMetaData>,
}

/// Account meta data for cross-program invocations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AccountMetaData {
    /// Account pubkey
    pub pubkey: Pubkey,
    /// Is signer
    pub is_signer: bool,
    /// Is writable
    pub is_writable: bool,
}

/// Authorization Program state
#[account]
pub struct AuthorizationState {
    /// Program owner
    pub owner: Pubkey,
    /// Secondary authorities
    pub sub_owners: Vec<Pubkey>,
    /// Processor program ID
    pub processor_program_id: Pubkey,
    /// Unique ID for executions
    pub execution_counter: u64,
    /// Address of the Valence Registry
    pub valence_registry: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Authorization
#[account]
pub struct Authorization {
    /// Unique identifier
    pub label: String,
    /// Owner of this authorization
    pub owner: Pubkey,
    /// Whether authorization is active
    pub is_active: bool,
    /// Who can use this authorization
    pub permission_type: PermissionType,
    /// If permission type is allowlist
    pub allowed_users: Vec<Pubkey>,
    /// Earliest valid timestamp
    pub not_before: i64,
    /// Expiration timestamp
    pub expiration: Option<i64>,
    /// Concurrent execution limit
    pub max_concurrent_executions: u32,
    /// Message priority level
    pub priority: Priority,
    /// Atomic or NonAtomic execution
    pub subroutine_type: SubroutineType,
    /// Current number of in-flight executions
    pub current_executions: u32,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Current execution tracking
#[account]
pub struct CurrentExecution {
    /// Unique execution ID
    pub id: u64,
    /// Related authorization
    pub authorization_label: String,
    /// Transaction initiator
    pub sender: Pubkey,
    /// Start timestamp
    pub start_time: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Instruction context for initializing the authorization program
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The program state account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<AuthorizationState>() + 32 * 10, // Extra space for future sub_owners
        seeds = [b"authorization_state".as_ref()],
        bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The account paying for the initialization
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for creating an authorization
#[derive(Accounts)]
#[instruction(label: String)]
pub struct CreateAuthorization<'info> {
    /// The program state account
    #[account(
        mut,
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
        constraint = authorization_state.owner == owner.key() || authorization_state.sub_owners.contains(&owner.key()) @ AuthorizationError::NotAuthorized,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The new authorization account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<Authorization>() + label.len() + 4 + 32 * 50, // Extra space for allowed_users
        seeds = [b"authorization".as_ref(), label.as_bytes()],
        bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    /// The authorization owner
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for modifying an authorization
#[derive(Accounts)]
pub struct ModifyAuthorization<'info> {
    /// The program state account
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The authorization to modify
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization.label.as_bytes()],
        bump = authorization.bump,
        constraint = authorization.owner == owner.key() || authorization_state.owner == owner.key() || authorization_state.sub_owners.contains(&owner.key()) @ AuthorizationError::NotAuthorized,
    )]
    pub authorization: Account<'info, Authorization>,
    
    /// The signer modifying the authorization
    pub owner: Signer<'info>,
}

/// Instruction context for disabling an authorization
#[derive(Accounts)]
pub struct DisableAuthorization<'info> {
    /// The program state account
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The authorization to disable
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization.label.as_bytes()],
        bump = authorization.bump,
        constraint = authorization.owner == owner.key() || authorization_state.owner == owner.key() || authorization_state.sub_owners.contains(&owner.key()) @ AuthorizationError::NotAuthorized,
    )]
    pub authorization: Account<'info, Authorization>,
    
    /// The signer disabling the authorization
    pub owner: Signer<'info>,
}

/// Instruction context for enabling an authorization
#[derive(Accounts)]
pub struct EnableAuthorization<'info> {
    /// The program state account
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The authorization to enable
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization.label.as_bytes()],
        bump = authorization.bump,
        constraint = authorization.owner == owner.key() || authorization_state.owner == owner.key() || authorization_state.sub_owners.contains(&owner.key()) @ AuthorizationError::NotAuthorized,
    )]
    pub authorization: Account<'info, Authorization>,
    
    /// The signer enabling the authorization
    pub owner: Signer<'info>,
}

/// Instruction context for sending messages
#[derive(Accounts)]
#[instruction(authorization_label: String)]
pub struct SendMessages<'info> {
    /// The program state account
    #[account(
        mut,
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The authorization being used
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), authorization_label.as_bytes()],
        bump,
        constraint = authorization.is_active @ AuthorizationError::AuthorizationDisabled,
        constraint = authorization.not_before <= Clock::get()?.unix_timestamp @ AuthorizationError::AuthorizationNotYetValid,
        constraint = match authorization.expiration {
            Some(expiration) => Clock::get()?.unix_timestamp < expiration,
            None => true
        } @ AuthorizationError::AuthorizationExpired,
        constraint = authorization.current_executions < authorization.max_concurrent_executions @ AuthorizationError::TooManyExecutions,
        constraint = match authorization.permission_type {
            PermissionType::Public => true,
            PermissionType::OwnerOnly => authorization.owner == sender.key(),
            PermissionType::Allowlist => authorization.allowed_users.contains(&sender.key()),
        } @ AuthorizationError::NotAuthorized,
    )]
    pub authorization: Account<'info, Authorization>,
    
    /// The new execution tracking account
    #[account(
        init,
        payer = sender,
        space = 8 + std::mem::size_of::<CurrentExecution>() + authorization_label.len() + 4,
        seeds = [b"execution".as_ref(), &authorization_state.execution_counter.to_le_bytes()],
        bump
    )]
    pub execution: Account<'info, CurrentExecution>,
    
    /// The account sending the messages
    #[account(mut)]
    pub sender: Signer<'info>,
    
    /// The processor program to forward messages to
    /// We verify this matches what's stored in the authorization state
    #[account(
        constraint = processor_program.key() == authorization_state.processor_program_id @ AuthorizationError::InvalidProcessorProgram,
    )]
    pub processor_program: Program<'info, crate::ID>,
    
    /// The Solana clock sysvar
    pub clock: Sysvar<'info, Clock>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for receiving a callback
#[derive(Accounts)]
#[instruction(execution_id: u64)]
pub struct ReceiveCallback<'info> {
    /// The program state account
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    /// The execution being completed
    #[account(
        mut,
        close = sender,
        seeds = [b"execution".as_ref(), &execution_id.to_le_bytes()],
        bump = execution.bump,
    )]
    pub execution: Account<'info, CurrentExecution>,
    
    /// The authorization account
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), execution.authorization_label.as_bytes()],
        bump,
        constraint = authorization.current_executions > 0 @ AuthorizationError::InvalidExecutionState,
    )]
    pub authorization: Account<'info, Authorization>,
    
    /// The processor program calling us
    #[account(
        constraint = processor_program.key() == authorization_state.processor_program_id @ AuthorizationError::InvalidProcessorProgram,
    )]
    pub processor_program: Signer<'info>,
    
    /// The original message sender, who will receive the closed execution account's rent
    /// We don't need to verify this as we stored it in the execution account
    #[account(mut)]
    pub sender: AccountInfo<'info>,
} 