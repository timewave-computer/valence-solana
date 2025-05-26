use anchor_lang::prelude::*;
use litesvm::LiteSVM;
use anchor_lang::solana_program::pubkey::Pubkey;
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token_2022::instruction as token_instruction;
use token_transfer::utils::token_helpers;

#[test]
fn test_get_token_program_id() {
    // Test that the function returns the correct token program ID
    let token_program_id = token_helpers::get_token_program_id();
    assert_eq!(token_program_id, spl_token_2022::id());
}

#[test]
fn test_token_account_exists() {
    // Initialize LiteSVM
    let mut svm = LiteSVM::new();
    
    // Create a token account with data and lamports
    let account_keypair = Keypair::new();
    let account_pubkey = account_keypair.pubkey();
    
    // Create account with space and lamports
    let rent = svm.get_rent().minimum_balance(100);
    let create_account_ix = system_instruction::create_account(
        &svm.payer().pubkey(),
        &account_pubkey,
        rent,
        100,
        &spl_token_2022::id(),
    );
    
    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix],
        Some(&svm.payer().pubkey()),
        &[&svm.payer(), &account_keypair],
        svm.last_blockhash(),
    );
    
    // Process transaction
    svm.process_transaction(transaction).unwrap();
    
    // Fetch account info
    let account = svm.get_account(&account_pubkey).unwrap();
    let account_info = AccountInfo::new(
        &account_pubkey,
        false,
        false,
        &mut account.lamports,
        &mut account.data,
        &account.owner,
        false,
        0,
    );
    
    // Test account exists
    assert!(token_helpers::token_account_exists(&account_info));
    
    // Test with non-existent account
    let empty_keypair = Keypair::new();
    let empty_pubkey = empty_keypair.pubkey();
    let empty_account_option = svm.get_account(&empty_pubkey);
    
    // Empty account should return None from SVM
    assert!(empty_account_option.is_none());
    
    // Create an AccountInfo with 0 lamports and empty data
    let mut zero_lamports = 0;
    let mut empty_data = vec![];
    let system_program_id = Pubkey::default();
    
    let empty_account_info = AccountInfo::new(
        &empty_pubkey,
        false,
        false,
        &mut zero_lamports,
        &mut empty_data,
        &system_program_id,
        false,
        0,
    );
    
    // Test account doesn't exist
    assert!(!token_helpers::token_account_exists(&empty_account_info));
}

#[test]
fn test_transfer_tokens() {
    // Initialize LiteSVM
    let mut svm = LiteSVM::new();
    
    // Add token-2022 program
    svm.add_program_from_name(&spl_token_2022::id(), "spl_token_2022").unwrap();
    
    // Create mint
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let mint_authority = svm.payer();
    
    // Create mint account
    let mint_rent = svm.get_rent().minimum_balance(spl_token_2022::state::Mint::LEN);
    let create_mint_account_ix = system_instruction::create_account(
        &svm.payer().pubkey(),
        &mint_pubkey,
        mint_rent,
        spl_token_2022::state::Mint::LEN as u64,
        &spl_token_2022::id(),
    );
    
    // Initialize mint
    let init_mint_ix = token_instruction::initialize_mint(
        &spl_token_2022::id(),
        &mint_pubkey,
        &mint_authority.pubkey(),
        None,
        9,
    ).unwrap();
    
    // Create token accounts for source and destination
    let source_keypair = Keypair::new();
    let source_pubkey = source_keypair.pubkey();
    let dest_keypair = Keypair::new();
    let dest_pubkey = dest_keypair.pubkey();
    
    // Create source account
    let token_account_rent = svm.get_rent().minimum_balance(spl_token_2022::state::Account::LEN);
    let create_source_account_ix = system_instruction::create_account(
        &svm.payer().pubkey(),
        &source_pubkey,
        token_account_rent,
        spl_token_2022::state::Account::LEN as u64,
        &spl_token_2022::id(),
    );
    
    // Initialize source account
    let init_source_ix = token_instruction::initialize_account(
        &spl_token_2022::id(),
        &source_pubkey,
        &mint_pubkey,
        &svm.payer().pubkey(),
    ).unwrap();
    
    // Create destination account
    let create_dest_account_ix = system_instruction::create_account(
        &svm.payer().pubkey(),
        &dest_pubkey,
        token_account_rent,
        spl_token_2022::state::Account::LEN as u64,
        &spl_token_2022::id(),
    );
    
    // Initialize destination account
    let init_dest_ix = token_instruction::initialize_account(
        &spl_token_2022::id(),
        &dest_pubkey,
        &mint_pubkey,
        &svm.payer().pubkey(),
    ).unwrap();
    
    // Mint tokens to source account
    let mint_amount = 1_000_000_000;
    let mint_to_ix = token_instruction::mint_to(
        &spl_token_2022::id(),
        &mint_pubkey,
        &source_pubkey,
        &mint_authority.pubkey(),
        &[],
        mint_amount,
    ).unwrap();
    
    // Create and process setup transaction
    let setup_transaction = Transaction::new_signed_with_payer(
        &[
            create_mint_account_ix,
            init_mint_ix,
            create_source_account_ix,
            init_source_ix,
            create_dest_account_ix,
            init_dest_ix,
            mint_to_ix,
        ],
        Some(&svm.payer().pubkey()),
        &[&svm.payer(), &mint_keypair, &source_keypair, &dest_keypair],
        svm.last_blockhash(),
    );
    
    // Process setup transaction
    svm.process_transaction(setup_transaction).unwrap();
    
    // Check source balance before transfer
    let source_account = svm.get_account(&source_pubkey).unwrap();
    let token_account = spl_token_2022::state::Account::unpack(&source_account.data).unwrap();
    assert_eq!(token_account.amount, mint_amount);
    
    // Create accounts for Anchor's CpiContext
    let transfer_amount = 500_000_000;
    let authority = svm.payer();
    
    // Create transfer instruction manually
    let transfer_ix = token_instruction::transfer(
        &spl_token_2022::id(),
        &source_pubkey,
        &dest_pubkey,
        &authority.pubkey(),
        &[],
        transfer_amount,
    ).unwrap();
    
    // Create and process transfer transaction
    let transfer_transaction = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&svm.payer().pubkey()),
        &[&svm.payer()],
        svm.last_blockhash(),
    );
    
    // Process transfer transaction
    svm.process_transaction(transfer_transaction).unwrap();
    
    // Check balances after transfer
    let source_account = svm.get_account(&source_pubkey).unwrap();
    let source_token_account = spl_token_2022::state::Account::unpack(&source_account.data).unwrap();
    assert_eq!(source_token_account.amount, mint_amount - transfer_amount);
    
    let dest_account = svm.get_account(&dest_pubkey).unwrap();
    let dest_token_account = spl_token_2022::state::Account::unpack(&dest_account.data).unwrap();
    assert_eq!(dest_token_account.amount, transfer_amount);
} 