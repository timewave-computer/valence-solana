use anchor_lang::prelude::*;
use crate::state::AuthorizationState;

pub fn handler(ctx: Context<Initialize>, processor_id: Pubkey, registry_id: Pubkey) -> Result<()> {
    msg!("Initializing authorization program");
    
    let state = &mut ctx.accounts.authorization_state;
    state.owner = ctx.accounts.owner.key();
    state.sub_owners = Vec::new();
    state.processor_id = processor_id;
    state.registry_id = registry_id;
    state.execution_counter = 0;
    state.bump = ctx.bumps.authorization_state;
    state.last_zk_sequence = 0;
    state.zk_sequence_counter = 0;
    state.reserved = [0u8; 64];
    
    Ok(())
}




#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<AuthorizationState>() + 32 * 10, // Extra space for sub_owners
        seeds = [b"authorization_state".as_ref()],
        bump
    )]
    pub authorization_state: Account<'info, AuthorizationState>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
} 