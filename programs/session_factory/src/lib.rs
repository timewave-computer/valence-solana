use anchor_lang::prelude::*;

declare_id!("FacTryNutKBJ4HrNYBQQZ7oMNd7yUtJxgwRqDKmULjY");

pub mod error;
pub mod state;
pub mod instructions;

use instructions::*;

#[program]
pub mod session_factory {
    use super::*;

    /// Initialize the Session Factory with owner
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize_handler(ctx)
    }

    /// Create a new session with initial state
    pub fn create_session(
        ctx: Context<CreateSession>, 
        eval_program_id: Pubkey,
        initial_namespaces: Vec<[u8; 32]>,
    ) -> Result<()> {
        create_session_handler(ctx, eval_program_id, initial_namespaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deterministic_session_addresses() {
        // Test that session addresses are deterministically generated
        let program_id = ID;
        let owner = Pubkey::new_unique();
        
        // Test multiple sessions for same owner
        for i in 0u64..5 {
            let (pda, bump) = Pubkey::find_program_address(
                &[
                    b"session",
                    owner.as_ref(),
                    &i.to_le_bytes(),
                ],
                &program_id,
            );
            
            // Verify same inputs produce same address
            let (pda2, bump2) = Pubkey::find_program_address(
                &[
                    b"session",
                    owner.as_ref(),
                    &i.to_le_bytes(),
                ],
                &program_id,
            );
            
            assert_eq!(pda, pda2);
            assert_eq!(bump, bump2);
        }
    }
    
    #[test]
    fn test_unique_session_addresses() {
        // Test that different inputs produce different addresses
        let program_id = ID;
        let owner1 = Pubkey::new_unique();
        let owner2 = Pubkey::new_unique();
        
        let (pda1, _) = Pubkey::find_program_address(
            &[
                b"session",
                owner1.as_ref(),
                &0u64.to_le_bytes(),
            ],
            &program_id,
        );
        
        let (pda2, _) = Pubkey::find_program_address(
            &[
                b"session",
                owner2.as_ref(),
                &0u64.to_le_bytes(),
            ],
            &program_id,
        );
        
        assert_ne!(pda1, pda2);
    }
    
    #[test]
    fn test_factory_state_pda() {
        // Test factory state PDA generation
        let program_id = ID;
        
        let (pda, bump) = Pubkey::find_program_address(
            &[b"factory_state"],
            &program_id,
        );
        
        // Verify it's deterministic
        let (pda2, bump2) = Pubkey::find_program_address(
            &[b"factory_state"],
            &program_id,
        );
        
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }
} 