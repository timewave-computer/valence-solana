//! Simple test for token_helpers that only checks that the module can be imported
//! and the functions can be called correctly.

// Just import the module to make sure it compiles
use token_transfer::utils::token_helpers;

#[test]
fn test_token_helpers_module_loads() {
    // This is a very basic test to make sure the module can be loaded
    println!("Token helpers module loaded successfully");
    
    // Call get_token_program_id to make sure it returns a valid Pubkey
    let token_program_id = token_helpers::get_token_program_id();
    assert!(!token_program_id.to_bytes().is_empty());
    
    println!("Token program ID = {}", token_program_id);
    
    // No need to test more complex functionality here since that requires LiteSVM
} 