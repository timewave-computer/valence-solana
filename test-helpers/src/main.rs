use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use solana_program::pubkey::Pubkey;

mod token_helpers {
    use super::*;

    /// Get the token program ID
    pub fn get_token_program_id() -> Pubkey {
        token_2022::ID
    }

    /// Transfer tokens using Token-2022
    #[allow(dead_code)]
    pub fn transfer_tokens<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, token_2022::TransferChecked<'info>>,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        token_2022::transfer_checked(ctx, amount, decimals)
    }

    /// Check if a token account exists
    #[allow(dead_code)]
    pub fn token_account_exists(account_info: &AccountInfo) -> bool {
        account_info.data_len() > 0 && account_info.lamports() > 0
    }
}

fn main() {
    println!("Testing token_helpers module...");
    
    // Get token program ID
    let token_program_id = token_helpers::get_token_program_id();
    println!("Token program ID: {}", token_program_id);
    
    // Verify it's the expected program ID
    assert_eq!(token_program_id, token_2022::ID);
    
    println!("Token helpers test passed!");
} 