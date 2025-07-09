// Scheduler state definitions

use anchor_lang::prelude::*;
use crate::error::SchedulerError;

#[account]
pub struct SchedulerState {
    /// Authority that can control the scheduler
    pub authority: Pubkey,
    /// Whether the scheduler is paused
    pub is_paused: bool,
    /// Maximum number of shards that can be registered
    pub max_shards: u16,
    /// Maximum queue size per shard
    pub max_queue_size: u16,
    /// Total number of capabilities scheduled
    pub total_scheduled: u64,
    /// Total number of capabilities processed
    pub total_processed: u64,
    /// Total number of capabilities failed
    pub total_failed: u64,
    /// Number of active shards
    pub active_shard_count: u16,
    /// Current queue depth across all shards
    pub global_queue_depth: u32,
    /// Last queue processing timestamp
    pub last_processed_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl SchedulerState {
    pub const SPACE: usize = 8 +   // discriminator
        32 +  // authority (Pubkey)
        1 +   // is_paused (bool)
        2 +   // max_shards (u16)
        2 +   // max_queue_size (u16)
        8 +   // total_scheduled (u64)
        8 +   // total_processed (u64)
        8 +   // total_failed (u64)
        2 +   // active_shard_count (u16)
        4 +   // global_queue_depth (u32)
        8 +   // last_processed_at (i64)
        1;    // bump (u8)
        
    /// Update scheduling statistics
    pub fn update_scheduled(&mut self, count: u32) {
        self.total_scheduled += count as u64;
        self.global_queue_depth += count;
    }
    
    /// Update processing statistics
    pub fn update_processed(&mut self, success: bool) {
        if success {
            self.total_processed += 1;
        } else {
            self.total_failed += 1;
        }
        
        if self.global_queue_depth > 0 {
            self.global_queue_depth -= 1;
        }
        
        let clock = Clock::get().unwrap_or_default();
        self.last_processed_at = clock.unix_timestamp;
    }
    
    /// Register a new shard
    pub fn register_shard(&mut self) -> Result<()> {
        if self.active_shard_count >= self.max_shards {
            return Err(SchedulerError::MaxCapacityReached.into());
        }
        self.active_shard_count += 1;
        Ok(())
    }
    
    /// Unregister a shard
    pub fn unregister_shard(&mut self) {
        if self.active_shard_count > 0 {
            self.active_shard_count -= 1;
        }
    }
} 