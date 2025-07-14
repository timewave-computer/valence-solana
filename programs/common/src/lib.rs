use anchor_lang::prelude::*;

/// Program IDs for the Valence Protocol
pub mod program_ids {
    use super::*;
    
    pub const SYSTEM_PROGRAM_ID: Pubkey = anchor_lang::system_program::ID;
    
    // These would be replaced with actual deployed program IDs
    pub const REGISTRY_ID: Pubkey = solana_program::pubkey!("11111111111111111111111111111112");
    pub const SHARD_ID: Pubkey = solana_program::pubkey!("11111111111111111111111111111113");
}

/// Macro for validating program IDs
#[macro_export]
macro_rules! validate_program {
    ($account:expr, $expected:expr) => {
        require_keys_eq!(
            $account.key(),
            $expected,
            $crate::error::CommonError::InvalidProgram
        )
    };
    ($account:expr, $expected:expr, $error:expr) => {
        require_keys_eq!(
            $account.key(),
            $expected,
            $error
        )
    };
}

/// Bounds checking utilities
pub mod bounds {
    use super::*;
    
    /// Safely read data from an account with bounds checking
    pub fn read_data_safe(
        account: &AccountInfo,
        offset: usize,
        len: usize,
    ) -> Result<Vec<u8>> {
        let data = account.try_borrow_data()?;
        
        // Check bounds
        require!(
            offset.saturating_add(len) <= data.len(),
            error::CommonError::OutOfBounds
        );
        
        Ok(data[offset..offset + len].to_vec())
    }
    
    /// Safely write data to an account with bounds checking
    pub fn write_data_safe(
        account: &AccountInfo,
        offset: usize,
        data: &[u8],
    ) -> Result<()> {
        let mut account_data = account.try_borrow_mut_data()?;
        
        // Check bounds
        require!(
            offset.saturating_add(data.len()) <= account_data.len(),
            error::CommonError::OutOfBounds
        );
        
        account_data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    
    /// Check for arithmetic overflow before performing operation
    pub fn checked_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b)
            .ok_or(error::CommonError::ArithmeticOverflow.into())
    }
    
    pub fn checked_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b)
            .ok_or(error::CommonError::ArithmeticOverflow.into())
    }
    
    pub fn checked_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b)
            .ok_or(error::CommonError::ArithmeticOverflow.into())
    }
}

/// Account indexing for O(1) lookups
pub mod indexing {
    use super::*;
    use std::collections::HashMap;
    
    pub struct AccountIndex<'a> {
        accounts: HashMap<Pubkey, &'a AccountInfo<'a>>,
    }
    
    impl<'a> AccountIndex<'a> {
        /// Build an index from a slice of accounts
        pub fn build(accounts: &'a [AccountInfo<'a>]) -> Self {
            let mut map = HashMap::with_capacity(accounts.len());
            for account in accounts {
                map.insert(account.key(), account);
            }
            Self { accounts: map }
        }
        
        /// Get an account by key with O(1) lookup
        pub fn get(&self, key: &Pubkey) -> Option<&'a AccountInfo<'a>> {
            self.accounts.get(key).copied()
        }
        
        /// Get an account or return error
        pub fn get_required(&self, key: &Pubkey) -> Result<&'a AccountInfo<'a>> {
            self.get(key)
                .ok_or(error::CommonError::AccountNotFound.into())
        }
        
        /// Check if account exists
        pub fn contains(&self, key: &Pubkey) -> bool {
            self.accounts.contains_key(key)
        }
        
        /// Get number of indexed accounts
        pub fn len(&self) -> usize {
            self.accounts.len()
        }
        
        /// Check if the index is empty
        pub fn is_empty(&self) -> bool {
            self.accounts.is_empty()
        }
    }
}

/// Common errors
pub mod error {
    use super::*;
    
    #[error_code]
    pub enum CommonError {
        #[msg("Invalid program ID")]
        InvalidProgram,
        #[msg("Out of bounds access")]
        OutOfBounds,
        #[msg("Arithmetic overflow")]
        ArithmeticOverflow,
        #[msg("Account not found in index")]
        AccountNotFound,
    }
}