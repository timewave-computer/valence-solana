use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(
        init,
        payer = owner,
        space = FactoryState::SIZE,
        seeds = [b"factory_state"],
        bump
    )]
    pub factory_state: Account<'info, FactoryState>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    
    factory_state.owner = ctx.accounts.owner.key();
    factory_state.total_sessions_created = 0;
    factory_state.bump = ctx.bumps.factory_state;
    
    msg!("Session Factory initialized with owner: {}", factory_state.owner);
    
    Ok(())
} 