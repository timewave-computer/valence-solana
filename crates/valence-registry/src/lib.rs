// Valence Registry - Client-side registry for functions and shards
//
// This crate provides client-side utilities for working with the valence-kernel's
// hardcoded function registry and shard deployments. It aligns with the simplified
// architecture where the kernel uses a hardcoded registry rather than complex storage.

// ================================
// Module Declarations
// ================================

/// Registry error types
pub mod error;

/// Function registry aligned with kernel's hardcoded approach
pub mod functions;

/// Shard registry for deployment tracking
pub mod shards;

/// Simplified IDL generation for shard integration
pub mod idl;

// ================================
// Public API Re-exports
// ================================

// Re-export error types
pub use error::{RegistryError, Result};

// Re-export function registry components
pub use functions::{
    FunctionInfo, FunctionEntry, FunctionRegistry,
};

// Re-export shard registry components
pub use shards::{
    ShardMetadata, ShardInstance, ShardRegistry,
    AuditInfo, ShardInterface,
};

// Re-export IDL components
pub use idl::{
    ValenceIdl, IdlMetadata, BuildInfo, IdlGenerator,
    extract_functions_from_idl, extract_guards_from_idl,
    is_compatible_version,
};

// ================================
// Registry Constants
// ================================

/// Current registry format version
pub const REGISTRY_VERSION: &str = "0.1.0";

/// Maximum number of functions per registry
pub const MAX_FUNCTIONS_PER_REGISTRY: usize = 1024;

/// Maximum number of shards per registry
pub const MAX_SHARDS_PER_REGISTRY: usize = 256;

// ================================
// Convenience Functions
// ================================

/// Create a new function registry with default settings
pub fn create_function_registry() -> FunctionRegistry {
    FunctionRegistry::new()
}

/// Create a new shard registry with default settings
pub fn create_shard_registry() -> ShardRegistry {
    ShardRegistry::new()
}

/// Generate IDL for a Valence shard
pub fn generate_shard_idl(
    name: &str,
    program_id: &anchor_lang::prelude::Pubkey,
    functions: Vec<String>,
    guards: Vec<String>,
) -> Result<ValenceIdl> {
    IdlGenerator::generate(name, program_id, functions, guards)
}

// ================================
// Tests
// ================================

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_registry_constants() {
        assert_eq!(REGISTRY_VERSION, "0.1.0");
        assert!(MAX_FUNCTIONS_PER_REGISTRY > 0);
        assert!(MAX_SHARDS_PER_REGISTRY > 0);
    }

    #[test]
    fn test_convenience_functions() {
        let function_registry = create_function_registry();
        assert_eq!(function_registry.function_count(), 0);

        let shard_registry = create_shard_registry();
        assert_eq!(shard_registry.shard_count(), 0);
    }

    #[test]
    fn test_idl_generation_convenience() {
        let program_id = Pubkey::new_unique();
        let functions = vec!["test_function".to_string()];
        let guards = vec!["test_guard".to_string()];

        let idl = generate_shard_idl("test_shard", &program_id, functions, guards);
        assert!(idl.is_ok());

        let idl = idl.unwrap();
        assert_eq!(idl.name, "test_shard");
        assert_eq!(idl.metadata.shard, "valence");
    }

    #[test]
    fn test_integration() {
        // Test that all components work together
        let mut function_registry = create_function_registry();
        let mut shard_registry = create_shard_registry();

        // Create a test function
        let program_id = Pubkey::new_unique();
        let function_info = FunctionInfo::new(
            1001,
            program_id,
            "test_function".to_string(),
            1,
            10_000,
        );

        let function_entry = FunctionEntry::new(
            function_info,
            "Test function".to_string(),
            vec!["test".to_string()],
        );

        // Register the function
        let result = function_registry.register_function(function_entry);
        assert!(result.is_ok());

        // Create a shard that uses this function
        let shard_metadata = ShardMetadata {
            name: "test_shard".to_string(),
            version: "1.0.0".to_string(),
            description: "A test shard".to_string(),
            website: None,
            repository: None,
            audits: vec![],
        };

        let mut shard_instance = ShardInstance::new(
            program_id,
            Pubkey::new_unique(),
            shard_metadata,
            "localnet".to_string(),
        );

        // Link the function to the shard
        shard_instance.add_provided_function(1001);

        // Register the shard
        let result = shard_registry.register_shard(shard_instance);
        assert!(result.is_ok());

        // Verify the integration
        let providers = shard_registry.find_function_providers(1001);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].metadata.name, "test_shard");

        // Generate IDL for the shard
        let idl = generate_shard_idl(
            "test_shard",
            &program_id,
            vec!["test_function".to_string()],
            vec![],
        );
        assert!(idl.is_ok());
    }
}