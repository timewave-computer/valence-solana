// Standardized PDA derivation patterns for Valence Protocol
use anchor_lang::prelude::*;

/// Standard PDA seeds for the authorization program
pub struct AuthorizationSeeds;

impl AuthorizationSeeds {
    /// Authorization state PDA seed
    pub const AUTHORIZATION_STATE: &'static [u8] = b"authorization_state";
    
    /// Individual authorization PDA seed
    pub const AUTHORIZATION: &'static [u8] = b"authorization";
    
    /// Execution tracking PDA seed
    pub const EXECUTION: &'static [u8] = b"execution";
    
    /// ZK registry PDA seed
    pub const ZK_REGISTRY: &'static [u8] = b"zk_registry";
    
    /// Replay protection PDA seed
    pub const REPLAY_PROTECTION: &'static [u8] = b"replay_protection";
}

/// PDA derivation helpers for authorization program
impl AuthorizationSeeds {
    /// Derive authorization state PDA
    pub fn authorization_state(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::AUTHORIZATION_STATE],
            program_id
        )
    }
    
    /// Derive authorization PDA from label
    pub fn authorization(label: &str, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::AUTHORIZATION, label.as_bytes()],
            program_id
        )
    }
    
    /// Derive execution PDA from execution ID
    pub fn execution(execution_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::EXECUTION, &execution_id.to_le_bytes()],
            program_id
        )
    }
    
    /// Derive ZK registry PDA from program ID
    pub fn zk_registry(zk_program_id: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::ZK_REGISTRY, zk_program_id.as_ref()],
            program_id
        )
    }
    
    /// Derive replay protection PDA from message hash
    pub fn replay_protection(message_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::REPLAY_PROTECTION, message_hash.as_ref()],
            program_id
        )
    }
}

/// Standard PDA seeds for the base account program
pub struct BaseAccountSeeds;

impl BaseAccountSeeds {
    /// Base account PDA seed
    pub const BASE_ACCOUNT: &'static [u8] = b"base_account";
    
    /// Vault authority PDA seed
    pub const VAULT: &'static [u8] = b"vault";
    
    /// Approval nonce PDA seed
    pub const APPROVAL_NONCE: &'static [u8] = b"approval_nonce";
}

/// PDA derivation helpers for base account program
impl BaseAccountSeeds {
    /// Derive base account PDA from owner
    pub fn base_account(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::BASE_ACCOUNT, owner.as_ref()],
            program_id
        )
    }
    
    /// Derive vault authority PDA from base account
    pub fn vault_authority(base_account: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::VAULT, base_account.as_ref()],
            program_id
        )
    }
    
    /// Derive approval nonce PDA
    pub fn approval_nonce(owner: &Pubkey, nonce: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::APPROVAL_NONCE, owner.as_ref(), &nonce.to_le_bytes()],
            program_id
        )
    }
}

/// Standard PDA seeds for the storage account program
pub struct StorageAccountSeeds;

impl StorageAccountSeeds {
    /// Storage account PDA seed
    pub const STORAGE_ACCOUNT: &'static [u8] = b"storage_account";
    
    /// Storage authority PDA seed
    pub const STORAGE_AUTHORITY: &'static [u8] = b"storage_authority";
    
    /// Storage item PDA seed
    pub const STORAGE_ITEM: &'static [u8] = b"storage_item";
}

/// PDA derivation helpers for storage account program
impl StorageAccountSeeds {
    /// Derive storage account PDA from owner
    pub fn storage_account(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::STORAGE_ACCOUNT, owner.as_ref()],
            program_id
        )
    }
    
    /// Derive storage authority PDA from storage account
    pub fn storage_authority(storage_account: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::STORAGE_AUTHORITY, storage_account.as_ref()],
            program_id
        )
    }
    
    /// Derive storage item PDA from storage account and key
    pub fn storage_item(storage_account: &Pubkey, key: &str, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::STORAGE_ITEM, storage_account.as_ref(), key.as_bytes()],
            program_id
        )
    }
}

/// Standard PDA seeds for the processor program
pub struct ProcessorSeeds;

impl ProcessorSeeds {
    /// Processor state PDA seed
    pub const PROCESSOR_STATE: &'static [u8] = b"processor_state";
    
    /// Message batch PDA seed
    pub const MESSAGE_BATCH: &'static [u8] = b"message_batch";
    
    /// Pending callback PDA seed
    pub const PENDING_CALLBACK: &'static [u8] = b"pending_callback";
}

/// PDA derivation helpers for processor program
impl ProcessorSeeds {
    /// Derive processor state PDA
    pub fn processor_state(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::PROCESSOR_STATE],
            program_id
        )
    }
    
    /// Derive message batch PDA from execution ID
    pub fn message_batch(execution_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::MESSAGE_BATCH, &execution_id.to_le_bytes()],
            program_id
        )
    }
    
    /// Derive pending callback PDA from execution ID
    pub fn pending_callback(execution_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::PENDING_CALLBACK, &execution_id.to_le_bytes()],
            program_id
        )
    }
}

/// Standard PDA seeds for the ZK verifier program
pub struct ZKVerifierSeeds;

impl ZKVerifierSeeds {
    /// Verification key PDA seed
    pub const VERIFICATION_KEY: &'static [u8] = b"verification_key";
    
    /// SMT state PDA seed
    pub const SMT_STATE: &'static [u8] = b"smt_state";
    
    /// SMT leaf PDA seed
    pub const SMT_LEAF: &'static [u8] = b"smt_leaf";
}

/// PDA derivation helpers for ZK verifier program
impl ZKVerifierSeeds {
    /// Derive verification key PDA from key ID
    pub fn verification_key(key_id: &str, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::VERIFICATION_KEY, key_id.as_bytes()],
            program_id
        )
    }
    
    /// Derive SMT state PDA from owner
    pub fn smt_state(owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SMT_STATE, owner.as_ref()],
            program_id
        )
    }
    
    /// Derive SMT leaf PDA from SMT state and key
    pub fn smt_leaf(smt_state: &Pubkey, key: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SMT_LEAF, smt_state.as_ref(), key.as_ref()],
            program_id
        )
    }
} 