use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization, PermissionType, Priority, SubroutineType};
use crate::error::AuthorizationError;

pub fn handler(
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

impl<'info> ModifyAuthorization<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, ModifyAuthorization<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(Accounts)]
pub struct ModifyAuthorization<'info> {
    #[account(
        seeds = [b"authorization_state".as_ref()],
        bump = authorization_state.bump,
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(
        mut,
        seeds = [b"authorization".as_ref(), &authorization.label[..authorization.label_length as usize]],
        bump = authorization.bump,
        constraint = authorization.owner == owner.key() || authorization_state.owner == owner.key() || authorization_state.sub_owners.contains(&owner.key()) @ AuthorizationError::NotAuthorized,
    )]
    pub authorization: Account<'info, Authorization>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
} 