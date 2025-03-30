use anchor_lang::prelude::*;
use std::collections::HashSet;
use crate::error::BaseAccountError;

/// Base Account state - holds asset information and approved libraries
#[account]
pub struct AccountState {
    /// Account owner
    pub owner: Pubkey,
    /// Approved library programs that can execute on this account
    pub approved_libraries: Vec<Pubkey>,
    /// PDA for token custody
    pub vault_authority: Pubkey,
    /// PDA bump seed for vault derivation
    pub vault_bump_seed: u8,
    /// Associated token accounts owned by this account
    pub token_accounts: Vec<Pubkey>,
    /// Number of instructions executed
    pub instruction_count: u64,
    /// Timestamp of the last activity
    pub last_activity: i64,
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            owner: Pubkey::default(),
            approved_libraries: Vec::default(),
            vault_authority: Pubkey::default(),
            vault_bump_seed: 0,
            token_accounts: Vec::default(),
            instruction_count: 0,
            last_activity: 0,
            reserved: [0; 64],
        }
    }
}

/// Approval nonce for one-time operations
#[account]
#[derive(Default)]
pub struct ApprovalNonce {
    /// Library program ID that's approved
    pub library: Pubkey,
    /// Unique approval nonce
    pub nonce: u64,
    /// The owner who created this approval
    pub owner: Pubkey,
    /// When this approval expires
    pub expiration: i64,
    /// Whether this approval has been used
    pub is_used: bool,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl AccountState {
    /// Calculate space needed for account creation
    pub fn get_space(max_libraries: usize, max_token_accounts: usize) -> usize {
        // Base size: owner + vault_authority + vault_bump_seed + instruction_count + last_activity + reserved
        let base_size = 32 + 32 + 1 + 8 + 8 + 64;
        
        // Add space for vectors
        let libraries_size = 4 + (max_libraries * 32); // 4 bytes for length + 32 bytes per pubkey
        let token_accounts_size = 4 + (max_token_accounts * 32); // 4 bytes for length + 32 bytes per pubkey
        
        base_size + libraries_size + token_accounts_size
    }
    
    /// Check if a library is approved to operate on this account
    pub fn is_library_approved(&self, library: &Pubkey) -> bool {
        self.approved_libraries.contains(library)
    }
    
    /// Add a library to the approved list if not already present
    pub fn approve_library(&mut self, library: Pubkey) -> Result<()> {
        if self.is_library_approved(&library) {
            return Err(BaseAccountError::LibraryAlreadyApproved.into());
        }
        
        self.approved_libraries.push(library);
        Ok(())
    }
    
    /// Remove a library from the approved list
    pub fn remove_approved_library(&mut self, library: &Pubkey) -> Result<()> {
        if !self.is_library_approved(library) {
            return Err(BaseAccountError::LibraryNotApproved.into());
        }
        
        self.approved_libraries.retain(|&x| x != *library);
        Ok(())
    }
    
    /// Record a token account
    pub fn add_token_account(&mut self, token_account: Pubkey) -> Result<()> {
        if !self.token_accounts.contains(&token_account) {
            self.token_accounts.push(token_account);
        }
        Ok(())
    }
    
    /// Remove a token account from tracking
    pub fn remove_token_account(&mut self, token_account: &Pubkey) -> Result<()> {
        if let Some(index) = self.token_accounts.iter().position(|x| x == token_account) {
            self.token_accounts.remove(index);
        }
        Ok(())
    }
    
    /// Update activity tracking after an instruction
    pub fn record_instruction_execution(&mut self) {
        self.instruction_count += 1;
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
}

impl ApprovalNonce {
    pub const SPACE: usize = 32 + 8 + 32 + 8 + 1 + 1; // library + nonce + owner + expiration + is_used + bump
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