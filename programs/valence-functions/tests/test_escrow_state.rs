// Integration tests for EscrowState and related functionality
// Tests the escrow state management, validation, and lifecycle

use anchor_lang::prelude::*;
use valence_functions::states::*;

#[cfg(test)]
mod escrow_state_tests {
    use super::*;

    /// Helper function to create a mock clock
    fn mock_clock(timestamp: i64) -> Clock {
        Clock {
            slot: 1000,
            epoch_start_timestamp: timestamp - 1000,
            epoch: 10,
            leader_schedule_epoch: 10,
            unix_timestamp: timestamp,
        }
    }

    /// Helper function to create a basic escrow state
    fn create_test_escrow(clock: &Clock) -> EscrowState {
        EscrowBuilder::new()
            .seller(Pubkey::new_unique())
            .asset_mint(Pubkey::new_unique())
            .price(1000)
            .duration(24 * 60 * 60) // 24 hours
            .build(clock)
            .unwrap()
    }

    /// Test suite for EscrowState basic functionality
    mod escrow_basic_tests {
        use super::*;

        #[test]
        fn test_escrow_creation() {
            let clock = mock_clock(1640995200);
            let seller = Pubkey::new_unique();
            let asset_mint = Pubkey::new_unique();
            let price = 1000u64;
            let duration = 24 * 60 * 60i64;

            let escrow = EscrowBuilder::new()
                .seller(seller)
                .asset_mint(asset_mint)
                .price(price)
                .duration(duration)
                .build(&clock)
                .unwrap();

            assert_eq!(escrow.seller, seller);
            assert_eq!(escrow.asset_mint, asset_mint);
            assert_eq!(escrow.price, price);
            assert_eq!(escrow.created_at, clock.unix_timestamp);
            assert_eq!(escrow.expires_at, clock.unix_timestamp + duration);
            assert_eq!(escrow.status, EscrowStatus::Open);
            assert!(escrow.buyer.is_none());
        }

        #[test]
        fn test_escrow_default_duration() {
            let clock = mock_clock(1640995200);
            
            let escrow = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .build(&clock)
                .unwrap();

            // Should default to 24 hours
            assert_eq!(escrow.expires_at - escrow.created_at, 24 * 60 * 60);
        }

        #[test]
        fn test_escrow_validation_success() {
            let clock = mock_clock(1640995200);
            let escrow = create_test_escrow(&clock);
            
            // Should validate successfully
            assert!(escrow.validate().is_ok());
        }

        #[test]
        fn test_escrow_validation_failures() {
            let clock = mock_clock(1640995200);
            
            // Test invalid price (zero)
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(0) // Invalid
                .build(&clock);
            assert!(result.is_err());

            // Test duration too short
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(30 * 60) // 30 minutes, less than MIN_DURATION
                .build(&clock);
            assert!(result.is_err());

            // Test duration too long
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(31 * 24 * 60 * 60) // 31 days, more than MAX_DURATION
                .build(&clock);
            assert!(result.is_err());
        }

        #[test]
        fn test_escrow_constants() {
            assert_eq!(EscrowState::MIN_DURATION, 60 * 60); // 1 hour
            assert_eq!(EscrowState::MAX_DURATION, 30 * 24 * 60 * 60); // 30 days
            assert!(EscrowState::LEN > 100); // Should have reasonable size
        }
    }

    /// Test suite for EscrowState lifecycle management
    mod escrow_lifecycle_tests {
        use super::*;

        #[test]
        fn test_escrow_expiration() {
            let start_time = 1640995200;
            let clock_start = mock_clock(start_time);
            let duration = 24 * 60 * 60;
            
            let escrow = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(duration)
                .build(&clock_start)
                .unwrap();

            // Should not be expired at start
            assert!(!escrow.is_expired(&clock_start));

            // Should not be expired just before expiration
            let clock_before = mock_clock(start_time + duration - 1);
            assert!(!escrow.is_expired(&clock_before));

            // Should be expired at expiration time
            let clock_expired = mock_clock(start_time + duration);
            assert!(escrow.is_expired(&clock_expired));

            // Should be expired after expiration
            let clock_after = mock_clock(start_time + duration + 1);
            assert!(escrow.is_expired(&clock_after));
        }

        #[test]
        fn test_remaining_time() {
            let start_time = 1640995200;
            let duration = 24 * 60 * 60;
            let clock = mock_clock(start_time);
            
            let escrow = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(duration)
                .build(&clock)
                .unwrap();

            // At start, should have full duration remaining
            assert_eq!(escrow.remaining_time(&clock), duration);

            // Halfway through, should have half remaining
            let clock_mid = mock_clock(start_time + duration / 2);
            assert_eq!(escrow.remaining_time(&clock_mid), duration / 2);

            // After expiration, should have 0 remaining
            let clock_after = mock_clock(start_time + duration + 100);
            assert_eq!(escrow.remaining_time(&clock_after), 0);
        }

        #[test]
        fn test_escrow_status_transitions() {
            let clock = mock_clock(1640995200);
            let mut escrow = create_test_escrow(&clock);

            // Should start as Open
            assert_eq!(escrow.status, EscrowStatus::Open);

            // Can transition from Open to Committed
            assert!(escrow.transition_status(EscrowStatus::Committed).is_ok());
            assert_eq!(escrow.status, EscrowStatus::Committed);

            // Can transition from Committed to Completed
            assert!(escrow.transition_status(EscrowStatus::Completed).is_ok());
            assert_eq!(escrow.status, EscrowStatus::Completed);

            // Cannot transition from Completed to other states
            assert!(escrow.transition_status(EscrowStatus::Open).is_err());
            assert!(escrow.transition_status(EscrowStatus::Cancelled).is_err());
        }

        #[test]
        fn test_escrow_cancellation_transitions() {
            let clock = mock_clock(1640995200);
            let mut escrow = create_test_escrow(&clock);

            // Can cancel from Open
            assert!(escrow.transition_status(EscrowStatus::Cancelled).is_ok());
            assert_eq!(escrow.status, EscrowStatus::Cancelled);

            // Reset to test cancellation from Committed
            escrow.status = EscrowStatus::Committed;
            assert!(escrow.transition_status(EscrowStatus::Cancelled).is_ok());
        }

        #[test]
        fn test_invalid_status_transitions() {
            let clock = mock_clock(1640995200);
            let mut escrow = create_test_escrow(&clock);

            // Cannot go directly from Open to Completed
            assert!(escrow.transition_status(EscrowStatus::Completed).is_err());

            // Cannot go from Committed back to Open
            escrow.status = EscrowStatus::Committed;
            assert!(escrow.transition_status(EscrowStatus::Open).is_err());
        }
    }

    /// Test suite for EscrowState access control
    mod escrow_access_control_tests {
        use super::*;

        #[test]
        fn test_seller_cancellation_rights() {
            let clock = mock_clock(1640995200);
            let escrow = create_test_escrow(&clock);
            let seller = escrow.seller;
            let other_user = Pubkey::new_unique();

            // Seller can cancel when Open
            assert!(escrow.can_be_cancelled(&seller, &clock));
            
            // Other users cannot cancel
            assert!(!escrow.can_be_cancelled(&other_user, &clock));
        }

        #[test]
        fn test_committed_escrow_cancellation() {
            let clock = mock_clock(1640995200);
            let mut escrow = create_test_escrow(&clock);
            let seller = escrow.seller;
            
            // Set buyer and status to committed
            escrow.buyer = Some(Pubkey::new_unique());
            escrow.status = EscrowStatus::Committed;

            // Seller cannot cancel committed escrow that hasn't expired
            assert!(!escrow.can_be_cancelled(&seller, &clock));

            // But can cancel after expiration
            let expired_clock = mock_clock(clock.unix_timestamp + 25 * 60 * 60); // 25 hours later
            assert!(escrow.can_be_cancelled(&seller, &expired_clock));
        }

        #[test]
        fn test_interaction_permissions() {
            let clock = mock_clock(1640995200);
            let mut escrow = create_test_escrow(&clock);
            let seller = escrow.seller;
            let buyer = Pubkey::new_unique();
            let other_user = Pubkey::new_unique();

            // When Open, anyone can interact (to become buyer)
            assert!(escrow.can_interact(&seller));
            assert!(escrow.can_interact(&buyer));
            assert!(escrow.can_interact(&other_user));

            // When Committed, only seller and buyer can interact
            escrow.buyer = Some(buyer);
            escrow.status = EscrowStatus::Committed;
            
            assert!(escrow.can_interact(&seller));
            assert!(escrow.can_interact(&buyer));
            assert!(!escrow.can_interact(&other_user));

            // When Completed, nobody can interact
            escrow.status = EscrowStatus::Completed;
            assert!(!escrow.can_interact(&seller));
            assert!(!escrow.can_interact(&buyer));
            assert!(!escrow.can_interact(&other_user));
        }

        #[test]
        fn test_ready_for_completion() {
            let clock = mock_clock(1640995200);
            let mut escrow = create_test_escrow(&clock);

            // Not ready without buyer
            assert!(!escrow.is_ready_for_completion(&clock));

            // Not ready with buyer but wrong status
            escrow.buyer = Some(Pubkey::new_unique());
            assert!(!escrow.is_ready_for_completion(&clock));

            // Ready with buyer and committed status
            escrow.status = EscrowStatus::Committed;
            assert!(escrow.is_ready_for_completion(&clock));

            // Not ready if expired
            let expired_clock = mock_clock(clock.unix_timestamp + 25 * 60 * 60);
            assert!(!escrow.is_ready_for_completion(&expired_clock));
        }
    }

    /// Test suite for StateValidator trait implementation
    mod state_validator_tests {
        use super::*;

        #[test]
        fn test_operation_permissions() {
            let clock = mock_clock(1640995200);
            let mut escrow = create_test_escrow(&clock);

            // Open status allows commit
            assert!(escrow.allows_operation("commit"));
            assert!(!escrow.allows_operation("complete"));
            assert!(escrow.allows_operation("cancel"));

            // Committed status allows complete and cancel
            escrow.status = EscrowStatus::Committed;
            assert!(!escrow.allows_operation("commit"));
            assert!(escrow.allows_operation("complete"));
            assert!(escrow.allows_operation("cancel"));

            // Completed status allows nothing
            escrow.status = EscrowStatus::Completed;
            assert!(!escrow.allows_operation("commit"));
            assert!(!escrow.allows_operation("complete"));
            assert!(!escrow.allows_operation("cancel"));

            // Unknown operations not allowed
            assert!(!escrow.allows_operation("unknown"));
        }

        #[test]
        fn test_validation_consistency_checks() {
            let clock = mock_clock(1640995200);
            
            // Test that committed escrow must have buyer
            let mut escrow = create_test_escrow(&clock);
            escrow.status = EscrowStatus::Committed;
            // buyer is None, should fail validation
            assert!(escrow.validate().is_err());

            // Fix by adding buyer
            escrow.buyer = Some(Pubkey::new_unique());
            assert!(escrow.validate().is_ok());

            // Test that open escrow should not have buyer
            escrow.status = EscrowStatus::Open;
            // buyer is Some, should fail validation
            assert!(escrow.validate().is_err());

            // Fix by removing buyer
            escrow.buyer = None;
            assert!(escrow.validate().is_ok());
        }
    }

    /// Test suite for PDA generation
    mod pda_tests {
        use super::*;

        #[test]
        fn test_escrow_state_pda_generation() {
            let seller = Pubkey::new_unique();
            let asset_mint = Pubkey::new_unique();
            let nonce = 42u64;
            let program_id = Pubkey::new_unique();

            let (pda1, bump1) = pda::escrow_state(&seller, &asset_mint, nonce, &program_id);
            let (pda2, bump2) = pda::escrow_state(&seller, &asset_mint, nonce, &program_id);

            // Should be deterministic
            assert_eq!(pda1, pda2);
            assert_eq!(bump1, bump2);

            // Different inputs should produce different PDAs
            let (pda3, _) = pda::escrow_state(&seller, &asset_mint, nonce + 1, &program_id);
            assert_ne!(pda1, pda3);
        }

        #[test]
        fn test_escrow_vault_pda_generation() {
            let escrow_state = Pubkey::new_unique();
            let program_id = Pubkey::new_unique();

            let (pda1, bump1) = pda::escrow_vault(&escrow_state, &program_id);
            let (pda2, bump2) = pda::escrow_vault(&escrow_state, &program_id);

            // Should be deterministic
            assert_eq!(pda1, pda2);
            assert_eq!(bump1, bump2);

            // Different escrow state should produce different vault PDA
            let different_escrow = Pubkey::new_unique();
            let (pda3, _) = pda::escrow_vault(&different_escrow, &program_id);
            assert_ne!(pda1, pda3);
        }

        #[test]
        fn test_pda_seeds_constants() {
            // Verify seed constants are as expected
            assert_eq!(seeds::ESCROW_STATE, b"escrow_state");
            assert_eq!(seeds::ESCROW_VAULT, b"escrow_vault");
        }
    }

    /// Test suite for EscrowBuilder
    mod escrow_builder_tests {
        use super::*;

        #[test]
        fn test_builder_pattern() {
            let clock = mock_clock(1640995200);
            let seller = Pubkey::new_unique();
            let asset_mint = Pubkey::new_unique();
            let price = 1000u64;
            let duration = 12 * 60 * 60i64; // 12 hours

            let escrow = EscrowBuilder::new()
                .seller(seller)
                .asset_mint(asset_mint)
                .price(price)
                .duration(duration)
                .build(&clock)
                .unwrap();

            assert_eq!(escrow.seller, seller);
            assert_eq!(escrow.asset_mint, asset_mint);
            assert_eq!(escrow.price, price);
            assert_eq!(escrow.expires_at - escrow.created_at, duration);
        }

        #[test]
        fn test_builder_missing_required_fields() {
            let clock = mock_clock(1640995200);

            // Missing seller
            let result = EscrowBuilder::new()
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .build(&clock);
            assert!(result.is_err());

            // Missing asset_mint
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .price(1000)
                .build(&clock);
            assert!(result.is_err());

            // Missing price
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .build(&clock);
            assert!(result.is_err());
        }

        #[test]
        fn test_builder_default() {
            let builder1 = EscrowBuilder::new();
            let builder2 = EscrowBuilder::default();
            
            // Both should create equivalent builders (we can't directly compare due to no PartialEq)
            // But we know they're equivalent because they both fail to build without required fields
            let clock = mock_clock(1640995200);
            assert!(builder1.build(&clock).is_err());
            assert!(builder2.build(&clock).is_err());
        }
    }
}