// State structures for ZK Proof Verifier Program

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
    /// SMT (Sparse Merkle Tree) verification key
    SMT,
}

/// SMT (Sparse Merkle Tree) node for efficient state commitments
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct SMTNode {
    /// Hash of the node
    pub hash: [u8; 32],
    /// Left child hash (empty for leaf nodes)
    pub left: Option<[u8; 32]>,
    /// Right child hash (empty for leaf nodes)
    pub right: Option<[u8; 32]>,
    /// Whether this is a leaf node
    pub is_leaf: bool,
}

/// SMT proof for membership/non-membership verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SMTProof {
    /// The key being proven
    pub key: [u8; 32],
    /// The value (for membership proofs)
    pub value: Option<[u8; 32]>,
    /// Merkle path from leaf to root
    pub path: Vec<[u8; 32]>,
    /// Bit vector indicating left/right direction for each path element
    pub directions: Vec<bool>,
    /// Root hash of the SMT
    pub root: [u8; 32],
}

/// SMT state account for managing sparse merkle trees
#[account]
pub struct SMTState {
    /// Owner of the SMT
    pub owner: Pubkey,
    /// Root hash of the SMT
    pub root: [u8; 32],
    /// Height of the SMT (typically 256 for 32-byte keys)
    pub height: u8,
    /// Number of leaves in the SMT
    pub leaf_count: u64,
    /// Last update timestamp
    pub last_updated: i64,
    /// Whether the SMT is frozen (read-only)
    pub is_frozen: bool,
    /// Bump seed for PDA
    pub bump: u8,
}

impl SMTState {
    pub const SPACE: usize = 32 + 32 + 1 + 8 + 8 + 1 + 1; // owner + root + height + leaf_count + last_updated + is_frozen + bump
}

/// SMT leaf node account
#[account]
pub struct SMTLeaf {
    /// The key for this leaf
    pub key: [u8; 32],
    /// The value for this leaf
    pub value: [u8; 32],
    /// Hash of the leaf (hash(key, value))
    pub leaf_hash: [u8; 32],
    /// Timestamp when leaf was created/updated
    pub timestamp: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl SMTLeaf {
    pub const SPACE: usize = 32 + 32 + 32 + 8 + 1; // key + value + leaf_hash + timestamp + bump
    
    /// Calculate the hash of this leaf
    pub fn calculate_hash(&self) -> [u8; 32] {
        use anchor_lang::solana_program::hash::{hash, Hasher};
        let mut hasher = Hasher::default();
        hasher.hash(&self.key);
        hasher.hash(&self.value);
        hasher.result().to_bytes()
    }
    
    /// Verify the leaf hash is correct
    pub fn verify_hash(&self) -> bool {
        self.leaf_hash == self.calculate_hash()
    }
}

/// ZK Verifier Program state
#[account]
pub struct VerifierState {
    /// Program owner
    pub owner: Pubkey,
    /// Total number of registered verification keys
    pub total_keys: u64,
    /// Total number of successful verifications
    pub successful_verifications: u64,
    /// Total number of failed verifications
    pub failed_verifications: u64,
    /// Whether the verifier is paused
    pub is_paused: bool,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Verification key registry entry
#[account]
pub struct VerificationKey {
    /// Program ID this key is for
    pub program_id: Pubkey,
    /// The verification key data
    pub key_data: Vec<u8>,
    /// Type of verification key
    pub key_type: VerificationKeyType,
    /// Whether this key is active
    pub is_active: bool,
    /// Timestamp when key was registered
    pub registered_at: i64,
    /// Timestamp when key was last updated
    pub updated_at: i64,
    /// Number of successful verifications with this key
    pub verification_count: u64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Instruction context for initializing the verifier
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The verifier state account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<VerifierState>(),
        seeds = [b"verifier_state".as_ref()],
        bump
    )]
    pub verifier_state: Account<'info, VerifierState>,
    
    /// The account paying for the initialization
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for registering a verification key
#[derive(Accounts)]
#[instruction(program_id: Pubkey, verification_key: Vec<u8>)]
pub struct RegisterVerificationKey<'info> {
    /// The verifier state account
    #[account(
        mut,
        seeds = [b"verifier_state".as_ref()],
        bump = verifier_state.bump,
        constraint = verifier_state.owner == owner.key() @ crate::error::VerifierError::NotAuthorized,
        constraint = !verifier_state.is_paused @ crate::error::VerifierError::VerifierPaused,
    )]
    pub verifier_state: Account<'info, VerifierState>,
    
    /// The verification key account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<VerificationKey>() + verification_key.len() + 64, // Extra space
        seeds = [b"verification_key".as_ref(), program_id.as_ref()],
        bump
    )]
    pub verification_key_account: Account<'info, VerificationKey>,
    
    /// The owner of the verifier
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for verifying a proof
#[derive(Accounts)]
#[instruction(program_id: Pubkey)]
pub struct VerifyProof<'info> {
    /// The verifier state account
    #[account(
        mut,
        seeds = [b"verifier_state".as_ref()],
        bump = verifier_state.bump,
        constraint = !verifier_state.is_paused @ crate::error::VerifierError::VerifierPaused,
    )]
    pub verifier_state: Account<'info, VerifierState>,
    
    /// The verification key account
    #[account(
        mut,
        seeds = [b"verification_key".as_ref(), program_id.as_ref()],
        bump = verification_key_account.bump,
        constraint = verification_key_account.is_active @ crate::error::VerifierError::VerificationKeyInactive,
    )]
    pub verification_key_account: Account<'info, VerificationKey>,
    
    /// The account requesting verification (pays for compute)
    #[account(mut)]
    pub verifier: Signer<'info>,
}

/// Instruction context for updating a verification key
#[derive(Accounts)]
pub struct UpdateVerificationKey<'info> {
    /// The verifier state account
    #[account(
        seeds = [b"verifier_state".as_ref()],
        bump = verifier_state.bump,
        constraint = verifier_state.owner == owner.key() @ crate::error::VerifierError::NotAuthorized,
    )]
    pub verifier_state: Account<'info, VerifierState>,
    
    /// The verification key account
    #[account(
        mut,
        seeds = [b"verification_key".as_ref(), verification_key_account.program_id.as_ref()],
        bump = verification_key_account.bump
    )]
    pub verification_key_account: Account<'info, VerificationKey>,
    
    /// The owner of the verifier
    pub owner: Signer<'info>,
} 