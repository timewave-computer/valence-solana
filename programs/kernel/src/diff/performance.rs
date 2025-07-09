// Performance optimization for diff operations

use anchor_lang::prelude::*;
use crate::diff::types::{DiffOperation, DiffBatch};
use std::collections::HashMap;

/// Performance optimizer for diff operations
pub struct DiffPerformanceOptimizer;

impl DiffPerformanceOptimizer {
    /// Optimize a batch of diff operations
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
                DiffOperation::Add { key, .. } => key,
                DiffOperation::Update { key, .. } => key,
                DiffOperation::Remove { key, .. } => key,
                DiffOperation::Move { old_key, .. } => old_key,
            };

            if seen.contains_key(key) {
                removed += 1;
                false
            } else {
                seen.insert(key.clone(), true);
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
                _ => true,
            }
        });

        initial_len - operations.len()
    }

    /// Sort operations for better cache locality
    fn sort_for_locality(operations: &mut [DiffOperation]) {
        operations.sort_by(|a, b| {
            let key_a = match a {
                DiffOperation::Add { key, .. } => key,
                DiffOperation::Update { key, .. } => key,
                DiffOperation::Remove { key, .. } => key,
                DiffOperation::Move { old_key, .. } => old_key,
            };
            let key_b = match b {
                DiffOperation::Add { key, .. } => key,
                DiffOperation::Update { key, .. } => key,
                DiffOperation::Remove { key, .. } => key,
                DiffOperation::Move { old_key, .. } => old_key,
            };
            key_a.cmp(key_b)
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