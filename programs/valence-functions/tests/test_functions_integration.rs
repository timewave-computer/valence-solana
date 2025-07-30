// Integration tests for valence-functions core functionality
// Tests the main function implementations and their integration

use anchor_lang::prelude::*;
use valence_functions::functions::*;
use valence_functions::*;

#[cfg(test)]
mod function_integration_tests {
    use super::*;

    /// Test suite for identity function
    mod identity_tests {
        use super::*;
        use valence_functions::functions::identity::*;

        #[test]
        fn test_identity_basic_functionality() {
            // Test basic identity functionality
            assert_eq!(identity(42).unwrap(), 42);
            assert_eq!(identity(0).unwrap(), 0);
            assert_eq!(identity(u64::MAX).unwrap(), u64::MAX);
        }

        #[test]
        fn test_identity_metadata() {
            // Verify function metadata constants
            assert_eq!(FUNCTION_ID, 1001);
            assert_eq!(FUNCTION_NAME, "identity");
            assert_eq!(FUNCTION_VERSION, 1);
            assert_eq!(COMPUTE_UNITS, 1_000);
        }

        #[test]
        fn test_identity_edge_cases() {
            // Test edge cases
            let test_cases = vec![
                1u64,
                42u64,
                1000u64,
                u64::MAX - 1,
                u64::MAX,
            ];

            for case in test_cases {
                let result = identity(case).unwrap();
                assert_eq!(result, case, "Identity failed for input: {}", case);
            }
        }
    }

    /// Test suite for math_add function
    mod math_add_tests {
        use super::*;
        use valence_functions::functions::math_add::*;

        #[test]
        fn test_math_add_normal_operations() {
            let test_cases = vec![
                (0, 0, 0),
                (1, 1, 2),
                (10, 20, 30),
                (100, 200, 300),
                (u64::MAX / 2, u64::MAX / 2, u64::MAX - 1),
            ];

            for (a, b, expected) in test_cases {
                let input = AddInput { a, b };
                let result = math_add(input).unwrap();
                assert_eq!(result, expected, "Addition failed for {} + {}", a, b);
            }
        }

        #[test]
        fn test_math_add_overflow_protection() {
            let overflow_cases = vec![
                (u64::MAX, 1),
                (u64::MAX, u64::MAX),
                (u64::MAX - 1, 2),
            ];

            for (a, b) in overflow_cases {
                let input = AddInput { a, b };
                let result = math_add(input);
                assert!(result.is_err(), "Should overflow for {} + {}", a, b);
            }
        }

        #[test]
        fn test_math_add_metadata() {
            // Verify function metadata
            assert_eq!(FUNCTION_ID, 1002);
            assert_eq!(FUNCTION_NAME, "math_add");
            assert_eq!(FUNCTION_VERSION, 1);
            assert_eq!(COMPUTE_UNITS, 2_000);
        }

        #[test]
        fn test_math_add_zero_identity() {
            // Test that adding zero acts as identity
            let values = vec![0, 1, 42, 1000, u64::MAX];
            
            for value in values {
                let input = AddInput { a: value, b: 0 };
                assert_eq!(math_add(input).unwrap(), value);
                
                let input = AddInput { a: 0, b: value };
                assert_eq!(math_add(input).unwrap(), value);
            }
        }

        #[test]
        fn test_math_add_commutativity() {
            // Test that a + b = b + a
            let test_pairs = vec![
                (1, 2),
                (10, 20),
                (100, 200),
                (42, 58),
            ];

            for (a, b) in test_pairs {
                let input1 = AddInput { a, b };
                let input2 = AddInput { a: b, b: a };
                
                let result1 = math_add(input1).unwrap();
                let result2 = math_add(input2).unwrap();
                
                assert_eq!(result1, result2, "Addition not commutative for {} and {}", a, b);
            }
        }
    }

    /// Test suite for token_validate function
    mod token_validate_tests {
        use super::*;
        use valence_functions::functions::token_validate::*;

        #[test]
        fn test_token_validate_success() {
            let mint = Pubkey::new_unique();
            let token_account = Pubkey::new_unique();
            let owner = Pubkey::new_unique();

            let input = TokenValidateInput {
                expected_mint: mint,
                min_balance: 500, // Less than mock balance of 1000
                token_account,
                owner,
            };

            let result = token_validate(input).unwrap();
            assert!(result.valid);
            assert_eq!(result.balance, 1000);
            assert_eq!(result.mint, mint);
        }

        #[test]
        fn test_token_validate_insufficient_balance() {
            let input = TokenValidateInput {
                expected_mint: Pubkey::new_unique(),
                min_balance: 2000, // More than mock balance of 1000
                token_account: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
            };

            let result = token_validate(input);
            assert!(result.is_err());
        }

        #[test]
        fn test_token_validate_metadata() {
            assert_eq!(FUNCTION_ID, 1003);
            assert_eq!(FUNCTION_NAME, "token_validate");
            assert_eq!(FUNCTION_VERSION, 1);
            assert_eq!(COMPUTE_UNITS, 5_000);
        }

        #[test]
        fn test_token_validate_boundary_conditions() {
            let test_cases = vec![
                (1001, false), // Just above mock balance (1000)
                (1000, true),  // Exactly mock balance
                (999, true),   // Just below mock balance - should pass
                (0, true),     // Zero minimum
            ];

            for (min_balance, should_pass) in test_cases {
                let input = TokenValidateInput {
                    expected_mint: Pubkey::new_unique(),
                    min_balance,
                    token_account: Pubkey::new_unique(),
                    owner: Pubkey::new_unique(),
                };

                let result = token_validate(input);
                if should_pass {
                    assert!(result.is_ok(), "Should pass for min_balance: {}", min_balance);
                } else {
                    assert!(result.is_err(), "Should fail for min_balance: {}", min_balance);
                }
            }
        }
    }

    /// Test suite for Environment functionality
    mod environment_tests {
        use super::*;

        #[test]
        fn test_environment_creation() {
            let env = Environment::from_kernel_context(
                1000,  // slot
                10,    // epoch
                Pubkey::new_unique(), // tx_submitter
                Pubkey::new_unique(), // session
                Pubkey::new_unique(), // caller
                1640995200, // timestamp (arbitrary)
            );

            assert_eq!(env.slot, 1000);
            assert_eq!(env.epoch, 10);
            assert_eq!(env.timestamp, 1640995200);
            assert!(env.is_valid());
        }

        #[test]
        fn test_environment_validation() {
            // Valid environment
            let valid_env = Environment {
                slot: 1000,
                epoch: 10,
                tx_submitter: Pubkey::new_unique(),
                session: Pubkey::new_unique(),
                caller: Pubkey::new_unique(),
                timestamp: 1640995200,
                recent_blockhash: [0u8; 32],
            };
            assert!(valid_env.is_valid());

            // Invalid environment - zero timestamp
            let invalid_env = Environment {
                timestamp: 0,
                ..valid_env.clone()
            };
            assert!(!invalid_env.is_valid());

            // Invalid environment - zero slot
            let invalid_env = Environment {
                slot: 0,
                ..valid_env
            };
            assert!(!invalid_env.is_valid());
        }

        #[test]
        fn test_environment_default() {
            let env = Environment::default();
            assert_eq!(env.slot, 0);
            assert_eq!(env.epoch, 0);
            assert_eq!(env.timestamp, 0);
            assert!(!env.is_valid()); // Should be invalid by default
        }
    }

    /// Cross-function integration tests
    mod cross_function_tests {
        use super::*;

        #[test]
        fn test_function_registry_ids_unique() {
            // Verify all function IDs are unique
            use std::collections::HashSet;
            let mut ids = HashSet::new();
            
            ids.insert(identity::FUNCTION_ID);
            ids.insert(math_add::FUNCTION_ID);
            ids.insert(token_validate::FUNCTION_ID);
            
            // Should have same number of unique IDs as total IDs
            assert_eq!(ids.len(), 3, "Function IDs are not unique");
        }

        #[test]
        fn test_function_names_unique() {
            // Verify all function names are unique
            use std::collections::HashSet;
            let mut names = HashSet::new();
            
            names.insert(identity::FUNCTION_NAME);
            names.insert(math_add::FUNCTION_NAME);
            names.insert(token_validate::FUNCTION_NAME);
            
            assert_eq!(names.len(), 3, "Function names are not unique");
        }

        #[test]
        fn test_compute_units_reasonable() {
            // Verify compute units are reasonable
            assert!(identity::COMPUTE_UNITS > 0);
            assert!(identity::COMPUTE_UNITS < 10_000); // Simple function should be cheap
            
            assert!(math_add::COMPUTE_UNITS > identity::COMPUTE_UNITS); // More complex
            assert!(math_add::COMPUTE_UNITS < 10_000);
            
            assert!(token_validate::COMPUTE_UNITS > math_add::COMPUTE_UNITS); // Most complex
            assert!(token_validate::COMPUTE_UNITS < 50_000); // But not excessive
        }

        #[test]
        fn test_all_functions_have_version_1() {
            // Verify all functions start at version 1
            assert_eq!(identity::FUNCTION_VERSION, 1);
            assert_eq!(math_add::FUNCTION_VERSION, 1);
            assert_eq!(token_validate::FUNCTION_VERSION, 1);
        }
    }
}