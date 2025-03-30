use anchor_lang::prelude::*;

#[account]
pub struct LibraryConfig {
    /// The authority that can update the configuration
    pub authority: Pubkey,
    /// Whether this library is active
    pub is_active: bool,
    /// The processor program ID (if restricted)
    pub processor_program_id: Option<Pubkey>,
    /// Maximum transfer size in tokens (0 for unlimited)
    pub max_transfer_amount: u64,
    /// Maximum batch transfer size (0 for unlimited)
    pub max_batch_size: u8,
    /// Whether to enforce recipient allowlisting
    pub enforce_recipient_allowlist: bool,
    /// List of allowed recipient addresses (if enforce_recipient_allowlist is true)
    pub allowed_recipients: Vec<Pubkey>,
    /// Whether to enforce source allowlisting
    pub enforce_source_allowlist: bool,
    /// List of allowed source addresses (if enforce_source_allowlist is true)
    pub allowed_sources: Vec<Pubkey>,
    /// Whether to enforce token mint allowlisting
    pub enforce_mint_allowlist: bool,
    /// List of allowed token mints (if enforce_mint_allowlist is true)
    pub allowed_mints: Vec<Pubkey>,
    /// Whether to validate that token accounts belong to the provided owner (additional security)
    pub validate_account_ownership: bool,
    /// Whether to enable slippage protection (for future integration with swaps)
    pub enable_slippage_protection: bool,
    /// Default slippage tolerance in basis points (e.g., 100 = 1%)
    pub default_slippage_bps: u16,
    /// Fee in basis points (e.g., 25 = 0.25%)
    pub fee_bps: u16,
    /// Fee collector address
    pub fee_collector: Option<Pubkey>,
    /// Number of transfers executed
    pub transfer_count: u64,
    /// Total volume transferred (raw sum of amounts)
    pub total_volume: u64,
    /// Total fees collected
    pub total_fees_collected: u64,
    /// Last updated timestamp
    pub last_updated: i64,
    /// Reserved for future use
    pub reserved: [u8; 64],
}

impl LibraryConfig {
    pub const BASE_SIZE: usize = 8 + // discriminator
        32 + // authority
        1 + // is_active
        1 + 32 + // processor_program_id Option<Pubkey>
        8 + // max_transfer_amount
        1 + // max_batch_size
        1 + // enforce_recipient_allowlist
        4 + // allowed_recipients vector length
        1 + // enforce_source_allowlist
        4 + // allowed_sources vector length
        1 + // enforce_mint_allowlist
        4 + // allowed_mints vector length
        1 + // validate_account_ownership
        1 + // enable_slippage_protection
        2 + // default_slippage_bps
        2 + // fee_bps
        1 + 32 + // fee_collector Option<Pubkey>
        8 + // transfer_count
        8 + // total_volume
        8 + // total_fees_collected
        8 + // last_updated
        64; // reserved

    pub fn size(
        allowed_recipients_len: usize,
        allowed_sources_len: usize,
        allowed_mints_len: usize,
    ) -> usize {
        Self::BASE_SIZE
            + (allowed_recipients_len * 32) // Size of each Pubkey in allowed_recipients
            + (allowed_sources_len * 32) // Size of each Pubkey in allowed_sources
            + (allowed_mints_len * 32) // Size of each Pubkey in allowed_mints
    }

    pub fn increment_transfer_count(&mut self) {
        self.transfer_count = self.transfer_count.saturating_add(1);
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }

    pub fn add_volume(&mut self, amount: u64) {
        self.total_volume = self.total_volume.saturating_add(amount);
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }

    pub fn add_fees_collected(&mut self, amount: u64) {
        self.total_fees_collected = self.total_fees_collected.saturating_add(amount);
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }
    
    pub fn is_recipient_allowed(&self, recipient: &Pubkey) -> bool {
        if !self.enforce_recipient_allowlist {
            return true;
        }
        self.allowed_recipients.contains(recipient)
    }
    
    pub fn is_source_allowed(&self, source: &Pubkey) -> bool {
        if !self.enforce_source_allowlist {
            return true;
        }
        self.allowed_sources.contains(source)
    }
    
    pub fn is_mint_allowed(&self, mint: &Pubkey) -> bool {
        if !self.enforce_mint_allowlist {
            return true;
        }
        self.allowed_mints.contains(mint)
    }
} 