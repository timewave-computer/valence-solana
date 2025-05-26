use anchor_lang::prelude::*;
use crate::state::AccountState;

pub fn handler(
    ctx: Context<Initialize>,
    max_libraries: u8,
    max_token_accounts: u8,
) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    
    // Initialize the account state
    account_state.owner = ctx.accounts.signer.key();
    account_state.approved_libraries = Vec::with_capacity(max_libraries as usize);
    account_state.token_accounts = Vec::with_capacity(max_token_accounts as usize);
    account_state.instruction_count = 0;
    account_state.last_activity = Clock::get()?.unix_timestamp;
    
    // Save vault authority and bump
    let (vault_authority, vault_bump) = Pubkey::find_program_address(
        &[
            b"vault",
            account_state.to_account_info().key.as_ref(),
        ],
        ctx.program_id,
    );
    account_state.vault_authority = vault_authority;
    account_state.vault_bump_seed = vault_bump;
    
    msg!("Base account initialized with owner: {}", account_state.owner);
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + AccountState::get_space(
            0, // No approved libraries yet
            0  // No token accounts yet
        ),
    )]
    pub account: Account<'info, AccountState>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
} 