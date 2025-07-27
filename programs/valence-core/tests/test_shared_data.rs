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
async fn test_shared_data_reentrancy() {
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
    
    // Test reentrancy protection through shared data
    let mut shared_data = SessionSharedData::default();
    assert!(!shared_data.is_entered());
    
    // First entry should succeed
    assert!(shared_data.enter_protected_section().is_ok());
    assert!(shared_data.is_entered());
    
    // Create session with reentrancy flag already set
    let session_params = CreateSessionParams {
        scope: SessionScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data,
        metadata: [0u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Verify the flag persists
    let session_account = context.get_account(&session_keypair.pubkey()).await;
    let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
    assert!(session.shared_data.is_entered());
    assert_eq!(session.shared_data.reentrancy_flag, 1);
}

#[tokio::test]
async fn test_shared_data_cpi_depth() {
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
    
    // Test CPI depth tracking
    let mut shared_data = SessionSharedData::default();
    
    // Increment CPI depth multiple times
    for i in 1..=3 {
        assert!(shared_data.check_and_increment_cpi_depth().is_ok());
        assert_eq!(shared_data.current_cpi_depth(), i);
    }
    
    let session_params = CreateSessionParams {
        scope: SessionScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data,
        metadata: [0u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Verify CPI depth persists
    let session_account = context.get_account(&session_keypair.pubkey()).await;
    let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
    assert_eq!(session.shared_data.current_cpi_depth(), 3);
    assert!(!session.shared_data.is_at_max_cpi_depth());
}

#[tokio::test]
async fn test_shared_data_feature_flags() {
    let mut context = TestContext::new().await;
    context.initialize_program().await.unwrap();
    
    let allowlist_keypair = Keypair::new();
    context.initialize_allowlist(&allowlist_keypair).await.unwrap();
    
    // Test multiple sessions with different feature flags
    for (i, flags) in [
        (SessionSharedData::FLAG_PAUSED, "paused"),
        (SessionSharedData::FLAG_DEBUG, "debug"),
        (SessionSharedData::FLAG_ATOMIC, "atomic"),
        (SessionSharedData::FLAG_CROSS_PROTOCOL, "cross_protocol"),
    ].iter().enumerate() {
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
        
        let mut shared_data = SessionSharedData::default();
        shared_data.set_flag(*flags.0);
        
        let session_params = CreateSessionParams {
            scope: SessionScope::User,
            guard_data: guard_data_keypair.pubkey(),
            bound_to: None,
            shared_data,
            metadata: [i as u8; 64],
        };
        
        context.create_session(
            &session_keypair,
            context.authority.pubkey(),
            session_params,
        ).await.unwrap();
        
        // Verify flag is set
        let session_account = context.get_account(&session_keypair.pubkey()).await;
        let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
        assert!(session.shared_data.has_flag(*flags.0), "Flag {} should be set", flags.1);
        
        // Test flag-specific behavior
        match *flags.0 {
            SessionSharedData::FLAG_PAUSED => assert!(session.shared_data.is_paused()),
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_shared_data_custom_data() {
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
    
    // Test custom data storage
    let mut shared_data = SessionSharedData::default();
    let custom_data = shared_data.custom_data_mut();
    
    // Write pattern to custom data
    for i in 0..32 {
        custom_data[i] = i as u8;
    }
    
    let session_params = CreateSessionParams {
        scope: SessionScope::User,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data,
        metadata: [0u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Verify custom data persists
    let session_account = context.get_account(&session_keypair.pubkey()).await;
    let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
    
    for i in 0..32 {
        assert_eq!(session.shared_data.custom_data()[i], i as u8);
    }
}

#[tokio::test]
async fn test_shared_data_max_cpi_depth() {
    let mut shared_data = SessionSharedData::default();
    
    // Increment to max depth
    for _ in 0..SessionSharedData::MAX_CPI_DEPTH {
        assert!(shared_data.check_and_increment_cpi_depth().is_ok());
    }
    
    assert!(shared_data.is_at_max_cpi_depth());
    assert_eq!(shared_data.current_cpi_depth(), SessionSharedData::MAX_CPI_DEPTH);
    
    // Next increment should fail
    assert!(shared_data.check_and_increment_cpi_depth().is_err());
    
    // Decrement and try again
    shared_data.decrement_cpi_depth();
    assert_eq!(shared_data.current_cpi_depth(), SessionSharedData::MAX_CPI_DEPTH - 1);
    assert!(!shared_data.is_at_max_cpi_depth());
    
    // Should be able to increment again
    assert!(shared_data.check_and_increment_cpi_depth().is_ok());
    assert!(shared_data.is_at_max_cpi_depth());
}

#[tokio::test]
async fn test_shared_data_flag_operations() {
    let mut shared_data = SessionSharedData::default();
    
    // Test individual flag operations
    assert_eq!(shared_data.feature_flags, 0);
    
    // Set multiple flags
    shared_data.set_flag(SessionSharedData::FLAG_DEBUG | SessionSharedData::FLAG_ATOMIC);
    assert!(shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
    assert!(shared_data.has_flag(SessionSharedData::FLAG_ATOMIC));
    assert!(!shared_data.has_flag(SessionSharedData::FLAG_PAUSED));
    
    // Toggle flags
    shared_data.toggle_flag(SessionSharedData::FLAG_DEBUG);
    assert!(!shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
    assert!(shared_data.has_flag(SessionSharedData::FLAG_ATOMIC));
    
    shared_data.toggle_flag(SessionSharedData::FLAG_DEBUG);
    assert!(shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
    
    // Clear specific flag
    shared_data.feature_flags &= !SessionSharedData::FLAG_DEBUG;
    assert!(!shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
    assert!(shared_data.has_flag(SessionSharedData::FLAG_ATOMIC));
    
    // Test pause functionality
    assert!(!shared_data.is_paused());
    shared_data.set_paused(true);
    assert!(shared_data.is_paused());
    assert!(shared_data.has_flag(SessionSharedData::FLAG_PAUSED));
    
    shared_data.set_paused(false);
    assert!(!shared_data.is_paused());
    assert!(!shared_data.has_flag(SessionSharedData::FLAG_PAUSED));
}

#[tokio::test]
async fn test_shared_data_serialization() {
    let mut shared_data = SessionSharedData::default();
    
    // Set various fields
    shared_data.version = 1;
    shared_data.enter_protected_section().unwrap();
    shared_data.check_and_increment_cpi_depth().unwrap();
    shared_data.check_and_increment_cpi_depth().unwrap();
    shared_data.set_flag(SessionSharedData::FLAG_DEBUG | SessionSharedData::FLAG_ATOMIC);
    
    let custom = shared_data.custom_data_mut();
    custom[0] = 0xFF;
    custom[31] = 0xAA;
    
    // Serialize
    let mut buffer = Vec::new();
    shared_data.serialize(&mut buffer).unwrap();
    
    // Deserialize
    let deserialized = SessionSharedData::try_from_slice(&buffer).unwrap();
    
    // Verify all fields
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.reentrancy_flag, 1);
    assert_eq!(deserialized.cpi_depth, 2);
    assert_eq!(deserialized.feature_flags, SessionSharedData::FLAG_DEBUG | SessionSharedData::FLAG_ATOMIC);
    assert_eq!(deserialized.custom_data()[0], 0xFF);
    assert_eq!(deserialized.custom_data()[31], 0xAA);
}

#[tokio::test]
async fn test_shared_data_integration() {
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
    
    // Create session with complex shared data state
    let mut shared_data = SessionSharedData::default();
    shared_data.enter_protected_section().unwrap();
    shared_data.exit_protected_section();
    
    for _ in 0..2 {
        shared_data.check_and_increment_cpi_depth().unwrap();
    }
    
    shared_data.set_flag(SessionSharedData::FLAG_DEBUG | SessionSharedData::FLAG_CROSS_PROTOCOL);
    
    let custom = shared_data.custom_data_mut();
    for i in 0..32 {
        custom[i] = (i * 2) as u8;
    }
    
    let session_params = CreateSessionParams {
        scope: SessionScope::Protocol,
        guard_data: guard_data_keypair.pubkey(),
        bound_to: None,
        shared_data,
        metadata: [99u8; 64],
    };
    
    context.create_session(
        &session_keypair,
        context.authority.pubkey(),
        session_params,
    ).await.unwrap();
    
    // Execute operations to verify shared data doesn't interfere
    let batch = OperationBatch::new(
        vec![SessionOperation::UpdateMetadata { metadata: [100u8; 64] }],
        vec![],
    );
    
    context.execute_operations(
        &session_keypair,
        &guard_data_keypair,
        &allowlist_keypair,
        batch,
        vec![],
    ).await.unwrap();
    
    // Verify all shared data persists correctly
    let session_account = context.get_account(&session_keypair.pubkey()).await;
    let session: Session = Session::try_from_slice(&session_account.data[8..]).unwrap();
    
    assert_eq!(session.shared_data.current_cpi_depth(), 2);
    assert!(session.shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
    assert!(session.shared_data.has_flag(SessionSharedData::FLAG_CROSS_PROTOCOL));
    assert_eq!(session.metadata[0], 100); // Updated by operation
    
    for i in 0..32 {
        assert_eq!(session.shared_data.custom_data()[i], (i * 2) as u8);
    }
}