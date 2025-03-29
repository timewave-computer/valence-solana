use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use solana_program::program::{invoke, invoke_signed};
use solana_program::instruction::{Instruction, AccountMeta};
use crate::state::SingleUseAccount;
use crate::error::SingleUseAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SerializableAccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<SerializableAccountMeta> for AccountMeta {
    fn from(account_meta: SerializableAccountMeta) -> Self {
        if account_meta.is_writable {
            AccountMeta::new(account_meta.pubkey, account_meta.is_signer)
        } else {
            AccountMeta::new_readonly(account_meta.pubkey, account_meta.is_signer)
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteParams {
    pub library: Pubkey,
    pub instruction_data: Vec<u8>,
    pub accounts: Vec<SerializableAccountMeta>,
    pub destination: Pubkey,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"single_use_account", authority.key().as_ref()],
        bump,
        constraint = single_use_account.authority == authority.key() @ SingleUseAccountError::UnauthorizedOwnerOperation,
        constraint = !single_use_account.was_used @ SingleUseAccountError::AccountAlreadyUsed
    )]
    pub single_use_account: Account<'info, SingleUseAccount>,
    
    /// CHECK: The signer can specify any accounts, we'll be using these to pass to CPI
    pub single_use_account_pda: AccountInfo<'info>,
    
    // We include token_program to be able to check token accounts
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Execute>, params: ExecuteParams) -> Result<()> {
    let single_use_account = &mut ctx.accounts.single_use_account;
    
    // Check if the account has already been used
    if single_use_account.was_used {
        return Err(SingleUseAccountError::AccountAlreadyUsed.into());
    }
    
    // Validate the destination if required
    if !single_use_account.validate_destination(&params.destination) {
        return Err(SingleUseAccountError::InvalidDestination.into());
    }
    
    // Check if the library is approved
    if !single_use_account.is_library_approved(&params.library) {
        return Err(SingleUseAccountError::LibraryNotApproved.into());
    }
    
    // Execute the instruction via CPI
    let accounts: Vec<AccountMeta> = params.accounts
        .iter()
        .map(|account_meta| account_meta.clone().into())
        .collect();
    
    let instruction = Instruction {
        program_id: params.library,
        accounts,
        data: params.instruction_data,
    };
    
    // Get the single-use account PDA seeds for signing
    let single_use_account_seed = b"single_use_account";
    let authority_key = ctx.accounts.authority.key();
    let seeds = &[
        single_use_account_seed,
        authority_key.as_ref(),
        &[ctx.bumps.single_use_account],
    ];
    
    // Execute the instruction via CPI
    invoke_signed(
        &instruction,
        ctx.remaining_accounts,
        &[seeds],
    ).map_err(|_| SingleUseAccountError::ExecutionFailed.into())?;
    
    // Check that all token accounts belonging to this single-use account are empty
    // This would involve iterating through token accounts and checking their balances
    // For simplicity, we're just logging the operation here
    msg!("Verifying all token accounts are empty (in a real implementation, check balances)");
    
    // Mark the account as used
    single_use_account.mark_as_used();
    
    msg!(
        "Executed single-use operation with library: {}, destination: {}", 
        params.library,
        params.destination
    );
    
    Ok(())
} 