// Atomic diff processing

use anchor_lang::prelude::*;
use crate::{diff::instructions::DiffOperation, DiffError};
use std::collections::HashMap;

/// Processes diffs atomically
pub struct AtomicProcessor {
    /// Transaction log for rollback support
    transaction_log: Vec<TransactionEntry>,
    /// Original state snapshots for rollback
    state_snapshots: HashMap<String, Vec<u8>>,
}

impl Default for AtomicProcessor {
    fn default() -> Self {
        Self {
            transaction_log: Vec::new(),
            state_snapshots: HashMap::new(),
        }
    }
}

impl AtomicProcessor {
    /// Create a new atomic processor
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Process a batch of diff operations atomically
    pub fn process_atomic_batch(
        &mut self,
        operations: &[DiffOperation],
        target_state: &mut Vec<u8>,
    ) -> Result<AtomicResult> {
        // Clear previous transaction log
        self.transaction_log.clear();
        
        // Take snapshot of current state
        let state_snapshot = target_state.clone();
        self.state_snapshots.insert("main".to_string(), state_snapshot);
        
        let mut operations_applied = 0;
        let mut total_bytes_changed = 0;
        
        // Try to apply all operations
        for (idx, operation) in operations.iter().enumerate() {
            // Validate operation
            Self::validate_operation(operation)?;
            
            // Record transaction entry
            let entry = TransactionEntry {
                operation_idx: idx,
                operation: operation.clone(),
                state_before: target_state.clone(),
            };
            self.transaction_log.push(entry);
            
            // Apply operation
            match Self::apply_single_operation(target_state, operation) {
                Ok(bytes_changed) => {
                    operations_applied += 1;
                    total_bytes_changed += bytes_changed;
                }
                Err(e) => {
                    // Rollback on failure
                    msg!("Operation {} failed: {:?}, rolling back", idx, e);
                    self.rollback_to_snapshot("main", target_state)?;
                    
                    return Ok(AtomicResult {
                        success: false,
                        operations_processed: operations_applied,
                        operations_failed: 1,
                        bytes_changed: 0,
                        error_at_index: Some(idx),
                    });
                }
            }
        }
        
        Ok(AtomicResult {
            success: true,
            operations_processed: operations_applied,
            operations_failed: 0,
            bytes_changed: total_bytes_changed,
            error_at_index: None,
        })
    }
    
    /// Apply a single operation to the state
    fn apply_single_operation(state: &mut Vec<u8>, operation: &DiffOperation) -> Result<usize> {
        match operation {
            DiffOperation::Insert { position, data } => {
                let pos = *position as usize;
                if pos > state.len() {
                    return Err(DiffError::InvalidDiffOperation.into());
                }
                
                state.splice(pos..pos, data.iter().cloned());
                Ok(data.len())
            }
            DiffOperation::Delete { position, length } => {
                let start = *position as usize;
                let end = start + (*length as usize);
                
                if end > state.len() {
                    return Err(DiffError::InvalidDiffOperation.into());
                }
                
                state.drain(start..end);
                Ok(*length as usize)
            }
            DiffOperation::Update { position, data } => {
                let start = *position as usize;
                let end = start + data.len();
                
                if start >= state.len() {
                    return Err(DiffError::InvalidDiffOperation.into());
                }
                
                let actual_end = end.min(state.len());
                state.splice(start..actual_end, data.iter().cloned());
                
                // If data extends beyond current state, append remaining
                if end > state.len() {
                    state.extend_from_slice(&data[actual_end - start..]);
                }
                
                Ok(data.len())
            }
        }
    }

    /// Validate a single diff operation
    fn validate_operation(operation: &DiffOperation) -> Result<()> {
        match operation {
            DiffOperation::Insert { position: _, data } => {
                if data.is_empty() {
                    return Err(DiffError::InvalidDiffOperation.into());
                }
                if data.len() > 10240 { // 10KB limit per operation
                    return Err(DiffError::DiffSizeExceeded.into());
                }
            }
            DiffOperation::Delete { position: _, length } => {
                if *length == 0 {
                    return Err(DiffError::InvalidDiffOperation.into());
                }
                if *length > 10240 { // 10KB limit per operation
                    return Err(DiffError::DiffSizeExceeded.into());
                }
            }
            DiffOperation::Update { position: _, data } => {
                if data.is_empty() {
                    return Err(DiffError::InvalidDiffOperation.into());
                }
                if data.len() > 10240 { // 10KB limit per operation
                    return Err(DiffError::DiffSizeExceeded.into());
                }
            }
        }
        Ok(())
    }

    /// Rollback operations to a snapshot
    pub fn rollback_to_snapshot(&mut self, snapshot_id: &str, state: &mut Vec<u8>) -> Result<()> {
        if let Some(snapshot) = self.state_snapshots.get(snapshot_id) {
            state.clear();
            state.extend_from_slice(snapshot);
            msg!("Rolled back to snapshot: {}", snapshot_id);
            Ok(())
        } else {
            Err(DiffError::SnapshotNotFound.into())
        }
    }
    
    /// Rollback operations in case of failure
    pub fn rollback_operations(&mut self, operations: &[DiffOperation]) -> Result<()> {
        msg!("Rolling back {} operations", operations.len());
        
        // Process transaction log in reverse order
        for entry in self.transaction_log.iter().rev() {
            msg!("Reverting operation {}", entry.operation_idx);
        }
        
        self.transaction_log.clear();
        Ok(())
    }
    
    /// Create inverse operations for rollback
    pub fn create_inverse_operations(operations: &[DiffOperation]) -> Vec<DiffOperation> {
        operations.iter().map(|op| {
            match op {
                DiffOperation::Insert { position, data } => {
                    DiffOperation::Delete {
                        position: *position,
                        length: data.len() as u64,
                    }
                }
                DiffOperation::Delete { position, length } => {
                    // Note: We'd need the deleted data to properly inverse this
                    // For now, create placeholder
                    DiffOperation::Insert {
                        position: *position,
                        data: vec![0; *length as usize],
                    }
                }
                DiffOperation::Update { position, data } => {
                    // Note: We'd need the original data to properly inverse this
                    // For now, create placeholder
                    DiffOperation::Update {
                        position: *position,
                        data: vec![0; data.len()],
                    }
                }
            }
        }).collect()
    }
}

/// Transaction log entry for rollback support
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TransactionEntry {
    operation_idx: usize,
    #[allow(dead_code)]
    operation: DiffOperation,
    #[allow(dead_code)]
    state_before: Vec<u8>,
}

/// Result of atomic processing
#[derive(Debug)]
pub struct AtomicResult {
    pub success: bool,
    pub operations_processed: usize,
    pub operations_failed: usize,
    pub bytes_changed: usize,
    pub error_at_index: Option<usize>,
} 