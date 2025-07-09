// Priority scheduling engine

use anchor_lang::prelude::*;

/// Manages priority-based scheduling
pub struct PriorityEngine;

impl PriorityEngine {
    /// Calculate priority for capability execution
    pub fn calculate_priority(
        _capability_id: &str,
        shard_load: u64,
        base_priority: u8,
    ) -> u8 {
        // Simple priority calculation (to be enhanced)
        let load_factor = if shard_load > 100 { 0 } else { 1 };
        base_priority.saturating_add(load_factor)
    }

    /// Sort capabilities by priority
    pub fn sort_by_priority(mut capabilities: Vec<PriorityCapability>) -> Vec<PriorityCapability> {
        capabilities.sort_by(|a, b| b.priority.cmp(&a.priority));
        capabilities
    }

    /// Get next high-priority capability
    pub fn get_next_high_priority(capabilities: &[PriorityCapability]) -> Option<&PriorityCapability> {
        capabilities.iter().max_by_key(|c| c.priority)
    }
}

/// Capability with priority information
#[derive(Debug, Clone)]
pub struct PriorityCapability {
    pub capability_id: String,
    pub shard_id: Pubkey,
    pub priority: u8,
    pub timestamp: i64,
} 