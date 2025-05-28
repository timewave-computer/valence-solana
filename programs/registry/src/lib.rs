use anchor_lang::prelude::*;

// Program ID declaration
declare_id!("VaLRegGhzqJwVmQhciS9JT6czSZJzBGmyReBt3SXXyL");

pub mod state;
pub mod instructions;
pub mod error;
pub mod cache;

use state::*;
use instructions::list_libraries::LibraryInfoResponse;

/// lib.rs - Main program entry point with instruction routing
/// state.rs - Account structures and data types
/// error.rs - Error handling for the program
/// instructions/ - Individual instruction handlers
///    mod.rs - Module exports
///    initialize.rs - Handler for initialize instruction
///    register_library.rs - Handler for library registration
///    update_library_status.rs - Handler for updating library status
///    query_library.rs - Handler for querying library information

#[program]
pub mod registry {
    use super::*;

    /// Initialize the Registry Program with an owner and authorization program
    pub fn initialize(
        ctx: Context<Initialize>,
        authorization_program_id: Pubkey,
        account_factory: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, authorization_program_id, account_factory)
    }

    /// Register a new library with the registry
    pub fn register_library(
        ctx: Context<RegisterLibrary>,
        library_type: String,
        description: String,
        is_approved: bool,
    ) -> Result<()> {
        instructions::register_library::handler(ctx, library_type, description, is_approved)
    }

    /// Update an existing library's status
    pub fn update_library_status(
        ctx: Context<UpdateLibraryStatus>,
        is_approved: bool,
    ) -> Result<()> {
        instructions::update_library_status::handler(ctx, is_approved)
    }

    /// Update an existing library's version
    pub fn update_library_version(
        ctx: Context<UpdateLibraryVersion>,
        new_version: String,
    ) -> Result<()> {
        instructions::update_library_version::handler(ctx, new_version)
    }

    /// Check version compatibility for library dependencies
    pub fn check_version_compatibility(
        ctx: Context<CheckVersionCompatibility>,
    ) -> Result<bool> {
        instructions::check_version_compatibility::handler(ctx)
    }

    /// Query a library's information
    pub fn query_library(
        ctx: Context<QueryLibrary>,
    ) -> Result<LibraryInfo> {
        instructions::query_library::handler(ctx)
    }

    /// List approved libraries with pagination
    pub fn list_libraries(
        ctx: Context<ListLibraries>,
        start_after: Option<Pubkey>,
        limit: u8,
    ) -> Result<Vec<LibraryInfoResponse>> {
        instructions::list_libraries::handler(ctx, start_after, limit)
    }

    /// Register a ZK program with verification key
    pub fn register_zk_program(
        ctx: Context<RegisterZKProgram>,
        program_id: Pubkey,
        verification_key_hash: [u8; 32],
        program_type: String,
        description: String,
    ) -> Result<()> {
        instructions::register_zk_program::handler(ctx, program_id, verification_key_hash, program_type, description)
    }

    /// Update ZK program status
    pub fn update_zk_program_status(
        ctx: Context<UpdateZKProgramStatus>,
        is_active: bool,
    ) -> Result<()> {
        instructions::update_zk_program_status::handler(ctx, is_active)
    }

    /// Query ZK program information
    pub fn query_zk_program(
        ctx: Context<QueryZKProgram>,
    ) -> Result<ZKProgramInfo> {
        instructions::query_zk_program::handler(ctx)
    }

    /// Verify ZK program registration
    pub fn verify_zk_program(
        ctx: Context<VerifyZKProgram>,
        program_id: Pubkey,
    ) -> Result<bool> {
        instructions::verify_zk_program::handler(ctx, program_id)
    }

    /// Add a dependency to a library
    pub fn add_dependency(
        ctx: Context<AddDependency>,
        dependency: LibraryDependency,
    ) -> Result<()> {
        instructions::add_dependency::handler(ctx, dependency)
    }

    /// Remove a dependency from a library
    pub fn remove_dependency(
        ctx: Context<RemoveDependency>,
        dependency_program_id: Pubkey,
    ) -> Result<()> {
        instructions::remove_dependency::handler(ctx, dependency_program_id)
    }

    /// Resolve library dependencies and create dependency graph
    pub fn resolve_dependencies(
        ctx: Context<ResolveDependencies>,
        library_program_id: Pubkey,
    ) -> Result<Vec<Pubkey>> {
        instructions::resolve_dependencies::handler(ctx, library_program_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    

    #[test]
    fn test_library_dependency_creation() {
        let program_id = Pubkey::new_unique();
        let dependency = LibraryDependency {
            program_id,
            required_version: "1.0.0".to_string(),
            is_optional: false,
            dependency_type: DependencyType::Runtime,
        };

        assert_eq!(dependency.program_id, program_id);
        assert_eq!(dependency.required_version, "1.0.0");
        assert!(!dependency.is_optional);
        assert_eq!(dependency.dependency_type, DependencyType::Runtime);
    }

    #[test]
    fn test_dependency_type_default() {
        let default_type = DependencyType::default();
        assert_eq!(default_type, DependencyType::Runtime);
    }

    #[test]
    fn test_dependency_graph_space_calculation() {
        let space_0 = DependencyGraph::space(0);
        let space_5 = DependencyGraph::space(5);
        let space_20 = DependencyGraph::space(20);

        // Base size: 8 + 32 + 4 + 1 + 8 + 1 = 54 bytes
        assert_eq!(space_0, 54);
        // With 5 dependencies: 54 + (5 * 32) = 214 bytes
        assert_eq!(space_5, 214);
        // With 20 dependencies: 54 + (20 * 32) = 694 bytes
        assert_eq!(space_20, 694);
    }

    #[test]
    fn test_library_info_serialization() {
        let program_id = Pubkey::new_unique();
        let dependency = LibraryDependency {
            program_id: Pubkey::new_unique(),
            required_version: "1.2.0".to_string(),
            is_optional: true,
            dependency_type: DependencyType::Dev,
        };

        let library_info = LibraryInfo {
            program_id,
            library_type: "token_transfer".to_string(),
            description: "A library for token transfers".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 1234567890,
            dependencies: vec![dependency],
            bump: 255,
        };

        // Test that the structure can be serialized/deserialized
        let serialized = library_info.try_to_vec().unwrap();
        let deserialized: LibraryInfo = LibraryInfo::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, program_id);
        assert_eq!(deserialized.library_type, "token_transfer");
        assert_eq!(deserialized.dependencies.len(), 1);
        assert_eq!(deserialized.dependencies[0].required_version, "1.2.0");
    }

    #[test]
    fn test_zk_program_info_serialization() {
        let program_id = Pubkey::new_unique();
        let verification_key_hash = [1u8; 32];

        let zk_program_info = ZKProgramInfo {
            program_id,
            verification_key_hash,
            program_type: "sp1_verifier".to_string(),
            description: "SP1 proof verifier".to_string(),
            is_active: true,
            registered_at: 1234567890,
            last_verified: 1234567900,
            verification_count: 42,
            bump: 255,
        };

        // Test serialization/deserialization
        let serialized = zk_program_info.try_to_vec().unwrap();
        let deserialized: ZKProgramInfo = ZKProgramInfo::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, program_id);
        assert_eq!(deserialized.verification_key_hash, verification_key_hash);
        assert_eq!(deserialized.program_type, "sp1_verifier");
        assert_eq!(deserialized.verification_count, 42);
    }

    #[test]
    fn test_registry_state_space() {
        let space = std::mem::size_of::<RegistryState>();
        // RegistryState: 32 + 32 + 32 + 1 = 97 bytes (plus discriminator = 105)
        assert!(space >= 97);
    }

    #[test]
    fn test_dependency_graph_space() {
        let space = std::mem::size_of::<DependencyGraph>();
        // DependencyGraph base: 32 + 4 + 1 + 8 + 1 = 46 bytes (plus discriminator = 54)
        assert!(space >= 46);
    }
}

// Integration tests module
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::instructions::update_library_version::{is_valid_version, is_version_greater};
    use crate::instructions::resolve_dependencies::topological_sort;

    #[test]
    fn test_version_validation() {
        assert!(is_valid_version("1.0.0"));
        assert!(is_valid_version("10.20.30"));
        assert!(is_valid_version("0.0.1"));
        
        assert!(!is_valid_version("1.0"));
        assert!(!is_valid_version("1.0.0.0"));
        assert!(!is_valid_version("1.a.0"));
        assert!(!is_valid_version(""));
        assert!(!is_valid_version("v1.0.0"));
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_version_greater("1.0.1", "1.0.0"));
        assert!(is_version_greater("1.1.0", "1.0.0"));
        assert!(is_version_greater("2.0.0", "1.9.9"));
        
        assert!(!is_version_greater("1.0.0", "1.0.0"));
        assert!(!is_version_greater("1.0.0", "1.0.1"));
        assert!(!is_version_greater("0.9.9", "1.0.0"));
    }

    #[test]
    fn test_topological_sort_simple() {
        let root = Pubkey::new_unique();
        let dep1 = Pubkey::new_unique();
        
        let dependencies = vec![
            LibraryDependency {
                program_id: dep1,
                required_version: "1.0.0".to_string(),
                is_optional: false,
                dependency_type: DependencyType::Runtime,
            }
        ];

        let result = topological_sort(root, &dependencies).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&root));
        assert!(result.contains(&dep1));
    }

    #[test]
    fn test_topological_sort_empty() {
        let root = Pubkey::new_unique();
        let dependencies = vec![];

        let result = topological_sort(root, &dependencies).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root);
    }

    #[test]
    fn test_topological_sort_too_large() {
        let root = Pubkey::new_unique();
        let mut dependencies = vec![];
        
        // Create 25 dependencies (exceeds limit of 20)
        for _ in 0..25 {
            dependencies.push(LibraryDependency {
                program_id: Pubkey::new_unique(),
                required_version: "1.0.0".to_string(),
                is_optional: false,
                dependency_type: DependencyType::Runtime,
            });
        }

        let result = topological_sort(root, &dependencies);
        assert!(result.is_err());
    }
}

