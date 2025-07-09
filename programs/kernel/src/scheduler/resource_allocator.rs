// Resource allocation across shards

use anchor_lang::prelude::*;
use crate::error::SchedulerError;

/// Manages resource allocation across multiple shards
pub struct ResourceAllocator;

impl ResourceAllocator {
    /// Allocate resources to shards based on requirements
    pub fn allocate_resources(
        shard_requirements: &[ShardResourceRequirement],
        total_available: u64,
    ) -> Result<Vec<ShardResourceAllocation>> {
        let mut allocations = Vec::new();
        let total_requested: u64 = shard_requirements.iter().map(|r| r.requested).sum();
        
        if total_requested > total_available {
            return Err(SchedulerError::ResourceAllocationFailed.into());
        }

        // Simple proportional allocation
        for req in shard_requirements {
            allocations.push(ShardResourceAllocation {
                shard_id: req.shard_id,
                allocated: req.requested, // For now, allocate exactly what's requested
            });
        }

        Ok(allocations)
    }

    /// Check if resources are available for allocation
    pub fn check_availability(required: u64, available: u64) -> bool {
        required <= available
    }
}

/// Resource requirement for a shard
#[derive(Debug, Clone)]
pub struct ShardResourceRequirement {
    pub shard_id: Pubkey,
    pub requested: u64,
}

/// Resource allocation result
#[derive(Debug, Clone)]
pub struct ShardResourceAllocation {
    pub shard_id: Pubkey,
    pub allocated: u64,
} 