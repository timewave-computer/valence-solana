// Processor state definitions

use anchor_lang::prelude::*;

#[account]
pub struct ProcessorState {
    /// Authority that can control the processor
    pub authority: Pubkey,
    /// Whether the processor is currently paused
    pub is_paused: bool,
    /// Total number of capability executions processed
    pub total_executions: u64,
    /// Total number of successful executions
    pub total_successes: u64,
    /// Total number of failed executions
    pub total_failures: u64,
    /// Total gas consumed across all executions
    pub total_gas_consumed: u64,
    /// Average gas per execution (scaled by 1000 for precision)
    pub average_gas_scaled: u64,
    /// Last execution timestamp
    pub last_execution_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl ProcessorState {
    pub const SPACE: usize = 8 +   // discriminator
        32 +  // authority (Pubkey)
        1 +   // is_paused (bool)
        8 +   // total_executions (u64)
        8 +   // total_successes (u64)
        8 +   // total_failures (u64)
        8 +   // total_gas_consumed (u64)
        8 +   // average_gas_scaled (u64)
        8 +   // last_execution_at (i64)
        1;    // bump (u8)
        
    /// Update execution statistics
    pub fn update_stats(&mut self, success: bool, gas_used: u64) {
        self.total_executions += 1;
        
        if success {
            self.total_successes += 1;
        } else {
            self.total_failures += 1;
        }
        
        self.total_gas_consumed += gas_used;
        
        // Calculate average gas (scaled by 1000 for precision)
        if self.total_executions > 0 {
            self.average_gas_scaled = (self.total_gas_consumed * 1000) / self.total_executions;
        }
        
        let clock = Clock::get().unwrap_or_default();
        self.last_execution_at = clock.unix_timestamp;
    }
} 