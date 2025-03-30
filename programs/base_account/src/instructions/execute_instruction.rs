use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_program::program::{invoke_signed};
use solana_program::instruction::Instruction;
use crate::state::{AccountState, ApprovalNonce};
use crate::error::BaseAccountError;

pub fn handler(ctx: Context<ExecuteInstruction>, ix_data: Vec<u8>) -> Result<()> {
    let account_state = &mut ctx.accounts.account;
    let current_timestamp = Clock::get()?.unix_timestamp;
    
    // The instruction can be executed either:
    // 1. By the account owner directly
    // 2. By an approved library using an approval nonce
    
    let using_approval_nonce = ctx.accounts.approval_nonce.is_some();
    let library_key = ctx.accounts.library.key();
    
    // Check execution permission
    if ctx.accounts.signer.key() == account_state.owner {
        // Owner is executing directly - verify library is approved
        if !account_state.is_library_approved(&library_key) {
            return Err(BaseAccountError::LibraryNotApproved.into());
        }
    } else if using_approval_nonce {
        // Library is executing via approval nonce
        let approval_nonce = ctx.accounts.approval_nonce.as_ref().unwrap();
        
        // Verify approval nonce is valid
        if approval_nonce.library != library_key {
            return Err(BaseAccountError::InvalidApprovalNonce.into());
        }
        if approval_nonce.owner != account_state.owner {
            return Err(BaseAccountError::InvalidOwner.into());
        }
        if approval_nonce.is_used {
            return Err(BaseAccountError::ApprovalNonceUsed.into());
        }
        if approval_nonce.expiration < current_timestamp {
            return Err(BaseAccountError::ApprovalNonceExpired.into());
        }
        
        // Mark approval nonce as used
        let approval_nonce_mut = ctx.accounts.approval_nonce.as_mut().unwrap();
        approval_nonce_mut.is_used = true;
    } else {
        // Neither owner nor valid approval nonce
        return Err(BaseAccountError::Unauthorized.into());
    }
    
    // Create the instruction to execute
    let accounts: Vec<AccountMeta> = ctx.remaining_accounts
        .iter()
        .map(|a| {
            if a.is_writable {
                AccountMeta::new(a.key(), a.is_signer)
            } else {
                AccountMeta::new_readonly(a.key(), a.is_signer)
            }
        })
        .collect();
    
    let instruction = Instruction {
        program_id: library_key,
        accounts,
        data: ix_data,
    };
    
    // Execute the instruction
    let vault_seeds = &[
        b"vault".as_ref(),
        account_state.to_account_info().key.as_ref(),
        &[account_state.vault_bump_seed],
    ];
    
    // Try to execute the instruction with the vault authority as signer
    let result = invoke_signed(
        &instruction,
        ctx.remaining_accounts,
        &[vault_seeds]
    );
    
    if result.is_err() {
        return Err(BaseAccountError::InstructionExecutionFailed.into());
    }
    
    // Update execution statistics
    account_state.instruction_count += 1;
    account_state.last_activity = current_timestamp;
    
    msg!("Executed instruction for library: {}", library_key);
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteInstruction<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// CHECK: Library program validity is verified in the handler
    pub library: UncheckedAccount<'info>,
    
    /// Optional approval nonce for library execution
    #[account(mut)]
    pub approval_nonce: Option<Account<'info, ApprovalNonce>>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
} 