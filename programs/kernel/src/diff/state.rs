// Diff state definitions

use anchor_lang::prelude::*;

#[account]
pub struct DiffState {
    /// Authority that can control the diff
    pub authority: Pubkey,
    /// Whether the diff is paused
    pub is_paused: bool,
    /// Total number of diffs calculated
    pub total_diffs: u64,
    /// Total number of successful diff applications
    pub total_applied: u64,
    /// Total number of failed diff applications
    pub total_failed: u64,
    /// Total bytes processed
    pub total_bytes_processed: u64,
    /// Average diff size (scaled by 1000 for precision)
    pub average_diff_size_scaled: u64,
    /// Last diff calculation timestamp
    pub last_diff_at: i64,
    /// Maximum diff size allowed
    pub max_diff_size: u32,
    /// Bump seed for PDA
    pub bump: u8,
}

impl DiffState {
    pub const SPACE: usize = 8 +   // discriminator
        32 +  // authority (Pubkey)
        1 +   // is_paused (bool)
        8 +   // total_diffs (u64)
        8 +   // total_applied (u64)
        8 +   // total_failed (u64)
        8 +   // total_bytes_processed (u64)
        8 +   // average_diff_size_scaled (u64)
        8 +   // last_diff_at (i64)
        4 +   // max_diff_size (u32)
        1;    // bump (u8)
        
    /// Update diff statistics
    pub fn update_stats(&mut self, success: bool, diff_size: usize) {
        self.total_diffs += 1;
        
        if success {
            self.total_applied += 1;
        } else {
            self.total_failed += 1;
        }
        
        self.total_bytes_processed += diff_size as u64;
        
        // Calculate average diff size (scaled by 1000 for precision)
        if self.total_diffs > 0 {
            self.average_diff_size_scaled = (self.total_bytes_processed * 1000) / self.total_diffs;
        }
        
        let clock = Clock::get().unwrap_or_default();
        self.last_diff_at = clock.unix_timestamp;
    }
    
    /// Check if diff size is within allowed limits
    pub fn validate_diff_size(&self, diff_size: usize) -> Result<()> {
        if diff_size > self.max_diff_size as usize {
            return Err(crate::DiffError::DiffSizeExceeded.into());
        }
        Ok(())
    }
} 