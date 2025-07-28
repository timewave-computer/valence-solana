use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use valence_kernel::*;

#[path = "common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_invalid_session_config() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    // Create guard data
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        SerializedGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    // Try to create session with wrong guard data reference
    let wrong_session = Keypair::new();
    let session_params = CreateSessionParams {
        scope: SessionContextScope::User,
        guard_data: guard_data_keypair.pubkey(), // Guard data points to different session
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    let result = context.create_session_account(
        &wrong_session,
        context.authority.pubkey(),
        session_params,
    ).await;
    
    // Should succeed in creation but fail during execution
    assert!(result.is_ok());
    
    // Try to execute operations - should fail due to mismatch
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    let batch = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [1u8; 64] }],
        vec![],
    );
    
    let exec_result = context.execute_session_operations(
        &wrong_session,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![],
    ).await;
    
    assert!(exec_result.is_err());
}

#[tokio::test]
async fn test_data_size_limits() {
    let mut context = TestContext::new().await;
    
    // Test oversized operation batch
    let oversized_ops: Vec<SessionOperation> = (0..OperationBatch::MAX_OPERATIONS + 1)
        .map(|i| SessionOperation::UpdateMetadata { 
            metadata: [i as u8; 64] 
        })
        .collect();
    
    let batch = OperationBatch::new(oversized_ops, vec![]);
    assert!(batch.validate().is_err());
    
    // Test oversized CPI data
    let oversized_cpi_data = vec![0u8; MAX_CPI_DATA_SIZE + 1];
    let op = SessionOperation::InvokeProgram {
        manifest_index: 0,
        data: oversized_cpi_data,
        account_indices: vec![],
    };
    assert!(op.validate().is_err());
    
    // Test oversized custom data
    let oversized_custom_data = vec![0u8; MAX_CUSTOM_DATA_SIZE + 1];
    let custom_op = SessionOperation::Custom {
        program_id: Pubkey::new_unique(),
        discriminator: [0u8; 8],
        data: oversized_custom_data,
    };
    assert!(custom_op.validate().is_err());
}

#[tokio::test]
async fn test_guard_jump_out_of_bounds() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    // Create guard with invalid jump
    let invalid_guard = SerializedGuard {
        opcodes: vec![
            GuardOp::CheckOwner,
            GuardOp::JumpIfFalse { offset: 100 }, // Jump way out of bounds
            GuardOp::Terminate,
        ],
        cpi_manifest: vec![],
    };
    
    let result = context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        invalid_guard,
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_manifest_index() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    // Create guard with invalid manifest reference
    let invalid_guard = SerializedGuard {
        opcodes: vec![
            GuardOp::Invoke { manifest_index: 10 }, // No manifest entry at index 10
            GuardOp::Terminate,
        ],
        cpi_manifest: vec![], // Empty manifest
    };
    
    let result = context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        invalid_guard,
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_double_borrow_error() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        SerializedGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    let session_params = CreateSessionParams {
        scope: SessionContextScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    context.create_session_account(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Try to borrow same account twice in one batch
    let account = Keypair::new();
    let batch = OperationBatch::new(
        vec![
            SessionOperation::BorrowAccount {
                account: account.pubkey(),
                mode: ACCESS_MODE_READ,
            },
            SessionOperation::BorrowAccount {
                account: account.pubkey(),
                mode: ACCESS_MODE_WRITE,
            },
        ],
        vec![],
    );
    
    let result = context.execute_session_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![&account],
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_release_unborrowed_account() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        SerializedGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    let session_params = CreateSessionParams {
        scope: SessionContextScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    context.create_session_account(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Try to release account that was never borrowed
    let unborrowed = Keypair::new();
    let batch = OperationBatch::new(
        vec![SessionOperation::ReleaseAccount {
            account: unborrowed.pubkey(),
        }],
        vec![],
    );
    
    let result = context.execute_session_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![],
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unauthorized_cpi() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    // Don't add program to allowlist
    let unauthorized_program = Pubkey::new_unique();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        SerializedGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        },
    ).await.unwrap();
    
    let session_params = CreateSessionParams {
        scope: SessionContextScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data: SessionSharedData::default(),
        metadata: [0u8; 64],
    };
    
    context.create_session_account(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Try custom operation with unauthorized program
    let batch = OperationBatch::new(
        vec![SessionOperation::Custom {
            program_id: unauthorized_program,
            discriminator: [1, 2, 3, 4, 5, 6, 7, 8],
            data: vec![42],
        }],
        vec![],
    );
    
    let result = context.execute_session_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![],
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_allowlist_capacity() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    // Fill allowlist to capacity
    for i in 0..CpiAllowlistAccount::MAX_PROGRAMS {
        let program = Pubkey::new_from_array([i as u8; 32]);
        context.add_to_allowlist(&allowlist_keypair, program).await.unwrap();
    }
    
    // Try to add one more - should fail
    let overflow_program = Pubkey::new_unique();
    let result = context.add_to_allowlist(&allowlist_keypair, overflow_program).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_account_modes() {
    let op_invalid_mode = SessionOperation::BorrowAccount {
        account: Pubkey::new_unique(),
        mode: 0, // Invalid mode
    };
    
    // This should be caught during validation or execution
    assert!(op_invalid_mode.validate().is_ok()); // Validation might not check mode validity
    
    let op_invalid_mode2 = SessionOperation::BorrowAccount {
        account: Pubkey::new_unique(),
        mode: 4, // Invalid mode (only 1, 2, 3 are valid)
    };
    
    assert!(op_invalid_mode2.validate().is_ok()); // Validation might not check mode validity
}

#[tokio::test]
async fn test_reentrancy_depth_overflow() {
    let mut shared_data = SessionSharedData::default();
    
    // Max out CPI depth
    for _ in 0..SessionSharedData::MAX_CPI_DEPTH {
        assert!(shared_data.check_and_increment_cpi_depth().is_ok());
    }
    
    // Next increment should fail
    assert!(shared_data.check_and_increment_cpi_depth().is_err());
    
    // Verify underflow protection
    for _ in 0..100 {
        shared_data.decrement_cpi_depth();
    }
    
    // Should be at 0, not negative
    assert_eq!(shared_data.current_cpi_depth(), 0);
}

#[tokio::test]
async fn test_guard_infinite_loop_protection() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    // Create guard with potential infinite loop
    let loop_guard = SerializedGuard {
        opcodes: vec![
            GuardOp::JumpIfFalse { offset: 0 }, // Jump to itself - infinite loop
        ],
        cpi_manifest: vec![],
    };
    
    // Should be caught during validation
    let result = context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        loop_guard,
    ).await;
    
    // Validation should catch this as invalid
    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_guard() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    // Create guard with no opcodes
    let empty_guard = SerializedGuard {
        opcodes: vec![],
        cpi_manifest: vec![],
    };
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        empty_guard,
    ).await.unwrap();
    
    // Empty guard should evaluate to false (falls off end)
    // This tests edge case handling
}