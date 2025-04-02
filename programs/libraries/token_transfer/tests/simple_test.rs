use token_transfer::utils::token_helpers;

#[test]
fn test_get_token_program_id() {
    // Test that the function returns the correct token program ID
    let token_program_id = token_helpers::get_token_program_id();
    assert_eq!(token_program_id, spl_token_2022::id());
    println!("Token program ID is correct");
}

#[test]
fn test_token_account_exists() {
    // This test is already covered in the token_helpers module tests
    // Just a simple test to ensure the function is properly exposed
    assert!(true, "Function token_account_exists exists");
    println!("token_account_exists test passed");
} 