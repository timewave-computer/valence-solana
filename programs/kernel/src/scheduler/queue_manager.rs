// Execution queue management

use anchor_lang::prelude::*;
use crate::SchedulerError;
use std::collections::{VecDeque, HashMap};

/// Manages execution queues across multiple shards
pub struct QueueManager {
    /// Per-shard queues
    queues: HashMap<Pubkey, VecDeque<QueueItem>>,
    /// Global priority queue
    priority_queue: VecDeque<QueueItem>,
    /// Processing items
    processing: HashMap<String, QueueItem>,
}

impl Default for QueueManager {
    fn default() -> Self {
        Self {
            queues: HashMap::new(),
            priority_queue: VecDeque::new(),
            processing: HashMap::new(),
        }
    }
}

impl QueueManager {
    /// Create a new queue manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add capabilities to execution queue
    pub fn enqueue_capabilities(
        &mut self,
        shard_id: Pubkey,
        capabilities: &[String],
        priority: u8,
    ) -> Result<()> {
        if capabilities.is_empty() {
            return Err(SchedulerError::InvalidOrderingConstraint.into());
        }

        let clock = Clock::get()?;
        let shard_queue = self.queues.entry(shard_id).or_insert_with(VecDeque::new);

        for capability_id in capabilities {
            let item = QueueItem {
                shard_id,
                capability_id: capability_id.clone(),
                priority,
                timestamp: clock.unix_timestamp,
                status: QueueItemStatus::Pending,
            };

            // Add to shard queue
            shard_queue.push_back(item.clone());

            // Add to priority queue if high priority
            if priority >= 7 {
                self.priority_queue.push_back(item);
            }
        }

        msg!("Enqueued {} capabilities for shard {} with priority {}", 
             capabilities.len(), shard_id, priority);
        Ok(())
    }

    /// Process next item in queue
    pub fn dequeue_next(&mut self) -> Result<Option<QueueItem>> {
        // Check priority queue first
        if let Some(mut item) = self.priority_queue.pop_front() {
            item.status = QueueItemStatus::Processing;
            self.processing.insert(item.capability_id.clone(), item.clone());
            return Ok(Some(item));
        }

        // Then check shard queues
        for (_, queue) in self.queues.iter_mut() {
            if let Some(mut item) = queue.pop_front() {
                item.status = QueueItemStatus::Processing;
                self.processing.insert(item.capability_id.clone(), item.clone());
                return Ok(Some(item));
            }
        }

        Ok(None)
    }

    /// Mark item as completed
    pub fn mark_completed(&mut self, capability_id: &str) -> Result<()> {
        self.processing.remove(capability_id);
        msg!("Marked capability {} as completed", capability_id);
        Ok(())
    }

    /// Mark item as failed
    pub fn mark_failed(&mut self, capability_id: &str) -> Result<()> {
        if let Some(mut item) = self.processing.remove(capability_id) {
            item.status = QueueItemStatus::Failed;
            // Re-queue with lower priority
            if item.priority > 0 {
                item.priority -= 1;
                let shard_queue = self.queues.entry(item.shard_id).or_insert_with(VecDeque::new);
                shard_queue.push_back(item);
            }
        }
        msg!("Marked capability {} as failed", capability_id);
        Ok(())
    }

    /// Get queue status
    pub fn get_queue_status(&self) -> QueueStatus {
        let total_items: usize = self.queues.values().map(|q| q.len()).sum();
        let processing_items = self.processing.len();
        let failed_items = self.queues.values()
            .flat_map(|q| q.iter())
            .filter(|item| item.status == QueueItemStatus::Failed)
            .count();

        QueueStatus {
            total_items: total_items as u64,
            processing_items: processing_items as u64,
            failed_items: failed_items as u64,
        }
    }

    /// Clear queue for a specific shard
    pub fn clear_shard_queue(&mut self, shard_id: Pubkey) -> Result<()> {
        self.queues.remove(&shard_id);
        
        // Remove from priority queue
        self.priority_queue.retain(|item| item.shard_id != shard_id);
        
        // Remove from processing
        self.processing.retain(|_, item| item.shard_id != shard_id);
        
        msg!("Cleared queue for shard {}", shard_id);
        Ok(())
    }
}

/// Item in the execution queue
#[derive(Debug, Clone)]
pub struct QueueItem {
    pub shard_id: Pubkey,
    pub capability_id: String,
    pub priority: u8,
    pub timestamp: i64,
    pub status: QueueItemStatus,
}

/// Status of a queue item
#[derive(Debug, Clone, PartialEq)]
pub enum QueueItemStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Queue status information
#[derive(Debug)]
pub struct QueueStatus {
    pub total_items: u64,
    pub processing_items: u64,
    pub failed_items: u64,
} 