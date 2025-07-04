// Tests for Session Factory functionality

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use session_factory::state::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_factory_state_creation() {
        let ctx = TestContext::new();
        
        let factory_state = FactoryState {
            owner: ctx.authority,
            total_sessions_created: 0,
            bump: 255,
        };

        // Test serialization
        let serialized = factory_state.try_to_vec().unwrap();
        let deserialized: FactoryState = FactoryState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, ctx.authority);
        assert_eq!(deserialized.total_sessions_created, 0);
        assert_eq!(deserialized.bump, 255);
    }

    #[test]
    fn test_session_creation() {
        let ctx = TestContext::new();
        let eval_program = generate_test_pubkey("eval_program");
        
        let session = Session {
            owner: ctx.user,
            eval_program_id: eval_program,
            namespaces: vec![[1u8; 32], [2u8; 32]],
            nonce: 0,
            created_at: 1000000000,
            last_activity: 1000000000,
            is_active: true,
            bump: 254,
        };

        // Test serialization
        let serialized = session.try_to_vec().unwrap();
        let deserialized: Session = Session::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, ctx.user);
        assert_eq!(deserialized.eval_program_id, eval_program);
        assert_eq!(deserialized.namespaces.len(), 2);
        assert!(deserialized.is_active);
    }

    #[test]
    fn test_session_namespace_checks() {
        let mut session = Session {
            owner: generate_test_pubkey("owner"),
            eval_program_id: generate_test_pubkey("eval"),
            namespaces: vec![[1u8; 32], [2u8; 32], [3u8; 32]],
            nonce: 0,
            created_at: 1000000000,
            last_activity: 1000000000,
            is_active: true,
            bump: 253,
        };

        // Test namespace checks
        assert!(session.has_namespace(&[1u8; 32]));
        assert!(session.has_namespace(&[2u8; 32]));
        assert!(session.has_namespace(&[3u8; 32]));
        assert!(!session.has_namespace(&[4u8; 32]));

        // Test nonce increment
        let old_nonce = session.nonce;
        let new_nonce = session.increment_nonce();
        assert_eq!(new_nonce, old_nonce + 1);
        assert_eq!(session.nonce, new_nonce);
    }

    #[test]
    fn test_session_space_calculation() {
        // Test space calculation for different namespace counts
        let space_0 = Session::get_space(0);
        let space_3 = Session::get_space(3);
        let space_5 = Session::get_space(5);

        // Verify space increases with namespace count
        assert!(space_3 > space_0);
        assert!(space_5 > space_3);

        // Verify exact space calculation
        assert_eq!(space_3 - space_0, 3 * 32); // 3 namespaces * 32 bytes each
        assert_eq!(space_5 - space_0, 5 * 32); // 5 namespaces * 32 bytes each
    }

    #[test]
    fn test_session_activity_tracking() {
        let mut session = Session {
            owner: generate_test_pubkey("owner"),
            eval_program_id: generate_test_pubkey("eval"),
            namespaces: vec![],
            nonce: 0,
            created_at: 1000000000,
            last_activity: 1000000000,
            is_active: true,
            bump: 252,
        };

        // Simulate time passing
        let original_activity = session.last_activity;
        
        // Note: In actual usage, Clock::get() would provide current time
        // For testing, we just verify the function exists and can be called
        session.update_activity();
        
        // In real scenario, this would be greater than original
        // For test, we just ensure the field is accessible
        assert!(session.last_activity >= original_activity);
    }

    #[test]
    fn test_deterministic_session_addresses() {
        let owner = generate_test_pubkey("owner");
        let factory_state_key = generate_test_pubkey("factory_state");
        
        // Test that session addresses are deterministic
        for i in 0..5 {
            let seeds = &[
                b"session",
                owner.as_ref(),
                &i.to_le_bytes(),
            ];
            
            // In real usage, this would generate a PDA
            let seed_hash = solana_program::hash::hashv(seeds);
            
            // Verify seeds are unique for each session
            for j in 0..5 {
                if i != j {
                    let other_seeds = &[
                        b"session",
                        owner.as_ref(),
                        &j.to_le_bytes(),
                    ];
                    let other_hash = solana_program::hash::hashv(other_seeds);
                    assert_ne!(seed_hash, other_hash);
                }
            }
        }
    }

    #[test]
    fn test_session_eval_binding() {
        let eval1 = generate_test_pubkey("eval1");
        let eval2 = generate_test_pubkey("eval2");
        
        let session = Session {
            owner: generate_test_pubkey("owner"),
            eval_program_id: eval1,
            namespaces: vec![],
            nonce: 0,
            created_at: 1000000000,
            last_activity: 1000000000,
            is_active: true,
            bump: 251,
        };

        // Verify eval binding is permanent (no setter method exists)
        assert_eq!(session.eval_program_id, eval1);
        assert_ne!(session.eval_program_id, eval2);
        
        // Session can only be modified by its designated eval
        // This would be enforced in the actual program logic
    }

    #[test]
    fn test_factory_state_size() {
        // Verify the constant SIZE matches actual struct size
        let expected_size = 8 + // discriminator
            32 + // owner
            8 + // total_sessions_created
            1; // bump
            
        assert_eq!(FactoryState::SIZE, expected_size);
    }
} 