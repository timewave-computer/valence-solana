// Simplified IDL generation aligned with valence-kernel architecture
//
// This module provides basic IDL (Interface Definition Language) generation
// for shard integration with the valence ecosystem.

use crate::error::Result;
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

// ================================
// Simplified IDL Structures
// ================================

/// Simplified Valence IDL format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceIdl {
    /// IDL format version
    pub version: String,
    /// Program name
    pub name: String,
    /// Program ID
    pub program_id: String,
    /// Shard metadata
    pub metadata: IdlMetadata,
    /// Available instructions
    pub instructions: Vec<String>,
    /// Account types
    pub accounts: Vec<String>,
    /// Error definitions
    pub errors: Vec<String>,
}

/// IDL metadata specific to Valence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlMetadata {
    /// Shard framework identifier
    pub shard: String,
    /// Functions this program provides
    pub functions: Vec<String>,
    /// Guards this program implements
    pub guards: Vec<String>,
    /// Build information
    pub build_info: BuildInfo,
}

/// Build information for reproducible builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// Rust compiler version
    pub rust_version: String,
    /// Anchor framework version
    pub anchor_version: String,
    /// Build timestamp
    pub build_timestamp: i64,
    /// Git commit hash (if available)
    pub git_commit: Option<String>,
}

// ================================
// IDL Generator
// ================================

/// Simple IDL generator for Valence shards
pub struct IdlGenerator;

impl IdlGenerator {
    /// Generate a basic IDL for a Valence shard
    pub fn generate(
        name: &str,
        program_id: &Pubkey,
        functions: Vec<String>,
        guards: Vec<String>,
    ) -> Result<ValenceIdl> {
        let build_info = Self::collect_build_info();
        
        Ok(ValenceIdl {
            version: "0.1.0".to_string(),
            name: name.to_string(),
            program_id: program_id.to_string(),
            metadata: IdlMetadata {
                shard: "valence".to_string(),
                functions,
                guards,
                build_info,
            },
            instructions: vec![
                "initialize".to_string(),
                "execute_batch".to_string(),
                "create_session".to_string(),
                "update_session".to_string(),
            ],
            accounts: vec![
                "Session".to_string(),
                "GuardAccount".to_string(),
                "SessionAccountLookup".to_string(),
            ],
            errors: vec![
                "InvalidParameters".to_string(),
                "Unauthorized".to_string(),
                "BorrowCapacityExceeded".to_string(),
            ],
        })
    }
    
    /// Generate IDL for a function program
    pub fn generate_function_idl(
        name: &str,
        program_id: &Pubkey,
        function_name: &str,
    ) -> Result<ValenceIdl> {
        Self::generate(
            name,
            program_id,
            vec![function_name.to_string()],
            vec![],
        )
    }
    
    /// Collect build information
    fn collect_build_info() -> BuildInfo {
        BuildInfo {
            rust_version: std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            anchor_version: "0.31.1".to_string(), // Should match workspace version
            build_timestamp: chrono::Utc::now().timestamp(),
            git_commit: std::option_env!("GIT_COMMIT").map(String::from),
        }
    }
    
    /// Validate an IDL structure
    pub fn validate_idl(idl: &ValenceIdl) -> Result<()> {
        // Basic validation
        if idl.name.is_empty() {
            return Err(crate::error::RegistryError::InvalidMetadata);
        }
        
        if idl.program_id.is_empty() {
            return Err(crate::error::RegistryError::InvalidMetadata);
        }
        
        // Validate program ID format
        if Pubkey::try_from(idl.program_id.as_str()).is_err() {
            return Err(crate::error::RegistryError::InvalidMetadata);
        }
        
        Ok(())
    }
}

// ================================
// Helper Functions
// ================================

/// Extract function information from IDL
pub fn extract_functions_from_idl(idl: &ValenceIdl) -> Vec<String> {
    idl.metadata.functions.clone()
}

/// Extract guard information from IDL
pub fn extract_guards_from_idl(idl: &ValenceIdl) -> Vec<String> {
    idl.metadata.guards.clone()
}

/// Check if IDL is compatible with a specific Valence version
pub fn is_compatible_version(idl: &ValenceIdl, target_version: &str) -> bool {
    // Simple version compatibility check
    // In a real implementation, this would use semantic versioning
    if idl.version == target_version {
        return true;
    }
    
    // For now, only consider 0.1.x versions compatible with each other
    idl.version.starts_with("0.1.") && target_version.starts_with("0.1.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idl_generation() {
        let program_id = Pubkey::new_unique();
        let functions = vec!["identity".to_string(), "math_add".to_string()];
        let guards = vec!["owner_guard".to_string()];
        
        let idl = IdlGenerator::generate("test_program", &program_id, functions, guards).unwrap();
        
        // Test basic structure
        assert_eq!(idl.name, "test_program");
        assert_eq!(idl.program_id, program_id.to_string());
        assert_eq!(idl.version, "0.1.0");
        
        // Test metadata
        assert_eq!(idl.metadata.shard, "valence");
        assert_eq!(idl.metadata.functions.len(), 2);
        assert_eq!(idl.metadata.guards.len(), 1);
        
        // Test build info
        assert!(!idl.metadata.build_info.rust_version.is_empty());
        assert_eq!(idl.metadata.build_info.anchor_version, "0.31.1");
        assert!(idl.metadata.build_info.build_timestamp > 0);
    }
    
    #[test]
    fn test_function_idl_generation() {
        let program_id = Pubkey::new_unique();
        let idl = IdlGenerator::generate_function_idl("math_functions", &program_id, "add").unwrap();
        
        assert_eq!(idl.name, "math_functions");
        assert_eq!(idl.metadata.functions, vec!["add"]);
        assert!(idl.metadata.guards.is_empty());
    }
    
    #[test]
    fn test_idl_validation() {
        let program_id = Pubkey::new_unique();
        let mut idl = IdlGenerator::generate("test", &program_id, vec![], vec![]).unwrap();
        
        // Valid IDL should pass
        assert!(IdlGenerator::validate_idl(&idl).is_ok());
        
        // Empty name should fail
        idl.name = String::new();
        assert!(IdlGenerator::validate_idl(&idl).is_err());
        
        // Invalid program ID should fail
        idl.name = "test".to_string();
        idl.program_id = "invalid_pubkey".to_string();
        assert!(IdlGenerator::validate_idl(&idl).is_err());
    }
    
    #[test]
    fn test_helper_functions() {
        let program_id = Pubkey::new_unique();
        let functions = vec!["func1".to_string(), "func2".to_string()];
        let guards = vec!["guard1".to_string()];
        
        let idl = IdlGenerator::generate("test", &program_id, functions.clone(), guards.clone()).unwrap();
        
        assert_eq!(extract_functions_from_idl(&idl), functions);
        assert_eq!(extract_guards_from_idl(&idl), guards);
        
        assert!(is_compatible_version(&idl, "0.1.0"));
        assert!(is_compatible_version(&idl, "0.1.1"));
        assert!(!is_compatible_version(&idl, "0.2.0"));
    }
}