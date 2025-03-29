use anchor_lang::prelude::*;

#[account]
pub struct SingleUseAccount {
    /// The authority (owner) of the single-use account
    pub authority: Pubkey,
    /// The authorization token used to validate operations
    pub auth_token: Pubkey,
    /// Set of approved library addresses that can be used with this account
    pub approved_libraries: Vec<Pubkey>,
    /// Number of token accounts managed by this single-use account
    pub token_account_count: u32,
    /// Total number of instructions executed
    pub instruction_count: u64,
    /// Timestamp of the last activity
    pub last_activity: i64,
    /// Whether the account has been used
    pub was_used: bool,
    /// Required destination for funds (if any)
    pub required_destination: Option<Pubkey>,
    /// Expiration timestamp after which the account can be recovered
    pub expiration_time: Option<i64>,
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl SingleUseAccount {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // auth_token
        8 + 32 * 10 + // approved_libraries (assuming max 10 libraries)
        4 + // token_account_count
        8 + // instruction_count
        8 + // last_activity
        1 + // was_used
        1 + 32 + // required_destination (Option<Pubkey>)
        1 + 8 + // expiration_time (Option<i64>)
        64; // reserved
    
    pub fn is_library_approved(&self, library: &Pubkey) -> bool {
        self.approved_libraries.contains(library)
    }
    
    pub fn approve_library(&mut self, library: Pubkey) -> Result<()> {
        if !self.approved_libraries.contains(&library) {
            self.approved_libraries.push(library);
        }
        Ok(())
    }
    
    pub fn increment_instruction_count(&mut self) {
        self.instruction_count = self.instruction_count.saturating_add(1);
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn increment_token_account_count(&mut self) {
        self.token_account_count = self.token_account_count.saturating_add(1);
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration_time {
            let current_time = Clock::get().unwrap().unix_timestamp;
            current_time > expiration
        } else {
            false
        }
    }
    
    pub fn mark_as_used(&mut self) {
        self.was_used = true;
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn validate_destination(&self, destination: &Pubkey) -> bool {
        if let Some(required) = self.required_destination {
            required == *destination
        } else {
            true // No required destination, any destination is valid
        }
    }
} 