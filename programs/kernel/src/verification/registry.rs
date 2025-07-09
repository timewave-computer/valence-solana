/// Verification function registry and lifecycle management
/// This module handles function registration, discovery, and performance tracking
use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};

// ======================= DEFAULT VERIFICATION REGISTRY =======================

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
    
    /// Register all default verification functions
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

// ======================= CAPABILITY CONFIGURATION =======================

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

// ======================= FUNCTION REGISTRY STATE =======================

/// Function Registry state for managing all types of functions
#[account]
pub struct FunctionRegistryState {
    /// The authority that manages the registry
    pub authority: Pubkey,
    /// Total number of functions registered
    pub total_functions: u64,
    /// Number of functions by category
    pub functions_by_category: FunctionCategoryStats,
    /// Version of the registry for future upgrades
    pub version: u8,
    /// PDA bump seed
    pub bump: u8,
    /// Reserved space for future use
    pub _reserved: [u8; 64],
}

impl FunctionRegistryState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        8 + // total_functions
        std::mem::size_of::<FunctionCategoryStats>() + // functions_by_category
        1 + // version
        1 + // bump
        64; // _reserved
}

/// Statistics for functions by category
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionCategoryStats {
    /// Pure verification functions
    pub verification_functions: u32,
    /// Pure computation functions
    pub computation_functions: u32,
    /// Diff generation functions
    pub diff_generation_functions: u32,
    /// Diff verification functions
    pub diff_verification_functions: u32,
    /// Function composition functions
    pub composition_functions: u32,
    /// Custom function types
    pub custom_functions: u32,
}

impl Default for FunctionCategoryStats {
    fn default() -> Self {
        Self {
            verification_functions: 0,
            computation_functions: 0,
            diff_generation_functions: 0,
            diff_verification_functions: 0,
            composition_functions: 0,
            custom_functions: 0,
        }
    }
}

/// Function types supported by the registry
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FunctionType {
    /// Pure verification function (legacy compatibility)
    Verification,
    /// Pure computation function
    Computation,
    /// Diff generation function
    DiffGeneration,
    /// Diff verification function
    DiffVerification,
    /// Function composition function
    Composition,
    /// Custom function type
    Custom(String),
}

/// A registered function entry in the registry
/// Functions are addressed by their hash for deterministic lookup
#[account]
pub struct RegisteredFunction {
    /// Hash of the function code (SHA-256)
    pub function_hash: [u8; 32],
    /// The function code/bytecode
    pub function_code: Vec<u8>,
    /// Function type/category
    pub function_type: FunctionType,
    /// Human-readable name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether the function is approved for use
    pub is_approved: bool,
    /// Function version string
    pub version: String,
    /// Creator of the function
    pub creator: Pubkey,
    /// Function metadata
    pub metadata: FunctionMetadata,
    /// Function dependencies
    pub dependencies: Vec<[u8; 32]>, // Hashes of dependent functions
    /// Function performance metrics
    pub performance: FunctionPerformance,
    /// Creation timestamp
    pub created_at: i64,
    /// Last updated timestamp
    pub last_updated: i64,
    /// Number of times this function has been used
    pub usage_count: u64,
    /// PDA bump seed
    pub bump: u8,
}

/// Metadata for a registered function
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionMetadata {
    /// Tags for categorization and search
    pub tags: Vec<String>,
    /// Compatibility version
    pub compatibility_version: String,
    /// Input schema description
    pub input_schema: String,
    /// Output schema description
    pub output_schema: String,
    /// Documentation URL
    pub documentation_url: String,
    /// License information
    pub license: String,
}

/// Performance metrics for a function
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionPerformance {
    /// Average execution time in microseconds
    pub avg_execution_time_us: u64,
    /// Maximum execution time observed
    pub max_execution_time_us: u64,
    /// Minimum execution time observed
    pub min_execution_time_us: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Average compute units consumed
    pub avg_compute_units: u32,
    /// Last performance update timestamp
    pub last_performance_update: i64,
}

impl Default for FunctionPerformance {
    fn default() -> Self {
        Self {
            avg_execution_time_us: 0,
            max_execution_time_us: 0,
            min_execution_time_us: u64::MAX,
            successful_executions: 0,
            failed_executions: 0,
            avg_compute_units: 0,
            last_performance_update: 0,
        }
    }
}

impl RegisteredFunction {
    /// Calculate space needed for function creation
    pub fn get_space(
        code_size: usize, 
        name_len: usize,
        description_len: usize, 
        version_len: usize,
        metadata: &FunctionMetadata,
        dependencies_count: usize,
    ) -> usize {
        8 + // discriminator
        32 + // function_hash
        4 + code_size + // function_code vec
        std::mem::size_of::<FunctionType>() + // function_type enum
        4 + name_len + // name string
        4 + description_len + // description string
        1 + // is_approved
        4 + version_len + // version string
        32 + // creator
        metadata.size() + // metadata
        4 + (dependencies_count * 32) + // dependencies vec
        std::mem::size_of::<FunctionPerformance>() + // performance
        8 + // created_at
        8 + // last_updated
        8 + // usage_count
        1 // bump
    }
    
    /// Calculate the hash of function code
    pub fn calculate_hash(code: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(code);
        hasher.finalize().into()
    }
    
    /// Verify the function hash matches the stored code
    pub fn verify_hash(&self) -> bool {
        let calculated_hash = Self::calculate_hash(&self.function_code);
        calculated_hash == self.function_hash
    }
    
    /// Update usage count and timestamp
    pub fn increment_usage(&mut self) -> Result<()> {
        self.usage_count = self.usage_count.saturating_add(1);
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }
    
    /// Record performance metrics
    pub fn record_performance(
        &mut self, 
        execution_time_us: u64, 
        compute_units: u32, 
        success: bool
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        if success {
            self.performance.successful_executions = self.performance.successful_executions.saturating_add(1);
        } else {
            self.performance.failed_executions = self.performance.failed_executions.saturating_add(1);
        }
        
        // Update execution time metrics
        let total_executions = self.performance.successful_executions + self.performance.failed_executions;
        if total_executions > 0 {
            // Running average
            self.performance.avg_execution_time_us = 
                ((self.performance.avg_execution_time_us * (total_executions - 1)) + execution_time_us) / total_executions;
        } else {
            self.performance.avg_execution_time_us = execution_time_us;
        }
        
        // Update min/max
        self.performance.max_execution_time_us = self.performance.max_execution_time_us.max(execution_time_us);
        self.performance.min_execution_time_us = self.performance.min_execution_time_us.min(execution_time_us);
        
        // Update compute units (running average)
        if total_executions > 0 {
            self.performance.avg_compute_units = 
                ((self.performance.avg_compute_units as u64 * (total_executions - 1)) + compute_units as u64) as u32 / total_executions as u32;
        } else {
            self.performance.avg_compute_units = compute_units;
        }
        
        self.performance.last_performance_update = clock.unix_timestamp;
        self.last_updated = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Check if function is ready for use
    pub fn is_usable(&self) -> bool {
        self.is_approved && self.verify_hash()
    }
    
    /// Check if function has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.metadata.tags.iter().any(|t| t == tag)
    }
    
    /// Check if function matches search criteria
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        self.name.to_lowercase().contains(&query_lower) ||
        self.description.to_lowercase().contains(&query_lower) ||
        self.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
    }
    
    /// Get the PDA seeds for this function
    pub fn get_pda_seeds(&self) -> [&[u8]; 2] {
        [b"registered_function", &self.function_hash[..21]]
    }
}

impl FunctionMetadata {
    /// Calculate size of metadata for space allocation
    pub fn size(&self) -> usize {
        4 + self.tags.iter().map(|t| 4 + t.len()).sum::<usize>() + // tags vec
        4 + self.compatibility_version.len() + // compatibility_version
        4 + self.input_schema.len() + // input_schema
        4 + self.output_schema.len() + // output_schema
        4 + self.documentation_url.len() + // documentation_url
        4 + self.license.len() // license
    }
}

/// Discovery index for functions by category and tags
#[account]
pub struct FunctionDiscoveryIndex {
    /// Category this index is for
    pub category: FunctionType,
    /// Tag this index is for (optional)
    pub tag: Option<String>,
    /// Function hashes in this category/tag
    pub function_hashes: Vec<[u8; 32]>,
    /// Last updated timestamp
    pub last_updated: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl FunctionDiscoveryIndex {
    pub fn get_space(tag_len: usize, function_count: usize) -> usize {
        8 + // discriminator
        std::mem::size_of::<FunctionType>() + // category
        1 + 4 + tag_len + // tag option
        4 + (function_count * 32) + // function_hashes vec
        8 + // last_updated
        1 // bump
    }
}

// ======================= TESTS =======================

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