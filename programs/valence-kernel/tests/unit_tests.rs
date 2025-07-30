// Unit tests for valence-kernel with namespace architecture
#[cfg(test)]
mod tests {
    use valence_kernel::{
        namespace::*,
        KernelOperation, OperationBatch,
        ACCESS_MODE_READ, ACCESS_MODE_WRITE,
        MAX_BATCH_ACCOUNTS, MAX_BATCH_OPERATIONS,
    };
    use anchor_lang::prelude::*;
    
    // ================================
    // Namespace Tests
    // ================================
    
    #[test]
    fn test_namespace_path_creation() {
        // Valid paths
        let path = NamespacePath::new("shard").unwrap();
        assert_eq!(path.as_str().unwrap(), "shard");
        assert_eq!(path.depth(), 1);
        
        let path = NamespacePath::new("shard/session").unwrap();
        assert_eq!(path.as_str().unwrap(), "shard/session");
        assert_eq!(path.depth(), 2);
        
        let path = NamespacePath::new("shard/session/trade").unwrap();
        assert_eq!(path.as_str().unwrap(), "shard/session/trade");
        assert_eq!(path.depth(), 3);
        
        // Invalid paths
        assert!(NamespacePath::new("").is_err());
        assert!(NamespacePath::new("/shard").is_err());
        assert!(NamespacePath::new("shard/").is_err());
        assert!(NamespacePath::new("shard//session").is_err());
    }
    
    #[test]
    fn test_namespace_hierarchy() {
        let parent = NamespacePath::new("shard").unwrap();
        let child = NamespacePath::new("shard/session").unwrap();
        let grandchild = NamespacePath::new("shard/session/trade").unwrap();
        
        assert!(parent.is_parent_of(&child));
        assert!(parent.is_parent_of(&grandchild));
        assert!(child.is_parent_of(&grandchild));
        assert!(!child.is_parent_of(&parent));
        
        // Test parent extraction
        assert_eq!(child.parent().unwrap().as_str().unwrap(), "shard");
        assert_eq!(grandchild.parent().unwrap().as_str().unwrap(), "shard/session");
        assert!(parent.parent().is_none());
    }
    
    #[test]
    fn test_namespace_child_creation() {
        let parent = NamespacePath::new("shard").unwrap();
        let child = parent.child("session").unwrap();
        assert_eq!(child.as_str().unwrap(), "shard/session");
        
        // Invalid child names
        assert!(parent.child("").is_err());
        
        // Valid child with slash (creates deeper hierarchy)
        let deep_child = parent.child("has/slash").unwrap();
        assert_eq!(deep_child.as_str().unwrap(), "shard/has/slash");
    }
    
    // ================================
    // Access Mode Tests  
    // ================================
    
    #[test]
    fn test_access_mode_flags() {
        assert_eq!(ACCESS_MODE_READ, 1);
        assert_eq!(ACCESS_MODE_WRITE, 2);
        assert_eq!(ACCESS_MODE_READ | ACCESS_MODE_WRITE, 3);
    }
    
    // ================================
    // Operation Tests
    // ================================
    
    #[test]
    fn test_kernel_operation_types() {
        // Test operation enum variants exist
        let _borrow_op = KernelOperation::BorrowAccount {
            account_index: 0,
            mode: ACCESS_MODE_READ,
        };
        
        let _release_op = KernelOperation::ReleaseAccount {
            account_index: 0,
        };
        
        // Verify the operations can be created successfully without errors
    }
    
    #[test]
    fn test_operation_batch_structure() {
        // Test that we can create batch structures with proper defaults
        let default_accounts = [Pubkey::default(); MAX_BATCH_ACCOUNTS];
        let mut default_operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS] = Default::default();
        
        // Initialize array manually since KernelOperation doesn't implement Copy
        default_operations.fill(None);
        
        let _batch = OperationBatch {
            accounts: default_accounts,
            accounts_len: 0,
            operations: default_operations,
            operations_len: 0,
        };
        
        // Basic structure test - if we reach here, construction succeeded
    }
}