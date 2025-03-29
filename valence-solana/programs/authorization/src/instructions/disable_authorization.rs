use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<DisableAuthorization>) -> Result<()> {
    let authorization = &mut ctx.accounts.authorization;
    
    // Only disable if currently active
    if authorization.is_active {
        authorization.is_active = false;
        msg!("Authorization disabled: {}", authorization.label);
    } else {
        msg!("Authorization already disabled: {}", authorization.label);
    }
    
    Ok(())
} 