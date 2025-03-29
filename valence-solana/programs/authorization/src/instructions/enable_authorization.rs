use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<EnableAuthorization>) -> Result<()> {
    let authorization = &mut ctx.accounts.authorization;
    
    // Only enable if currently inactive
    if !authorization.is_active {
        authorization.is_active = true;
        msg!("Authorization enabled: {}", authorization.label);
    } else {
        msg!("Authorization already enabled: {}", authorization.label);
    }
    
    Ok(())
} 