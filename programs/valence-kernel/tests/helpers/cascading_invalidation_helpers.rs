// Helper functions for testing cascading invalidation
use anchor_lang::prelude::*;
use valence_kernel::state::{Session, CreateSessionParams};

/// Creates a test session with default values
pub fn create_test_session_with_defaults() -> Session {
    let params = CreateSessionParams {
        namespace_path: pad_namespace_path("test"),
        namespace_path_len: 4,
        metadata: [0u8; 64],
        parent_session: None,
    };
    
    let clock = Clock {
        slot: 0,
        epoch_start_timestamp: 0,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: 1234567890,
    };
    
    Session::new(
        params,
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        &clock,
    ).expect("Failed to create test session")
}

/// Creates a parent-child session relationship
pub fn create_parent_child_sessions() -> (Session, Session, Pubkey, Pubkey) {
    let parent_key = Pubkey::new_unique();
    let child_key = Pubkey::new_unique();
    
    // Create parent
    let mut parent = create_test_session_with_defaults();
    
    // Create child with parent reference
    let child_params = CreateSessionParams {
        namespace_path: pad_namespace_path("test/child"),
        namespace_path_len: 10,
        metadata: [0u8; 64],
        parent_session: Some(parent_key),
    };
    
    let clock = Clock {
        slot: 0,
        epoch_start_timestamp: 0,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: 1234567890,
    };
    
    let child = Session::new(
        child_params,
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        &clock,
    ).expect("Failed to create child session");
    
    // Track child in parent
    parent.track_child_session(child_key)
        .expect("Failed to track child session");
    
    (parent, child, parent_key, child_key)
}

/// Pads a namespace path to the required size
pub fn pad_namespace_path(namespace: &str) -> [u8; 128] {
    let mut padded = [0u8; 128];
    let bytes = namespace.as_bytes();
    if bytes.len() <= 128 {
        padded[..bytes.len()].copy_from_slice(bytes);
    }
    padded
}

/// Verifies a session has been properly invalidated
pub fn assert_session_invalidated(session: &Session) {
    assert!(!session.active, "Session should be inactive");
    assert!(session.nonce > 0, "Session nonce should be incremented");
}

/// Creates a session hierarchy for testing
pub fn create_session_hierarchy() -> Vec<(Session, Pubkey)> {
    let mut sessions = Vec::new();
    
    // Create grandparent
    let grandparent = create_test_session_with_defaults();
    let grandparent_key = Pubkey::new_unique();
    sessions.push((grandparent, grandparent_key));
    
    // Create parent
    let parent_params = CreateSessionParams {
        namespace_path: pad_namespace_path("grandparent/parent"),
        namespace_path_len: 18,
        metadata: [0u8; 64],
        parent_session: Some(grandparent_key),
    };
    
    let clock = Clock::default();
    let parent = Session::new(
        parent_params,
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        &clock,
    ).expect("Failed to create parent session");
    let parent_key = Pubkey::new_unique();
    
    // Track in grandparent
    sessions[0].0.track_child_session(parent_key)
        .expect("Failed to track parent in grandparent");
    
    sessions.push((parent, parent_key));
    
    // Create children
    for i in 0..3 {
        let child_params = CreateSessionParams {
            namespace_path: pad_namespace_path(&format!("grandparent/parent/child{}", i)),
            namespace_path_len: 25,
            metadata: [0u8; 64],
            parent_session: Some(parent_key),
        };
        
        let child = Session::new(
            child_params,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &clock,
        ).expect("Failed to create child session");
        let child_key = Pubkey::new_unique();
        
        // Track in parent
        sessions[1].0.track_child_session(child_key)
            .expect("Failed to track child in parent");
        
        sessions.push((child, child_key));
    }
    
    sessions
}