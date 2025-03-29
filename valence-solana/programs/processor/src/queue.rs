use anchor_lang::prelude::*;
use crate::state::{QueueState, Priority};
use crate::error::ProcessorError;

/// Default capacity for queues
pub const DEFAULT_QUEUE_CAPACITY: u64 = 100;

/// Queue manager for handling priority queues
pub struct QueueManager<'a> {
    /// High priority queue
    pub high_priority: &'a mut QueueState,
    /// Medium priority queue
    pub medium_priority: &'a mut QueueState,
    /// Low priority queue
    pub low_priority: &'a mut QueueState,
}

impl<'a> QueueManager<'a> {
    /// Create a new queue manager
    pub fn new(
        high_priority: &'a mut QueueState,
        medium_priority: &'a mut QueueState,
        low_priority: &'a mut QueueState,
    ) -> Self {
        Self {
            high_priority,
            medium_priority,
            low_priority,
        }
    }
    
    /// Enqueue a message batch with the given priority
    pub fn enqueue(&mut self, priority: &Priority) -> Result<u64> {
        match priority {
            Priority::High => self.high_priority.enqueue(),
            Priority::Medium => self.medium_priority.enqueue(),
            Priority::Low => self.low_priority.enqueue(),
        }
    }
    
    /// Dequeue the next message batch based on priority
    pub fn dequeue(&mut self) -> Result<(u64, Priority)> {
        // Try high priority first
        if !self.high_priority.is_empty() {
            return Ok((self.high_priority.dequeue()?, Priority::High));
        }
        
        // Then medium priority
        if !self.medium_priority.is_empty() {
            return Ok((self.medium_priority.dequeue()?, Priority::Medium));
        }
        
        // Finally low priority
        if !self.low_priority.is_empty() {
            return Ok((self.low_priority.dequeue()?, Priority::Low));
        }
        
        // No messages available
        Err(error!(ProcessorError::QueueEmpty))
    }
    
    /// Check if all queues are empty
    pub fn is_empty(&self) -> bool {
        self.high_priority.is_empty() && 
        self.medium_priority.is_empty() && 
        self.low_priority.is_empty()
    }
    
    /// Get the total number of messages in all queues
    pub fn total_count(&self) -> u64 {
        self.high_priority.count + 
        self.medium_priority.count + 
        self.low_priority.count
    }
    
    /// Get the next batch to process without removing it
    pub fn peek_next_batch(&self) -> Option<(u64, Priority)> {
        // Try high priority first
        if !self.high_priority.is_empty() {
            return self.high_priority.next_dequeue_index().map(|idx| (idx, Priority::High));
        }
        
        // Then medium priority
        if !self.medium_priority.is_empty() {
            return self.medium_priority.next_dequeue_index().map(|idx| (idx, Priority::Medium));
        }
        
        // Finally low priority
        if !self.low_priority.is_empty() {
            return self.low_priority.next_dequeue_index().map(|idx| (idx, Priority::Low));
        }
        
        None
    }
    
    /// Initialize new queue states with default capacity
    pub fn initialize_new_queues() -> (QueueState, QueueState, QueueState) {
        (
            QueueState::new(DEFAULT_QUEUE_CAPACITY),
            QueueState::new(DEFAULT_QUEUE_CAPACITY),
            QueueState::new(DEFAULT_QUEUE_CAPACITY),
        )
    }
}

/// Helper function to convert priority to string
pub fn priority_to_string(priority: &Priority) -> &'static str {
    match priority {
        Priority::High => "high",
        Priority::Medium => "medium",
        Priority::Low => "low",
    }
}

/// Helper function to derive message batch PDA
pub fn derive_message_batch_pda(
    execution_id: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"message_batch", execution_id.to_le_bytes().as_ref()],
        program_id,
    )
}

/// Helper function to derive pending callback PDA
pub fn derive_pending_callback_pda(
    execution_id: u64,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"pending_callback", execution_id.to_le_bytes().as_ref()],
        program_id,
    )
} 