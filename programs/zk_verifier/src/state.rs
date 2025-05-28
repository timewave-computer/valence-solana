// State structures for ZK Verifier Program

use anchor_lang::prelude::*;

/// Type of verification key
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum VerificationKeyType {
    /// SP1 proof system verification key
    SP1,
    /// Groth16 verification key
    Groth16,
    /// PLONK verification key
    PLONK,
}

/// Global verifier state
#[account]
pub struct VerifierState {
    /// Owner of the verifier
    pub owner: Pubkey,
    /// Root hash of the ZK coprocessor (like Solidity implementation)
    pub coprocessor_root: [u8; 32],
    /// Generic verifier address
    pub verifier: Pubkey,
    /// Total number of registered verification keys
    pub total_keys: u64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Verification key storage - maps program_id + registry_id to verification key
#[account]
pub struct VerificationKey {
    /// Program ID that owns this verification key
    pub program_id: Pubkey,
    /// Registry ID for this verification key
    pub registry_id: u64,
    /// The verification key data (32 bytes hash like Solidity)
    pub vk_hash: [u8; 32],
    /// Type of verification key
    pub key_type: VerificationKeyType,
    /// Whether this key is active
    pub is_active: bool,
    /// Bump seed for PDA
    pub bump: u8,
}

impl VerifierState {
    pub const SPACE: usize = 8 + 32 + 32 + 32 + 8 + 1; // discriminator + owner + coprocessor_root + verifier + total_keys + bump
}

impl VerificationKey {
    pub const SPACE: usize = 8 + 32 + 8 + 32 + 1 + 1 + 1; // discriminator + program_id + registry_id + vk_hash + key_type + is_active + bump
}





 