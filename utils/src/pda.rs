// PDA derivation utilities for Valence Solana programs
use anchor_lang::prelude::*;

/// PDA derivation utilities
pub struct PdaDeriver;

impl PdaDeriver {
    /// Derive authorization state PDA
    pub fn derive_authorization_state(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"authorization_state"], program_id)
    }
    
    /// Derive authorization PDA
    pub fn derive_authorization(label: &str, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"authorization", label.as_bytes()], program_id)
    }
    
    /// Derive execution PDA
    pub fn derive_execution(execution_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"execution", &execution_id.to_le_bytes()],
            program_id,
        )
    }
    
    /// Derive processor state PDA
    pub fn derive_processor_state(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"processor_state"], program_id)
    }
    
    /// Derive message batch PDA
    pub fn derive_message_batch(execution_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"message_batch", &execution_id.to_le_bytes()],
            program_id,
        )
    }
    
    /// Derive registry state PDA
    pub fn derive_registry_state(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"registry_state"], program_id)
    }
    
    /// Derive library entry PDA
    pub fn derive_library_entry(library_id: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"library", library_id.as_ref()],
            program_id,
        )
    }
    
    /// Derive vault authority PDA
    pub fn derive_vault_authority(account_key: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"vault", account_key.as_ref()],
            program_id,
        )
    }
    
    /// Derive ZK verifier state PDA
    pub fn derive_zk_verifier_state(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"zk_verifier_state"], program_id)
    }
    
    /// Derive verification key PDA
    pub fn derive_verification_key(registry_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"verification_key", &registry_id.to_le_bytes()],
            program_id,
        )
    }
    
    /// Derive SMT state PDA
    pub fn derive_smt_state(tree_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"smt_state", &tree_id.to_le_bytes()],
            program_id,
        )
    }
} 