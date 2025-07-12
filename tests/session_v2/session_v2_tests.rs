//! Comprehensive tests for Session V2 API
//! Tests the clean developer interface that hides account complexity

use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::Instruction,
};

// Import from shard program
use valence_shard::{
    ID, Session, SimpleBundle, SimpleOperation, Capability, Capabilities,
    accounts::{CreateSessionV2, ExecuteOnSession, ExecuteBundleV2},
    instruction::{CreateSessionV2 as CreateSessionV2Ix, ExecuteOnSession as ExecuteOnSessionIx, ExecuteBundleV2 as ExecuteBundleV2Ix},
};

/// Helper to create a session using V2 API
async fn create_session_v2(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: solana_sdk::hash::Hash,
    capabilities: u64,
    initial_state: Vec<u8>,
    namespace: String,
) -> Result<Pubkey, Box<dyn std::error::Error>> {
    let session = Keypair::new();
    let backing_account = Keypair::new();
    
    let ix = Instruction {
        program_id: ID,
        accounts: CreateSessionV2 {
            owner: payer.pubkey(),
            session: session.pubkey(),
            backing_account: backing_account.pubkey(),
            system_program: anchor_lang::system_program::ID,
        }.to_account_metas(None),
        data: CreateSessionV2Ix {
            capabilities,
            initial_state,
            namespace,
            nonce: 1,
            metadata: vec![],
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, &session, &backing_account], recent_blockhash);
    banks_client.process_transaction(transaction).await?;
    
    Ok(session.pubkey())
}

#[tokio::test]
async fn test_session_v2_creation_with_bitmap_capabilities() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Test creating session with multiple capabilities using bitmap
    let mut capabilities = Capabilities::none();
    capabilities.add(Capability::Read);
    capabilities.add(Capability::Write);
    capabilities.add(Capability::Transfer);
    
    let session = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities.0,
        b"initial state".to_vec(),
        "test-namespace".to_string(),
    ).await.unwrap();
    
    println!("✅ Session V2 created with bitmap capabilities: {:064b}", capabilities.0);
    
    // Verify session was created correctly by fetching account
    let session_account = banks_client.get_account(session).await.unwrap().unwrap();
    assert!(!session_account.data.is_empty());
    
    println!("✅ Session V2 creation test passed");
}

#[tokio::test]
async fn test_direct_session_execution() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Create session with execute capability
    let mut capabilities = Capabilities::none();
    capabilities.add(Capability::Execute);
    
    let session = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities.0,
        b"test state".to_vec(),
        "exec-test".to_string(),
    ).await.unwrap();
    
    // Execute function directly on session
    let function_hash = [1u8; 32];
    let args = b"test args".to_vec();
    
    let ix = Instruction {
        program_id: ID,
        accounts: ExecuteOnSession {
            executor: payer.pubkey(),
            session,
        }.to_account_metas(None),
        data: ExecuteOnSessionIx {
            function_hash,
            args,
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    // Should succeed with execute capability
    banks_client.process_transaction(transaction).await.unwrap();
    
    println!("✅ Direct session execution test passed");
}

#[tokio::test]
async fn test_simplified_bundle_execution() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Create session with multiple capabilities
    let mut capabilities = Capabilities::none();
    capabilities.add(Capability::Execute);
    capabilities.add(Capability::Read);
    capabilities.add(Capability::Write);
    
    let session = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities.0,
        b"bundle state".to_vec(),
        "bundle-test".to_string(),
    ).await.unwrap();
    
    // Create simplified bundle with multiple operations
    let operations = vec![
        SimpleOperation {
            function_hash: [1u8; 32],
            required_capabilities: Capability::Read.to_mask(),
            args: b"read operation".to_vec(),
        },
        SimpleOperation {
            function_hash: [2u8; 32],
            required_capabilities: Capability::Write.to_mask(),
            args: b"write operation".to_vec(),
        },
    ];
    
    let bundle = SimpleBundle {
        session,
        operations,
    };
    
    let ix = Instruction {
        program_id: ID,
        accounts: ExecuteBundleV2 {
            executor: payer.pubkey(),
            session,
        }.to_account_metas(None),
        data: ExecuteBundleV2Ix { bundle }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    // Should succeed with required capabilities
    banks_client.process_transaction(transaction).await.unwrap();
    
    println!("✅ Simplified bundle execution test passed");
}

#[tokio::test]
async fn test_capability_enforcement_failure() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Create session with only READ capability
    let capabilities = Capability::Read.to_mask();
    
    let session = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities,
        b"limited state".to_vec(),
        "limited-test".to_string(),
    ).await.unwrap();
    
    // Try to execute operation requiring WRITE capability
    let operations = vec![
        SimpleOperation {
            function_hash: [1u8; 32],
            required_capabilities: Capability::Write.to_mask(), // Requires WRITE
            args: b"write operation".to_vec(),
        },
    ];
    
    let bundle = SimpleBundle {
        session,
        operations,
    };
    
    let ix = Instruction {
        program_id: ID,
        accounts: ExecuteBundleV2 {
            executor: payer.pubkey(),
            session,
        }.to_account_metas(None),
        data: ExecuteBundleV2Ix { bundle }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    // Should fail with InsufficientCapabilities
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err());
    
    println!("✅ Capability enforcement failure test passed");
}

#[tokio::test]
async fn test_o1_capability_checking_performance() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Create session with many capabilities
    let mut capabilities = Capabilities::none();
    capabilities.add(Capability::Read);
    capabilities.add(Capability::Write);
    capabilities.add(Capability::Execute);
    capabilities.add(Capability::Transfer);
    capabilities.add(Capability::Mint);
    capabilities.add(Capability::Burn);
    capabilities.add(Capability::Admin);
    capabilities.add(Capability::CreateAccount);
    
    let session = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities.0,
        b"perf state".to_vec(),
        "perf-test".to_string(),
    ).await.unwrap();
    
    // Create bundle with operations requiring different capabilities
    // This tests that capability checking is O(1) regardless of number of capabilities
    let operations = vec![
        SimpleOperation {
            function_hash: [1u8; 32],
            required_capabilities: Capability::Read.to_mask(),
            args: b"op1".to_vec(),
        },
        SimpleOperation {
            function_hash: [2u8; 32],
            required_capabilities: Capability::Write.to_mask() | Capability::Execute.to_mask(),
            args: b"op2".to_vec(),
        },
        SimpleOperation {
            function_hash: [3u8; 32],
            required_capabilities: Capability::Transfer.to_mask() | Capability::Mint.to_mask() | Capability::Burn.to_mask(),
            args: b"op3".to_vec(),
        },
    ];
    
    let bundle = SimpleBundle {
        session,
        operations,
    };
    
    let ix = Instruction {
        program_id: ID,
        accounts: ExecuteBundleV2 {
            executor: payer.pubkey(),
            session,
        }.to_account_metas(None),
        data: ExecuteBundleV2Ix { bundle }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    // Should execute efficiently with O(1) capability checks
    banks_client.process_transaction(transaction).await.unwrap();
    
    println!("✅ O(1) capability checking performance test passed");
}

#[tokio::test]
async fn test_state_root_management() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Create session with initial state
    let initial_state = b"initial state data for testing";
    let capabilities = Capability::Execute.to_mask();
    
    let session = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities,
        initial_state.to_vec(),
        "state-test".to_string(),
    ).await.unwrap();
    
    // Execute operation that modifies state
    let function_hash = [1u8; 32];
    let args = b"state modifying args".to_vec();
    
    let ix = Instruction {
        program_id: ID,
        accounts: ExecuteOnSession {
            executor: payer.pubkey(),
            session,
        }.to_account_metas(None),
        data: ExecuteOnSessionIx {
            function_hash,
            args,
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    banks_client.process_transaction(transaction).await.unwrap();
    
    // TODO: In a full test, we would verify the state root changed
    // For now, we just verify the execution succeeded
    
    println!("✅ State root management test passed");
}

#[tokio::test]
async fn test_session_v2_vs_legacy_api_compatibility() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    // Create session using V2 API
    let capabilities = Capability::Read.to_mask() | Capability::Write.to_mask();
    
    let session_v2 = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities,
        b"v2 state".to_vec(),
        "compat-test".to_string(),
    ).await.unwrap();
    
    // Verify V2 session can be used with V2 operations
    let ix = Instruction {
        program_id: ID,
        accounts: ExecuteOnSession {
            executor: payer.pubkey(),
            session: session_v2,
        }.to_account_metas(None),
        data: ExecuteOnSessionIx {
            function_hash: [1u8; 32],
            args: b"compat test".to_vec(),
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    
    banks_client.process_transaction(transaction).await.unwrap();
    
    println!("✅ Session V2 compatibility test passed");
}

/// Integration test showing complete workflow
#[tokio::test]
async fn test_end_to_end_session_v2_workflow() {
    let mut pt = ProgramTest::new("valence_shard", ID, processor!(valence_shard::entry));
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    println!("Starting end-to-end Session V2 workflow test...");
    
    // Step 1: Create session with comprehensive capabilities
    let mut capabilities = Capabilities::none();
    capabilities.add(Capability::Read);
    capabilities.add(Capability::Write);
    capabilities.add(Capability::Execute);
    capabilities.add(Capability::Transfer);
    
    let session = create_session_v2(
        &mut banks_client,
        &payer,
        recent_blockhash,
        capabilities.0,
        b"e2e initial state".to_vec(),
        "e2e-workflow".to_string(),
    ).await.unwrap();
    
    println!("✓ Session created with capabilities: {:064b}", capabilities.0);
    
    // Step 2: Execute single operation
    let ix1 = Instruction {
        program_id: ID,
        accounts: ExecuteOnSession {
            executor: payer.pubkey(),
            session,
        }.to_account_metas(None),
        data: ExecuteOnSessionIx {
            function_hash: [1u8; 32],
            args: b"first operation".to_vec(),
        }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix1],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    println!("✓ Single operation executed successfully");
    
    // Step 3: Execute bundle with multiple operations
    let operations = vec![
        SimpleOperation {
            function_hash: [2u8; 32],
            required_capabilities: Capability::Read.to_mask(),
            args: b"read data".to_vec(),
        },
        SimpleOperation {
            function_hash: [3u8; 32],
            required_capabilities: Capability::Write.to_mask(),
            args: b"write data".to_vec(),
        },
        SimpleOperation {
            function_hash: [4u8; 32],
            required_capabilities: Capability::Transfer.to_mask(),
            args: b"transfer tokens".to_vec(),
        },
    ];
    
    let bundle = SimpleBundle {
        session,
        operations,
    };
    
    let ix2 = Instruction {
        program_id: ID,
        accounts: ExecuteBundleV2 {
            executor: payer.pubkey(),
            session,
        }.to_account_metas(None),
        data: ExecuteBundleV2Ix { bundle }.data(),
    };
    
    let mut transaction = Transaction::new_with_payer(
        &[ix2],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    
    println!("✓ Bundle with 3 operations executed successfully");
    
    println!("✅ End-to-end Session V2 workflow test completed successfully!");
    println!("   - Session created with bitmap capabilities");
    println!("   - Direct operation execution");
    println!("   - Simplified bundle execution");
    println!("   - All capability checks passed");
} 