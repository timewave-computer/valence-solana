// Token Transfer Library Tests
use anchor_lang::prelude::*;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    instruction::Instruction,
};
use borsh::BorshSerialize;
use token_transfer::{
    state::LibraryConfig,
    instructions::{
        initialize::{InitializeParams, Initialize},
        transfer_sol::{TransferSolParams, TransferSol},
    },
};

// Helper function to create instruction data
fn create_instruction_data<T: BorshSerialize>(discriminator: u8, params: &T) -> Vec<u8> {
    let mut data = vec![discriminator];
    data.extend_from_slice(&params.try_to_vec().unwrap());
    data
}

// Test for initializing the token transfer library
#[tokio::test]
async fn test_initialize() {
    let program_id = token_transfer::ID;
    let mut program_test = ProgramTest::new(
        "token_transfer",
        program_id,
        None,
    );

    // Generate keypairs for test accounts
    let payer = Keypair::new();
    let authority = Keypair::new();
    let processor_program_id = Pubkey::new_unique();

    // Add accounts with initial lamports
    program_test.add_account(
        payer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        authority.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive the library config address
    let (library_config_pda, _) = Pubkey::find_program_address(
        &[b"library_config"],
        &program_id,
    );

    // Create initialization parameters
    let init_params = InitializeParams {
        authority: authority.pubkey(),
        processor_program_id,
        max_transfer_amount: 1_000_000,
        max_batch_size: 10,
        fee_collector: None,
        enforce_recipient_allowlist: false,
        allowed_recipients: None,
        enforce_source_allowlist: false,
        allowed_sources: None,
        enforce_mint_allowlist: false,
        allowed_mints: None,
    };

    // Create the initialization instruction
    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(library_config_pda, false),
            AccountMeta::new_readonly(processor_program_id, false),
            AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        ],
        data: create_instruction_data(0, &init_params), // 0 is the discriminator for Initialize
    };

    // Create and send transaction
    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // Submit transaction and verify success
    banks_client.process_transaction(tx).await.unwrap();

    // Fetch and verify the library config account
    let library_config_account = banks_client
        .get_account(library_config_pda)
        .await
        .unwrap()
        .unwrap();

    let library_config: LibraryConfig = anchor_lang::AccountDeserialize::try_deserialize(
        &mut library_config_account.data.as_ref(),
    )
    .unwrap();

    assert_eq!(library_config.authority, authority.pubkey());
    assert_eq!(library_config.processor_program_id, Some(processor_program_id));
    assert_eq!(library_config.max_transfer_amount, 1_000_000);
    assert_eq!(library_config.max_batch_size, 10);
    assert_eq!(library_config.is_active, true);
    assert_eq!(library_config.transfer_count, 0);
}

// Test for SOL transfer functionality
#[tokio::test]
async fn test_transfer_sol() {
    let program_id = token_transfer::ID;
    let mut program_test = ProgramTest::new(
        "token_transfer",
        program_id,
        None,
    );

    // Generate keypairs for test accounts
    let payer = Keypair::new();
    let authority = Keypair::new();
    let sender = Keypair::new();
    let recipient = Keypair::new();
    let processor_program_id = Pubkey::new_unique();

    // Add accounts with initial lamports
    program_test.add_account(
        payer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        sender.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    program_test.add_account(
        recipient.pubkey(),
        Account {
            lamports: 1_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive the library config address
    let (library_config_pda, _) = Pubkey::find_program_address(
        &[b"library_config"],
        &program_id,
    );

    // Initialize library config first
    let init_params = InitializeParams {
        authority: authority.pubkey(),
        processor_program_id,
        max_transfer_amount: 1_000_000,
        max_batch_size: 10,
        fee_collector: None,
        enforce_recipient_allowlist: false,
        allowed_recipients: None,
        enforce_source_allowlist: false,
        allowed_sources: None,
        enforce_mint_allowlist: false,
        allowed_mints: None,
    };

    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(library_config_pda, false),
            AccountMeta::new_readonly(processor_program_id, false),
            AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        ],
        data: create_instruction_data(0, &init_params), // 0 is the discriminator for Initialize
    };

    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();

    // Now test SOL transfer
    let transfer_amount = 1_000_000; // 0.001 SOL
    let transfer_params = TransferSolParams {
        amount: transfer_amount,
        fee_amount: None,
        memo: Some("Test transfer".to_string()),
    };

    let transfer_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(library_config_pda, false),
            AccountMeta::new_readonly(processor_program_id, false),
            AccountMeta::new(sender.pubkey(), true),
            AccountMeta::new(recipient.pubkey(), false),
            AccountMeta::new_readonly(anchor_lang::system_program::ID, false),
        ],
        data: create_instruction_data(2, &transfer_params), // 2 is the discriminator for TransferSol
    };

    // Get initial balances
    let initial_sender_balance = banks_client.get_balance(sender.pubkey()).await.unwrap();
    let initial_recipient_balance = banks_client.get_balance(recipient.pubkey()).await.unwrap();

    // Create and send transfer transaction
    let transfer_tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client.process_transaction(transfer_tx).await.unwrap();

    // Verify balances changed correctly
    let final_sender_balance = banks_client.get_balance(sender.pubkey()).await.unwrap();
    let final_recipient_balance = banks_client.get_balance(recipient.pubkey()).await.unwrap();

    assert!(final_sender_balance < initial_sender_balance);
    assert_eq!(final_recipient_balance, initial_recipient_balance + transfer_amount);

    // Verify library config was updated
    let library_config_account = banks_client
        .get_account(library_config_pda)
        .await
        .unwrap()
        .unwrap();

    let library_config: LibraryConfig = anchor_lang::AccountDeserialize::try_deserialize(
        &mut library_config_account.data.as_ref(),
    )
    .unwrap();

    assert_eq!(library_config.transfer_count, 1);
    assert_eq!(library_config.total_volume, transfer_amount);
} 