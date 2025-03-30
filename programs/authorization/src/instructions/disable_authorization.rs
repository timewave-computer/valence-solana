use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization};
use crate::error::AuthorizationError;

pub fn handler(ctx: Context<DisableAuthorization>) -> Result<()> {
    ctx.accounts.authorization.is_active = false;
    msg!("Disabled authorization: {}", ctx.accounts.authorization.get_label());
    Ok(())
}

#[derive(Accounts)]
pub struct DisableAuthorization<'info> {
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