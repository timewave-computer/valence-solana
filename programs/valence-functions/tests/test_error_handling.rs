// Tests for error handling and edge cases in valence-functions
// Comprehensive error scenario testing and boundary condition validation

use anchor_lang::prelude::*;
use valence_functions::*;
use valence_functions::states::{EscrowBuilder, EscrowState, EscrowStatus, FunctionsError};

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    /// Test suite for FunctionsError handling
    mod functions_error_tests {
        use super::*;

        #[test]
        fn test_functions_error_codes() {
            // Verify error codes are properly defined
            let invalid_params = FunctionsError::InvalidParameters;
            let unauthorized = FunctionsError::Unauthorized;
            let invalid_state = FunctionsError::InvalidState;
            let operation_failed = FunctionsError::OperationFailed;

            // Test that they can be converted to anchor errors
            let _anchor_err1: anchor_lang::error::Error = invalid_params.into();
            let _anchor_err2: anchor_lang::error::Error = unauthorized.into();
            let _anchor_err3: anchor_lang::error::Error = invalid_state.into();
            let _anchor_err4: anchor_lang::error::Error = operation_failed.into();
        }

        #[test]
        fn test_functions_error_messages() {
            // Test error message formatting (this is mostly for documentation)
            match FunctionsError::InvalidParameters {
                FunctionsError::InvalidParameters => {}, // Expected
                _ => panic!("Error variant mismatch"),
            }
        }
    }

    /// Test suite for math function error handling
    mod math_error_tests {
        use super::*;
        use valence_functions::functions::math_add::*;

        #[test]
        fn test_math_overflow_detection() {
            let overflow_cases = vec![
                (u64::MAX, 1),
                (u64::MAX, u64::MAX),
                (u64::MAX - 1, 2),
                (u64::MAX / 2 + 1, u64::MAX / 2 + 1),
            ];

            for (a, b) in overflow_cases {
                let input = AddInput { a, b };
                let result = math_add(input);
                
                assert!(result.is_err(), "Should detect overflow for {} + {}", a, b);
                
                // Verify it's the correct error type
                match result.unwrap_err() {
                    anchor_lang::error::Error::AnchorError(anchor_err) => {
                        // Should be our MathError::Overflow
                        assert!(anchor_err.error_msg.contains("overflow") || 
                               anchor_err.error_msg.contains("Overflow"));
                    }
                    _ => panic!("Expected AnchorError for overflow"),
                }
            }
        }

        #[test]
        fn test_math_edge_case_success() {
            // Test cases that should NOT overflow
            let safe_cases = vec![
                (0, 0),
                (1, 0),
                (0, 1),
                (u64::MAX / 2, u64::MAX / 2),
                (u64::MAX - 1, 0),
                (0, u64::MAX - 1),
            ];

            for (a, b) in safe_cases {
                let input = AddInput { a, b };
                let result = math_add(input);
                
                assert!(result.is_ok(), "Should not overflow for {} + {}", a, b);
                assert_eq!(result.unwrap(), a.wrapping_add(b));
            }
        }

        #[test]
        fn test_math_boundary_conditions() {
            // Test exactly at the boundary
            let input = AddInput { a: u64::MAX - 1, b: 1 };
            let result = math_add(input).unwrap();
            assert_eq!(result, u64::MAX);

            // Test just over the boundary
            let input = AddInput { a: u64::MAX - 1, b: 2 };
            assert!(math_add(input).is_err());
        }
    }

    /// Test suite for token validation error handling
    mod token_validation_error_tests {
        use super::*;
        use valence_functions::functions::token_validate::*;

        #[test]
        fn test_token_insufficient_balance_error() {
            let input = TokenValidateInput {
                expected_mint: Pubkey::new_unique(),
                min_balance: 2000, // More than mock balance of 1000
                token_account: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
            };

            let result = token_validate(input);
            assert!(result.is_err());

            // Verify it's the correct error type
            match result.unwrap_err() {
                anchor_lang::error::Error::AnchorError(anchor_err) => {
                    assert!(anchor_err.error_msg.contains("Insufficient balance"));
                }
                _ => panic!("Expected AnchorError for insufficient balance"),
            }
        }

        #[test]
        fn test_token_validation_boundary_conditions() {
            // Test exactly at balance limit
            let input = TokenValidateInput {
                expected_mint: Pubkey::new_unique(),
                min_balance: 1000, // Exactly the mock balance
                token_account: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
            };
            assert!(token_validate(input).is_ok());

            // Test just over balance limit
            let input = TokenValidateInput {
                expected_mint: Pubkey::new_unique(),
                min_balance: 1001, // Just over mock balance
                token_account: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
            };
            assert!(token_validate(input).is_err());
        }

        #[test]
        fn test_token_validation_zero_balance_requirement() {
            let input = TokenValidateInput {
                expected_mint: Pubkey::new_unique(),
                min_balance: 0, // Zero requirement should always pass
                token_account: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
            };
            
            let result = token_validate(input).unwrap();
            assert!(result.valid);
            assert_eq!(result.balance, 1000);
        }

        #[test]
        fn test_token_validation_max_balance_requirement() {
            let input = TokenValidateInput {
                expected_mint: Pubkey::new_unique(),
                min_balance: u64::MAX, // Should definitely fail
                token_account: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
            };
            assert!(token_validate(input).is_err());
        }
    }

    /// Helper function for creating mock clocks
    fn mock_clock(timestamp: i64) -> Clock {
        Clock {
            slot: 1000,
            epoch_start_timestamp: timestamp - 1000,
            epoch: 10,
            leader_schedule_epoch: 10,
            unix_timestamp: timestamp,
        }
    }

    /// Test suite for escrow state error handling
    mod escrow_error_tests {
        use super::*;


        #[test]
        fn test_escrow_validation_errors() {
            let clock = mock_clock(1640995200);

            // Test zero price error
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(0) // Invalid price
                .duration(24 * 60 * 60)
                .build(&clock);
            assert!(result.is_err());

            // Test duration too short
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(30 * 60) // 30 minutes, less than 1 hour minimum
                .build(&clock);
            assert!(result.is_err());

            // Test duration too long
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(31 * 24 * 60 * 60) // 31 days, more than 30 day maximum
                .build(&clock);
            assert!(result.is_err());
        }

        #[test]
        fn test_escrow_duration_boundary_conditions() {
            let clock = mock_clock(1640995200);

            // Test minimum duration boundary
            let min_duration = EscrowState::MIN_DURATION;
            
            // Just below minimum should fail
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(min_duration - 1)
                .build(&clock);
            assert!(result.is_err());

            // Exactly minimum should succeed
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(min_duration)
                .build(&clock);
            assert!(result.is_ok());

            // Test maximum duration boundary
            let max_duration = EscrowState::MAX_DURATION;
            
            // Exactly maximum should succeed
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(max_duration)
                .build(&clock);
            assert!(result.is_ok());

            // Just over maximum should fail
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .duration(max_duration + 1)
                .build(&clock);
            assert!(result.is_err());
        }

        #[test]
        fn test_escrow_status_transition_errors() {
            let clock = mock_clock(1640995200);
            let mut escrow = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .build(&clock)
                .unwrap();

            // Invalid transitions should fail
            assert!(escrow.transition_status(EscrowStatus::Completed).is_err()); // Open -> Completed invalid
            
            escrow.status = EscrowStatus::Committed;
            escrow.buyer = Some(Pubkey::new_unique());
            assert!(escrow.transition_status(EscrowStatus::Open).is_err()); // Committed -> Open invalid

            escrow.status = EscrowStatus::Completed;
            assert!(escrow.transition_status(EscrowStatus::Open).is_err()); // Completed -> Open invalid
            assert!(escrow.transition_status(EscrowStatus::Committed).is_err()); // Completed -> Committed invalid
            assert!(escrow.transition_status(EscrowStatus::Cancelled).is_err()); // Completed -> Cancelled invalid

            escrow.status = EscrowStatus::Cancelled;
            assert!(escrow.transition_status(EscrowStatus::Open).is_err()); // Cancelled -> Open invalid
            assert!(escrow.transition_status(EscrowStatus::Committed).is_err()); // Cancelled -> Committed invalid
            assert!(escrow.transition_status(EscrowStatus::Completed).is_err()); // Cancelled -> Completed invalid
        }

        #[test]
        fn test_escrow_state_consistency_errors() {
            let clock = mock_clock(1640995200);
            let mut escrow = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .build(&clock)
                .unwrap();

            // Open escrow with buyer should fail validation
            escrow.buyer = Some(Pubkey::new_unique());
            assert!(escrow.validate().is_err());

            // Committed escrow without buyer should fail validation
            escrow.status = EscrowStatus::Committed;
            escrow.buyer = None;
            assert!(escrow.validate().is_err());

            // Fix the state
            escrow.buyer = Some(Pubkey::new_unique());
            assert!(escrow.validate().is_ok());
        }

        #[test]
        fn test_escrow_builder_missing_fields() {
            let clock = mock_clock(1640995200);

            // Test each required field missing
            let result = EscrowBuilder::new()
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .build(&clock);
            assert!(result.is_err()); // Missing seller

            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .price(1000)
                .build(&clock);
            assert!(result.is_err()); // Missing asset_mint

            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .build(&clock);
            assert!(result.is_err()); // Missing price

            // All fields present should work
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000)
                .build(&clock);
            assert!(result.is_ok());
        }
    }

    /// Test suite for environment error handling
    mod environment_error_tests {
        use super::*;

        #[test]
        fn test_environment_validation_edge_cases() {
            // Test various invalid environments
            let invalid_envs = vec![
                Environment {
                    slot: 0, // Invalid slot
                    timestamp: 1640995200,
                    ..Default::default()
                },
                Environment {
                    slot: 1000,
                    timestamp: 0, // Invalid timestamp
                    ..Default::default()
                },
                Environment {
                    slot: 0,
                    timestamp: 0, // Both invalid
                    ..Default::default()
                },
                Environment::default(), // Default should be invalid
            ];

            for env in invalid_envs {
                assert!(!env.is_valid(), "Environment should be invalid: {:?}", env);
            }
        }

        #[test]
        fn test_environment_valid_edge_cases() {
            // Test minimum valid values
            let valid_envs = vec![
                Environment {
                    slot: 1,
                    timestamp: 1,
                    ..Default::default()
                },
                Environment {
                    slot: u64::MAX,
                    timestamp: i64::MAX,
                    ..Default::default()
                },
            ];

            for env in valid_envs {
                assert!(env.is_valid(), "Environment should be valid: {:?}", env);
            }
        }

        #[test]
        fn test_environment_from_kernel_context_edge_cases() {
            // Test with edge case values
            let env = Environment::from_kernel_context(
                0, // Zero slot - should still create but be invalid
                0, // Zero epoch
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                0, // Zero timestamp - should still create but be invalid
            );

            assert_eq!(env.slot, 0);
            assert_eq!(env.timestamp, 0);
            assert!(!env.is_valid()); // Should be invalid

            // Test with maximum values
            let env = Environment::from_kernel_context(
                u64::MAX,
                u64::MAX,
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                i64::MAX,
            );

            assert_eq!(env.slot, u64::MAX);
            assert_eq!(env.timestamp, i64::MAX);
            assert!(env.is_valid()); // Should be valid
        }
    }

    /// Test suite for comprehensive error scenarios
    mod comprehensive_error_tests {
        use super::*;

        #[test]
        fn test_error_propagation_consistency() {
            // Test that all our error types can be properly converted to anchor errors
            let functions_errors = vec![
                FunctionsError::InvalidParameters,
                FunctionsError::Unauthorized,
                FunctionsError::InvalidState,
                FunctionsError::OperationFailed,
            ];

            for error in functions_errors {
                let anchor_error: anchor_lang::error::Error = error.into();
                // Should not panic and should be properly formatted
                let _error_string = format!("{:?}", anchor_error);
            }
        }

        #[test]
        fn test_concurrent_error_scenarios() {
            // Test multiple error conditions that might occur together
            let clock = mock_clock(1640995200);
            
            // Try to create an escrow with multiple invalid properties
            let result = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(0) // Invalid price
                .duration(30) // Invalid duration (too short)
                .build(&clock);
            
            // Should fail (and should fail on the first validation error encountered)
            assert!(result.is_err());
        }

        #[test]
        fn test_error_recovery_scenarios() {
            let clock = mock_clock(1640995200);
            
            // Test incomplete builders step by step
            let builder1 = EscrowBuilder::new();
            assert!(builder1.build(&clock).is_err()); // Should fail - missing fields
            
            let builder2 = EscrowBuilder::new().seller(Pubkey::new_unique());
            assert!(builder2.build(&clock).is_err()); // Still missing fields
            
            let builder3 = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique());
            assert!(builder3.build(&clock).is_err()); // Still missing price
            
            let builder4 = EscrowBuilder::new()
                .seller(Pubkey::new_unique())
                .asset_mint(Pubkey::new_unique())
                .price(1000);
            assert!(builder4.build(&clock).is_ok()); // Now should work
        }
    }
}