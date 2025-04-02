use anchor_lang::prelude::*;
use anchor_spl::{token::{TokenAccount, Token, Mint}, associated_token::AssociatedToken};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
};
use token_transfer::{
    state::LibraryConfig,
    instructions::{
        initialize::{InitializeParams, Initialize},
        transfer_token::{TransferTokenParams, TransferToken},
        transfer_sol::{TransferSolParams, TransferSol},
        batch_transfer::{BatchTransferParams, TransferDestination, BatchTransfer},
        transfer_with_authority::{TransferWithAuthorityParams, TransferWithAuthority},
    },
};
use litesvm::LiteSVM;
use solana_program::{
    pubkey::Pubkey,
    instruction::AccountMeta,
};
use spl_token_2022::{
    instruction as token_instruction,
    state::{Account as TokenAccount},
    id as token_program_id,
};
use spl_associated_token_account::instruction as associated_token_instruction;
use token_transfer::{
    utils::token_helpers,
    instruction::{
        TokenTransferParams, 
    },
};
use std::str::FromStr;

// Utility function to create a program test for token_transfer
async fn setup_program_test() -> (ProgramTest, Keypair, Keypair, Pubkey) {
    // Create program test environment
    let mut program_test = ProgramTest::new(
        "token_transfer",
        token_transfer::ID,
        processor!(token_transfer::process_instruction),
    );

    // Generate keypairs for test accounts
    let payer = Keypair::new();
    let authority = Keypair::new();
    let processor_program = Keypair::new();
    let processor_program_id = processor_program.pubkey();

    // Add accounts with initial lamports
    program_test.add_account(
        payer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            ..Account::default()
        },
    );

    program_test.add_account(
        authority.pubkey(),
        Account {
            lamports: 10_000_000_000,
            ..Account::default()
        },
    );

    (program_test, payer, authority, processor_program_id)
}

// Test for initializing the token transfer library
#[tokio::test]
async fn test_initialize() {
    let (mut program_test, payer, authority, processor_program_id) = setup_program_test().await;
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive the library config address
    let (library_config_pda, _) = Pubkey::find_program_address(
        &[b"library_config"],
        &token_transfer::ID,
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
    let init_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &Initialize {
                payer: payer.pubkey(),
                library_config: library_config_pda,
                processor_program: processor_program_id,
                system_program: solana_program::system_program::ID,
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::InitializeInstruction { params: init_params }).unwrap(),
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
    assert_eq!(library_config.processor_program_id, processor_program_id);
    assert_eq!(library_config.max_transfer_amount, 1_000_000);
    assert_eq!(library_config.max_batch_size, 10);
    assert_eq!(library_config.is_active, true);
    assert_eq!(library_config.transfer_count, 0);
}

// Test for token transfer functionality
#[tokio::test]
async fn test_transfer_token() {
    let (mut program_test, payer, authority, processor_program_id) = setup_program_test().await;
    
    // Add SPL Token program
    program_test.add_program("spl_token", spl_token::id(), None);
    program_test.add_program("spl_associated_token_account", spl_associated_token::id(), None);
    
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive the library config address
    let (library_config_pda, _) = Pubkey::find_program_address(
        &[b"library_config"],
        &token_transfer::ID,
    );

    // Initialize library config
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

    let init_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &Initialize {
                payer: payer.pubkey(),
                library_config: library_config_pda,
                processor_program: processor_program_id,
                system_program: solana_program::system_program::ID,
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::InitializeInstruction { params: init_params }).unwrap(),
    };

    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create a test token mint
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    
    // Create source and destination owners
    let source_owner = Keypair::new();
    let destination_owner = Keypair::new();
    
    // Fund the account owners
    let fund_ix = system_instruction::transfer(
        &payer.pubkey(),
        &source_owner.pubkey(),
        1_000_000_000,
    );
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[fund_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    let fund_ix = system_instruction::transfer(
        &payer.pubkey(),
        &destination_owner.pubkey(),
        1_000_000_000,
    );
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[fund_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create mint
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);
    
    let create_mint_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_keypair.pubkey(),
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    );
    
    let initialize_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        &payer.pubkey(),
        None,
        6,
    ).unwrap();
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_mint_ix, initialize_mint_ix],
            Some(&payer.pubkey()),
            &[&payer, &mint_keypair],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create source token account
    let source_token_account = spl_associated_token_account::get_associated_token_address(
        &source_owner.pubkey(),
        &mint_pubkey,
    );
    
    let create_source_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &source_owner.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_source_account_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create destination token account
    let destination_token_account = spl_associated_token_account::get_associated_token_address(
        &destination_owner.pubkey(),
        &mint_pubkey,
    );
    
    let create_dest_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &destination_owner.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_dest_account_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Mint tokens to source account
    let mint_amount = 1_000_000;
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &source_token_account,
        &payer.pubkey(),
        &[],
        mint_amount,
    ).unwrap();
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[mint_to_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create fee collector account
    let fee_collector = Keypair::new();
    let fee_collector_token_account = spl_associated_token_account::get_associated_token_address(
        &fee_collector.pubkey(),
        &mint_pubkey,
    );
    
    let create_fee_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &fee_collector.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_fee_account_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create transfer instruction
    let transfer_amount = 500_000;
    let fee_amount = 5_000;
    let transfer_params = TransferTokenParams {
        amount: transfer_amount,
        fee_amount: Some(fee_amount),
        slippage_bps: Some(10), // 0.1%
        memo: Some("Test transfer".to_string()),
    };
    
    let transfer_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &TransferToken {
                library_config: library_config_pda,
                processor_program: processor_program_id,
                source_account: source_token_account,
                destination_account: destination_token_account,
                mint: mint_pubkey,
                authority: source_owner.pubkey(),
                fee_collector: Some(fee_collector_token_account),
                token_program: spl_token::id(),
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::TransferTokenInstruction { params: transfer_params }).unwrap(),
    };
    
    // Process the transfer
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&payer.pubkey()),
            &[&payer, &source_owner],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Verify the transfer results
    let source_account = banks_client.get_account(source_token_account).await.unwrap().unwrap();
    let source_token = spl_token::state::Account::unpack(&source_account.data).unwrap();
    
    let destination_account = banks_client.get_account(destination_token_account).await.unwrap().unwrap();
    let destination_token = spl_token::state::Account::unpack(&destination_account.data).unwrap();
    
    let fee_account = banks_client.get_account(fee_collector_token_account).await.unwrap().unwrap();
    let fee_token = spl_token::state::Account::unpack(&fee_account.data).unwrap();
    
    assert_eq!(source_token.amount, mint_amount - transfer_amount - fee_amount);
    assert_eq!(destination_token.amount, transfer_amount);
    assert_eq!(fee_token.amount, fee_amount);
    
    // Verify library stats
    let library_config_account = banks_client.get_account(library_config_pda).await.unwrap().unwrap();
    let library_config: LibraryConfig = anchor_lang::AccountDeserialize::try_deserialize(
        &mut library_config_account.data.as_ref(),
    ).unwrap();
    
    assert_eq!(library_config.transfer_count, 1);
    assert_eq!(library_config.total_volume, transfer_amount);
    assert_eq!(library_config.total_fees_collected, fee_amount);
}

// Test for SOL transfer functionality
#[tokio::test]
async fn test_transfer_sol() {
    let (mut program_test, payer, authority, processor_program_id) = setup_program_test().await;
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive the library config address
    let (library_config_pda, _) = Pubkey::find_program_address(
        &[b"library_config"],
        &token_transfer::ID,
    );

    // Initialize library config
    let init_params = InitializeParams {
        authority: authority.pubkey(),
        processor_program_id,
        max_transfer_amount: 1_000_000_000,
        max_batch_size: 10,
        fee_collector: None,
        enforce_recipient_allowlist: false,
        allowed_recipients: None,
        enforce_source_allowlist: false,
        allowed_sources: None,
        enforce_mint_allowlist: false,
        allowed_mints: None,
    };

    let init_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &Initialize {
                payer: payer.pubkey(),
                library_config: library_config_pda,
                processor_program: processor_program_id,
                system_program: solana_program::system_program::ID,
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::InitializeInstruction { params: init_params }).unwrap(),
    };

    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create sender and recipient
    let sender = Keypair::new();
    let recipient = Keypair::new();
    let fee_collector = Keypair::new();
    
    // Fund the sender account with 10 SOL
    let fund_amount = 10_000_000_000;
    let fund_ix = system_instruction::transfer(
        &payer.pubkey(),
        &sender.pubkey(),
        fund_amount,
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[fund_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create recipient account (just minimum rent)
    let rent = banks_client.get_rent().await.unwrap();
    let min_rent = rent.minimum_balance(0);
    
    let fund_recipient_ix = system_instruction::transfer(
        &payer.pubkey(),
        &recipient.pubkey(),
        min_rent,
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[fund_recipient_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create fee collector account (just minimum rent)
    let fund_fee_collector_ix = system_instruction::transfer(
        &payer.pubkey(),
        &fee_collector.pubkey(),
        min_rent,
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[fund_fee_collector_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Get initial balances
    let initial_sender_balance = banks_client.get_balance(sender.pubkey()).await.unwrap();
    let initial_recipient_balance = banks_client.get_balance(recipient.pubkey()).await.unwrap();
    let initial_fee_collector_balance = banks_client.get_balance(fee_collector.pubkey()).await.unwrap();
    
    // Create transfer parameters
    let transfer_amount = 5_000_000_000; // 5 SOL
    let fee_amount = 100_000_000; // 0.1 SOL
    let transfer_params = TransferSolParams {
        amount: transfer_amount,
        fee_amount: Some(fee_amount),
        memo: Some("Test SOL transfer".to_string()),
    };
    
    // Create transfer instruction
    let transfer_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &TransferSol {
                library_config: library_config_pda,
                processor_program: processor_program_id,
                sender: sender.pubkey(),
                recipient: recipient.pubkey(),
                fee_collector: Some(fee_collector.pubkey()),
                system_program: solana_program::system_program::ID,
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::TransferSolInstruction { params: transfer_params }).unwrap(),
    };
    
    // Process the transfer
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&payer.pubkey()),
            &[&payer, &sender],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Verify the balances after transfer
    let final_sender_balance = banks_client.get_balance(sender.pubkey()).await.unwrap();
    let final_recipient_balance = banks_client.get_balance(recipient.pubkey()).await.unwrap();
    let final_fee_collector_balance = banks_client.get_balance(fee_collector.pubkey()).await.unwrap();
    
    // Account for transaction fee in sender account (approximate)
    assert!(initial_sender_balance - final_sender_balance >= transfer_amount + fee_amount);
    assert_eq!(final_recipient_balance, initial_recipient_balance + transfer_amount);
    assert_eq!(final_fee_collector_balance, initial_fee_collector_balance + fee_amount);
    
    // Verify library stats
    let library_config_account = banks_client.get_account(library_config_pda).await.unwrap().unwrap();
    let library_config: LibraryConfig = anchor_lang::AccountDeserialize::try_deserialize(
        &mut library_config_account.data.as_ref(),
    ).unwrap();
    
    assert_eq!(library_config.transfer_count, 1);
    assert_eq!(library_config.total_volume, transfer_amount);
    assert_eq!(library_config.total_fees_collected, fee_amount);
}

// Test for batch transfer functionality
#[tokio::test]
async fn test_batch_transfer() {
    let (mut program_test, payer, authority, processor_program_id) = setup_program_test().await;
    
    // Add SPL Token program
    program_test.add_program("spl_token", spl_token::id(), None);
    program_test.add_program("spl_associated_token_account", spl_associated_token::id(), None);
    
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive the library config address
    let (library_config_pda, _) = Pubkey::find_program_address(
        &[b"library_config"],
        &token_transfer::ID,
    );

    // Initialize library config with larger batch size
    let init_params = InitializeParams {
        authority: authority.pubkey(),
        processor_program_id,
        max_transfer_amount: 1_000_000,
        max_batch_size: 5, // Allow up to 5 transfers in a batch
        fee_collector: None,
        enforce_recipient_allowlist: false,
        allowed_recipients: None,
        enforce_source_allowlist: false,
        allowed_sources: None,
        enforce_mint_allowlist: false,
        allowed_mints: None,
    };

    let init_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &Initialize {
                payer: payer.pubkey(),
                library_config: library_config_pda,
                processor_program: processor_program_id,
                system_program: solana_program::system_program::ID,
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::InitializeInstruction { params: init_params }).unwrap(),
    };

    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create a test token mint
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    
    // Create source owner
    let source_owner = Keypair::new();
    
    // Create destination owners (3 recipients)
    let destination_owners = vec![
        Keypair::new(),
        Keypair::new(), 
        Keypair::new(),
    ];
    
    // Fund the account owners
    let fund_ix = system_instruction::transfer(
        &payer.pubkey(),
        &source_owner.pubkey(),
        1_000_000_000,
    );
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[fund_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create mint
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);
    
    let create_mint_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_keypair.pubkey(),
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    );
    
    let initialize_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        &payer.pubkey(),
        None,
        6,
    ).unwrap();
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_mint_ix, initialize_mint_ix],
            Some(&payer.pubkey()),
            &[&payer, &mint_keypair],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create source token account
    let source_token_account = spl_associated_token_account::get_associated_token_address(
        &source_owner.pubkey(),
        &mint_pubkey,
    );
    
    let create_source_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &source_owner.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_source_account_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create destination token accounts
    let destination_token_accounts = destination_owners.iter().map(|owner| {
        spl_associated_token_account::get_associated_token_address(
            &owner.pubkey(),
            &mint_pubkey,
        )
    }).collect::<Vec<_>>();
    
    for (owner, _account) in destination_owners.iter().zip(destination_token_accounts.iter()) {
        let create_dest_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &owner.pubkey(),
            &mint_pubkey,
            &spl_token::id(),
        );
        
        banks_client.process_transaction(
            Transaction::new_signed_with_payer(
                &[create_dest_account_ix],
                Some(&payer.pubkey()),
                &[&payer],
                recent_blockhash,
            )
        ).await.unwrap();
    }
    
    // Create fee collector account
    let fee_collector = Keypair::new();
    let fee_collector_token_account = spl_associated_token_account::get_associated_token_address(
        &fee_collector.pubkey(),
        &mint_pubkey,
    );
    
    let create_fee_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &fee_collector.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_fee_account_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Mint tokens to source account
    let mint_amount = 10_000_000;
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &source_token_account,
        &payer.pubkey(),
        &[],
        mint_amount,
    ).unwrap();
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[mint_to_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create batch transfer parameters
    let transfer_destinations = vec![
        TransferDestination {
            destination: destination_token_accounts[0],
            amount: 1_000_000,
            memo: Some("Transfer 1".to_string()),
        },
        TransferDestination {
            destination: destination_token_accounts[1],
            amount: 2_000_000,
            memo: Some("Transfer 2".to_string()),
        },
        TransferDestination {
            destination: destination_token_accounts[2],
            amount: 3_000_000,
            memo: Some("Transfer 3".to_string()),
        },
    ];
    
    let fee_amount = 10_000;
    let batch_params = BatchTransferParams {
        destinations: transfer_destinations.clone(),
        fee_amount: Some(fee_amount),
    };
    
    // Calculate total amount excluding fee
    let total_transfer_amount: u64 = transfer_destinations.iter().map(|d| d.amount).sum();
    
    // Create batch transfer instruction
    let batch_transfer_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &BatchTransfer {
                library_config: library_config_pda,
                processor_program: processor_program_id,
                source_account: source_token_account,
                authority: source_owner.pubkey(),
                fee_collector: Some(fee_collector_token_account),
                token_program: spl_token::id(),
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::BatchTransferInstruction { params: batch_params }).unwrap(),
    };
    
    // Add remaining accounts (destination token accounts)
    let mut accounts = batch_transfer_ix.accounts;
    for acc in destination_token_accounts.iter() {
        accounts.push(AccountMeta::new(*acc, false));
    }
    
    let modified_batch_transfer_ix = solana_program::instruction::Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };
    
    // Process the batch transfer
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[modified_batch_transfer_ix],
            Some(&payer.pubkey()),
            &[&payer, &source_owner],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Verify the transfer results
    let source_account = banks_client.get_account(source_token_account).await.unwrap().unwrap();
    let source_token = spl_token::state::Account::unpack(&source_account.data).unwrap();
    
    // Verify source account balance (original - all transfers - fee)
    assert_eq!(source_token.amount, mint_amount - total_transfer_amount - fee_amount);
    
    // Verify destination account balances
    for (i, dest_account) in destination_token_accounts.iter().enumerate() {
        let account = banks_client.get_account(*dest_account).await.unwrap().unwrap();
        let token = spl_token::state::Account::unpack(&account.data).unwrap();
        
        assert_eq!(token.amount, transfer_destinations[i].amount);
    }
    
    // Verify fee collector account
    let fee_account = banks_client.get_account(fee_collector_token_account).await.unwrap().unwrap();
    let fee_token = spl_token::state::Account::unpack(&fee_account.data).unwrap();
    
    assert_eq!(fee_token.amount, fee_amount);
    
    // Verify library stats
    let library_config_account = banks_client.get_account(library_config_pda).await.unwrap().unwrap();
    let library_config: LibraryConfig = anchor_lang::AccountDeserialize::try_deserialize(
        &mut library_config_account.data.as_ref(),
    ).unwrap();
    
    // Should have 3 transfers counted (one for each destination)
    assert_eq!(library_config.transfer_count, 3);
    assert_eq!(library_config.total_volume, total_transfer_amount);
    assert_eq!(library_config.total_fees_collected, fee_amount);
}

// Test error handling (max transfer limit exceeded)
#[tokio::test]
async fn test_error_max_transfer_exceeded() {
    let (mut program_test, payer, authority, processor_program_id) = setup_program_test().await;
    
    // Add SPL Token program
    program_test.add_program("spl_token", spl_token::id(), None);
    program_test.add_program("spl_associated_token_account", spl_associated_token::id(), None);
    
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Derive the library config address
    let (library_config_pda, _) = Pubkey::find_program_address(
        &[b"library_config"],
        &token_transfer::ID,
    );

    // Initialize library config with low max transfer amount
    let max_transfer_amount = 1_000; // Very low limit for testing
    let init_params = InitializeParams {
        authority: authority.pubkey(),
        processor_program_id,
        max_transfer_amount,
        max_batch_size: 10,
        fee_collector: None,
        enforce_recipient_allowlist: false,
        allowed_recipients: None,
        enforce_source_allowlist: false,
        allowed_sources: None,
        enforce_mint_allowlist: false,
        allowed_mints: None,
    };

    let init_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &Initialize {
                payer: payer.pubkey(),
                library_config: library_config_pda,
                processor_program: processor_program_id,
                system_program: solana_program::system_program::ID,
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::InitializeInstruction { params: init_params }).unwrap(),
    };

    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[init_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create a test token mint and accounts
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let source_owner = Keypair::new();
    let destination_owner = Keypair::new();
    
    // Fund accounts, create mint, and token accounts
    // (Similar setup code as previous tests)
    // ...
    
    // Set up token mint and accounts
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);
    
    // Create and initialize mint
    let create_mint_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_keypair.pubkey(),
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    );
    
    let initialize_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        &payer.pubkey(),
        None,
        6,
    ).unwrap();
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_mint_ix, initialize_mint_ix],
            Some(&payer.pubkey()),
            &[&payer, &mint_keypair],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create source and destination token accounts
    let source_token_account = spl_associated_token_account::get_associated_token_address(
        &source_owner.pubkey(),
        &mint_pubkey,
    );
    
    let destination_token_account = spl_associated_token_account::get_associated_token_address(
        &destination_owner.pubkey(),
        &mint_pubkey,
    );
    
    // Fund source owner
    let fund_ix = system_instruction::transfer(
        &payer.pubkey(),
        &source_owner.pubkey(),
        1_000_000_000,
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[fund_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Create token accounts
    let create_source_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &source_owner.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    let create_dest_account_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &destination_owner.pubkey(),
        &mint_pubkey,
        &spl_token::id(),
    );
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[create_source_account_ix, create_dest_account_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Mint tokens to source account
    let mint_amount = 100_000;
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &source_token_account,
        &payer.pubkey(),
        &[],
        mint_amount,
    ).unwrap();
    
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[mint_to_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        )
    ).await.unwrap();

    // Try to transfer more than the limit
    let transfer_amount = max_transfer_amount + 1000; // Exceeds max amount
    let transfer_params = TransferTokenParams {
        amount: transfer_amount,
        fee_amount: None,
        slippage_bps: None,
        memo: None,
    };
    
    // Create transfer instruction
    let transfer_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &TransferToken {
                library_config: library_config_pda,
                processor_program: processor_program_id,
                source_account: source_token_account,
                destination_account: destination_token_account,
                mint: mint_pubkey,
                authority: source_owner.pubkey(),
                fee_collector: None,
                token_program: spl_token::id(),
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::TransferTokenInstruction { params: transfer_params }).unwrap(),
    };
    
    // Process the transfer - should fail with TransferAmountExceedsLimit error
    let result = banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&payer.pubkey()),
            &[&payer, &source_owner],
            recent_blockhash,
        )
    ).await;
    
    // Verify that the transaction failed as expected
    assert!(result.is_err());
    
    // Try transferring an amount within the limit - should succeed
    let valid_transfer_amount = max_transfer_amount / 2;
    let valid_transfer_params = TransferTokenParams {
        amount: valid_transfer_amount,
        fee_amount: None,
        slippage_bps: None,
        memo: None,
    };
    
    let valid_transfer_ix = solana_program::instruction::Instruction {
        program_id: token_transfer::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &TransferToken {
                library_config: library_config_pda,
                processor_program: processor_program_id,
                source_account: source_token_account,
                destination_account: destination_token_account,
                mint: mint_pubkey,
                authority: source_owner.pubkey(),
                fee_collector: None,
                token_program: spl_token::id(),
            },
            None,
        ),
        data: anchor_lang::AnchorSerialize::try_to_vec(&token_transfer::instruction::TransferTokenInstruction { params: valid_transfer_params }).unwrap(),
    };
    
    // This should succeed
    banks_client.process_transaction(
        Transaction::new_signed_with_payer(
            &[valid_transfer_ix],
            Some(&payer.pubkey()),
            &[&payer, &source_owner],
            recent_blockhash,
        )
    ).await.unwrap();
    
    // Verify successful transfer
    let destination_account = banks_client.get_account(destination_token_account).await.unwrap().unwrap();
    let destination_token = spl_token::state::Account::unpack(&destination_account.data).unwrap();
    
    assert_eq!(destination_token.amount, valid_transfer_amount);
}

// Additional tests that could be implemented:
// - test_transfer_with_authority
// - test_error_insufficient_funds 
// - test_error_invalid_fee_amount
// - test_allowlist_validation 

fn create_associated_token_account(
    svm: &mut LiteSVM, 
    payer: &Keypair, 
    wallet: &Pubkey, 
    mint: &Pubkey
) -> Pubkey {
    // Find associated token address
    let token_address = spl_associated_token_account::get_associated_token_address_with_program_id(
        wallet,
        mint,
        &token_program_id(),
    );
    
    // Create associated token account
    let create_ata_ix = associated_token_instruction::create_associated_token_account(
        &payer.pubkey(),
        wallet,
        mint,
        &token_program_id(),
    );
    
    let tx = Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&payer.pubkey()),
        &[payer],
        svm.last_blockhash(),
    );
    
    svm.process_transaction(tx).unwrap();
    
    token_address
}

fn mint_to(
    svm: &mut LiteSVM,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Keypair,
    amount: u64,
) {
    let mint_to_ix = token_instruction::mint_to(
        &token_program_id(),
        mint,
        destination,
        &authority.pubkey(),
        &[],
        amount,
    ).unwrap();
    
    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&authority.pubkey()),
        &[authority],
        svm.last_blockhash(),
    );
    
    svm.process_transaction(tx).unwrap();
}

#[test]
fn test_token_transfer_library() {
    // Initialize the LiteSVM instance
    let mut svm = LiteSVM::new();
    
    // Add token program
    svm.add_program_from_name(&token_program_id(), "spl_token_2022").unwrap();
    
    // Add our token_transfer program
    let program_keypair = Keypair::new();
    let program_id = program_keypair.pubkey();
    
    // Create test accounts
    let authority = Keypair::new();
    let user = Keypair::new();
    let recipient = Keypair::new();
    let fee_receiver = Keypair::new();
    
    // Fund accounts
    let fund_lamports = 10_000_000_000; // 10 SOL
    svm.transfer(&svm.payer().pubkey(), &authority.pubkey(), fund_lamports).unwrap();
    svm.transfer(&svm.payer().pubkey(), &user.pubkey(), fund_lamports).unwrap();
    svm.transfer(&svm.payer().pubkey(), &recipient.pubkey(), fund_lamports).unwrap();
    svm.transfer(&svm.payer().pubkey(), &fee_receiver.pubkey(), fund_lamports).unwrap();
    
    // Create mint
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    
    // Create mint account
    let mint_rent = svm.get_rent().minimum_balance(Mint::LEN);
    let create_mint_account_ix = system_instruction::create_account(
        &authority.pubkey(),
        &mint_pubkey,
        mint_rent,
        Mint::LEN as u64,
        &token_program_id(),
    );
    
    // Initialize mint (6 decimals)
    let mint_decimals = 6;
    let init_mint_ix = token_instruction::initialize_mint(
        &token_program_id(),
        &mint_pubkey,
        &authority.pubkey(),
        None,
        mint_decimals,
    ).unwrap();
    
    // Process mint setup
    let mint_tx = Transaction::new_signed_with_payer(
        &[create_mint_account_ix, init_mint_ix],
        Some(&authority.pubkey()),
        &[&authority, &mint_keypair],
        svm.last_blockhash(),
    );
    
    svm.process_transaction(mint_tx).unwrap();
    
    // Create token accounts
    let authority_token_account = create_associated_token_account(
        &mut svm,
        &authority,
        &authority.pubkey(),
        &mint_pubkey,
    );
    
    let user_token_account = create_associated_token_account(
        &mut svm,
        &authority,
        &user.pubkey(),
        &mint_pubkey,
    );
    
    let recipient_token_account = create_associated_token_account(
        &mut svm,
        &authority,
        &recipient.pubkey(),
        &mint_pubkey,
    );
    
    let fee_token_account = create_associated_token_account(
        &mut svm,
        &authority,
        &fee_receiver.pubkey(),
        &mint_pubkey,
    );
    
    // Mint tokens
    let initial_mint_amount = 1000 * 10u64.pow(mint_decimals);
    
    // Mint to authority and user
    mint_to(&mut svm, &mint_pubkey, &authority_token_account, &authority, initial_mint_amount);
    mint_to(&mut svm, &mint_pubkey, &user_token_account, &authority, initial_mint_amount);
    
    // Find config PDA
    let (config_pda, _config_bump) = Pubkey::find_program_address(
        &[
            b"token_transfer_config",
            authority.pubkey().as_ref(),
        ],
        &program_id,
    );
    
    // Find token authority PDA
    let (_token_authority_pda, _token_authority_bump) = Pubkey::find_program_address(
        &[
            b"token_authority",
            user_token_account.as_ref(),
        ],
        &program_id,
    );
    
    // Test constants
    const TEST_FEE_BPS: u16 = 100; // 1%
    const TEST_TRANSFER_AMOUNT: u64 = 50 * 10u64.pow(6); // 50 tokens
    
    // Initialize the token transfer library
    // First build the instruction manually
    let init_accounts = vec![
        AccountMeta::new(authority.pubkey(), true),
        AccountMeta::new(config_pda, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    
    let max_transfer_amount = 1_000_000 * 10u64.pow(mint_decimals);
    let params = InitializeParams {
        processor_program_id: authority.pubkey(), // In test we use authority as processor
        max_transfer_amount,
        max_batch_size: 10,
        fee_bps: TEST_FEE_BPS,
        fee_collector: fee_receiver.pubkey(),
        slippage_bps: 0,
        validate_account_ownership: true,
        enforce_source_allowlist: false,
        enforce_recipient_allowlist: false,
        enforce_mint_allowlist: false,
        allowed_sources: vec![],
        allowed_recipients: vec![],
        allowed_mints: vec![],
    };
    
    let init_data = token_transfer::instruction::Initialize { params }.data();
    
    let init_ix = Instruction {
        program_id,
        accounts: init_accounts,
        data: init_data,
    };
    
    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&authority.pubkey()),
        &[&authority],
        svm.last_blockhash(),
    );
    
    svm.process_transaction(init_tx).unwrap();
    
    // Check user balances before transfer
    let pre_source_account = svm.get_account(&user_token_account).unwrap();
    let pre_source_token_account = TokenAccount::unpack(&pre_source_account.data).unwrap();
    let pre_source_balance = pre_source_token_account.amount;
    
    let pre_dest_account = svm.get_account(&recipient_token_account).unwrap();
    let pre_dest_token_account = TokenAccount::unpack(&pre_dest_account.data).unwrap();
    let pre_dest_balance = pre_dest_token_account.amount;
    
    let pre_fee_account = svm.get_account(&fee_token_account).unwrap();
    let pre_fee_token_account = TokenAccount::unpack(&pre_fee_account.data).unwrap();
    let pre_fee_balance = pre_fee_token_account.amount;
    
    // Transfer tokens with fee collection
    let transfer_accounts = vec![
        AccountMeta::new(authority.pubkey(), true),
        AccountMeta::new_readonly(config_pda, false),
        AccountMeta::new(user_token_account, false),
        AccountMeta::new(recipient_token_account, false),
        AccountMeta::new(fee_token_account, false),
        AccountMeta::new_readonly(token_program_id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::ID, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];
    
    let transfer_params = TokenTransferParams {
        amount: TEST_TRANSFER_AMOUNT,
        source_owner: user.pubkey(),
        destination_owner: recipient.pubkey(),
        slippage_bps: None,
        memo: Some("Test transfer".to_string()),
    };
    
    let transfer_data = token_transfer::instruction::TransferToken { params: transfer_params }.data();
    
    let transfer_ix = Instruction {
        program_id,
        accounts: transfer_accounts,
        data: transfer_data,
    };
    
    let transfer_tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&authority.pubkey()),
        &[&authority, &user],
        svm.last_blockhash(),
    );
    
    svm.process_transaction(transfer_tx).unwrap();
    
    // Verify balances after transfer
    let post_source_account = svm.get_account(&user_token_account).unwrap();
    let post_source_token_account = TokenAccount::unpack(&post_source_account.data).unwrap();
    let post_source_balance = post_source_token_account.amount;
    
    let post_dest_account = svm.get_account(&recipient_token_account).unwrap();
    let post_dest_token_account = TokenAccount::unpack(&post_dest_account.data).unwrap();
    let post_dest_balance = post_dest_token_account.amount;
    
    let post_fee_account = svm.get_account(&fee_token_account).unwrap();
    let post_fee_token_account = TokenAccount::unpack(&post_fee_account.data).unwrap();
    let post_fee_balance = post_fee_token_account.amount;
    
    // Calculate expected fee
    let expected_fee = TEST_TRANSFER_AMOUNT * TEST_FEE_BPS as u64 / 10000;
    let expected_transfer = TEST_TRANSFER_AMOUNT - expected_fee;
    
    // Assert correct balances
    assert_eq!(post_source_balance, pre_source_balance - TEST_TRANSFER_AMOUNT);
    assert_eq!(post_dest_balance, pre_dest_balance + expected_transfer);
    assert_eq!(post_fee_balance, pre_fee_balance + expected_fee);
} 