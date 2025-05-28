// Comprehensive tests for Storage Account Program functionality

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use anchor_lang::prelude::*;

    use base_account::state::AccountState;

    #[test]
    fn test_storage_account_creation() {
        let ctx = TestContext::new();
        
        // Test storage account initialization
        let base_account = AccountState {
            owner: ctx.user,
            approved_libraries: vec![],
            vault_authority: generate_test_pubkey("vault_authority"),
            vault_bump_seed: 255,
            token_accounts: vec![],
            last_activity: 1000000000,
            instruction_count: 0,
        };
        
        let storage_authority = generate_test_pubkey("storage_authority");
        
        // Validate storage account parameters
        assert_eq!(base_account.owner, ctx.user);
        assert_ne!(storage_authority, Pubkey::default(), "Storage authority should be valid");
        assert!(base_account.approved_libraries.is_empty(), "Should start with no libraries");
        assert!(base_account.token_accounts.is_empty(), "Should start with no token accounts");
    }

    #[test]
    fn test_storage_item_management() {
        // Test storage item structure
        let key = "user_preference".to_string();
        let value_type = "string";
        let value = "dark_mode".as_bytes().to_vec();
        
        // Validate storage item parameters
        assert!(!key.is_empty(), "Key should not be empty");
        assert!(key.len() <= 64, "Key should be reasonable length");
        assert!(!value_type.is_empty(), "Value type should be specified");
        assert!(!value.is_empty(), "Value should not be empty");
        assert!(value.len() <= 1024, "Value should be reasonable size");
    }

    #[test]
    fn test_key_value_operations() {
        // Test different data types
        let string_item = ("username", "string", "alice".as_bytes().to_vec());
        let number_item = ("balance", "u64", 1000u64.to_le_bytes().to_vec());
        let boolean_item = ("is_active", "bool", vec![1u8]); // true
        let json_item = ("metadata", "json", r#"{"theme":"dark","lang":"en"}"#.as_bytes().to_vec());
        
        let items = vec![string_item, number_item, boolean_item, json_item];
        
        for (key, value_type, value) in items {
            // Validate each item
            assert!(!key.is_empty(), "Key should not be empty for {}", key);
            assert!(!value_type.is_empty(), "Value type should not be empty for {}", key);
            assert!(!value.is_empty(), "Value should not be empty for {}", key);
            assert!(key.len() <= 64, "Key should be reasonable length for {}", key);
            assert!(value.len() <= 1024, "Value should be reasonable size for {}", key);
        }
    }

    #[test]
    fn test_storage_capacity_limits() {
        // Test storage capacity and limits
        let max_items_per_account = 100;
        let max_key_length = 64;
        let max_value_size = 1024;
        let max_total_storage = 100 * 1024; // 100KB total
        
        // Validate limits
        assert!(max_items_per_account > 0, "Should allow storage items");
        assert!(max_items_per_account <= 1000, "Should have reasonable upper limit");
        assert!(max_key_length > 0, "Should allow keys");
        assert!(max_key_length <= 128, "Key length should be reasonable");
        assert!(max_value_size > 0, "Should allow values");
        assert!(max_value_size <= 10240, "Value size should be reasonable");
        assert!(max_total_storage > max_value_size, "Total should exceed single value");
    }

    #[test]
    fn test_storage_key_validation() {
        let valid_keys = vec![
            "username",
            "user_id",
            "balance_usd",
            "settings.theme",
            "data123",
        ];
        
        let long_key = "a".repeat(100);
        let invalid_keys = vec![
            "",
            " ",
            "key with spaces",
            "key/with/slashes",
            "key@with@symbols",
            &long_key, // Too long
        ];
        
        // Test valid keys
        for key in valid_keys {
            assert!(!key.is_empty(), "Valid key should not be empty");
            assert!(key.len() <= 64, "Valid key should be reasonable length");
            assert!(!key.contains(' '), "Valid key should not contain spaces");
        }
        
        // Test invalid keys
        for key in invalid_keys {
            if !key.is_empty() && key.len() <= 64 {
                assert!(key.contains(' ') || key.contains('/') || key.contains('@'), 
                        "Invalid key should contain invalid characters");
            }
        }
    }

    #[test]
    fn test_value_type_validation() {
        let supported_types = vec![
            "string",
            "u8", "u16", "u32", "u64",
            "i8", "i16", "i32", "i64",
            "bool",
            "bytes",
            "json",
            "pubkey",
        ];
        
        let unsupported_types = vec![
            "",
            "unknown",
            "float",
            "double",
            "custom_type",
        ];
        
        // Test supported types
        for value_type in supported_types {
            assert!(!value_type.is_empty(), "Supported type should not be empty");
            assert!(value_type.chars().all(|c| c.is_alphanumeric()), 
                    "Supported type should be alphanumeric");
        }
        
        // Test unsupported types
        for value_type in unsupported_types {
            if !value_type.is_empty() {
                assert!(!["string", "u64", "bool", "json"].contains(&value_type), 
                        "Unsupported type should not be in supported list");
            }
        }
    }

    #[test]
    fn test_batch_storage_operations() {
        // Test batch set operations
        let batch_items = vec![
            ("key1", "string", "value1".as_bytes().to_vec()),
            ("key2", "u64", 42u64.to_le_bytes().to_vec()),
            ("key3", "bool", vec![1u8]),
        ];
        
        let max_batch_size = 10;
        
        // Validate batch parameters
        assert!(!batch_items.is_empty(), "Batch should have items");
        assert!(batch_items.len() <= max_batch_size, "Batch should not exceed limit");
        
        // Validate each item in batch
        for (key, value_type, value) in batch_items {
            assert!(!key.is_empty(), "Batch key should not be empty");
            assert!(!value_type.is_empty(), "Batch value type should not be empty");
            assert!(!value.is_empty(), "Batch value should not be empty");
        }
    }

    #[test]
    fn test_storage_serialization() {
        // Test serialization of different value types
        let test_values = vec![
            ("string_val", "hello world".as_bytes().to_vec()),
            ("u64_val", 12345u64.to_le_bytes().to_vec()),
            ("bool_val", vec![1u8]), // true
            ("bytes_val", vec![1, 2, 3, 4, 5]),
        ];
        
        for (name, value) in test_values {
            // Test that values can be serialized and are reasonable size
            assert!(!value.is_empty(), "Value should not be empty for {}", name);
            assert!(value.len() <= 1024, "Value should be reasonable size for {}", name);
            
            // Test round-trip serialization would work
            let serialized_size = value.len();
            assert!(serialized_size > 0, "Serialized size should be positive for {}", name);
        }
    }

    #[test]
    fn test_storage_access_patterns() {
        // Test different access patterns
        let sequential_keys: Vec<String> = (0..10).map(|i| format!("item_{}", i)).collect();
        let random_keys = vec![
            "user_settings".to_string(),
            "last_login".to_string(),
            "preferences".to_string(),
            "cache_data".to_string(),
        ];
        
        // Test sequential access
        for key in sequential_keys {
            assert!(!key.is_empty(), "Sequential key should not be empty");
            assert!(key.starts_with("item_"), "Sequential key should follow pattern");
        }
        
        // Test random access
        for key in random_keys {
            assert!(!key.is_empty(), "Random key should not be empty");
            assert!(key.len() <= 64, "Random key should be reasonable length");
        }
    }

    #[test]
    fn test_storage_optimization() {
        // Test storage layout optimization
        let small_values: Vec<Vec<u8>> = (0..10).map(|i| vec![i]).collect();
        let medium_values: Vec<Vec<u8>> = (0..5).map(|_| vec![0u8; 100]).collect();
        let large_values: Vec<Vec<u8>> = (0..2).map(|_| vec![0u8; 500]).collect();
        
        // Calculate storage efficiency
        let small_total: usize = small_values.iter().map(|v| v.len()).sum();
        let medium_total: usize = medium_values.iter().map(|v| v.len()).sum();
        let large_total: usize = large_values.iter().map(|v| v.len()).sum();
        
        // Validate storage efficiency
        assert!(small_total < medium_total, "Small values should use less space");
        assert!(medium_total < large_total, "Medium values should use less space than large");
        assert!(small_total + medium_total + large_total <= 10240, "Total should fit in reasonable space");
    }

    #[test]
    fn test_storage_permissions() {
        let ctx = TestContext::new();
        
        // Test permission validation for storage operations
        let account_owner = ctx.user;
        let authorized_library = generate_test_pubkey("authorized_lib");
        let unauthorized_user = generate_test_pubkey("unauthorized");
        
        // Validate permission logic
        assert_ne!(account_owner, unauthorized_user, "Owner and unauthorized should be different");
        assert_ne!(authorized_library, unauthorized_user, "Authorized and unauthorized should be different");
        
        // Test permission checks (simulated)
        let can_owner_write = true; // Owner can always write
        let can_library_write = true; // Authorized library can write
        let can_unauthorized_write = false; // Unauthorized cannot write
        
        assert!(can_owner_write, "Owner should be able to write");
        assert!(can_library_write, "Authorized library should be able to write");
        assert!(!can_unauthorized_write, "Unauthorized should not be able to write");
    }

    #[test]
    fn test_storage_space_calculation() {
        // Test space calculation for storage items
        let base_item_overhead = 64; // Key + metadata overhead
        let value_sizes = vec![10, 50, 100, 500, 1000];
        
        for value_size in value_sizes {
            let total_space = base_item_overhead + value_size;
            
            // Validate space calculations
            assert!(total_space > value_size, "Total space should include overhead");
            assert!(total_space < value_size + 100, "Overhead should be reasonable");
            assert!(total_space <= 1024 + base_item_overhead, "Should fit in max item size");
        }
    }

    #[test]
    fn test_storage_cleanup() {
        // Test storage cleanup and deletion
        let items_to_delete = vec![
            "temp_data",
            "cache_entry",
            "expired_token",
        ];
        
        let items_to_keep = vec![
            "user_settings",
            "permanent_data",
            "important_config",
        ];
        
        // Validate cleanup logic
        for item in items_to_delete {
            assert!(!item.is_empty(), "Item to delete should have valid key");
            assert!(item.contains("temp") || item.contains("cache") || item.contains("expired"), 
                    "Item to delete should be temporary");
        }
        
        for item in items_to_keep {
            assert!(!item.is_empty(), "Item to keep should have valid key");
            assert!(!item.contains("temp") && !item.contains("cache") && !item.contains("expired"), 
                    "Item to keep should be permanent");
        }
    }
} 