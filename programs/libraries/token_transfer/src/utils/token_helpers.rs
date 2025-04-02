use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use solana_program::pubkey::Pubkey;

/// Get the token program ID
/// 
/// @return Pubkey The program ID of Token-2022
pub fn get_token_program_id() -> Pubkey {
    token_2022::ID
}

/// Transfer tokens using Token-2022
/// 
/// @param ctx The CPI context for the transfer
/// @param amount The amount of tokens to transfer
/// @return Result<()> The result of the transfer
pub fn transfer_tokens<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, token_2022::Transfer<'info>>,
    amount: u64,
) -> Result<()> {
    token_2022::transfer(ctx, amount)
}

/// Check if a token account exists
/// 
/// @param account_info The account to check
/// @return bool Whether the account exists
pub fn token_account_exists(account_info: &AccountInfo) -> bool {
    account_info.data_len() > 0 && account_info.lamports() > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_token_program_id() {
        let id = get_token_program_id();
        assert_eq!(id, token_2022::ID);
    }
    
    #[test]
    fn test_token_account_exists() {
        // Create mock AccountInfo with zero lamports
        let key = Pubkey::new_unique();
        let mut lamports = 0;
        let mut data = vec![];
        
        let non_existent_account = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &Pubkey::default(),
            false,
            0,
        );
        
        // Test non-existent account
        assert!(!token_account_exists(&non_existent_account));
        
        // Create mock AccountInfo with lamports and data
        let mut lamports = 1000;
        let mut data = vec![0; 10];
        
        let existent_account = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &Pubkey::default(),
            false,
            0,
        );
        
        // Test existent account
        assert!(token_account_exists(&existent_account));
    }
}
