use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization, PermissionType, Priority, SubroutineType};
use crate::error::AuthorizationError;
use crate::validation::Validator;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateAuthorizationParams {
    pub label: String,
    pub permission_type: PermissionType,
    pub allowed_users: Option<Vec<Pubkey>>,
    pub not_before: i64,
    pub expiration: Option<i64>,
    pub max_concurrent_executions: u32,
    pub priority: Priority,
    pub subroutine_type: SubroutineType,
}

pub fn handler(
    ctx: Context<CreateAuthorization>,
    params: CreateAuthorizationParams,
) -> Result<()> {
    // Validate authorization creation parameters
    Validator::validate_authorization_creation(
        &params.label,
        &params.allowed_users,
        params.not_before,
        params.expiration,
        params.max_concurrent_executions,
    )?;
    
    let auth = &mut ctx.accounts.authorization;
    
    // Initialize authorization using the new String-based approach
    auth.label = params.label.clone();
    auth.owner = ctx.accounts.owner.key();
    auth.is_active = true;
    auth.permission_type = params.permission_type;
    auth.allowed_users = params.allowed_users.unwrap_or_default();
    auth.not_before = params.not_before;
    auth.expiration = params.expiration;
    auth.max_concurrent_executions = params.max_concurrent_executions;
    auth.priority = params.priority;
    auth.subroutine_type = params.subroutine_type;
    auth.current_executions = 0;
    auth.bump = ctx.bumps.authorization;
    
    msg!("Created new authorization with label: {}", params.label);
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(params: CreateAuthorizationParams)]
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
        space = Authorization::space(params.label.len(), params.allowed_users.as_ref().map_or(0, |v| v.len())),
        seeds = [b"authorization".as_ref(), params.label.as_bytes()],
        bump,
        // Additional constraints for validation
        constraint = params.label.len() <= 32 @ AuthorizationError::InvalidParameters,
        constraint = params.allowed_users.as_ref().map_or(0, |v| v.len()) <= 100 @ AuthorizationError::InvalidParameters,
        constraint = params.max_concurrent_executions > 0 && params.max_concurrent_executions <= 1000 @ AuthorizationError::InvalidParameters,
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
} 