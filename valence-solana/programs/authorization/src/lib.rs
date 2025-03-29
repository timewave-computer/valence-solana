use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as tx_instructions;

// Program ID declaration
declare_id!("AuthorizationProgramxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

pub mod state;
pub mod instructions;
pub mod error;
pub mod cache;

use state::*;
use instructions::*;
use error::*;

#[program]
pub mod authorization {
    use super::*;

    /// Initialize the Authorization Program with an owner and registry
    pub fn initialize(
        ctx: Context<Initialize>,
        valence_registry: Pubkey,
        processor_program_id: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, valence_registry, processor_program_id)
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
        instructions::create_authorization::handler(
            ctx,
            label,
            permission_type,
            allowed_users,
            not_before,
            expiration,
            max_concurrent_executions,
            priority,
            subroutine_type,
        )
    }

    /// Modify an existing authorization
    pub fn modify_authorization(
        ctx: Context<ModifyAuthorization>,
        permission_type: Option<PermissionType>,
        allowed_users: Option<Vec<Pubkey>>,
        not_before: Option<i64>,
        expiration: Option<i64>,
        max_concurrent_executions: Option<u32>,
        priority: Option<Priority>,
        subroutine_type: Option<SubroutineType>,
    ) -> Result<()> {
        instructions::modify_authorization::handler(
            ctx,
            permission_type,
            allowed_users,
            not_before,
            expiration,
            max_concurrent_executions,
            priority,
            subroutine_type,
        )
    }

    /// Disable an authorization
    pub fn disable_authorization(ctx: Context<DisableAuthorization>) -> Result<()> {
        instructions::disable_authorization::handler(ctx)
    }

    /// Enable an authorization
    pub fn enable_authorization(ctx: Context<EnableAuthorization>) -> Result<()> {
        instructions::enable_authorization::handler(ctx)
    }

    /// Send messages using an authorization
    pub fn send_messages(
        ctx: Context<SendMessages>,
        authorization_label: String,
        messages: Vec<ProcessorMessage>,
    ) -> Result<()> {
        instructions::send_messages::handler(ctx, authorization_label, messages)
    }

    /// Process callback from the Processor Program
    pub fn receive_callback(
        ctx: Context<ReceiveCallback>,
        execution_id: u64,
        result: ExecutionResult,
        executed_count: u32,
        error_data: Option<Vec<u8>>,
    ) -> Result<()> {
        instructions::receive_callback::handler(ctx, execution_id, result, executed_count, error_data)
    }

    /// Look up an authorization by label
    pub fn lookup_authorization(
        ctx: Context<LookupAuthorization>,
        label: String,
    ) -> Result<Pubkey> {
        instructions::lookup_authorization::handler(ctx, label)
    }
}

/// The source file structure will be:
/// 
/// lib.rs - Main program entry point with instruction routing
/// state.rs - Account structures and data types
/// error.rs - Error handling for the program
/// instructions/ - Individual instruction handlers
///    mod.rs - Module exports
///    initialize.rs - Handler for initialize instruction
///    create_authorization.rs - Handler for authorization creation
///    modify_authorization.rs - Handler for modifying authorizations
///    disable_authorization.rs - Handler for disabling authorizations
///    enable_authorization.rs - Handler for enabling authorizations
///    send_messages.rs - Handler for message sending
///    receive_callback.rs - Handler for callback processing
/// 