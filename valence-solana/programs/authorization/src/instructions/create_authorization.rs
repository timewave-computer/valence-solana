use anchor_lang::prelude::*;
use crate::state::*;
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
    if label.len() > 100 {
        return Err(AuthorizationError::LabelTooLong.into());
    }
    
    let allowed_users_vec = allowed_users.unwrap_or_default();
    if allowed_users_vec.len() > 50 {
        return Err(AuthorizationError::TooManyAllowedUsers.into());
    }
    
    // Check expiration logic
    if let Some(exp) = expiration {
        if exp <= not_before {
            return Err(error!(AuthorizationError::AuthorizationExpired));
        }
    }
    
    // Initialize the authorization
    let authorization = &mut ctx.accounts.authorization;
    authorization.label = label;
    authorization.owner = ctx.accounts.owner.key();
    authorization.is_active = true;
    authorization.permission_type = permission_type;
    authorization.allowed_users = allowed_users_vec;
    authorization.not_before = not_before;
    authorization.expiration = expiration;
    authorization.max_concurrent_executions = max_concurrent_executions;
    authorization.priority = priority;
    authorization.subroutine_type = subroutine_type;
    authorization.current_executions = 0;
    authorization.bump = *ctx.bumps.get("authorization").unwrap();
    
    msg!("Authorization created: {}", authorization.label);
    
    Ok(())
} 