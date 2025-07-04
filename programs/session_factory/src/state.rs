use anchor_lang::prelude::*;

/// Factory state for managing session creation
#[account]
pub struct FactoryState {
    /// The owner/authority of the factory
    pub owner: Pubkey,
    /// Total number of sessions created
    pub total_sessions_created: u64,
    /// PDA bump seed
    pub bump: u8,
}

impl FactoryState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // owner
        8 + // total_sessions_created
        1; // bump
}

/// Valence Session - holds assets and executes capability-verified operations
#[account]
pub struct Session {
    /// Session owner
    pub owner: Pubkey,
    /// The Eval program that has exclusive permission to execute calls
    pub eval_program_id: Pubkey,
    /// Namespaces this session belongs to
    pub namespaces: Vec<[u8; 32]>,
    /// Nonce for unique operations
    pub nonce: u64,
    /// Creation timestamp
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Whether the session is active
    pub is_active: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl Session {
    /// Calculate space needed for session creation
    pub fn get_space(namespace_count: usize) -> usize {
        8 + // discriminator
        32 + // owner
        32 + // eval_program_id
        4 + (namespace_count * 32) + // namespaces vec
        8 + // nonce
        8 + // created_at
        8 + // last_activity
        1 + // is_active
        1 // bump
    }
    
    /// Check if session belongs to a namespace
    pub fn has_namespace(&self, namespace: &[u8; 32]) -> bool {
        self.namespaces.contains(namespace)
    }
    
    /// Update activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
    
    /// Increment nonce
    pub fn increment_nonce(&mut self) -> u64 {
        self.nonce = self.nonce.saturating_add(1);
        self.update_activity();
        self.nonce
    }
} 