use anchor_lang::prelude::*;
use crate::state::{AuthorizationState, Authorization};
use crate::error::AuthorizationError;

pub fn handler(ctx: Context<EnableAuthorization>) -> Result<()> {
    ctx.accounts.authorization.is_active = true;
    msg!("Enabled authorization: {}", ctx.accounts.authorization.get_label());
    Ok(())
}

impl<'info> EnableAuthorization<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, EnableAuthorization<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(Accounts)]
pub struct EnableAuthorization<'info> {
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