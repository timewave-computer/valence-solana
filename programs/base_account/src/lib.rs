// Base account program for managing account state and library approvals
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod error;
pub mod state;

use state::AccountState;

#[program]
pub mod base_account {
    use super::*;

    pub fn initialize(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::*;

    #[test]
    fn test_account_state_space_calculation() {
        let space_no_libs = AccountState::get_space(0, 0);
        let space_with_libs = AccountState::get_space(5, 3);
        let space_max_libs = AccountState::get_space(10, 10);

        // Base size calculation: 8 + 32 + 4 + 32 + 1 + 4 + 8 + 8 = 97 bytes
        assert_eq!(space_no_libs, 97);
        // More libraries should require more space
        assert!(space_with_libs > space_no_libs);
        assert!(space_max_libs > space_with_libs);
        
        // Each library adds 32 bytes (Pubkey), each token account adds 32 bytes
        let expected_diff = (5 * 32) + (3 * 32); // 5 libraries + 3 token accounts
        assert_eq!(space_with_libs - space_no_libs, expected_diff);
    }

    #[test]
    fn test_account_state_serialization() {
        let owner = Pubkey::new_unique();
        let vault_authority = Pubkey::new_unique();
        let library1 = Pubkey::new_unique();
        let library2 = Pubkey::new_unique();
        let token_account = Pubkey::new_unique();

        let account_state = AccountState {
            owner,
            approved_libraries: vec![library1, library2],
            token_accounts: vec![token_account],
            instruction_count: 42,
            last_activity: 1234567890,
            vault_authority,
            vault_bump_seed: 255,
        };

        // Test that the structure can be serialized/deserialized
        let serialized = account_state.try_to_vec().unwrap();
        let deserialized: AccountState = AccountState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, owner);
        assert_eq!(deserialized.approved_libraries.len(), 2);
        assert_eq!(deserialized.approved_libraries[0], library1);
        assert_eq!(deserialized.approved_libraries[1], library2);
        assert_eq!(deserialized.token_accounts.len(), 1);
        assert_eq!(deserialized.token_accounts[0], token_account);
        assert_eq!(deserialized.instruction_count, 42);
        assert_eq!(deserialized.last_activity, 1234567890);
        assert_eq!(deserialized.vault_authority, vault_authority);
        assert_eq!(deserialized.vault_bump_seed, 255);
    }

    #[test]
    fn test_account_state_default() {
        let account_state = AccountState::default();
        
        assert_eq!(account_state.owner, Pubkey::default());
        assert!(account_state.approved_libraries.is_empty());
        assert!(account_state.token_accounts.is_empty());
        assert_eq!(account_state.instruction_count, 0);
        assert_eq!(account_state.last_activity, 0);
        assert_eq!(account_state.vault_authority, Pubkey::default());
        assert_eq!(account_state.vault_bump_seed, 0);
    }

    #[test]
    fn test_vault_authority_derivation() {
        let account_key = Pubkey::new_unique();
        let program_id = crate::ID;
        
        let (vault_authority, vault_bump) = Pubkey::find_program_address(
            &[b"vault", account_key.as_ref()],
            &program_id,
        );
        
        // PDA should be valid
        assert_ne!(vault_authority, Pubkey::default());
        // vault_bump is u8, so it's always <= 255, and should be a valid bump
        // vault_bump is u8, so it's always <= 255
        
        // Should be deterministic
        let (vault_authority2, vault_bump2) = Pubkey::find_program_address(
            &[b"vault", account_key.as_ref()],
            &program_id,
        );
        assert_eq!(vault_authority, vault_authority2);
        assert_eq!(vault_bump, vault_bump2);
    }

    #[test]
    fn test_different_accounts_produce_different_vault_authorities() {
        let account1 = Pubkey::new_unique();
        let account2 = Pubkey::new_unique();
        let program_id = crate::ID;
        
        let (vault1, _) = Pubkey::find_program_address(
            &[b"vault", account1.as_ref()],
            &program_id,
        );
        let (vault2, _) = Pubkey::find_program_address(
            &[b"vault", account2.as_ref()],
            &program_id,
        );
        
        assert_ne!(vault1, vault2);
    }

    #[test]
    fn test_approved_libraries_management() {
        let mut account_state = AccountState::default();
        let library1 = Pubkey::new_unique();
        let library2 = Pubkey::new_unique();
        
        // Add libraries
        account_state.approved_libraries.push(library1);
        account_state.approved_libraries.push(library2);
        
        assert_eq!(account_state.approved_libraries.len(), 2);
        assert!(account_state.approved_libraries.contains(&library1));
        assert!(account_state.approved_libraries.contains(&library2));
        
        // Remove a library
        account_state.approved_libraries.retain(|&lib| lib != library1);
        assert_eq!(account_state.approved_libraries.len(), 1);
        assert!(!account_state.approved_libraries.contains(&library1));
        assert!(account_state.approved_libraries.contains(&library2));
    }

    #[test]
    fn test_token_accounts_management() {
        let mut account_state = AccountState::default();
        let token_account1 = Pubkey::new_unique();
        let token_account2 = Pubkey::new_unique();
        
        // Add token accounts
        account_state.token_accounts.push(token_account1);
        account_state.token_accounts.push(token_account2);
        
        assert_eq!(account_state.token_accounts.len(), 2);
        assert!(account_state.token_accounts.contains(&token_account1));
        assert!(account_state.token_accounts.contains(&token_account2));
        
        // Remove a token account
        account_state.token_accounts.retain(|&acc| acc != token_account1);
        assert_eq!(account_state.token_accounts.len(), 1);
        assert!(!account_state.token_accounts.contains(&token_account1));
        assert!(account_state.token_accounts.contains(&token_account2));
    }

    #[test]
    fn test_instruction_count_tracking() {
        let mut account_state = AccountState::default();
        
        assert_eq!(account_state.instruction_count, 0);
        
        // Simulate instruction execution
        account_state.instruction_count += 1;
        assert_eq!(account_state.instruction_count, 1);
        
        account_state.instruction_count += 5;
        assert_eq!(account_state.instruction_count, 6);
    }

    #[test]
    fn test_last_activity_tracking() {
        let mut account_state = AccountState::default();
        
        assert_eq!(account_state.last_activity, 0);
        
        // Simulate activity update
        let current_time = 1234567890;
        account_state.last_activity = current_time;
        assert_eq!(account_state.last_activity, current_time);
        
        // Update to newer time
        let newer_time = current_time + 3600; // 1 hour later
        account_state.last_activity = newer_time;
        assert_eq!(account_state.last_activity, newer_time);
    }

    #[test]
    fn test_space_calculation_edge_cases() {
        // Test with maximum reasonable values
        let space_large = AccountState::get_space(255, 255);
        assert!(space_large > 0);
        
        // Test with zero values
        let space_zero = AccountState::get_space(0, 0);
        assert!(space_zero > 0);
        
        // Ensure space calculation is reasonable
        assert!(space_large > space_zero);
        
        // Each additional library/token account should add exactly 32 bytes
        let space_one_lib = AccountState::get_space(1, 0);
        let space_one_token = AccountState::get_space(0, 1);
        
        assert_eq!(space_one_lib - space_zero, 32);
        assert_eq!(space_one_token - space_zero, 32);
    }
}
