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
async fn test_owner_only_guard() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    // Create guard that checks owner
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    let owner = context.authority.pubkey();
    
    let compiled_guard = CompiledGuard {
        opcodes: vec![
            GuardOp::CheckOwner,
            GuardOp::JumpIfFalse { offset: 1 },
            GuardOp::Terminate,
            GuardOp::Abort,
        ],
        cpi_manifest: vec![],
    };
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        compiled_guard,
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
        owner,
        session_params,
    ).await.unwrap();
    
    // Owner should be able to execute
    let batch = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [1u8; 64] }],
        vec![],
    );
    
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![],
    ).await.unwrap();
    
    // Non-owner should fail
    let non_owner = Keypair::new();
    context.payer = non_owner; // Switch to non-owner
    
    let batch2 = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [2u8; 64] }],
        vec![],
    );
    
    let result = context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch2,
        vec![],
    ).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_complex_guard_logic() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    // Create guard with complex AND/OR logic:
    // (Owner AND NotExpired) OR (UsageLimit < 100)
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    let future_timestamp = context.get_clock().await.unix_timestamp + 3600;
    
    let compiled_guard = CompiledGuard {
        opcodes: vec![
            // Check first condition: Owner AND NotExpired
            GuardOp::CheckOwner,                          // 0
            GuardOp::JumpIfFalse { offset: 4 },           // 1: if not owner, jump to usage check at 5
            GuardOp::CheckExpiry { timestamp: future_timestamp }, // 2
            GuardOp::JumpIfFalse { offset: 2 },           // 3: if expired, jump to usage check at 5
            GuardOp::Terminate,                           // 4: both conditions met
            
            // Check second condition: UsageLimit
            GuardOp::CheckUsageLimit { limit: 100 },      // 5
            GuardOp::JumpIfFalse { offset: 1 },           // 6: if over limit, abort
            GuardOp::Terminate,                           // 7: usage under limit
            GuardOp::Abort,                               // 8: all conditions failed
        ],
        cpi_manifest: vec![],
    };
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        compiled_guard,
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
    
    // Test 1: Owner with valid timestamp - should pass via first branch
    let batch = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [1u8; 64] }],
        vec![],
    );
    
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![],
    ).await.unwrap();
    
    // Test 2: Non-owner with low usage - should pass via second branch
    let non_owner = Keypair::new();
    context.payer = non_owner;
    
    let batch2 = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [2u8; 64] }],
        vec![],
    );
    
    // This should succeed because usage is still low
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch2,
        vec![],
    ).await.unwrap();
}

#[tokio::test]
async fn test_time_based_guards() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    let current_time = context.get_clock().await.unix_timestamp;
    let start_time = current_time + 3600; // 1 hour from now
    let end_time = current_time + 7200;   // 2 hours from now
    
    // Create time window guard: NotBefore AND Expiry
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    let compiled_guard = CompiledGuard {
        opcodes: vec![
            GuardOp::CheckNotBefore { timestamp: start_time }, // 0
            GuardOp::JumpIfFalse { offset: 3 },                // 1: too early, abort
            GuardOp::CheckExpiry { timestamp: end_time },      // 2
            GuardOp::JumpIfFalse { offset: 1 },                // 3: expired, abort
            GuardOp::Terminate,                                // 4: in valid window
            GuardOp::Abort,                                    // 5: outside window
        ],
        cpi_manifest: vec![],
    };
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        compiled_guard,
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
    
    // Test 1: Too early - should fail
    let batch1 = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [1u8; 64] }],
        vec![],
    );
    
    let result1 = context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch1,
        vec![],
    ).await;
    assert!(result1.is_err());
    
    // Test 2: In window - should succeed
    context.warp_to_timestamp(start_time + 1).await;
    
    let batch2 = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [2u8; 64] }],
        vec![],
    );
    
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch2,
        vec![],
    ).await.unwrap();
    
    // Test 3: After window - should fail
    context.warp_to_timestamp(end_time + 1).await;
    
    let batch3 = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [3u8; 64] }],
        vec![],
    );
    
    let result3 = context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch3,
        vec![],
    ).await;
    assert!(result3.is_err());
}

#[tokio::test]
async fn test_guard_with_cpi_manifest() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    // Add programs to allowlist that will be used in CPI
    let cpi_program1 = Pubkey::new_unique();
    let cpi_program2 = Pubkey::new_unique();
    context.add_to_allowlist(&allowlist_keypair, cpi_program1).await.unwrap();
    context.add_to_allowlist(&allowlist_keypair, cpi_program2).await.unwrap();
    
    // Create guard with CPI manifest
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    let compiled_guard = CompiledGuard {
        opcodes: vec![
            GuardOp::Invoke { manifest_index: 0 }, // Invoke first CPI
            GuardOp::JumpIfFalse { offset: 3 },    // If failed, skip to abort
            GuardOp::Invoke { manifest_index: 1 }, // Invoke second CPI
            GuardOp::JumpIfFalse { offset: 1 },    // If failed, abort
            GuardOp::Terminate,                    // Both succeeded
            GuardOp::Abort,                        // Either failed
        ],
        cpi_manifest: vec![
            CPIManifestEntry {
                program_id: cpi_program1,
                data: vec![1, 2, 3, 4], // Mock instruction data
                account_indices: vec![],
            },
            CPIManifestEntry {
                program_id: cpi_program2,
                data: vec![5, 6, 7, 8],
                account_indices: vec![],
            },
        ],
    };
    
    context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        compiled_guard,
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
    
    // Execute operations - guard will attempt CPIs
    let batch = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [1u8; 64] }],
        vec![],
    );
    
    // In a real test environment with mock programs, this would test the CPI execution
    // For now, we're testing that the guard with CPI manifest can be created and validated
    let guard_account = context.get_account(&guard_data_keypair.pubkey()).await;
    let guard_data: GuardData = GuardData::try_from_slice(&guard_account.data[8..]).unwrap();
    assert_eq!(guard_data.compiled_guard.cpi_manifest.len(), 2);
}

#[tokio::test]
async fn test_guard_validation_errors() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let guard_data_keypair = Keypair::new();
    let session_keypair = Keypair::new();
    
    // Test 1: Invalid jump offset
    let invalid_guard = CompiledGuard {
        opcodes: vec![
            GuardOp::JumpIfFalse { offset: 100 }, // Jump out of bounds
        ],
        cpi_manifest: vec![],
    };
    
    let result = context.create_guard_data(
        &guard_data_keypair,
        session_keypair.pubkey(),
        invalid_guard,
    ).await;
    assert!(result.is_err());
    
    // Test 2: Invalid CPI manifest index
    let guard_data_keypair2 = Keypair::new();
    let invalid_guard2 = CompiledGuard {
        opcodes: vec![
            GuardOp::Invoke { manifest_index: 5 }, // No entry at index 5
            GuardOp::Terminate,
        ],
        cpi_manifest: vec![], // Empty manifest
    };
    
    let result2 = context.create_guard_data(
        &guard_data_keypair2,
        session_keypair.pubkey(),
        invalid_guard2,
    ).await;
    assert!(result2.is_err());
}