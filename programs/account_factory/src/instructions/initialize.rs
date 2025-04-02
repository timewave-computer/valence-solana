use anchor_lang::prelude::*;
use crate::state::FactoryState;
use crate::error::AccountFactoryError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitializeParams {
    pub creation_fee: u64,
    pub fee_receiver: Option<Pubkey>,
}

impl<'info> Initialize<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, Initialize<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = FactoryState::SIZE,
        seeds = [b"factory_state"],
        bump
    )]
    pub factory_state: Account<'info, FactoryState>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    let authority = ctx.accounts.authority.key();
    let current_time = Clock::get()?.unix_timestamp;
    
    // Initialize factory state
    factory_state.authority = authority;
    factory_state.template_count = 0;
    factory_state.account_count = 0;
    factory_state.last_activity = current_time;
    factory_state.is_paused = false;
    factory_state.creation_fee = params.creation_fee;
    factory_state.fee_receiver = params.fee_receiver.unwrap_or(authority);
    factory_state.reserved = [0; 64];
    
    msg!("Account Factory initialized with authority: {}", factory_state.authority);
    
    if factory_state.creation_fee > 0 {
        msg!("Creation fee set to {} lamports", factory_state.creation_fee);
        msg!("Fee receiver set to {}", factory_state.fee_receiver);
    }
    
    Ok(())
} 