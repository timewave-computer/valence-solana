use anchor_lang::prelude::*;
use crate::error::RegistryError;

/// Registry Program state
#[account]
pub struct RegistryState {
    /// Program owner
    pub owner: Pubkey,
    /// Authorization Program ID
    pub authorization_program_id: Pubkey,
    /// Account Factory address
    pub account_factory: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Library information
#[account]
pub struct LibraryInfo {
    /// Program ID of the library
    pub program_id: Pubkey,
    /// Type of library (e.g., "token_transfer", "vault_deposit")
    pub library_type: String,
    /// Human-readable description
    pub description: String,
    /// Whether the library is globally approved
    pub is_approved: bool,
    /// Version information
    pub version: String,
    /// Last updated timestamp
    pub last_updated: i64,
    /// Dependencies on other libraries
    pub dependencies: Vec<LibraryDependency>,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Library dependency information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LibraryDependency {
    /// Program ID of the dependency
    pub program_id: Pubkey,
    /// Required version (semantic versioning)
    pub required_version: String,
    /// Whether this is an optional dependency
    pub is_optional: bool,
    /// Dependency type (e.g., "runtime", "build", "dev")
    pub dependency_type: DependencyType,
}

/// Types of dependencies
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum DependencyType {
    /// Required at runtime
    Runtime,
    /// Required for building/compilation
    Build,
    /// Required for development/testing
    Dev,
    /// Optional enhancement
    Optional,
}

impl Default for DependencyType {
    fn default() -> Self {
        DependencyType::Runtime
    }
}

/// Dependency graph for resolving library dependencies
#[account]
pub struct DependencyGraph {
    /// Root library program ID
    pub root_library: Pubkey,
    /// Resolved dependency order (topologically sorted)
    pub resolved_order: Vec<Pubkey>,
    /// Whether the dependency graph is valid (no cycles)
    pub is_valid: bool,
    /// Last resolution timestamp
    pub last_resolved: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl DependencyGraph {
    /// Calculate space needed for this account
    pub fn space(dependency_count: usize) -> usize {
        8 + // discriminator
        32 + // root_library
        4 + (dependency_count * 32) + // resolved_order vec
        1 + // is_valid
        8 + // last_resolved
        1 // bump
    }
}

/// Instruction context for initializing the registry program
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The program state account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<RegistryState>(),
        seeds = [b"registry_state".as_ref()],
        bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The account paying for the initialization
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for registering a library
#[derive(Accounts)]
#[instruction(library_type: String, description: String)]
pub struct RegisterLibrary<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<LibraryInfo>() + 
                library_type.len() + 
                description.len() + 
                32 + // Extra space for version
                4 + 0, // Empty dependencies vector (4 bytes for length + 0 dependencies)
        seeds = [b"library_info".as_ref(), program_id.key().as_ref()],
        bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The program ID of the library being registered
    /// This is not a signer, just the public key
    pub program_id: UncheckedAccount<'info>,
    
    /// The owner of the registry
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for updating a library's status
#[derive(Accounts)]
pub struct UpdateLibraryStatus<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account
    #[account(
        mut,
        seeds = [b"library_info".as_ref(), library_info.program_id.as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The owner of the registry
    pub owner: Signer<'info>,
}

/// Instruction context for updating a library's version
#[derive(Accounts)]
pub struct UpdateLibraryVersion<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account
    #[account(
        mut,
        seeds = [b"library_info".as_ref(), library_info.program_id.as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The owner of the registry
    pub owner: Signer<'info>,
}

/// Instruction context for checking version compatibility
#[derive(Accounts)]
pub struct CheckVersionCompatibility<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account to check
    #[account(
        seeds = [b"library_info".as_ref(), library_info.program_id.as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
}

/// Instruction context for querying a library
#[derive(Accounts)]
pub struct QueryLibrary<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account
    #[account(
        seeds = [b"library_info".as_ref(), program_id.key().as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The program ID to query
    pub program_id: UncheckedAccount<'info>,
}

/// Instruction context for listing libraries
#[derive(Accounts)]
pub struct ListLibraries<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
}

/// Instruction context for adding a dependency to a library
#[derive(Accounts)]
#[instruction(dependency: LibraryDependency)]
pub struct AddDependency<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account to add dependency to
    #[account(
        mut,
        seeds = [b"library_info".as_ref(), library_info.program_id.as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The dependency library info account (must exist)
    #[account(
        seeds = [b"library_info".as_ref(), dependency.program_id.as_ref()],
        bump = dependency_library.bump
    )]
    pub dependency_library: Account<'info, LibraryInfo>,
    
    /// The owner of the registry
    #[account(mut)]
    pub owner: Signer<'info>,
}

/// Instruction context for removing a dependency from a library
#[derive(Accounts)]
#[instruction(dependency_program_id: Pubkey)]
pub struct RemoveDependency<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account to remove dependency from
    #[account(
        mut,
        seeds = [b"library_info".as_ref(), library_info.program_id.as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The owner of the registry
    pub owner: Signer<'info>,
}

/// Instruction context for resolving library dependencies
#[derive(Accounts)]
#[instruction(library_program_id: Pubkey)]
pub struct ResolveDependencies<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The library info account to resolve dependencies for
    #[account(
        seeds = [b"library_info".as_ref(), library_program_id.as_ref()],
        bump = library_info.bump
    )]
    pub library_info: Account<'info, LibraryInfo>,
    
    /// The dependency graph account
    #[account(
        init_if_needed,
        payer = payer,
        space = DependencyGraph::space(20), // Support up to 20 dependencies
        seeds = [b"dependency_graph".as_ref(), library_program_id.as_ref()],
        bump
    )]
    pub dependency_graph: Account<'info, DependencyGraph>,
    
    /// Account paying for the dependency graph creation
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// ZK Program information
#[account]
pub struct ZKProgramInfo {
    /// Program ID of the ZK program
    pub program_id: Pubkey,
    /// Hash of the verification key
    pub verification_key_hash: [u8; 32],
    /// Type of ZK program (e.g., "sp1_verifier", "groth16_verifier")
    pub program_type: String,
    /// Human-readable description
    pub description: String,
    /// Whether the ZK program is active
    pub is_active: bool,
    /// Registration timestamp
    pub registered_at: i64,
    /// Last verification timestamp
    pub last_verified: i64,
    /// Number of successful verifications
    pub verification_count: u64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Instruction context for registering a ZK program
#[derive(Accounts)]
#[instruction(program_id: Pubkey, verification_key_hash: [u8; 32], program_type: String, description: String)]
pub struct RegisterZKProgram<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The ZK program info account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<ZKProgramInfo>() + 
                program_type.len() + 
                description.len() + 
                64, // Extra space for strings
        seeds = [b"zk_program_info".as_ref(), program_id.as_ref()],
        bump
    )]
    pub zk_program_info: Account<'info, ZKProgramInfo>,
    
    /// The owner of the registry
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for updating ZK program status
#[derive(Accounts)]
pub struct UpdateZKProgramStatus<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump,
        constraint = registry_state.owner == owner.key() @ RegistryError::NotAuthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The ZK program info account
    #[account(
        mut,
        seeds = [b"zk_program_info".as_ref(), zk_program_info.program_id.as_ref()],
        bump = zk_program_info.bump
    )]
    pub zk_program_info: Account<'info, ZKProgramInfo>,
    
    /// The owner of the registry
    pub owner: Signer<'info>,
}

/// Instruction context for querying a ZK program
#[derive(Accounts)]
pub struct QueryZKProgram<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The ZK program info account
    #[account(
        seeds = [b"zk_program_info".as_ref(), program_id.key().as_ref()],
        bump = zk_program_info.bump
    )]
    pub zk_program_info: Account<'info, ZKProgramInfo>,
    
    /// The program ID to query
    pub program_id: UncheckedAccount<'info>,
}

/// Instruction context for verifying ZK program registration
#[derive(Accounts)]
pub struct VerifyZKProgram<'info> {
    /// The program state account
    #[account(
        seeds = [b"registry_state".as_ref()],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    /// The ZK program info account
    #[account(
        mut,
        seeds = [b"zk_program_info".as_ref(), program_id.key().as_ref()],
        bump = zk_program_info.bump
    )]
    pub zk_program_info: Account<'info, ZKProgramInfo>,
    
    /// The program ID to verify
    pub program_id: UncheckedAccount<'info>,
} 