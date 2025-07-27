use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use valence_core::*;

#[path = "common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_batch_operations() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    // Add test program for InvokeProgram operation
    let test_program = Pubkey::new_unique();
    context.add_to_allowlist(&allowlist_keypair, test_program).await.unwrap();
    
    // Setup session
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        CompiledGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    let session_params = CreateSessionParams {
        scope: SessionScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Create complex batch with multiple operation types
    let account1 = Keypair::new();
    let account2 = Keypair::new();
    
    let operations = vec![
        // Borrow accounts
        SessionOperation::BorrowAccount {
            account: account1.pubkey(),
            mode: MODE_READ_WRITE,
        },
        SessionOperation::BorrowAccount {
            account: account2.pubkey(),
            mode: MODE_READ,
        },
        
        // Update metadata
        SessionOperation::UpdateMetadata {
            metadata: [123u8; 64],
        },
        
        // Invoke program with borrowed accounts
        SessionOperation::InvokeProgram {
            manifest_index: 0,
            data: vec![1, 2, 3, 4, 5],
            account_indices: vec![0, 1], // Use both borrowed accounts
        },
        
        // Release one account
        SessionOperation::ReleaseAccount {
            account: account1.pubkey(),
        },
        
        // Custom operation
        SessionOperation::Custom {
            program_id: test_program,
            discriminator: [9, 8, 7, 6, 5, 4, 3, 2],
            data: vec![99, 98, 97],
        },
    ];
    
    let batch = OperationBatch {
        operations,
        auto_release: true,
        program_manifest: vec![
            ProgramManifestEntry {
                program_id: test_program,
            },
        ],
    };
    
    // Validate batch
    assert!(batch.validate().is_ok());
    assert_eq!(batch.compute_estimate(), 75_000); // Sum of all operation estimates
    assert!(batch.requires_write());
    
    // Execute batch
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![&account1, &account2],
    ).await.unwrap();
    
    // Verify session state
    let session_account = context.get_account(&session_keypair.pubkey()).await;
    let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
    
    assert_eq!(session.usage_count, 1);
    assert_eq!(session.metadata[0], 123);
    assert_eq!(session.borrowed_bitmap, 0); // All released due to auto_release
}

#[tokio::test]
async fn test_account_validation() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        CompiledGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    let session_params = CreateSessionParams {
        scope: SessionScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Test 1: Cannot borrow same account twice
    let account = Keypair::new();
    let batch_double_borrow = OperationBatch::new(
        vec![
            SessionOperation::BorrowAccount {
                account: account.pubkey(),
                mode: MODE_READ,
            },
            SessionOperation::BorrowAccount {
                account: account.pubkey(),
                mode: MODE_READ,
            },
        ],
        vec![],
    );
    
    let result = context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch_double_borrow,
        vec![&account],
    ).await;
    assert!(result.is_err());
    
    // Test 2: Cannot release account not borrowed
    let unborrowed = Keypair::new();
    let batch_invalid_release = OperationBatch::new(
        vec![
            SessionOperation::ReleaseAccount {
                account: unborrowed.pubkey(),
            },
        ],
        vec![],
    );
    
    let result2 = context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch_invalid_release,
        vec![],
    ).await;
    assert!(result2.is_err());
    
    // Test 3: Cannot use invalid account index in InvokeProgram
    let batch_invalid_index = OperationBatch::new(
        vec![
            SessionOperation::InvokeProgram {
                manifest_index: 0,
                data: vec![],
                account_indices: vec![7], // No account borrowed at index 7
            },
        ],
        vec![ProgramManifestEntry { program_id: Pubkey::new_unique() }],
    );
    
    let result3 = context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch_invalid_index,
        vec![],
    ).await;
    assert!(result3.is_err());
}

#[tokio::test]
async fn test_operation_data_validation() {
    let mut context = TestContext::new().await;
    
    // Test oversized CPI data
    let oversized_data = vec![0u8; MAX_CPI_DATA_SIZE + 1];
    let op = SessionOperation::InvokeProgram {
        manifest_index: 0,
        data: oversized_data,
        account_indices: vec![],
    };
    assert!(op.validate().is_err());
    
    // Test valid CPI data
    let valid_data = vec![0u8; MAX_CPI_DATA_SIZE];
    let op_valid = SessionOperation::InvokeProgram {
        manifest_index: 0,
        data: valid_data,
        account_indices: vec![],
    };
    assert!(op_valid.validate().is_ok());
    
    // Test oversized custom data
    let oversized_custom = vec![0u8; MAX_CUSTOM_DATA_SIZE + 1];
    let op_custom = SessionOperation::Custom {
        program_id: Pubkey::new_unique(),
        discriminator: [0u8; 8],
        data: oversized_custom,
    };
    assert!(op_custom.validate().is_err());
    
    // Test too many account indices
    let too_many_indices = vec![0u8; 9]; // Max is 8
    let op_indices = SessionOperation::InvokeProgram {
        manifest_index: 0,
        data: vec![],
        account_indices: too_many_indices,
    };
    assert!(op_indices.validate().is_err());
}

#[tokio::test]
async fn test_batch_validation() {
    // Test too many operations
    let too_many_ops: Vec<SessionOperation> = (0..OperationBatch::MAX_OPERATIONS + 1)
        .map(|_| SessionOperation::UpdateMetadata { metadata: [0u8; 64] })
        .collect();
    
    let batch = OperationBatch::new(too_many_ops, vec![]);
    assert!(batch.validate().is_err());
    
    // Test too many programs in manifest
    let too_many_programs: Vec<ProgramManifestEntry> = (0..OperationBatch::MAX_MANIFEST_SIZE + 1)
        .map(|i| ProgramManifestEntry {
            program_id: Pubkey::new_from_array([i as u8; 32]),
        })
        .collect();
    
    let batch2 = OperationBatch::new(vec![], too_many_programs);
    assert!(batch2.validate().is_err());
    
    // Test valid batch
    let valid_ops = vec![
        SessionOperation::UpdateMetadata { metadata: [1u8; 64] },
        SessionOperation::UpdateMetadata { metadata: [2u8; 64] },
    ];
    let valid_manifest = vec![
        ProgramManifestEntry { program_id: Pubkey::new_unique() },
    ];
    
    let valid_batch = OperationBatch::new(valid_ops, valid_manifest);
    assert!(valid_batch.validate().is_ok());
}

#[tokio::test]
async fn test_manual_release_vs_auto_release() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        CompiledGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    let session_params = CreateSessionParams {
        scope: SessionScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Test manual release (auto_release = false)
    let account1 = Keypair::new();
    let mut batch_manual = OperationBatch::new(
        vec![
            SessionOperation::BorrowAccount {
                account: account1.pubkey(),
                mode: MODE_READ,
            },
        ],
        vec![],
    );
    batch_manual.auto_release = false;
    
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch_manual,
        vec![&account1],
    ).await.unwrap();
    
    // Verify account is still borrowed
    let session_account = context.get_account(&session_keypair.pubkey()).await;
    let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
    assert_ne!(session.borrowed_bitmap, 0);
    
    // Manually release
    let batch_release = OperationBatch::new(
        vec![
            SessionOperation::ReleaseAccount {
                account: account1.pubkey(),
            },
        ],
        vec![],
    );
    
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch_release,
        vec![],
    ).await.unwrap();
    
    // Verify released
    let session_account2 = context.get_account(&session_keypair.pubkey()).await;
    let session2: Session = Session::try_from_slice(&session_account2.data[8..]).unwrap();
    assert_eq!(session2.borrowed_bitmap, 0);
}

#[tokio::test]
async fn test_access_modes() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        CompiledGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    let session_params = CreateSessionParams {
        scope: SessionScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Test different access modes
    let read_account = Keypair::new();
    let write_account = Keypair::new();
    let rw_account = Keypair::new();
    
    let batch = OperationBatch::new(
        vec![
            SessionOperation::BorrowAccount {
                account: read_account.pubkey(),
                mode: MODE_READ,
            },
            SessionOperation::BorrowAccount {
                account: write_account.pubkey(),
                mode: MODE_WRITE,
            },
            SessionOperation::BorrowAccount {
                account: rw_account.pubkey(),
                mode: MODE_READ_WRITE,
            },
        ],
        vec![],
    );
    
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![&read_account, &write_account, &rw_account],
    ).await.unwrap();
    
    // Verify modes are stored correctly
    let session_account = context.get_account(&session_keypair.pubkey()).await;
    let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
    
    assert_eq!(session.borrowed_accounts[0].mode, MODE_READ);
    assert!(session.borrowed_accounts[0].can_read());
    assert!(!session.borrowed_accounts[0].can_write());
    
    assert_eq!(session.borrowed_accounts[1].mode, MODE_WRITE);
    assert!(!session.borrowed_accounts[1].can_read());
    assert!(session.borrowed_accounts[1].can_write());
    
    assert_eq!(session.borrowed_accounts[2].mode, MODE_READ_WRITE);
    assert!(session.borrowed_accounts[2].can_read());
    assert!(session.borrowed_accounts[2].can_write());
}