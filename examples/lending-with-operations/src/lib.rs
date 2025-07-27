// ================================
// Lending Protocol with Generated Operations
// ================================

use anchor_lang::prelude::*;
use valence_sdk::{session, SessionOperations, track_operations};
use borsh::{BorshSerialize, BorshDeserialize};

// ================================
// Protocol State
// ================================

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LendingPoolState {
    pub total_deposits: u64,
    pub total_borrows: u64,
    pub interest_rate: u16, // Basis points
    pub ltv_ratio: u16,     // Basis points
}

// ================================
// Protocol Operations
// ================================

/// Type-safe operations for the lending protocol
#[derive(SessionOperations, BorshSerialize, BorshDeserialize, Clone, Debug)]
#[track_operations]
pub enum LendingProtocolOp {
    /// Initialize a new lending pool
    Initialize {
        interest_rate: u16,
        ltv_ratio: u16,
    },
    
    /// Deposit funds into the pool
    Deposit {
        amount: u64,
        #[validate(not_empty)]
        depositor: Pubkey,
    },
    
    /// Borrow against collateral
    Borrow {
        amount: u64,
        collateral_amount: u64,
        borrower: Pubkey,
    },
    
    /// Repay borrowed funds
    Repay {
        amount: u64,
        include_interest: bool,
    },
    
    /// Liquidate undercollateralized position
    Liquidate {
        position: Pubkey,
        max_repay: u64,
    },
    
    /// Update pool parameters (admin only)
    UpdateParams {
        new_interest_rate: Option<u16>,
        new_ltv_ratio: Option<u16>,
    },
}

// ================================
// Usage Example
// ================================

#[cfg(test)]
mod tests {
    use super::*;
    use valence_sdk::{session, duration};
    
    #[test]
    fn test_lending_operations() {
        // Create a session for the lending protocol
        let mut lending_session = session::<LendingPoolState>()
            .for_entity(Pubkey::new_unique())
            .with_state(Pubkey::new_unique()) // Pool state
            .expires_in(duration::days(30))
            .build()
            .unwrap();
        
        // Now we can use the generated methods!
        
        // Initialize the pool
        lending_session.initialize(500, 8000).unwrap(); // 5% interest, 80% LTV
        
        // Alice deposits
        let alice = Pubkey::new_unique();
        lending_session.deposit(1_000_000, alice).unwrap();
        
        // Bob borrows against collateral
        let bob = Pubkey::new_unique();
        lending_session.borrow(500_000, 700_000, bob).unwrap();
        
        // Bob repays with interest
        lending_session.repay(520_000, true).unwrap();
        
        // Admin updates parameters
        lending_session.update_params(Some(600), None).unwrap(); // Update to 6% interest
        
        // Check operation history
        let history = lending_session.history();
        assert_eq!(history.successful_operations().len(), 5);
        assert_eq!(history.success_rate(), 1.0);
    }
    
    #[test]
    fn test_batch_operations() {
        let mut session = session::<LendingPoolState>()
            .for_entity(Pubkey::new_unique())
            .with_state(Pubkey::new_unique())
            .expires_in(duration::hours(1))
            .build()
            .unwrap();
        
        // Execute multiple operations atomically
        let result = session.batch()
            .add(|_state| {
                // First operation
                Ok(())
            })
            .add(|_state| {
                // Second operation
                Ok(())
            })
            .add(|_state| {
                // Third operation
                Ok(())
            })
            .execute()
            .await
            .unwrap();
        
        assert_eq!(result.completed_operations, 3);
    }
}

// ================================
// Advanced Usage with Guards
// ================================

pub mod advanced {
    use super::*;
    use valence_functions::{GuardFunction, Environment};
    
    /// Guard that ensures only whitelisted users can borrow
    pub struct WhitelistGuard {
        whitelist: Vec<Pubkey>,
    }
    
    impl GuardFunction for WhitelistGuard {
        type State = LendingPoolState;
        
        fn check(&self, _state: &Self::State, operation: &[u8], env: &Environment) -> Result<bool> {
            // Deserialize operation
            if let Ok(op) = LendingProtocolOp::try_from_slice(operation) {
                match op {
                    LendingProtocolOp::Borrow { borrower, .. } => {
                        Ok(self.whitelist.contains(&borrower))
                    }
                    _ => Ok(true), // Other operations allowed
                }
            } else {
                Ok(false)
            }
        }
    }
    
    #[test]
    fn test_guarded_operations() {
        let whitelist = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let guard = WhitelistGuard { whitelist: whitelist.clone() };
        
        let mut session = session::<LendingPoolState>()
            .for_entity(Pubkey::new_unique())
            .with_state(Pubkey::new_unique())
            .guard(guard)
            .expires_in(duration::days(7))
            .build()
            .unwrap();
        
        // Whitelisted user can borrow
        session.borrow(100_000, 150_000, whitelist[0]).unwrap();
        
        // Non-whitelisted user would be rejected by guard
        // session.borrow(100_000, 150_000, Pubkey::new_unique()).unwrap_err();
    }
}