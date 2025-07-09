/// Address derivation utilities and PDA helpers for Valence Protocol
/// 
/// This module provides utilities for deriving addresses and PDAs
/// used by the Valence Protocol programs.

use solana_sdk::pubkey::Pubkey;


/// Find the session execution PDA
pub fn find_execution_pda(program_id: &Pubkey, session: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"session_execution", session.as_ref()],
        program_id,
    )
}

/// Find the shard state PDA
pub fn find_shard_state_pda(program_id: &Pubkey, shard_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"shard", shard_program_id.as_ref()],
        program_id,
    )
}

/// Find the capability PDA
pub fn find_capability_pda(program_id: &Pubkey, shard: &Pubkey, capability_id: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"capability",
            shard.as_ref(),
            capability_id.as_bytes(),
        ],
        program_id,
    )
}

/// Find the session registration PDA
pub fn find_session_registration_pda(program_id: &Pubkey, shard: &Pubkey, session: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"session_registration",
            shard.as_ref(),
            session.as_ref(),
        ],
        program_id,
    )
}

/// Find the namespace PDA
pub fn find_namespace_pda(program_id: &Pubkey, shard: &Pubkey, namespace: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"namespace",
            shard.as_ref(),
            namespace.as_bytes(),
        ],
        program_id,
    )
}

/// Find the execution record PDA
pub fn find_execution_record_pda(program_id: &Pubkey, shard: &Pubkey, execution_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"execution_record",
            shard.as_ref(),
            &execution_id.to_le_bytes(),
        ],
        program_id,
    )
}

/// Find the registry state PDA
pub fn find_registry_state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"registry_state"],
        program_id,
    )
}

/// Find the library entry PDA
pub fn find_library_pda(program_id: &Pubkey, library_id: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"library",
            library_id.as_bytes(),
        ],
        program_id,
    )
}

/// Find the ZK program entry PDA
pub fn find_zk_program_pda(program_id: &Pubkey, program_id_str: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"zk_program",
            program_id_str.as_bytes(),
        ],
        program_id,
    )
}

/// Find the dependency entry PDA
pub fn find_dependency_pda(program_id: &Pubkey, dependent: &str, dependency: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"dependency",
            dependent.as_bytes(),
            dependency.as_bytes(),
        ],
        program_id,
    )
}

/// Account structures for instruction contexts
/// These match the account structures in the programs
/// 
/// Note: This sub-module is named 'accounts' to describe what it contains (account structures),
/// while the parent module is named 'addresses' to describe its primary purpose (address derivation).

/// Kernel program account contexts
pub mod accounts {
    use super::*;

    /// Shard program account contexts
    #[derive(Debug, Clone)]
    pub struct InitializeShard {
        pub shard_state: Pubkey,
        pub authority: Pubkey,
        pub program_id: Pubkey,
        pub system_program: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct GrantCapability {
        pub authority: Pubkey,
        pub shard_state: Pubkey,
        pub capability: Pubkey,
        pub system_program: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct UpdateCapability {
        pub authority: Pubkey,
        pub shard_state: Pubkey,
        pub capability: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct RevokeCapability {
        pub authority: Pubkey,
        pub shard_state: Pubkey,
        pub capability: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct AddNamespace {
        pub authority: Pubkey,
        pub shard_state: Pubkey,
        pub namespace_account: Pubkey,
        pub system_program: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct RemoveNamespace {
        pub authority: Pubkey,
        pub shard_state: Pubkey,
        pub namespace_account: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct CheckNamespaceAccess {
        pub shard_state: Pubkey,
        pub session_registration: Pubkey,
        pub namespace_account: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct RecordExecution {
        pub caller: Pubkey,
        pub shard_state: Pubkey,
        pub capability: Pubkey,
        pub execution_record: Pubkey,
        pub system_program: Pubkey,
    }

    /// Registry program account contexts
    #[derive(Debug, Clone)]
    pub struct InitializeRegistry {
        pub registry_state: Pubkey,
        pub authority: Pubkey,
        pub system_program: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct RegisterLibrary {
        pub authority: Pubkey,
        pub registry_state: Pubkey,
        pub library_entry: Pubkey,
        pub system_program: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct RegisterZkProgram {
        pub authority: Pubkey,
        pub registry_state: Pubkey,
        pub zk_program_entry: Pubkey,
        pub system_program: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct AddDependency {
        pub authority: Pubkey,
        pub registry_state: Pubkey,
        pub dependent_library: Pubkey,
        pub dependency_library: Pubkey,
        pub dependency_entry: Pubkey,
        pub system_program: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct UpdateLibraryStatus {
        pub authority: Pubkey,
        pub registry_state: Pubkey,
        pub library_entry: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct VerifyZkProgram {
        pub verifier: Pubkey,
        pub registry_state: Pubkey,
        pub zk_program_entry: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct QueryLibrary {
        pub registry_state: Pubkey,
        pub library_entry: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct CheckVersionCompatibility {
        pub registry_state: Pubkey,
        pub library_entry: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct ListLibraries {
        pub registry_state: Pubkey,
    }

    #[derive(Debug, Clone)]
    pub struct ResolveDependencies {
        pub registry_state: Pubkey,
        pub library_entry: Pubkey,
    }
}

/// Helper functions for account validation
impl accounts::InitializeShard {
    pub fn new(program_id: &Pubkey, authority: &Pubkey, shard_program_id: &Pubkey) -> Self {
        let shard_state_pda = find_shard_state_pda(program_id, shard_program_id);
        Self {
            shard_state: shard_state_pda.0,
            authority: *authority,
            program_id: *shard_program_id,
            system_program: solana_sdk::system_program::ID,
        }
    }
}

impl accounts::GrantCapability {
    pub fn new(program_id: &Pubkey, authority: &Pubkey, shard_state: &Pubkey, capability_id: &str) -> Self {
        let capability_pda = find_capability_pda(program_id, shard_state, capability_id);
        Self {
            authority: *authority,
            shard_state: *shard_state,
            capability: capability_pda.0,
            system_program: solana_sdk::system_program::ID,
        }
    }
}

impl accounts::InitializeRegistry {
    pub fn new(program_id: &Pubkey, authority: &Pubkey) -> Self {
        let registry_state_pda = find_registry_state_pda(program_id);
        Self {
            registry_state: registry_state_pda.0,
            authority: *authority,
            system_program: solana_sdk::system_program::ID,
        }
    }
}

impl accounts::RegisterLibrary {
    pub fn new(program_id: &Pubkey, authority: &Pubkey, library_id: &str) -> Self {
        let registry_state_pda = find_registry_state_pda(program_id);
        let library_pda = find_library_pda(program_id, library_id);
        Self {
            authority: *authority,
            registry_state: registry_state_pda.0,
            library_entry: library_pda.0,
            system_program: solana_sdk::system_program::ID,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pda_generation() {
        let program_id = Pubkey::new_unique();
        let shard_id = Pubkey::new_unique();
        let capability_id = "test_capability";
        
        // Test capability PDA
        let (pda1, bump1) = find_capability_pda(&program_id, &shard_id, capability_id);
        let (pda2, bump2) = find_capability_pda(&program_id, &shard_id, capability_id);
        
        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
        
    }
    
    #[test]
    fn test_account_constructors() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let shard_address = Pubkey::new_unique();
        
        // Test InitializeRegistry account constructor
        let registry_accounts = accounts::InitializeRegistry::new(&program_id, &authority);
        assert_eq!(registry_accounts.authority, authority);
        assert_eq!(registry_accounts.system_program, solana_sdk::system_program::ID);
    }
} 