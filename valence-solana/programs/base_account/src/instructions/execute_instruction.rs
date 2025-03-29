use anchor_lang::prelude::*;
use solana_program::instruction::{Instruction, AccountMeta};
use solana_program::program::{invoke, invoke_signed};
use crate::state::BaseAccount;
use crate::error::BaseAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteInstructionParams {
    pub library: Pubkey,
    pub instruction_data: Vec<u8>,
    pub accounts: Vec<SerializableAccountMeta>,
}

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

#[derive(Accounts)]
pub struct ExecuteInstruction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"base_account", authority.key().as_ref()],
        bump,
        constraint = base_account.authority == authority.key() @ BaseAccountError::UnauthorizedOwnerOperation
    )]
    pub base_account: Account<'info, BaseAccount>,
    
    /// CHECK: The signer can specify any account, we'll be using this to pass to CPI
    pub base_account_pda: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ExecuteInstruction>, params: ExecuteInstructionParams) -> Result<()> {
    let base_account = &mut ctx.accounts.base_account;
    
    // Check if the library is approved
    if !base_account.is_library_approved(&params.library) {
        return Err(BaseAccountError::LibraryNotApproved.into());
    }
    
    // Increment the instruction count
    base_account.increment_instruction_count();
    
    // Convert the accounts
    let accounts: Vec<AccountMeta> = params.accounts
        .iter()
        .map(|account_meta| account_meta.clone().into())
        .collect();
    
    // Create the instruction
    let instruction = Instruction {
        program_id: params.library,
        accounts,
        data: params.instruction_data,
    };
    
    // Get the base account PDA seeds for signing
    let base_account_seed = b"base_account";
    let authority_key = ctx.accounts.authority.key();
    let seeds = &[
        base_account_seed,
        authority_key.as_ref(),
        &[ctx.bumps.base_account],
    ];
    
    // Execute the instruction via CPI
    invoke_signed(
        &instruction,
        ctx.remaining_accounts,
        &[seeds],
    ).map_err(|_| BaseAccountError::InstructionExecutionFailed.into())?;
    
    msg!(
        "Executed instruction through library: {}, instruction count: {}",
        params.library,
        base_account.instruction_count
    );
    
    Ok(())
} 