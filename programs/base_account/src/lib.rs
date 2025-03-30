use anchor_lang::prelude::*;

declare_id!("BaseSixYBPg1K1xjRXVBvd7LVLmATG2sFjUXzVZrfG6");

pub mod error;
pub mod state;
pub mod instructions;

use state::*;
use error::BaseAccountError;

#[program]
pub mod base_account {
    use super::*;
    
    /// Initialize a new base account
    pub fn initialize(
        ctx: Context<Initialize>,
        max_libraries: u8, 
        max_token_accounts: u8
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
    
    /// Approve a library to operate on this account
    pub fn approve_library(ctx: Context<ApproveLibrary>) -> Result<()> {
        let account_state = &mut ctx.accounts.account;
        let library = ctx.accounts.library.key();
        
        // Only the owner can approve libraries
        if account_state.owner != ctx.accounts.signer.key() {
            return Err(BaseAccountError::Unauthorized.into());
        }
        
        // Check if library is already approved
        if account_state.is_library_approved(&library) {
            return Err(BaseAccountError::LibraryAlreadyApproved.into());
        }
        
        // Add library to approved list
        account_state.approve_library(library)?;
        
        msg!("Approved library: {}", library);
        Ok(())
    }
    
    /// Revoke approval for a library
    pub fn revoke_library(ctx: Context<RevokeLibrary>) -> Result<()> {
        let account_state = &mut ctx.accounts.account;
        let library = ctx.accounts.library.key();
        
        // Only the owner can revoke libraries
        if account_state.owner != ctx.accounts.signer.key() {
            return Err(BaseAccountError::Unauthorized.into());
        }
        
        // Check if library is already approved
        if !account_state.is_library_approved(&library) {
            return Err(BaseAccountError::LibraryNotApproved.into());
        }
        
        // Remove library from approved list
        account_state.remove_approved_library(&library)?;
        
        msg!("Revoked library approval: {}", library);
        Ok(())
    }
    
    /// Create an associated token account for a given mint
    pub fn create_token_account(ctx: Context<CreateTokenAccount>) -> Result<()> {
        let account_state = &mut ctx.accounts.account;
        let token_account = ctx.accounts.token_account.key();
        let mint = ctx.accounts.mint.key();
        
        // Only the owner can create token accounts
        if account_state.owner != ctx.accounts.signer.key() {
            return Err(BaseAccountError::Unauthorized.into());
        }
        
        // Check if token account already exists in state
        if account_state.token_accounts.contains(&token_account) {
            return Err(BaseAccountError::TokenAccountAlreadyExists.into());
        }
        
        // Add token account to state
        account_state.token_accounts.push(token_account);
        account_state.last_activity = Clock::get()?.unix_timestamp;
        
        msg!("Created token account for mint {}: {}", mint, token_account);
        Ok(())
    }
    
    /// Close an associated token account
    pub fn close_token_account(ctx: Context<CloseTokenAccount>) -> Result<()> {
        let account_state = &mut ctx.accounts.account;
        let token_account = ctx.accounts.token_account.key();
        
        // Only the owner can close token accounts
        if account_state.owner != ctx.accounts.signer.key() {
            return Err(BaseAccountError::Unauthorized.into());
        }
        
        // Check if token account exists in state
        if !account_state.token_accounts.contains(&token_account) {
            return Err(BaseAccountError::TokenAccountNotFound.into());
        }
        
        // Remove token account from state
        account_state.token_accounts.retain(|&x| x != token_account);
        account_state.last_activity = Clock::get()?.unix_timestamp;
        
        msg!("Closed token account: {}", token_account);
        Ok(())
    }
    
    /// Transfer ownership of the account to a new owner
    pub fn transfer_ownership(ctx: Context<TransferOwnership>) -> Result<()> {
        let account_state = &mut ctx.accounts.account;
        let new_owner = ctx.accounts.new_owner.key();
        
        // Only the current owner can transfer ownership
        if account_state.owner != ctx.accounts.signer.key() {
            return Err(BaseAccountError::Unauthorized.into());
        }
        
        // Update the owner
        account_state.owner = new_owner;
        account_state.last_activity = Clock::get()?.unix_timestamp;
        
        msg!("Transferred ownership to: {}", new_owner);
        Ok(())
    }
    
    /// Create a one-time approval nonce for a library
    pub fn create_approval_nonce(
        ctx: Context<CreateApprovalNonce>,
        expiration: i64
    ) -> Result<()> {
        let account_state = &ctx.accounts.account;
        let approval_nonce = &mut ctx.accounts.approval_nonce;
        let library = ctx.accounts.library.key();
        
        // Only the owner can create approval nonces
        if account_state.owner != ctx.accounts.signer.key() {
            return Err(BaseAccountError::Unauthorized.into());
        }
        
        // Check if library is approved
        if !account_state.is_library_approved(&library) {
            return Err(BaseAccountError::LibraryNotApproved.into());
        }
        
        // Initialize the approval nonce
        approval_nonce.library = library;
        approval_nonce.nonce = Clock::get()?.unix_timestamp as u64;  // Use current timestamp as nonce
        approval_nonce.owner = ctx.accounts.signer.key();
        approval_nonce.expiration = expiration;
        approval_nonce.is_used = false;
        approval_nonce.bump = *ctx.bumps.get("approval_nonce").unwrap();
        
        msg!("Created approval nonce for library: {}, expires at: {}", library, expiration);
        Ok(())
    }
    
    /// Execute an instruction on behalf of a library
    pub fn execute_instruction(
        ctx: Context<ExecuteInstruction>,
        ix_data: Vec<u8>
    ) -> Result<()> {
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
        
        let instruction = solana_program::instruction::Instruction {
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
        let result = solana_program::program::invoke_signed(
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
}

// Account validation structs
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

#[derive(Accounts)]
pub struct ApproveLibrary<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// Address of the library program to approve
    /// CHECK: Library program validity is verified elsewhere
    pub library: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevokeLibrary<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// Address of the library program to revoke
    /// CHECK: Library program validity is verified elsewhere
    pub library: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateTokenAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    #[account(
        seeds = [b"vault", account.key().as_ref()],
        bump = account.vault_bump_seed
    )]
    /// CHECK: This is a PDA used as a token authority
    pub vault_authority: UncheckedAccount<'info>,
    
    pub mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = vault_authority,
    )]
    pub token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CloseTokenAccount<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    #[account(
        mut,
        constraint = token_account.owner == account.vault_authority @ BaseAccountError::InvalidVaultAuthority
    )]
    pub token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    /// CHECK: This is the destination for the closed token account funds
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[derive(Accounts)]
pub struct TransferOwnership<'info> {
    #[account(mut)]
    pub account: Account<'info, AccountState>,
    
    /// CHECK: This is the new owner address
    pub new_owner: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(expiration: i64)]
pub struct CreateApprovalNonce<'info> {
    pub account: Account<'info, AccountState>,
    
    #[account(
        init,
        payer = signer,
        space = 8 + ApprovalNonce::SPACE,
        seeds = [
            b"approval",
            account.key().as_ref(),
            library.key().as_ref(),
            &(Clock::get()?.unix_timestamp as u64).to_le_bytes()
        ],
        bump
    )]
    pub approval_nonce: Account<'info, ApprovalNonce>,
    
    /// CHECK: Library program validity is verified elsewhere
    pub library: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
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