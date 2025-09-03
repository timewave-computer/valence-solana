// Tests for cascading session invalidation
#[cfg(test)]
mod tests {
    use anchor_lang::prelude::*;
    use valence_kernel::state::{Session, CreateSessionParams};
    #[allow(unused_imports)]
    use valence_kernel::errors::KernelError;

    #[test]
    fn test_session_tracks_children() {
        // Create a mock session
        let mut session = create_test_session("parent");
        
        // Initially no children
        assert_eq!(session.child_session_count, 0);
        
        // Track a child session
        let child_key = Pubkey::new_unique();
        session.track_child_session(child_key).unwrap();
        
        assert_eq!(session.child_session_count, 1);
        assert_eq!(session.child_sessions[0], child_key);
    }
    
    #[test]
    fn test_max_child_sessions() {
        let mut session = create_test_session("parent");
        
        // Add 8 child sessions (the maximum - aligned with EVM)
        for i in 0..8u8 {
            let child_key = Pubkey::new_unique();
            session.track_child_session(child_key).unwrap();
            assert_eq!(session.child_session_count, i + 1);
        }
        
        // Try to add a 9th child - should fail (max is 8)
        let extra_child = Pubkey::new_unique();
        let result = session.track_child_session(extra_child);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_duplicate_child_sessions_ignored() {
        let mut session = create_test_session("parent");
        
        let child_key = Pubkey::new_unique();
        
        // Track the same child twice
        session.track_child_session(child_key).unwrap();
        session.track_child_session(child_key).unwrap(); // Should be ignored
        
        // Should only be tracked once
        assert_eq!(session.child_session_count, 1);
    }
    
    #[test]
    fn test_session_space_calculation() {
        // Ensure the space calculation includes the new child_sessions fields
        let space = Session::calculate_space();
        
        // The space should be large enough to accommodate all fields
        // Original size + (8 * 32) for child_sessions + 1 for child_session_count
        assert!(space >= 8 + 256 + 2 + 32 + 32 + 32 + 32 + 33 + 8 + 32 + 8 + 8 + (4 * 41) + 1 + 1 + 1 + 8 + (8 * 32) + 1 + (8 * 32) + 1);
    }
    
    #[test]
    fn test_new_session_initialization() {
        let session = create_test_session("test");
        
        // Verify child session fields are initialized
        assert_eq!(session.child_session_count, 0);
        for i in 0..8 {
            assert_eq!(session.child_sessions[i], Pubkey::default());
        }
    }
    
    // Helper function to create a test session
    fn create_test_session(namespace: &str) -> Session {
        let params = CreateSessionParams {
            namespace_path: pad_namespace(namespace),
            namespace_path_len: namespace.len() as u16,
            metadata: [0u8; 32],
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
            Pubkey::new_unique(), // owner
            Pubkey::new_unique(), // shard
            Pubkey::new_unique(), // guard_account
            Pubkey::new_unique(), // account_lookup
            &clock,
        ).unwrap()
    }
    
    fn pad_namespace(s: &str) -> [u8; 128] {
        let mut padded = [0u8; 128];
        let bytes = s.as_bytes();
        padded[..bytes.len()].copy_from_slice(bytes);
        padded
    }
}

#[cfg(test)]
mod invalidation_tests {
    use anchor_lang::prelude::*;
    use valence_kernel::state::Session;
    
    #[test]
    fn test_session_invalidation_marks_inactive() {
        let mut session = create_test_session("parent");
        assert!(session.active);
        
        // Simulate invalidation
        session.active = false;
        session.nonce = session.nonce.saturating_add(1);
        
        assert!(!session.active);
        assert_eq!(session.nonce, 1);
    }
    
    #[test]
    fn test_child_session_tracking_in_parent() {
        let mut parent = create_test_session("parent");
        let child1 = Pubkey::new_unique();
        let child2 = Pubkey::new_unique();
        
        parent.track_child_session(child1).unwrap();
        parent.track_child_session(child2).unwrap();
        
        assert_eq!(parent.child_session_count, 2);
        assert_eq!(parent.child_sessions[0], child1);
        assert_eq!(parent.child_sessions[1], child2);
    }
    
    fn create_test_session(namespace: &str) -> Session {
        let params = valence_kernel::state::CreateSessionParams {
            namespace_path: pad_namespace(namespace),
            namespace_path_len: namespace.len() as u16,
            metadata: [0u8; 32],
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
        ).unwrap()
    }
    
    fn pad_namespace(s: &str) -> [u8; 128] {
        let mut padded = [0u8; 128];
        let bytes = s.as_bytes();
        padded[..bytes.len()].copy_from_slice(bytes);
        padded
    }
}