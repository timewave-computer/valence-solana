use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::AuthorizationError;

pub fn handler(
    ctx: Context<ModifyAuthorization>,
    permission_type: Option<PermissionType>,
    allowed_users: Option<Vec<Pubkey>>,
    not_before: Option<i64>,
    expiration: Option<i64>,
    max_concurrent_executions: Option<u32>,
    priority: Option<Priority>,
    subroutine_type: Option<SubroutineType>,
) -> Result<()> {
    let authorization = &mut ctx.accounts.authorization;
    
    // Handle permission type update
    if let Some(new_permission_type) = permission_type {
        authorization.permission_type = new_permission_type;
    }
    
    // Handle allowed users update
    if let Some(new_allowed_users) = allowed_users {
        if new_allowed_users.len() > 50 {
            return Err(AuthorizationError::TooManyAllowedUsers.into());
        }
        authorization.allowed_users = new_allowed_users;
    }
    
    // Handle validity period updates
    if let Some(new_not_before) = not_before {
        authorization.not_before = new_not_before;
    }
    
    if let Some(new_expiration) = expiration {
        // Ensure expiration is after not_before
        if new_expiration <= authorization.not_before {
            return Err(error!(AuthorizationError::AuthorizationExpired));
        }
        authorization.expiration = Some(new_expiration);
    }
    
    // Handle concurrent execution limit update
    if let Some(new_max_concurrent_executions) = max_concurrent_executions {
        // If reducing the limit, ensure it's not below current executions
        if new_max_concurrent_executions < authorization.current_executions {
            return Err(error!(AuthorizationError::TooManyExecutions));
        }
        authorization.max_concurrent_executions = new_max_concurrent_executions;
    }
    
    // Handle priority update
    if let Some(new_priority) = priority {
        authorization.priority = new_priority;
    }
    
    // Handle subroutine type update
    if let Some(new_subroutine_type) = subroutine_type {
        authorization.subroutine_type = new_subroutine_type;
    }
    
    msg!("Authorization modified: {}", authorization.label);
    
    Ok(())
} 