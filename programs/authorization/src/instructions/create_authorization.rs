use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization, PermissionType, Priority, SubroutineType};
use crate::error::AuthorizationError;

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
        space = 8 +  // account discriminator
               32 + // [u8; 32] (label)
               1 + // u8 (label_length)
               32 + // Pubkey (owner)
               1 + // bool (is_active)
               1 + // enum (permission_type) 
               4 + 50 * 32 + // Vec<Pubkey> (allowed_users) with max 50 entries
               8 + // i64 (not_before)
               1 + 8 + // Option<i64> (expiration)
               4 + // u32 (max_concurrent_executions)
               1 + // enum (priority)
               1 + // enum (subroutine_type)
               4 + // u32 (current_executions)
               1, // u8 (bump)
        seeds = [b"authorization".as_ref(), label.as_bytes()],
        bump
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
} 