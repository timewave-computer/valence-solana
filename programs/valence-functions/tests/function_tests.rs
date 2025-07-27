// Function system tests for valence-functions

use anchor_lang::prelude::*;
use valence_functions::*;
use valence_functions::function_core::PureFunction;
use valence_functions::functions::common::IdentityFunction;
use valence_functions::functions::composition::{ComposedFunction, VersionedFunction};

fn mock_environment() -> Environment {
    Environment {
        caller: Pubkey::new_unique(),
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [0u8; 32],
    }
}

#[test]
fn test_pure_functions() {
    let identity = IdentityFunction;
    let env = mock_environment();
    
    let input = 42u64;
    let result = identity.execute(&input, &env).unwrap();
    assert_eq!(result, input);
    assert!(identity.is_deterministic());
    assert!(identity.metadata().is_some());
}

#[test]
fn test_function_composition() {
    let f1 = IdentityFunction;
    let f2 = IdentityFunction;
    let _composed = ComposedFunction::new(f1, f2);
    
    let versioned = VersionedFunction::new(1, IdentityFunction);
    assert_eq!(versioned.version, 1);
    assert!(!versioned.deprecated);
} 