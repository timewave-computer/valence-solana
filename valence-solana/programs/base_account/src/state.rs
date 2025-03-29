use anchor_lang::prelude::*;
use std::collections::HashSet;

#[account]
pub struct BaseAccount {
    /// The authority (owner) of the base account
    pub authority: Pubkey,
    /// The authorization token used to validate operations
    pub auth_token: Pubkey,
    /// Set of approved library addresses that can be used with this account
    pub approved_libraries: HashSet<Pubkey>,
    /// Number of token accounts managed by this base account
    pub token_account_count: u32,
    /// Total number of instructions executed
    pub instruction_count: u64,
    /// Timestamp of the last activity
    pub last_activity: i64,
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl BaseAccount {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // auth_token
        32 + // approved_libraries (simplified, actual size varies)
        4 + // token_account_count
        8 + // instruction_count
        8 + // last_activity
        64; // reserved
        
    pub fn is_library_approved(&self, library: &Pubkey) -> bool {
        self.approved_libraries.contains(library)
    }
    
    pub fn approve_library(&mut self, library: Pubkey) -> Result<()> {
        self.approved_libraries.insert(library);
        Ok(())
    }
    
    pub fn increment_instruction_count(&mut self) {
        self.instruction_count = self.instruction_count.saturating_add(1);
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn increment_token_account_count(&mut self) {
        self.token_account_count = self.token_account_count.saturating_add(1);
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct LibraryInfo {
    /// The library address
    pub address: Pubkey,
    /// The timestamp when the library was registered
    pub registered_at: i64,
    /// Flag indicating if the library is approved
    pub is_approved: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecutionStatistics {
    /// Total number of instructions executed
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Timestamp of the last execution
    pub last_execution: i64,
} 