// ================================
// Valence Registry
// ================================

pub mod error;
pub mod functions;
pub mod idl;
pub mod protocols;
pub mod storage;

pub use error::*;
pub use functions::*;
pub use idl::*;
pub use protocols::*;
pub use storage::*;

// ================================
// Registry Tests
// ================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // Test helpers
    fn mock_function_metadata() -> valence_functions::function_core::FunctionMetadata {
        valence_functions::function_core::FunctionMetadata {
            content_hash: [1u8; 32],
            name: "test_function".to_string(),
            version: 1,
            author: "test_author".to_string(),
            description: "A test function".to_string(),
            dependencies: vec![],
            supported_states: vec!["TestState".to_string()],
        }
    }

    fn mock_protocol_metadata() -> ProtocolMetadata {
        ProtocolMetadata {
            name: "test_protocol".to_string(),
            version: "1.0.0".to_string(),
            description: "A test protocol".to_string(),
            website: Some("https://test.com".to_string()),
            repository: Some("https://github.com/test/test".to_string()),
            audits: vec![],
        }
    }

    // ================================
    // Function Registry Tests
    // ================================

    #[tokio::test]
    async fn test_function_registry() {
        let registry = InMemoryFunctionRegistry::new();

        // Test function registration
        let entry = FunctionEntry::new(
            mock_function_metadata(),
            "fn test() { }".to_string(),
            vec!["test".to_string(), "example".to_string()],
        );

        let hash = registry.register_function(entry.clone()).await.unwrap();
        assert_eq!(hash, entry.content_hash);

        // Test function retrieval
        let retrieved = registry.get_function(&hash).await.unwrap();
        assert_eq!(retrieved.metadata.name, "test_function");
        assert_eq!(retrieved.tags.len(), 2);

        // Test duplicate registration
        let duplicate_result = registry.register_function(entry).await;
        assert!(duplicate_result.is_err());

        // Test search by tags
        let results = registry
            .search_by_tags(&["test".to_string()])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);

        let no_results = registry
            .search_by_tags(&["nonexistent".to_string()])
            .await
            .unwrap();
        assert_eq!(no_results.len(), 0);

        // Test listing functions
        let all_functions = registry.list_functions(10, 0).await.unwrap();
        assert_eq!(all_functions.len(), 1);

        // Test getting dependencies
        let deps = registry.get_dependencies(&hash).await.unwrap();
        assert_eq!(deps.len(), 0);
    }

    #[test]
    fn test_function_entry() {
        let metadata = mock_function_metadata();
        let source_code = "fn example() { return 42; }".to_string();
        let tags = vec!["math".to_string(), "example".to_string()];

        let entry = FunctionEntry::new(metadata.clone(), source_code.clone(), tags.clone());

        // Test hash computation
        let expected_hash = FunctionEntry::compute_hash(&source_code);
        assert_eq!(entry.content_hash, expected_hash);

        // Test hash verification
        assert!(entry.verify_hash());

        // Test with modified source (should fail verification)
        let mut modified_entry = entry.clone();
        modified_entry.source_code = "fn modified() { return 0; }".to_string();
        assert!(!modified_entry.verify_hash());

        // Test fields
        assert_eq!(entry.metadata.name, metadata.name);
        assert_eq!(entry.tags, tags);
        assert_eq!(entry.registry_version, 1);
    }

    // ================================
    // Protocol Registry Tests
    // ================================

    #[tokio::test]
    async fn test_protocol_registry() {
        let registry = InMemoryProtocolRegistry::new();

        let program_id = anchor_lang::prelude::Pubkey::new_unique();
        let authority = anchor_lang::prelude::Pubkey::new_unique();

        // Test protocol registration
        let instance = ProtocolInstance {
            protocol_id: [1u8; 32],
            program_id,
            authority,
            metadata: mock_protocol_metadata(),
            idl: serde_json::json!({"version": "1.0.0", "name": "test"}),
            deployed_at: chrono::Utc::now().timestamp(),
            network: "localnet".to_string(),
        };

        registry.register_protocol(instance.clone()).await.unwrap();

        // Test protocol retrieval
        let retrieved = registry.get_protocol(&program_id).await.unwrap();
        assert_eq!(retrieved.metadata.name, "test_protocol");
        assert_eq!(retrieved.network, "localnet");

        // Test search by function
        let results = registry.search_by_function(&[1u8; 32]).await.unwrap();
        assert_eq!(results.len(), 1);

        // Test list by network
        let localnet_protocols = registry.list_by_network("localnet").await.unwrap();
        assert_eq!(localnet_protocols.len(), 1);

        let mainnet_protocols = registry.list_by_network("mainnet").await.unwrap();
        assert_eq!(mainnet_protocols.len(), 0);

        // Test IDL retrieval
        let idl = registry.get_idl(&program_id).await.unwrap();
        assert_eq!(idl["name"], "test");
    }

    // ================================
    // Storage Tests
    // ================================

    #[tokio::test]
    async fn test_local_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        let content = b"Hello, World!";

        // Test storage
        let hash = storage.store(content).await.unwrap();
        assert!(!hash.is_empty());

        // Test existence check
        assert!(storage.exists(&hash).await.unwrap());
        assert!(!storage.exists("nonexistent").await.unwrap());

        // Test retrieval
        let retrieved = storage.retrieve(&hash).await.unwrap();
        assert_eq!(retrieved, content);

        // Test deletion
        storage.delete(&hash).await.unwrap();
        assert!(!storage.exists(&hash).await.unwrap());

        // Test retrieval after deletion
        let delete_result = storage.retrieve(&hash).await;
        assert!(delete_result.is_err());
    }

    #[tokio::test]
    async fn test_cached_storage() {
        let temp_dir = TempDir::new().unwrap();
        let backend = LocalStorage::new(temp_dir.path()).unwrap();
        let cached_storage = CachedStorage::new(backend, 100);

        let content = b"Cached content";

        // Test storage (should cache)
        let hash = cached_storage.store(content).await.unwrap();

        // Test retrieval (should hit cache)
        let retrieved1 = cached_storage.retrieve(&hash).await.unwrap();
        assert_eq!(retrieved1, content);

        // Test retrieval again (should hit cache again)
        let retrieved2 = cached_storage.retrieve(&hash).await.unwrap();
        assert_eq!(retrieved2, content);

        // Test existence (should check cache first)
        assert!(cached_storage.exists(&hash).await.unwrap());

        // Test deletion (should remove from cache)
        cached_storage.delete(&hash).await.unwrap();
        assert!(!cached_storage.exists(&hash).await.unwrap());
    }

    // ================================
    // SMT Tests
    // ================================

    #[test]
    fn test_registry_smt() {
        let mut smt = RegistrySMT::new();

        // Test initial state
        assert_eq!(smt.root(), [0u8; 32]);

        // Test insertion
        let key = [1u8; 32];
        let value = b"test value".to_vec();
        let new_root = smt.insert(key, value.clone()).unwrap();

        assert_ne!(new_root, [0u8; 32]);
        assert_eq!(smt.root(), new_root);

        // Test proof generation
        let proof = smt.generate_proof(vec![key]).unwrap();
        assert!(!proof.is_empty());

        // Test proof verification
        let value_hash: [u8; 32] = blake3::hash(&value).into();
        let verify_result = smt
            .verify_proof(vec![key], vec![value_hash], &proof)
            .unwrap();
        assert!(verify_result);

        // Test verification with wrong value
        let wrong_value = b"wrong value";
        let wrong_value_hash: [u8; 32] = blake3::hash(wrong_value).into();
        let verify_wrong = smt
            .verify_proof(vec![key], vec![wrong_value_hash], &proof)
            .unwrap();
        assert!(!verify_wrong);

        // Test multiple insertions
        let key2 = [2u8; 32];
        let value2 = b"second value".to_vec();
        let new_root2 = smt.insert(key2, value2).unwrap();

        assert_ne!(new_root2, new_root);
        assert_eq!(smt.root(), new_root2);
    }

    // ================================
    // IDL Tests
    // ================================

    #[test]
    fn test_idl_generation() {
        let program_id = anchor_lang::prelude::Pubkey::new_unique();
        let functions = vec!["function1".to_string(), "function2".to_string()];
        let guards = vec!["guard1".to_string()];

        let idl = IdlGenerator::generate("test_program", &program_id, functions, guards).unwrap();

        // Test IDL structure
        assert_eq!(idl.name, "test_program");
        assert_eq!(idl.program_id, program_id.to_string());
        assert_eq!(idl.version, "0.1.0");

        // Test metadata
        assert_eq!(idl.metadata.protocol, "valence");
        assert_eq!(idl.metadata.functions.len(), 2);
        assert_eq!(idl.metadata.guards.len(), 1);

        // Test that build info is populated
        assert!(!idl.metadata.build_info.rust_version.is_empty());
        assert!(!idl.metadata.build_info.anchor_version.is_empty());
        assert!(idl.metadata.build_info.build_timestamp > 0);
    }

    // ================================
    // Error Tests
    // ================================

    #[test]
    fn test_registry_errors() {
        // Test error creation and formatting
        let not_found_error = RegistryError::FunctionNotFound("abc123".to_string());
        assert!(format!("{}", not_found_error).contains("abc123"));

        let invalid_hash_error = RegistryError::InvalidContentHash;
        assert_eq!(format!("{}", invalid_hash_error), "Invalid content hash");

        let storage_error = RegistryError::StorageError("disk full".to_string());
        assert!(format!("{}", storage_error).contains("disk full"));

        // Test error conversion
        let json_error = serde_json::from_str::<HashMap<String, String>>("invalid json");
        assert!(json_error.is_err());

        let registry_error: RegistryError = json_error.unwrap_err().into();
        match registry_error {
            RegistryError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }
}
