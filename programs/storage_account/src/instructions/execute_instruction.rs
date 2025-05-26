use anchor_lang::prelude::*;
use crate::state::StorageAccount;
use crate::error::StorageAccountError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SerializableAccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl<'info> ExecuteInstruction<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, ExecuteInstruction<'info>>,
        _bumps: &std::collections::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteInstructionParams {
    pub library: Pubkey,
    pub instruction_data: Vec<u8>,
    pub accounts: Vec<SerializableAccountMeta>,
}

#[derive(Accounts)]
pub struct ExecuteInstruction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"storage_account", authority.key().as_ref()],
        bump,
        constraint = storage_account.authority == authority.key() @ StorageAccountError::UnauthorizedOwnerOperation
    )]
    pub storage_account: Account<'info, StorageAccount>,
    
    /// CHECK: This is validated in the handler
    pub storage_account_pda: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ExecuteInstruction>, params: ExecuteInstructionParams) -> Result<()> {
    let storage_account = &mut ctx.accounts.storage_account;
    
    // Check if the library is approved
    if !storage_account.is_library_approved(&params.library) {
        return Err(error!(StorageAccountError::UnauthorizedStorageOperation));
    }
    
    // In a real implementation, we would execute the instruction
    // using CPIs. For this example, we just update our state.
    
    // Increment the instruction count
    storage_account.increment_instruction_count();
    
    msg!(
        "Executed instruction through library: {}, instruction count: {}",
        params.library,
        storage_account.instruction_count
    );
    
    Ok(())
} 