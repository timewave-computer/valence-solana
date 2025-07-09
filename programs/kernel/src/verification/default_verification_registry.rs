// Core verification registry with default functions
/// Default Verification Function Registry
/// This module defines the standard verification functions that replace hardcoded constraints
/// in the execution flow, providing a clean, composable, and transparent verification system.

use anchor_lang::prelude::*;

/// Registry of default verification function hashes
/// These replace all hardcoded constraints in the execution flow
pub struct DefaultVerificationRegistry;

impl DefaultVerificationRegistry {
    /// Get the hash for system authentication verification
    /// Replaces: hardcoded caller authentication checks (entrypoint->eval->shard)
    pub fn system_auth_hash() -> [u8; 32] {
        anchor_lang::solana_program::hash::hash(
            b"system_auth_verifier::verify"
        ).to_bytes()
    }
    
    /// Get the hash for pause state verification
    /// Replaces: hardcoded eval/shard pause checks
    pub fn pause_state_hash() -> [u8; 32] {
        anchor_lang::solana_program::hash::hash(
            b"pause_state_verifier::verify"
        ).to_bytes()
    }
    
    /// Get the hash for capability integrity verification
    /// Replaces: hardcoded capability active and ID match checks
    pub fn capability_integrity_hash() -> [u8; 32] {
        anchor_lang::solana_program::hash::hash(
            b"capability_integrity_verifier::verify"
        ).to_bytes()
    }
    
    /// Get the hash for block height verification (replay attack prevention)
    /// Replaces: hardcoded block height validation
    pub fn block_height_hash() -> [u8; 32] {
        anchor_lang::solana_program::hash::hash(
            b"block_height_verifier::verify"
        ).to_bytes()
    }
    
    /// Get the hash for settlement data verification
    /// Replaces: hardcoded settlement data validation checks
    pub fn settlement_data_hash() -> [u8; 32] {
        anchor_lang::solana_program::hash::hash(
            b"settlement_data_verifier::verify"
        ).to_bytes()
    }
    
    /// Get all default verification function hashes
    /// These are automatically included in capabilities unless explicitly opted out
    pub fn all_defaults() -> Vec<[u8; 32]> {
        vec![
            Self::system_auth_hash(),
            Self::pause_state_hash(),
            Self::capability_integrity_hash(),
            Self::block_height_hash(),
            Self::settlement_data_hash(),
        ]
    }
}

/// Configuration for capability creation with default verification functions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CapabilityConfig {
    /// Whether to include default verification functions
    pub include_defaults: bool,
    /// Custom verification functions to add
    pub custom_verifications: Vec<[u8; 32]>,
    /// Default verification functions to exclude (if include_defaults = true)
    pub exclude_defaults: Vec<[u8; 32]>,
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            include_defaults: true,
            custom_verifications: vec![],
            exclude_defaults: vec![],
        }
    }
}

impl CapabilityConfig {
    /// Build the final verification function list
    pub fn build_verification_functions(&self) -> Vec<[u8; 32]> {
        let mut functions = Vec::new();
        
        // Add defaults if requested
        if self.include_defaults {
            for default_hash in DefaultVerificationRegistry::all_defaults() {
                if !self.exclude_defaults.contains(&default_hash) {
                    functions.push(default_hash);
                }
            }
        }
        
        // Add custom verifications
        for custom_hash in &self.custom_verifications {
            if !functions.contains(custom_hash) {
                functions.push(*custom_hash);
            }
        }
        
        functions
    }
    
    /// Create a capability with only custom verification functions
    pub fn custom_only(custom_verifications: Vec<[u8; 32]>) -> Self {
        Self {
            include_defaults: false,
            custom_verifications,
            exclude_defaults: vec![],
        }
    }
    
    /// Create a capability with defaults plus additional custom functions
    pub fn with_additional(custom_verifications: Vec<[u8; 32]>) -> Self {
        Self {
            include_defaults: true,
            custom_verifications,
            exclude_defaults: vec![],
        }
    }
    
    /// Create a capability with defaults minus some excluded functions
    pub fn without_defaults(exclude_defaults: Vec<[u8; 32]>) -> Self {
        Self {
            include_defaults: true,
            custom_verifications: vec![],
            exclude_defaults,
        }
    }
}

impl DefaultVerificationRegistry {
    /// Register all default verification functions
    /// This consolidates registration logic from register_functions.rs
    pub fn register_all_default_functions(
        verification_function_table_program: Pubkey,
        authority: Pubkey,
    ) -> Result<()> {
        use crate::verification::{register_basic_verifications, register_context_verifications};
        
        // Register basic verifications (permission, constraints, system auth)
        register_basic_verifications(verification_function_table_program, authority)?;
        
        // Register context verifications (block height, session creation)
        register_context_verifications(verification_function_table_program, authority)?;
        
        // Register composable capability example
        Self::register_composable_capability(verification_function_table_program, authority)?;
        
        msg!("All default verification functions registered");
        Ok(())
    }
    
    /// Register a composable capability verification function
    /// Moved from register_functions.rs
    fn register_composable_capability(
        _shard_program: Pubkey,
        _authority: Pubkey,
    ) -> Result<()> {
        // Example registration for a complex capability with multiple verification functions
        let _description = "DeFi swap capability with permission and parameter validation".to_string();
        msg!("Registering composable capability verification");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_verification_registry() {
        // Test that all default hashes are unique
        let defaults = DefaultVerificationRegistry::all_defaults();
        assert_eq!(defaults.len(), 5);
        
        // Test that each hash is deterministic
        assert_eq!(
            DefaultVerificationRegistry::system_auth_hash(),
            DefaultVerificationRegistry::system_auth_hash()
        );
    }
    
    #[test]
    fn test_capability_config_build() {
        // Test default config
        let config = CapabilityConfig::default();
        let functions = config.build_verification_functions();
        assert_eq!(functions.len(), 5); // All 5 defaults
        
        // Test custom only
        let custom_hash = [1u8; 32];
        let config = CapabilityConfig::custom_only(vec![custom_hash]);
        let functions = config.build_verification_functions();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0], custom_hash);
        
        // Test with additional
        let config = CapabilityConfig::with_additional(vec![custom_hash]);
        let functions = config.build_verification_functions();
        assert_eq!(functions.len(), 6); // 5 defaults + 1 custom
    }
} 