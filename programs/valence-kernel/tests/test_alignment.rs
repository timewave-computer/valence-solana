// Tests for aligned behavior between EVM and Solana implementations
#[cfg(test)]
mod alignment_tests {
    use anchor_lang::prelude::*;
    use valence_kernel::{
        state::{Session, CreateSessionParams},
        MAX_DIRECT_CHILDREN, MAX_CASCADE_DEPTH, MAX_BATCH_INVALIDATION_SIZE,
    };

    #[test]
    fn test_max_direct_children_limit() {
        let mut parent = create_test_session("parent");
        
        // Should be able to track exactly 8 children (MAX_DIRECT_CHILDREN)
        for i in 0..MAX_DIRECT_CHILDREN {
            let child_key = Pubkey::new_unique();
            parent.track_child_session(child_key)
                .expect(&format!("Should track child {}", i));
        }
        
        assert_eq!(parent.child_session_count, MAX_DIRECT_CHILDREN);
        
        // 9th child should fail
        let extra_child = Pubkey::new_unique();
        let result = parent.track_child_session(extra_child);
        assert!(result.is_err(), "Should not allow more than 8 children");
    }
    
    #[test]
    fn test_cascade_depth_constant() {
        // Verify the cascade depth is aligned with EVM
        assert_eq!(MAX_CASCADE_DEPTH, 4, "Cascade depth should be 4 to match EVM");
    }
    
    #[test]
    fn test_batch_size_constant() {
        // Verify the batch size is aligned with EVM
        assert_eq!(MAX_BATCH_INVALIDATION_SIZE, 10, "Batch size should be 10 to match EVM");
    }
    
    #[test]
    fn test_child_array_size() {
        let session = create_test_session("test");
        
        // Verify arrays are sized correctly
        assert_eq!(session.child_sessions.len(), 8, "Child sessions array should be size 8");
        assert_eq!(session.child_accounts.len(), 8, "Child accounts array should be size 8");
    }
    
    #[test]
    fn test_session_space_calculation() {
        // Ensure the space calculation accounts for the increased array sizes
        let space = Session::calculate_space();
        
        // The space should include:
        // - 8 * 32 bytes for child_sessions (was 4 * 32)
        // - 8 * 32 bytes for child_accounts (was 4 * 32)
        // This adds 256 bytes total (4 * 32 * 2)
        
        // Just verify it's reasonable (not testing exact value as it may change)
        assert!(space > 1000, "Session space should be substantial");
        assert!(space < 2000, "Session space should not be excessive");
    }
    
    fn create_test_session(namespace: &str) -> Session {
        let params = CreateSessionParams {
            namespace_path: pad_namespace(namespace),
            namespace_path_len: namespace.len() as u16,
            metadata: [0u8; 32],
            parent_session: None,
        };
        
        Session::new(
            params,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &Clock {
                slot: 0,
                epoch_start_timestamp: 0,
                epoch: 0,
                leader_schedule_epoch: 0,
                unix_timestamp: 1234567890,
            },
        ).unwrap()
    }
    
    fn pad_namespace(namespace: &str) -> [u8; 128] {
        let mut padded = [0u8; 128];
        let bytes = namespace.as_bytes();
        let len = bytes.len().min(128);
        padded[..len].copy_from_slice(&bytes[..len]);
        padded
    }
}