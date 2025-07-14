use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;
use template_shard::*;
use template_functions::hello_world_function::HelloWorldInput;
use template_functions::math_ops_function::{MathInput, MathOperation};
use borsh::BorshSerialize;

#[test]
fn test_function_hash_generation() {
    // Example of creating a function hash
    let function_code = b"hello_world_function_bytecode";
    let function_hash = hash(function_code);
    
    println!("Function hash: {:?}", function_hash.to_bytes());
    assert_eq!(function_hash.to_bytes().len(), 32);
}

#[test]
fn test_hello_world_serialization() {
    // Test that hello world input serializes correctly
    let input = HelloWorldInput {
        name: "Test User".to_string(),
    };
    
    let serialized = input.try_to_vec().unwrap();
    assert!(serialized.len() > 0);
    
    // Test deserialization
    let deserialized = HelloWorldInput::try_from_slice(&serialized).unwrap();
    assert_eq!(deserialized.name, "Test User");
}

#[test]
fn test_math_ops_serialization() {
    // Test math operations input
    let input = MathInput {
        a: 10,
        b: 5,
        operation: MathOperation::Add,
    };
    
    let serialized = input.try_to_vec().unwrap();
    let deserialized = MathInput::try_from_slice(&serialized).unwrap();
    
    assert_eq!(deserialized.a, 10);
    assert_eq!(deserialized.b, 5);
    matches!(deserialized.operation, MathOperation::Add);
}

#[test]
fn test_shard_pda_derivation() {
    let program_id = Pubkey::new_unique();
    let (pda, bump) = Pubkey::find_program_address(&[b"shard"], &program_id);
    
    // Verify we can derive the PDA
    assert_ne!(pda, Pubkey::default());
    assert!(bump > 0);
}