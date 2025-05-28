// Authorization Program for Valence Protocol
// Manages user authorizations and permissions for protocol execution

#![allow(clippy::too_many_arguments)]

use anchor_lang::prelude::*;

declare_id!("AUTHwqL7qxgqC8mT5kXxVPPbgU3eFFuwNJ5CKzrVR8Ed");

pub mod error;
pub mod state;
pub mod instructions;
pub mod validation;

use state::*;
use error::*;

#[program]
#[allow(clippy::too_many_arguments)]
pub mod authorization {
    use super::*;

    /// Initialize the authorization system
    pub fn initialize(
        ctx: Context<Initialize>,
        processor_program_id: Pubkey,
        registry_program_id: Pubkey,
    ) -> Result<()> {
        let state = &mut ctx.accounts.authorization_state;
        state.owner = ctx.accounts.owner.key();
        state.sub_owners = Vec::new();
        state.processor_id = processor_program_id;
        state.registry_id = registry_program_id;
        state.execution_counter = 0;
        state.last_zk_sequence = 0;
        state.zk_sequence_counter = 0;
        state.bump = ctx.bumps.authorization_state;
        state.reserved = [0; 64];
        
        msg!("Authorization system initialized");
        Ok(())
    }

    /// Create a new authorization
    #[allow(clippy::too_many_arguments)]
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
        // Validate authorization creation parameters
        validation::Validator::validate_authorization_creation(
            &label,
            &allowed_users,
            not_before,
            expiration,
            max_concurrent_executions,
        )?;
        
        let auth = &mut ctx.accounts.authorization;
        
        // Initialize authorization
        auth.label = label.clone();
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
        auth.bump = ctx.bumps.authorization;
        
        msg!("Created new authorization with label: {}", label);
        Ok(())
    }

    /// Send messages for execution
    pub fn send_messages(
        ctx: Context<SendMessages>,
        authorization_label: String,
        messages: Vec<ProcessorMessage>,
    ) -> Result<()> {
        let auth = &mut ctx.accounts.authorization;
        let state = &mut ctx.accounts.authorization_state;
        
        // Basic validation
        if !auth.is_active {
            return Err(AuthorizationError::AuthorizationInactive.into());
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
        
        // Generate execution ID
        let execution_id = state.execution_counter;
        state.execution_counter = state.execution_counter.checked_add(1)
            .ok_or(AuthorizationError::InvalidParameters)?;
        
        // Create execution record
        let execution = &mut ctx.accounts.current_execution;
        execution.id = execution_id;
        execution.authorization_label = authorization_label.clone();
        execution.sender = ctx.accounts.sender.key();
        execution.start_time = Clock::get()?.unix_timestamp;
        execution.bump = ctx.bumps.current_execution;
        
        // Increment active executions
        auth.current_executions = auth.current_executions.checked_add(1)
            .ok_or(AuthorizationError::InvalidParameters)?;
        
        msg!("Sending {} messages with execution ID: {}", messages.len(), execution_id);
        Ok(())
    }

    /// Receive callback from processor
    pub fn receive_callback(
        ctx: Context<ReceiveCallback>,
        execution_id: u64,
        result: ExecutionResult,
        _executed_count: u32,
        _error_data: Option<Vec<u8>>,
    ) -> Result<()> {
        let auth = &mut ctx.accounts.authorization;
        
        // Decrement active executions
        auth.current_executions = auth.current_executions.saturating_sub(1);
        
        msg!("Received callback for execution {}: {:?}", execution_id, result);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        space = AuthorizationState::space(0),
        seeds = [b"authorization_state"],
        bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    label: String,
    permission_type: PermissionType,
    allowed_users: Option<Vec<Pubkey>>,
    not_before: i64,
    expiration: Option<i64>,
    max_concurrent_executions: u32,
    priority: Priority,
    subroutine_type: SubroutineType
)]
pub struct CreateAuthorization<'info> {
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
        constraint = authorization_state.owner == owner.key() || authorization_state.sub_owners.contains(&owner.key()) @ AuthorizationError::NotAuthorized,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(
        init,
        payer = owner,
        space = Authorization::space(label.len(), allowed_users.as_ref().map_or(0, |v| v.len())),
        seeds = [b"authorization".as_ref(), label.as_bytes()],
        bump,
        // Additional constraints for validation
        constraint = label.len() <= 32 @ AuthorizationError::InvalidParameters,
        constraint = allowed_users.as_ref().map_or(0, |v| v.len()) <= 100 @ AuthorizationError::InvalidParameters,
        constraint = max_concurrent_executions > 0 && max_concurrent_executions <= 1000 @ AuthorizationError::InvalidParameters,
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(authorization_label: String, messages: Vec<ProcessorMessage>)]
pub struct SendMessages<'info> {
    #[account(
        mut,
        seeds = [b"authorization_state"],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(
        mut,
        seeds = [b"authorization", authorization_label.as_bytes()],
        bump = authorization.bump,
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(
        init,
        payer = sender,
        space = CurrentExecution::space(authorization_label.len()),
        seeds = [b"execution", authorization_state.execution_counter.to_le_bytes().as_ref()],
        bump
    )]
    pub current_execution: Account<'info, CurrentExecution>,
    
    #[account(mut)]
    pub sender: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReceiveCallback<'info> {
    #[account(
        mut,
        seeds = [b"authorization", authorization.label.as_bytes()],
        bump = authorization.bump,
    )]
    pub authorization: Account<'info, Authorization>,
    
    pub sender: Signer<'info>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_permission_type_variants() {
        let public = PermissionType::Public;
        let owner_only = PermissionType::OwnerOnly;
        let allowlist = PermissionType::Allowlist;

        assert_eq!(public, PermissionType::Public);
        assert_eq!(owner_only, PermissionType::OwnerOnly);
        assert_eq!(allowlist, PermissionType::Allowlist);
        assert_ne!(public, owner_only);
    }

    #[test]
    fn test_priority_levels() {
        let high = Priority::High;
        let medium = Priority::Medium;
        let low = Priority::Low;

        assert_eq!(high, Priority::High);
        assert_eq!(medium, Priority::Medium);
        assert_eq!(low, Priority::Low);
        assert_ne!(high, medium);
    }

    #[test]
    fn test_authorization_space_calculation() {
        let space_no_users = Authorization::space(10, 0); // 10 char label, 0 users
        let space_with_users = Authorization::space(10, 5); // 10 char label, 5 users
        let space_max_label = Authorization::space(32, 10); // 32 char label, 10 users

        // Base size should be reasonable
        assert!(space_no_users > 0);
        // More users should require more space
        assert!(space_with_users > space_no_users);
        assert!(space_max_label > space_with_users);
        
        // Each user adds 32 bytes (Pubkey)
        let expected_diff = 5 * 32;
        let actual_diff = space_with_users - space_no_users;
        // The actual difference might be larger due to Vec overhead and alignment
        assert!(actual_diff >= expected_diff);
    }

    #[test]
    fn test_processor_message_serialization() {
        let message = ProcessorMessage {
            program_id: Pubkey::new_unique(),
            data: vec![1, 2, 3, 4, 5],
            accounts: vec![
                AccountMetaData {
                    pubkey: Pubkey::new_unique(),
                    is_signer: true,
                    is_writable: false,
                }
            ],
        };

        // Test that the structure can be serialized/deserialized
        let serialized = message.try_to_vec().unwrap();
        let deserialized: ProcessorMessage = ProcessorMessage::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, message.program_id);
        assert_eq!(deserialized.data, vec![1, 2, 3, 4, 5]);
        assert_eq!(deserialized.accounts.len(), 1);
        assert_eq!(deserialized.accounts[0].pubkey, message.accounts[0].pubkey);
        assert!(deserialized.accounts[0].is_signer);
        assert!(!deserialized.accounts[0].is_writable);
    }

    #[test]
    fn test_authorization_state_space() {
        let space_no_sub_owners = AuthorizationState::space(0);
        let space_with_sub_owners = AuthorizationState::space(5);

        assert!(space_no_sub_owners > 0);
        assert!(space_with_sub_owners > space_no_sub_owners);
        
        // Each sub-owner adds 32 bytes (Pubkey)
        let expected_diff = 5 * 32;
        assert_eq!(space_with_sub_owners - space_no_sub_owners, expected_diff);
    }

    #[test]
    fn test_current_execution_space() {
        let space_short = CurrentExecution::space(5); // 5 char label
        let space_long = CurrentExecution::space(32); // 32 char label

        assert!(space_short > 0);
        assert!(space_long > space_short);
        
        // Label length difference should be reflected in space
        let expected_diff = 32 - 5;
        assert_eq!(space_long - space_short, expected_diff);
    }

    #[test]
    fn test_execution_result_variants() {
        let success = ExecutionResult::Success;
        let failure = ExecutionResult::Failure;

        assert_eq!(success, ExecutionResult::Success);
        assert_eq!(failure, ExecutionResult::Failure);
        assert_ne!(success, failure);
    }

    #[test]
    fn test_subroutine_type_variants() {
        let atomic = SubroutineType::Atomic;
        let non_atomic = SubroutineType::NonAtomic;

        assert_eq!(atomic, SubroutineType::Atomic);
        assert_eq!(non_atomic, SubroutineType::NonAtomic);
        assert_ne!(atomic, non_atomic);
    }

    #[test]
    fn test_authorization_serialization() {
        let owner = Pubkey::new_unique();
        let user1 = Pubkey::new_unique();
        let user2 = Pubkey::new_unique();

        let auth = Authorization {
            label: "test_auth".to_string(),
            owner,
            is_active: true,
            permission_type: PermissionType::Allowlist,
            allowed_users: vec![user1, user2],
            not_before: 1000,
            expiration: Some(2000),
            max_concurrent_executions: 5,
            priority: Priority::High,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 2,
            bump: 255,
        };

        // Test that the structure can be serialized/deserialized
        let serialized = auth.try_to_vec().unwrap();
        let deserialized: Authorization = Authorization::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.label, "test_auth");
        assert_eq!(deserialized.owner, owner);
        assert!(deserialized.is_active);
        assert_eq!(deserialized.permission_type, PermissionType::Allowlist);
        assert_eq!(deserialized.allowed_users.len(), 2);
        assert_eq!(deserialized.allowed_users[0], user1);
        assert_eq!(deserialized.allowed_users[1], user2);
        assert_eq!(deserialized.not_before, 1000);
        assert_eq!(deserialized.expiration, Some(2000));
        assert_eq!(deserialized.max_concurrent_executions, 5);
        assert_eq!(deserialized.priority, Priority::High);
        assert_eq!(deserialized.subroutine_type, SubroutineType::Atomic);
        assert_eq!(deserialized.current_executions, 2);
        assert_eq!(deserialized.bump, 255);
    }
} 