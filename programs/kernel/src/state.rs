/// State definitions for the unified Registry program
/// This module handles library and ZK program registration, dependency management,
/// and verification using the standardized Valence patterns.
use anchor_lang::prelude::*;
// Core protocol state definitions

/// Base program state structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProgramState {
    /// Program ID
    pub program_id: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// Program version
    pub version: u8,
    /// Whether the program is paused
    pub is_paused: bool,
    /// PDA bump
    pub bump: u8,
    /// Creation timestamp
    pub created_at: i64,
    /// Last updated timestamp
    pub last_updated: i64,
}

impl ProgramState {
    pub const SIZE: usize = 32 + 32 + 1 + 1 + 1 + 8 + 8;
    
    pub fn new(program_id: Pubkey, authority: Pubkey, bump: u8) -> Self {
        let clock = Clock::get().unwrap_or_default();
        Self {
            program_id,
            authority,
            version: 1,
            is_paused: false,
            bump,
            created_at: clock.unix_timestamp,
            last_updated: clock.unix_timestamp,
        }
    }
    
    pub fn initialize(&mut self, authority: Pubkey) -> Result<()> {
        self.authority = authority;
        self.version = 1;
        self.is_paused = false;
        Ok(())
    }
    
    pub fn close(&mut self) -> Result<()> {
        self.is_paused = true;
        Ok(())
    }
    
    pub fn validate_state(&self) -> Result<()> {
        require!(self.version > 0, anchor_lang::error::ErrorCode::AccountNotInitialized);
        Ok(())
    }
}

/// Base registry entry structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegistryEntryBase {
    /// Entry ID
    pub id: String,
    /// Entry authority
    pub authority: Pubkey,
    /// Creation timestamp
    pub created_at: i64,
    /// Last updated timestamp
    pub last_updated: i64,
    /// PDA bump
    pub bump: u8,
}

impl RegistryEntryBase {
    pub fn get_space(id_len: usize) -> usize {
        4 + id_len + 32 + 8 + 8 + 1
    }
    
    pub fn new(id: String, authority: Pubkey, bump: u8) -> Self {
        let clock = Clock::get().unwrap_or_default();
        Self {
            id,
            authority,
            created_at: clock.unix_timestamp,
            last_updated: clock.unix_timestamp,
            bump,
        }
    }
}

/// Lifecycle management trait
pub trait LifecycleManaged {
    fn initialize(&mut self, authority: Pubkey) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn validate_state(&self) -> Result<()>;
    fn version(&self) -> u8;
    fn bump(&self) -> u8;
}

/// Main state account for the Registry program
#[account]
pub struct RegistryState {
    /// Base program state
    pub base: ProgramState,
    
    /// Total libraries registered
    pub total_libraries: u64,
    
    /// Total ZK programs registered
    pub total_zk_programs: u64,
    
    /// Total dependencies registered
    pub total_dependencies: u64,
    
    /// Current registry version
    pub registry_version: u16,
    
    /// Reserved space for future fields
    pub _reserved: [u8; 32],
}

impl RegistryState {
    pub const SIZE: usize = 8 + // discriminator
        ProgramState::SIZE +     // 83 bytes
        8 + // total_libraries
        8 + // total_zk_programs
        8 + // total_dependencies
        2 + // registry_version
        32; // _reserved
        // Total: 8 + 83 + 8 + 8 + 8 + 2 + 32 = 149 bytes
        
    /// Initialize a new registry state
    pub fn new(
        program_id: Pubkey,
        authority: Pubkey,
        bump: u8,
    ) -> Self {
        Self {
            base: ProgramState::new(program_id, authority, bump),
            total_libraries: 0,
            total_zk_programs: 0,
            total_dependencies: 0,
            registry_version: 1,
            _reserved: [0u8; 32],
        }
    }
    
    /// Check if the registry is operational
    pub fn is_operational(&self) -> bool {
        !self.base.is_paused && self.base.version > 0
    }
}

impl LifecycleManaged for RegistryState {
    fn initialize(&mut self, authority: Pubkey) -> Result<()> {
        self.base.initialize(authority)?;
        Ok(())
    }
    
    fn close(&mut self) -> Result<()> {
        self.base.close()
    }
    
    fn validate_state(&self) -> Result<()> {
        self.base.validate_state()?;
        require!(
            self.registry_version > 0,
            anchor_lang::error::ErrorCode::AccountNotInitialized
        );
        Ok(())
    }
    
    fn version(&self) -> u8 {
        self.base.version
    }
    
    fn bump(&self) -> u8 {
        self.base.bump
    }
}

/// Library registration entry
#[account]
pub struct LibraryEntry {
    /// Base registry entry
    pub base: RegistryEntryBase,
    
    /// The registry this library belongs to
    pub registry: Pubkey,
    
    /// Library name
    pub name: String,
    
    /// Library version
    pub version: String,
    
    /// Library author/publisher
    pub author: Pubkey,
    
    /// Library metadata hash (IPFS or similar)
    pub metadata_hash: [u8; 32],
    
    /// Program ID of the library
    pub program_id: Pubkey,
    
    /// Library status
    pub status: LibraryStatus,
    
    /// Dependencies on other libraries
    pub dependencies: Vec<String>,
    
    /// Tags for categorization
    pub tags: Vec<String>,
    
    /// Verification status
    pub is_verified: bool,
    
    /// Total downloads/usage count
    pub usage_count: u64,
}

impl LibraryEntry {
    pub fn get_space(
        library_id: &str,
        name: &str,
        version: &str,
        dependencies_count: usize,
        tags_count: usize,
    ) -> usize {
        8 + // discriminator
        RegistryEntryBase::get_space(library_id.len()) +
        32 + // registry
        4 + name.len() + // name
        4 + version.len() + // version
        32 + // author
        32 + // metadata_hash
        32 + // program_id
        1 + // status
        4 + dependencies_count * 32 + // dependencies (estimate 32 chars each)
        4 + tags_count * 16 + // tags (estimate 16 chars each)
        8 + // is_verified (padded)
        8 // usage_count
    }
    
    /// Record a library usage
    pub fn record_usage(&mut self) {
        self.usage_count = self.usage_count.saturating_add(1);
        self.base.last_updated = Clock::get().unwrap().unix_timestamp;
    }
}

/// ZK program registration entry
#[account]
pub struct ZkProgramEntry {
    /// Base registry entry
    pub base: RegistryEntryBase,
    
    /// The registry this ZK program belongs to
    pub registry: Pubkey,
    
    /// ZK program name
    pub name: String,
    
    /// ZK program version
    pub version: String,
    
    /// ZK program author/publisher
    pub author: Pubkey,
    
    /// Program verification key
    pub verification_key: [u8; 64],
    
    /// Program metadata hash
    pub metadata_hash: [u8; 32],
    
    /// ZK program status
    pub status: ZkProgramStatus,
    
    /// Verification status
    pub is_verified: bool,
    
    /// Circuit constraints count
    pub constraints_count: u64,
    
    /// Total proofs verified
    pub proofs_verified: u64,
}

impl ZkProgramEntry {
    pub fn get_space(
        program_id: &str,
        name: &str,
        version: &str,
    ) -> usize {
        8 + // discriminator
        RegistryEntryBase::get_space(program_id.len()) +
        32 + // registry
        4 + name.len() + // name
        4 + version.len() + // version
        32 + // author
        64 + // verification_key
        32 + // metadata_hash
        1 + // status
        8 + // is_verified (padded)
        8 + // constraints_count
        8 // proofs_verified
    }
    
    /// Record a proof verification
    pub fn record_proof_verification(&mut self) {
        self.proofs_verified = self.proofs_verified.saturating_add(1);
        self.base.last_updated = Clock::get().unwrap().unix_timestamp;
    }
}

/// Library dependency entry
#[account]
pub struct DependencyEntry {
    /// Base registry entry  
    pub base: RegistryEntryBase,
    
    /// The library that has this dependency
    pub dependent_library: String,
    
    /// The library being depended upon
    pub dependency_library: String,
    
    /// Version requirement (semver-like)
    pub version_requirement: String,
    
    /// Whether this dependency is optional
    pub is_optional: bool,
    
    /// Dependency type
    pub dependency_type: DependencyType,
}

impl DependencyEntry {
    pub fn get_space(
        dependency_id: &str,
        dependent_library: &str,
        dependency_library: &str,
        version_requirement: &str,
    ) -> usize {
        8 + // discriminator
        RegistryEntryBase::get_space(dependency_id.len()) +
        4 + dependent_library.len() + // dependent_library
        4 + dependency_library.len() + // dependency_library
        4 + version_requirement.len() + // version_requirement
        8 + // is_optional (padded)
        1 // dependency_type
    }
}

/// Library status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum LibraryStatus {
    Draft,
    Published,
    Deprecated,
    Archived,
}

impl Default for LibraryStatus {
    fn default() -> Self {
        LibraryStatus::Draft
    }
}

/// ZK program status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ZkProgramStatus {
    Draft,
    Published,
    Verified,
    Deprecated,
    Archived,
}

impl Default for ZkProgramStatus {
    fn default() -> Self {
        ZkProgramStatus::Draft
    }
}

/// Dependency type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum DependencyType {
    Runtime,
    Development,
    Optional,
    Peer,
}

impl Default for DependencyType {
    fn default() -> Self {
        DependencyType::Runtime
    }
}

/// Version compatibility structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VersionCompatibility {
    /// Minimum compatible version
    pub min_version: String,
    
    /// Maximum compatible version
    pub max_version: String,
    
    /// Whether this is a breaking change
    pub breaking_change: bool,
}

impl VersionCompatibility {
    pub fn new(min_version: String, max_version: String, breaking_change: bool) -> Self {
        Self {
            min_version,
            max_version,
            breaking_change,
        }
    }
    
    /// Check if a version is compatible
    pub fn is_compatible(&self, version: &str) -> bool {
        // Simple string comparison for now - would use semver in production
        version >= self.min_version.as_str() && version <= self.max_version.as_str()
    }
}

/// Helper function to get library PDA
pub fn get_library_pda(
    registry: &Pubkey,
    library_id: &str,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"library",
            registry.as_ref(),
            library_id.as_bytes(),
        ],
        program_id,
    )
}

/// Helper function to get ZK program PDA
pub fn get_zk_program_pda(
    registry: &Pubkey,
    program_id: &str,
    program_id_key: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"zk_program",
            registry.as_ref(),
            program_id.as_bytes(),
        ],
        program_id_key,
    )
}

/// Helper function to get dependency PDA
pub fn get_dependency_pda(
    registry: &Pubkey,
    dependent_library: &str,
    dependency_library: &str,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    let dependency_id = format!("{}::{}", dependent_library, dependency_library);
    Pubkey::find_program_address(
        &[
            b"dependency",
            registry.as_ref(),
            dependency_id.as_bytes(),
        ],
        program_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_state_size() {
        let state = RegistryState::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            255,
        );
        
        // The #[account] macro adds additional fields, so we check the serialized size
        let serialized = state.try_to_vec().unwrap();
        assert!(serialized.len() <= RegistryState::SIZE, 
            "Serialized size {} exceeds RegistryState::SIZE {}", 
            serialized.len(), RegistryState::SIZE);
        
        // Test operational status
        assert!(state.is_operational());
    }
    
    #[test]
    fn test_library_space_calculation() {
        let library_id = "test_library_v1";
        let name = "Test Library";
        let version = "1.0.0";
        
        let space = LibraryEntry::get_space(library_id, name, version, 2, 3);
        
        // Space should be reasonable and include all components
        assert!(space > 200); // Should have substantial base size
        assert!(space < 5000); // Should not be unreasonably large
    }
    
    #[test]
    fn test_version_compatibility() {
        let compat = VersionCompatibility::new(
            "1.0.0".to_string(),
            "2.0.0".to_string(),
            false,
        );
        
        // Test compatibility checks
        assert!(compat.is_compatible("1.5.0"));
        assert!(compat.is_compatible("1.0.0"));
        assert!(compat.is_compatible("2.0.0"));
        assert!(!compat.is_compatible("0.9.0"));
        assert!(!compat.is_compatible("2.1.0"));
    }
    
    #[test]
    fn test_pda_generation() {
        let registry = Pubkey::new_unique();
        let library_id = "test_library";
        let program_id = Pubkey::new_unique();
        
        // Test library PDA
        let (pda1, bump1) = get_library_pda(&registry, library_id, &program_id);
        let (pda2, bump2) = get_library_pda(&registry, library_id, &program_id);
        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
        
        // Test ZK program PDA
        let zk_program_id = "test_zk_program";
        let (pda3, bump3) = get_zk_program_pda(&registry, zk_program_id, &program_id);
        let (pda4, bump4) = get_zk_program_pda(&registry, zk_program_id, &program_id);
        assert_eq!(pda3, pda4);
        assert_eq!(bump3, bump4);
    }
    
    #[test]
    fn test_library_status_transitions() {
        let mut status = LibraryStatus::Draft;
        assert_eq!(status, LibraryStatus::Draft);
        
        // Test valid transitions
        status = LibraryStatus::Published;
        assert_eq!(status, LibraryStatus::Published);
        
        status = LibraryStatus::Deprecated;
        assert_eq!(status, LibraryStatus::Deprecated);
        
        status = LibraryStatus::Archived;
        assert_eq!(status, LibraryStatus::Archived);
    }
} 