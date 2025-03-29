use anchor_lang::prelude::*;

#[account]
pub struct FactoryState {
    /// The authority allowed to register templates and manage the factory
    pub authority: Pubkey,
    /// Total number of templates registered
    pub template_count: u32,
    /// Total number of accounts created
    pub account_count: u64,
    /// Timestamp of the last activity
    pub last_activity: i64,
    /// Whether the factory is paused
    pub is_paused: bool,
    /// Fee in lamports for account creation (if any)
    pub creation_fee: u64,
    /// Fee receiver
    pub fee_receiver: Pubkey,
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl FactoryState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        4 + // template_count
        8 + // account_count
        8 + // last_activity
        1 + // is_paused
        8 + // creation_fee
        32 + // fee_receiver
        64; // reserved
    
    pub fn increment_template_count(&mut self) {
        self.template_count = self.template_count.saturating_add(1);
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn increment_account_count(&mut self) {
        self.account_count = self.account_count.saturating_add(1);
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn increment_account_count_by(&mut self, count: u64) {
        self.account_count = self.account_count.saturating_add(count);
        self.last_activity = Clock::get().unwrap().unix_timestamp;
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Base = 0,
    Storage = 1,
    SingleUse = 2,
}

#[account]
pub struct AccountTemplate {
    /// Unique identifier for the template
    pub template_id: String,
    /// The authority that created the template
    pub authority: Pubkey,
    /// The type of account this template creates
    pub account_type: u8,
    /// The version of this template
    pub version: u32,
    /// Whether this template is active
    pub is_active: bool,
    /// Whether to automatically fund the account with SOL during creation
    pub auto_fund_sol: bool,
    /// Amount of SOL to fund (in lamports)
    pub fund_amount_sol: u64,
    /// Whether to automatically create any token accounts
    pub create_token_accounts: bool,
    /// Array of token mints to create accounts for
    pub token_mints: Vec<Pubkey>,
    /// Whether to approve any libraries by default
    pub approve_libraries: bool,
    /// Array of pre-approved library addresses
    pub approved_libraries: Vec<Pubkey>,
    /// For Single-Use accounts: required destination
    pub required_destination: Option<Pubkey>,
    /// For Single-Use accounts: expiration time in seconds
    pub expiration_seconds: Option<u64>,
    /// Description of the template
    pub description: String,
    /// Timestamp of the last update
    pub last_update: i64,
    /// Number of accounts created with this template
    pub usage_count: u64,
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl AccountTemplate {
    pub const BASE_SIZE: usize = 8 + // discriminator
        4 + 32 + // template_id with max 32 chars
        32 + // authority
        1 + // account_type
        4 + // version
        1 + // is_active
        1 + // auto_fund_sol
        8 + // fund_amount_sol
        1 + // create_token_accounts
        4 + // token_mints vector length
        1 + // approve_libraries
        4 + // approved_libraries vector length
        1 + 32 + // required_destination (Option<Pubkey>)
        1 + 8 + // expiration_seconds (Option<u64>)
        4 + 100 + // description with max 100 chars
        8 + // last_update
        8 + // usage_count
        64; // reserved
    
    // Calculate the account size based on the number of token mints and approved libraries
    pub fn size(template_id_len: usize, token_mints_len: usize, approved_libraries_len: usize, description_len: usize) -> usize {
        Self::BASE_SIZE + 
            template_id_len.saturating_sub(32) + // If template ID is longer than 32 chars
            (token_mints_len * 32) + // Size of each token mint
            (approved_libraries_len * 32) + // Size of each approved library
            description_len.saturating_sub(100) // If description is longer than 100 chars
    }
    
    pub fn increment_usage_count(&mut self) {
        self.usage_count = self.usage_count.saturating_add(1);
        self.last_update = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn increment_usage_count_by(&mut self, count: u64) {
        self.usage_count = self.usage_count.saturating_add(count);
        self.last_update = Clock::get().unwrap().unix_timestamp;
    }
} 