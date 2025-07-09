// Comprehensive diff optimization module
// Combines atomic processing, batch optimization, and performance optimization

use anchor_lang::prelude::*;
use crate::{DiffError, diff::instructions::{DiffOperation, DiffBatch, DiffResult}};
use std::collections::HashMap;

// ======================= ATOMIC PROCESSOR =======================

/// Processes diffs atomically with rollback support
#[derive(Default)]
pub struct AtomicProcessor {
    /// Transaction log for rollback support
    transaction_log: Vec<TransactionEntry>,
    /// Original state snapshots for rollback
    state_snapshots: HashMap<String, Vec<u8>>,
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
            DiffOperation::Replace { position, data } => {
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
            _ => {
                // For key-value operations, we'd need a different state representation
                // For now, just return success
                Ok(0)
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
            DiffOperation::Replace { position: _, data } => {
                if data.is_empty() {
                    return Err(DiffError::InvalidDiffOperation.into());
                }
                if data.len() > 10240 { // 10KB limit per operation
                    return Err(DiffError::DiffSizeExceeded.into());
                }
            }
            _ => {} // Other operations are validated differently
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

// ======================= BATCH OPTIMIZER =======================

/// Optimizes batches of diff operations for efficiency
pub struct BatchOptimizer;

impl BatchOptimizer {
    /// Optimize a batch of diff operations
    pub fn optimize_batch(operations: Vec<DiffOperation>) -> Vec<DiffOperation> {
        if operations.is_empty() {
            return operations;
        }
        
        // Step 1: Merge adjacent operations
        let merged = Self::merge_adjacent_operations(operations);
        
        // Step 2: Eliminate redundant operations
        let reduced = Self::eliminate_redundant_operations(merged);
        
        // Step 3: Reorder for optimal application
        Self::reorder_operations(reduced)
    }
    
    /// Merge adjacent operations of the same type
    fn merge_adjacent_operations(operations: Vec<DiffOperation>) -> Vec<DiffOperation> {
        let mut merged = Vec::new();
        let mut i = 0;
        
        while i < operations.len() {
            match &operations[i] {
                DiffOperation::Insert { position, data } => {
                    let mut merged_data = data.clone();
                    let start_pos = *position;
                    i += 1;
                    
                    // Merge consecutive inserts
                    while i < operations.len() {
                        if let DiffOperation::Insert { position: next_pos, data: next_data } = &operations[i] {
                            if *next_pos == start_pos + merged_data.len() as u64 {
                                merged_data.extend_from_slice(next_data);
                                i += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    merged.push(DiffOperation::Insert {
                        position: start_pos,
                        data: merged_data,
                    });
                }
                DiffOperation::Delete { position, length } => {
                    let mut total_length = *length;
                    let start_pos = *position;
                    i += 1;
                    
                    // Merge consecutive deletes
                    while i < operations.len() {
                        if let DiffOperation::Delete { position: next_pos, length: next_length } = &operations[i] {
                            if *next_pos == start_pos {
                                total_length += next_length;
                                i += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    merged.push(DiffOperation::Delete {
                        position: start_pos,
                        length: total_length,
                    });
                }
                _ => {
                    merged.push(operations[i].clone());
                    i += 1;
                }
            }
        }
        
        merged
    }
    
    /// Eliminate redundant operations
    fn eliminate_redundant_operations(operations: Vec<DiffOperation>) -> Vec<DiffOperation> {
        let mut key_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut position_map: HashMap<u64, Vec<usize>> = HashMap::new();
        
        // Group operations by key or position
        for (idx, op) in operations.iter().enumerate() {
            match op {
                DiffOperation::Add { key, .. } |
                DiffOperation::Update { key, .. } |
                DiffOperation::Remove { key, .. } => {
                    key_map.entry(key.clone()).or_default().push(idx);
                }
                DiffOperation::Move { old_key, .. } => {
                    key_map.entry(old_key.clone()).or_default().push(idx);
                }
                DiffOperation::Insert { position, .. } |
                DiffOperation::Delete { position, .. } |
                DiffOperation::Replace { position, .. } => {
                    position_map.entry(*position).or_default().push(idx);
                }
            }
        }
        
        let mut keep_indices = vec![true; operations.len()];
        
        // Check for redundant operations
        for indices in key_map.values().chain(position_map.values()) {
            if indices.len() > 1 {
                // Keep only the last operation at each key/position
                for &idx in &indices[..indices.len() - 1] {
                    keep_indices[idx] = false;
                }
            }
        }
        
        operations.into_iter()
            .enumerate()
            .filter_map(|(idx, op)| {
                if keep_indices[idx] {
                    Some(op)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Reorder operations for optimal application
    fn reorder_operations(mut operations: Vec<DiffOperation>) -> Vec<DiffOperation> {
        operations.sort_by(|a, b| {
            match (a, b) {
                // Position-based operations
                (DiffOperation::Insert { position: p1, .. }, DiffOperation::Insert { position: p2, .. }) => p1.cmp(p2),
                (DiffOperation::Delete { position: p1, .. }, DiffOperation::Delete { position: p2, .. }) => p2.cmp(p1),
                (DiffOperation::Replace { position: p1, .. }, DiffOperation::Replace { position: p2, .. }) => p1.cmp(p2),
                
                // Key-based operations
                (DiffOperation::Add { key: k1, .. }, DiffOperation::Add { key: k2, .. }) => k1.cmp(k2),
                (DiffOperation::Update { key: k1, .. }, DiffOperation::Update { key: k2, .. }) => k1.cmp(k2),
                (DiffOperation::Remove { key: k1, .. }, DiffOperation::Remove { key: k2, .. }) => k1.cmp(k2),
                
                // Mixed operations - prioritize deletes, then updates, then inserts
                (DiffOperation::Delete { .. }, _) => std::cmp::Ordering::Less,
                (_, DiffOperation::Delete { .. }) => std::cmp::Ordering::Greater,
                (DiffOperation::Remove { .. }, _) => std::cmp::Ordering::Less,
                (_, DiffOperation::Remove { .. }) => std::cmp::Ordering::Greater,
                
                _ => std::cmp::Ordering::Equal,
            }
        });
        
        operations
    }
    
    /// Split batch if it exceeds size limit
    pub fn split_batch_by_size(operations: Vec<DiffOperation>, max_size: usize) -> Vec<Vec<DiffOperation>> {
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_size = 0;
        
        for op in operations {
            let op_size = op.size();
            
            if current_size + op_size > max_size && !current_batch.is_empty() {
                batches.push(current_batch);
                current_batch = Vec::new();
                current_size = 0;
            }
            
            current_size += op_size;
            current_batch.push(op);
        }
        
        if !current_batch.is_empty() {
            batches.push(current_batch);
        }
        
        batches
    }
}

// ======================= PERFORMANCE OPTIMIZER =======================

/// Performance optimizer for diff operations
pub struct PerformanceOptimizer;

impl PerformanceOptimizer {
    /// Optimize a batch of diff operations for performance
    pub fn optimize_batch(batch: &mut DiffBatch) -> Result<OptimizationStats> {
        let initial_count = batch.operations.len();
        let initial_size = batch.total_size();

        // Remove duplicates
        let duplicates_removed = Self::remove_duplicates(&mut batch.operations);

        // Merge consecutive updates
        let updates_merged = Self::merge_consecutive_updates(&mut batch.operations);

        // Remove no-ops
        let noops_removed = Self::remove_noops(&mut batch.operations);

        // Sort operations for better cache locality
        Self::sort_for_locality(&mut batch.operations);

        let final_count = batch.operations.len();
        let final_size = batch.total_size();

        Ok(OptimizationStats {
            initial_operation_count: initial_count,
            final_operation_count: final_count,
            operations_removed: initial_count - final_count,
            duplicates_removed,
            updates_merged,
            noops_removed,
            size_reduction: initial_size.saturating_sub(final_size),
            optimization_time_ms: 0, // Would be measured in real implementation
        })
    }

    /// Remove duplicate operations
    fn remove_duplicates(operations: &mut Vec<DiffOperation>) -> usize {
        let mut seen = HashMap::new();
        let mut removed = 0;

        operations.retain(|op| {
            let key = match op {
                DiffOperation::Add { key, .. } => key.clone(),
                DiffOperation::Update { key, .. } => key.clone(),
                DiffOperation::Remove { key, .. } => key.clone(),
                DiffOperation::Move { old_key, .. } => old_key.clone(),
                DiffOperation::Insert { position, .. } => position.to_string(),
                DiffOperation::Delete { position, .. } => position.to_string(),
                DiffOperation::Replace { position, .. } => position.to_string(),
            };

            if seen.contains_key(&key) {
                removed += 1;
                false
            } else {
                seen.insert(key, true);
                true
            }
        });

        removed
    }

    /// Merge consecutive update operations on the same key
    fn merge_consecutive_updates(operations: &mut Vec<DiffOperation>) -> usize {
        let mut merged = 0;
        let mut i = 0;

        while i < operations.len().saturating_sub(1) {
            if let (
                DiffOperation::Update { key: k1, old_value, .. },
                DiffOperation::Update { key: k2, new_value, .. }
            ) = (&operations[i], &operations[i + 1]) {
                if k1 == k2 {
                    // Merge the two updates
                    operations[i] = DiffOperation::Update {
                        key: k1.clone(),
                        old_value: old_value.clone(),
                        new_value: new_value.clone(),
                    };
                    operations.remove(i + 1);
                    merged += 1;
                    continue;
                }
            }
            i += 1;
        }

        merged
    }

    /// Remove no-op operations
    fn remove_noops(operations: &mut Vec<DiffOperation>) -> usize {
        let initial_len = operations.len();

        operations.retain(|op| {
            match op {
                DiffOperation::Update { old_value, new_value, .. } => old_value != new_value,
                DiffOperation::Move { old_key, new_key, .. } => old_key != new_key,
                DiffOperation::Delete { length, .. } => *length > 0,
                DiffOperation::Insert { data, .. } => !data.is_empty(),
                DiffOperation::Replace { data, .. } => !data.is_empty(),
                _ => true,
            }
        });

        initial_len - operations.len()
    }

    /// Sort operations for better cache locality
    fn sort_for_locality(operations: &mut [DiffOperation]) {
        operations.sort_by(|a, b| {
            let key_a = match a {
                DiffOperation::Add { key, .. } => key.clone(),
                DiffOperation::Update { key, .. } => key.clone(),
                DiffOperation::Remove { key, .. } => key.clone(),
                DiffOperation::Move { old_key, .. } => old_key.clone(),
                DiffOperation::Insert { position, .. } => position.to_string(),
                DiffOperation::Delete { position, .. } => position.to_string(),
                DiffOperation::Replace { position, .. } => position.to_string(),
            };
            let key_b = match b {
                DiffOperation::Add { key, .. } => key.clone(),
                DiffOperation::Update { key, .. } => key.clone(),
                DiffOperation::Remove { key, .. } => key.clone(),
                DiffOperation::Move { old_key, .. } => old_key.clone(),
                DiffOperation::Insert { position, .. } => position.to_string(),
                DiffOperation::Delete { position, .. } => position.to_string(),
                DiffOperation::Replace { position, .. } => position.to_string(),
            };
            key_a.cmp(&key_b)
        });
    }

    /// Estimate gas cost for a batch
    pub fn estimate_gas_cost(batch: &DiffBatch) -> u64 {
        let base_cost = 1000u64;
        let per_operation_cost = 100u64;
        let per_byte_cost = 10u64;

        base_cost
            + (batch.operations.len() as u64 * per_operation_cost)
            + (batch.total_size() as u64 * per_byte_cost)
    }

    /// Check if batch should be split for performance
    pub fn should_split_batch(batch: &DiffBatch, max_operations: usize, max_size: usize) -> bool {
        batch.operations.len() > max_operations || batch.total_size() > max_size
    }

    /// Split a batch into smaller batches
    pub fn split_batch(batch: DiffBatch, max_operations: usize) -> Vec<DiffBatch> {
        let mut batches = Vec::new();
        let mut current_batch = DiffBatch::new(batch.source, batch.target);

        for op in batch.operations {
            if current_batch.operations.len() >= max_operations {
                batches.push(current_batch);
                current_batch = DiffBatch::new(batch.source, batch.target);
            }
            current_batch.add_operation(op);
        }

        if !current_batch.operations.is_empty() {
            batches.push(current_batch);
        }

        batches
    }
}

/// Statistics from optimization
#[derive(Debug)]
pub struct OptimizationStats {
    pub initial_operation_count: usize,
    pub final_operation_count: usize,
    pub operations_removed: usize,
    pub duplicates_removed: usize,
    pub updates_merged: usize,
    pub noops_removed: usize,
    pub size_reduction: usize,
    pub optimization_time_ms: u64,
}

impl OptimizationStats {
    /// Calculate optimization efficiency percentage
    pub fn efficiency_percentage(&self) -> f64 {
        if self.initial_operation_count == 0 {
            0.0
        } else {
            (self.operations_removed as f64 / self.initial_operation_count as f64) * 100.0
        }
    }
} 