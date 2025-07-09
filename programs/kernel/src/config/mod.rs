/// Configuration and registry management for Valence Protocol
/// This module provides configuration management and registry functionality
use anchor_lang::prelude::*;

// ========== DEFAULT VERIFICATION HASHES ==========
// Merged from defaults.rs

/// Default verification function hashes
pub const DEFAULT_BASIC_PERMISSION_HASH: [u8; 32] = [1; 32];
pub const DEFAULT_SYSTEM_AUTH_HASH: [u8; 32] = [2; 32];
pub const DEFAULT_BLOCK_HEIGHT_HASH: [u8; 32] = [3; 32];
pub const DEFAULT_PARAMETER_CONSTRAINT_HASH: [u8; 32] = [4; 32];
pub const DEFAULT_SESSION_CREATION_HASH: [u8; 32] = [5; 32];

/// Default verification registry configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DefaultVerificationRegistry {
    /// Registry authority
    pub authority: Pubkey,
    /// Active verification functions
    pub active_verifications: Vec<String>,
    /// Configuration version
    pub version: u32,
    /// Registry settings
    pub settings: RegistrySettings,
}

/// Registry settings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegistrySettings {
    /// Maximum number of verification functions
    pub max_verifications: u32,
    /// Require verification for all capabilities
    pub require_verification: bool,
    /// Default verification timeout
    pub default_timeout: u64,
    /// Allow custom verification functions
    pub allow_custom_verifications: bool,
}

impl Default for RegistrySettings {
    fn default() -> Self {
        Self {
            max_verifications: 100,
            require_verification: true,
            default_timeout: 30,
            allow_custom_verifications: false,
        }
    }
}

impl DefaultVerificationRegistry {
    /// Create a new default registry
    pub fn new(authority: Pubkey) -> Self {
        Self {
            authority,
            active_verifications: vec![
                "basic_permission".to_string(),
                "system_auth".to_string(),
                "block_height".to_string(),
                "parameter_constraint".to_string(),
                "session_creation".to_string(),
            ],
            version: 1,
            settings: RegistrySettings::default(),
        }
    }
    
    /// Get default verification function hashes
    /// Merged from defaults.rs
    pub fn get_default_hashes() -> Vec<[u8; 32]> {
        vec![
            DEFAULT_BASIC_PERMISSION_HASH,
            DEFAULT_SYSTEM_AUTH_HASH,
            DEFAULT_BLOCK_HEIGHT_HASH,
            DEFAULT_PARAMETER_CONSTRAINT_HASH,
            DEFAULT_SESSION_CREATION_HASH,
        ]
    }
    
    /// Add a verification function to the registry
    pub fn add_verification(&mut self, verification_id: String) -> Result<()> {
        if self.active_verifications.len() >= self.settings.max_verifications as usize {
            return Err(anchor_lang::error::ErrorCode::AccountNotEnoughKeys.into());
        }
        
        if !self.active_verifications.contains(&verification_id) {
            self.active_verifications.push(verification_id);
        }
        
        Ok(())
    }
    
    /// Remove a verification function from the registry
    pub fn remove_verification(&mut self, verification_id: &str) -> Result<()> {
        self.active_verifications.retain(|id| id != verification_id);
        Ok(())
    }
    
    /// Check if a verification function is active
    pub fn is_verification_active(&self, verification_id: &str) -> bool {
        self.active_verifications.contains(&verification_id.to_string())
    }
    
    /// Update registry settings
    pub fn update_settings(&mut self, settings: RegistrySettings) -> Result<()> {
        self.settings = settings;
        self.version += 1;
        Ok(())
    }
}

/// Configuration state for on-chain storage
#[account]
pub struct ConfigState {
    /// Registry configuration
    pub registry: DefaultVerificationRegistry,
    /// Configuration bump
    pub bump: u8,
}

impl ConfigState {
    pub fn get_space() -> usize {
        8 + // discriminator
        32 + // authority
        4 + (50 * 20) + // active_verifications (assume max 20 items, 50 chars each)
        4 + // version
        4 + 1 + 8 + 1 + // settings
        1 // bump
    }
} 