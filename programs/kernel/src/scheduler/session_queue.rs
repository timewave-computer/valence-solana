// Session queue management - moved from sessions/lifecycle.rs
// This module handles queuing and batch processing of session operations

use anchor_lang::prelude::*;
use crate::error::SchedulerError;

/// Session operation queue for batch processing
#[account]
pub struct SessionOperationQueue {
    /// Queue authority
    pub authority: Pubkey,
    /// Pending operations
    pub pending_operations: Vec<PendingOperation>,
    /// Maximum queue size
    pub max_queue_size: u16,
    /// Total processed
    pub total_processed: u64,
    /// Total failed
    pub total_failed: u64,
    /// Queue creation timestamp
    pub created_at: i64,
    /// Last processed timestamp
    pub last_processed_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl SessionOperationQueue {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        4 + (100 * std::mem::size_of::<PendingOperation>()) + // pending_operations (max 100)
        2 + // max_queue_size
        8 + // total_processed
        8 + // total_failed
        8 + // created_at
        8 + // last_processed_at
        1; // bump
}

/// Pending operation in the queue
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PendingOperation {
    /// Operation type
    pub operation_type: OperationType,
    /// Target account (e.g., session PDA)
    pub target_account: Pubkey,
    /// Operation parameters
    pub params: Vec<u8>,
    /// Queued by
    pub queued_by: Pubkey,
    /// Queued at timestamp
    pub queued_at: i64,
    /// Execution deadline
    pub deadline: i64,
    /// Priority level (0-255)
    pub priority: u8,
}

/// Types of operations that can be queued
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OperationType {
    SessionInit,
    SessionActivation,
    SessionPause,
    SessionResume,
    SessionClose,
    CapabilityExecution,
}

/// Queue manager for processing operations
pub struct QueueManager;

impl QueueManager {
    /// Add operation to queue
    pub fn enqueue_operation(
        queue: &mut SessionOperationQueue,
        operation: PendingOperation,
    ) -> Result<()> {
        // Check queue capacity
        if queue.pending_operations.len() >= queue.max_queue_size as usize {
            return Err(SchedulerError::QueueFull.into());
        }

        // Add to queue
        queue.pending_operations.push(operation);
        
        Ok(())
    }

    /// Process operations from queue
    pub fn process_batch(
        queue: &mut SessionOperationQueue,
        max_batch_size: usize,
    ) -> Result<Vec<PendingOperation>> {
        let mut processed = Vec::new();
        let clock = Clock::get()?;
        
        // Sort by priority (higher first) then by queue time (FIFO)
        queue.pending_operations.sort_by(|a, b| {
            match b.priority.cmp(&a.priority) {
                std::cmp::Ordering::Equal => a.queued_at.cmp(&b.queued_at),
                other => other,
            }
        });
        
        // Process up to max_batch_size operations
        let idx = 0;
        while idx < queue.pending_operations.len() && processed.len() < max_batch_size {
            let op = &queue.pending_operations[idx];
            
            // Check if operation is still valid (not past deadline)
            if op.deadline < clock.unix_timestamp {
                // Remove expired operation
                queue.pending_operations.remove(idx);
                queue.total_failed += 1;
                // Don't increment idx since we removed an element
                continue;
            }
            
            // Add to processed batch
            processed.push(queue.pending_operations.remove(idx));
            // Don't increment idx since we removed an element
        }
        
        // Update queue stats
        queue.total_processed += processed.len() as u64;
        queue.last_processed_at = clock.unix_timestamp;
        
        Ok(processed)
    }

    /// Get queue statistics
    pub fn get_queue_stats(queue: &SessionOperationQueue) -> QueueStats {
        let mut type_counts = std::collections::HashMap::new();
        
        for op in &queue.pending_operations {
            *type_counts.entry(op.operation_type.clone()).or_insert(0) += 1;
        }
        
        QueueStats {
            pending_count: queue.pending_operations.len(),
            total_processed: queue.total_processed,
            total_failed: queue.total_failed,
            operation_type_counts: type_counts,
            oldest_operation_age: queue.pending_operations
                .iter()
                .map(|op| Clock::get().map(|c| c.unix_timestamp - op.queued_at).unwrap_or(0))
                .max()
                .unwrap_or(0),
        }
    }
}

/// Queue statistics
#[derive(Debug)]
pub struct QueueStats {
    pub pending_count: usize,
    pub total_processed: u64,
    pub total_failed: u64,
    pub operation_type_counts: std::collections::HashMap<OperationType, usize>,
    pub oldest_operation_age: i64,
}