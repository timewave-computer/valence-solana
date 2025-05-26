use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization, PermissionType, Priority, SubroutineType};
use crate::error::AuthorizationError;
// use crate::validation::Validator; // Temporarily disabled

pub fn handler(
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
    // TODO: Re-enable validation once validation module is working
    // Validator::validate_authorization_creation(
    //     &label,
    //     &allowed_users,
    //     not_before,
    //     expiration,
    //     max_concurrent_executions,
    // )?;
    
    let auth = &mut ctx.accounts.authorization;
    
    // Initialize authorization using the new String-based approach
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