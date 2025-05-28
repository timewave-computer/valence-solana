use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use token_transfer::utils::token_helpers;

#[test]
fn test_get_token_program_id() {
    // Test that the function returns the correct token program ID
    let token_program_id = token_helpers::get_token_program_id();
    assert_eq!(token_program_id, spl_token_2022::id());
}

#[test]
fn test_token_account_exists() {
    // Create mock AccountInfo with zero lamports
    let key = Pubkey::new_unique();
    let owner = Pubkey::default();
    let mut lamports = 0;
    let mut data = vec![];
    
    let non_existent_account = AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    
    // Test non-existent account
    assert!(!token_helpers::token_account_exists(&non_existent_account));
    
    // Create mock AccountInfo with lamports and data
    let mut lamports = 1000;
    let mut data = vec![0; 10];
    
    let existent_account = AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    
    // Test existent account
    assert!(token_helpers::token_account_exists(&existent_account));
} 