// Batch optimization logic for diff operations

use crate::diff::instructions::DiffOperation;
use std::collections::HashMap;

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
        let optimized = Self::reorder_operations(reduced);
        
        optimized
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
        let mut position_map: HashMap<u32, Vec<usize>> = HashMap::new();
        
        // Group operations by position
        for (idx, op) in operations.iter().enumerate() {
            let pos = match op {
                DiffOperation::Insert { position, .. } => *position,
                DiffOperation::Delete { position, .. } => *position,
                DiffOperation::Update { position, .. } => *position,
            };
            position_map.entry(pos as u32).or_default().push(idx);
        }
        
        let mut keep_indices = vec![true; operations.len()];
        
        // Check for redundant operations
        for indices in position_map.values() {
            if indices.len() > 1 {
                // Keep only the last operation at each position
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
        // Sort by position (descending for deletes, ascending for inserts)
        operations.sort_by(|a, b| {
            let pos_a = Self::get_position(a);
            let pos_b = Self::get_position(b);
            
            match (a, b) {
                (DiffOperation::Delete { .. }, DiffOperation::Delete { .. }) => {
                    // Deletes in descending order
                    pos_b.cmp(&pos_a)
                }
                (DiffOperation::Insert { .. }, DiffOperation::Insert { .. }) => {
                    // Inserts in ascending order
                    pos_a.cmp(&pos_b)
                }
                (DiffOperation::Delete { .. }, _) => std::cmp::Ordering::Less,
                (_, DiffOperation::Delete { .. }) => std::cmp::Ordering::Greater,
                _ => pos_a.cmp(&pos_b),
            }
        });
        
        operations
    }
    
    /// Get position from diff operation
    fn get_position(op: &DiffOperation) -> u64 {
        match op {
            DiffOperation::Insert { position, .. } => *position,
            DiffOperation::Delete { position, .. } => *position,
            DiffOperation::Update { position, .. } => *position,
        }
    }
    
    /// Calculate the total size of a batch
    pub fn calculate_batch_size(operations: &[DiffOperation]) -> usize {
        operations.iter().map(|op| {
            match op {
                DiffOperation::Insert { data, .. } => data.len(),
                DiffOperation::Delete { length, .. } => *length as usize,
                DiffOperation::Update { data, .. } => data.len(),
            }
        }).sum()
    }
    
    /// Split batch if it exceeds size limit
    pub fn split_batch_by_size(operations: Vec<DiffOperation>, max_size: usize) -> Vec<Vec<DiffOperation>> {
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_size = 0;
        
        for op in operations {
            let op_size = match &op {
                DiffOperation::Insert { data, .. } => data.len(),
                DiffOperation::Delete { length, .. } => *length as usize,
                DiffOperation::Update { data, .. } => data.len(),
            };
            
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_adjacent_inserts() {
        let operations = vec![
            DiffOperation::Insert { position: 0, data: vec![1, 2] },
            DiffOperation::Insert { position: 2, data: vec![3, 4] },
            DiffOperation::Insert { position: 4, data: vec![5, 6] },
        ];
        
        let optimized = BatchOptimizer::optimize_batch(operations);
        assert_eq!(optimized.len(), 1);
        
        if let DiffOperation::Insert { position, data } = &optimized[0] {
            assert_eq!(*position, 0);
            assert_eq!(*data, vec![1, 2, 3, 4, 5, 6]);
        }
    }
}