//! E2E test for capability enforcement

use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[tokio::test]
async fn test_capability_enforcement() {
    // Initialize test environment
    let mut pt = ProgramTest::new(
        "valence_shard",
        crate::ID,
        processor!(crate::entry)
    );
    
    // Add registry program
    pt.add_program(
        "valence_registry",
        valence_registry::ID,
        None
    );
    
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Step 1: Register transfer function with TRANSFER capability requirement
    let function_hash = [1u8; 32]; // Test hash for transfer function
    let transfer_capability = vec!["transfer".to_string()];
    
    let register_ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: valence_registry::ID,
        accounts: valence_registry::accounts::Register {
            authority: payer.pubkey(),
            function_entry: Pubkey::find_program_address(
                &[b"function", &function_hash],
                &valence_registry::ID,
            ).0,
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: valence_registry::instruction::Register {
            hash: function_hash,
            program: crate::ID,
            required_capabilities: transfer_capability.clone(),
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[register_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    // Step 2: Initialize shard
    let shard_config = Pubkey::find_program_address(
        &[b"shard_config", &payer.pubkey().to_bytes()],
        &crate::ID,
    ).0;
    
    let init_ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::Initialize {
            authority: payer.pubkey(),
            shard_config,
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: crate::instruction::Initialize {
            max_operations_per_bundle: 10,
            default_respect_deregistration: true,
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    // Step 3: Import the transfer function
    let function_import = Keypair::new();
    let import_ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::ImportFunction {
            authority: payer.pubkey(),
            shard_config,
            function_import: function_import.pubkey(),
            function_entry: Pubkey::find_program_address(
                &[b"function", &function_hash],
                &valence_registry::ID,
            ).0,
            registry_program: valence_registry::ID,
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: crate::instruction::ImportFunction {
            function_hash,
            respect_deregistration: Some(true),
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[import_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &function_import], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    // Step 4: Create session WITHOUT transfer capability
    let session_request = Keypair::new();
    let request_ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::RequestSession {
            owner: payer.pubkey(),
            session_request: session_request.pubkey(),
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: crate::instruction::RequestSession {
            capabilities: vec!["read".to_string()], // Only READ capability
            init_state_hash: [0u8; 32],
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[request_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &session_request], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    // Initialize the session
    let session = Keypair::new();
    let init_session_ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::InitializeSession {
            initializer: payer.pubkey(),
            session_request: session_request.pubkey(),
            session: session.pubkey(),
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: crate::instruction::InitializeSession {
            request_id: session_request.pubkey(),
            init_state_data: vec![],
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[init_session_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &session], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    // Step 5: Try to execute transfer function - should FAIL
    let bundle = crate::Bundle {
        operations: vec![crate::Operation {
            function_hash,
            args: vec![1, 2, 3], // dummy args
            expected_diff: None,
        }],
        mode: crate::ExecutionMode::Sync,
    };
    
    let execute_ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::ExecuteSyncBundle {
            executor: payer.pubkey(),
            session: session.pubkey(),
            shard_config,
        }.to_account_metas(None),
        data: crate::instruction::ExecuteSyncBundle { bundle }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[execute_ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    // This should fail with InsufficientCapabilities error
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
    
    // Step 6: Create session WITH transfer capability
    let session_request2 = Keypair::new();
    let request_ix2 = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::RequestSession {
            owner: payer.pubkey(),
            session_request: session_request2.pubkey(),
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: crate::instruction::RequestSession {
            capabilities: vec!["transfer".to_string()], // TRANSFER capability
            init_state_hash: [0u8; 32],
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[request_ix2],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &session_request2], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    // Initialize the session with transfer capability
    let session2 = Keypair::new();
    let init_session_ix2 = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::InitializeSession {
            initializer: payer.pubkey(),
            session_request: session_request2.pubkey(),
            session: session2.pubkey(),
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: crate::instruction::InitializeSession {
            request_id: session_request2.pubkey(),
            init_state_data: vec![],
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[init_session_ix2],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &session2], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    // Step 7: Execute transfer function - should SUCCEED
    let execute_ix2 = anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::ExecuteSyncBundle {
            executor: payer.pubkey(),
            session: session2.pubkey(),
            shard_config,
        }.to_account_metas(None),
        data: crate::instruction::ExecuteSyncBundle { 
            bundle: bundle.clone() 
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[execute_ix2],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    // This should succeed
    banks_client.process_transaction(transaction).await.unwrap();
    
    println!("✅ Capability enforcement test passed!");
    println!("   - Function execution failed without required capability");
    println!("   - Function execution succeeded with required capability");
}