// Diff computation logic

use anchor_lang::prelude::*;
use crate::diff::instructions::DiffOperation;

/// Internal representation for Myers' diff algorithm
#[derive(Debug, Clone)]
enum MyersDiff {
    Equal(usize),
    Insert(Vec<u8>),
    Delete(usize),
}

/// Calculates diffs between states
pub struct DiffCalculator;

impl DiffCalculator {
    /// Calculate diff between two byte arrays
    pub fn calculate_diff(state_a: &[u8], state_b: &[u8]) -> Vec<DiffOperation> {
        let mut operations = Vec::new();
        
        // Handle empty states
        if state_a.is_empty() && !state_b.is_empty() {
            operations.push(DiffOperation::Insert {
                position: 0,
                data: state_b.to_vec(),
            });
            return operations;
        }
        
        if !state_a.is_empty() && state_b.is_empty() {
            operations.push(DiffOperation::Delete {
                position: 0,
                length: state_a.len() as u64,
            });
            return operations;
        }
        
        // Use Myers' diff algorithm for byte-level comparison
        let diffs = Self::myers_diff(state_a, state_b);
        
        // Convert to DiffOperations
        let mut current_pos = 0;
        for diff in diffs {
            match diff {
                MyersDiff::Equal(len) => {
                    current_pos += len;
                }
                MyersDiff::Insert(data) => {
                    operations.push(DiffOperation::Insert {
                        position: current_pos as u64,
                        data: data.clone(),
                    });
                }
                MyersDiff::Delete(len) => {
                    operations.push(DiffOperation::Delete {
                        position: current_pos as u64,
                        length: len as u64,
                    });
                    current_pos += len;
                }
            }
        }
        
        // Optimize operations
        Self::optimize_operations(&mut operations);
        
        operations
    }
    
    /// Simplified Myers' diff algorithm
    fn myers_diff(a: &[u8], b: &[u8]) -> Vec<MyersDiff> {
        let mut diffs = Vec::new();
        let mut i = 0;
        let mut j = 0;
        
        while i < a.len() || j < b.len() {
            if i < a.len() && j < b.len() && a[i] == b[j] {
                // Count equal bytes
                let start_i = i;
                while i < a.len() && j < b.len() && a[i] == b[j] {
                    i += 1;
                    j += 1;
                }
                diffs.push(MyersDiff::Equal(i - start_i));
            } else if j < b.len() && (i >= a.len() || (i < a.len() && a[i] != b[j])) {
                // Insert from b
                let start_j = j;
                while j < b.len() && (i >= a.len() || (i < a.len() && a[i] != b[j])) {
                    j += 1;
                }
                diffs.push(MyersDiff::Insert(b[start_j..j].to_vec()));
            } else if i < a.len() {
                // Delete from a
                let start_i = i;
                while i < a.len() && (j >= b.len() || (j < b.len() && a[i] != b[j])) {
                    i += 1;
                }
                diffs.push(MyersDiff::Delete(i - start_i));
            }
        }
        
        diffs
    }
    
    /// Optimize diff operations by merging adjacent operations
    fn optimize_operations(operations: &mut Vec<DiffOperation>) {
        if operations.len() <= 1 {
            return;
        }
        
        let mut optimized = Vec::new();
        let mut i = 0;
        
        while i < operations.len() {
            match &operations[i] {
                DiffOperation::Insert { position, data } => {
                    // Merge consecutive inserts
                    let mut merged_data = data.clone();
                    let start_pos = *position;
                    i += 1;
                    
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
                    
                    optimized.push(DiffOperation::Insert {
                        position: start_pos,
                        data: merged_data,
                    });
                }
                _ => {
                    optimized.push(operations[i].clone());
                    i += 1;
                }
            }
        }
        
        *operations = optimized;
    }

    /// Apply diff operations to a state
    pub fn apply_diff(state: &mut Vec<u8>, operations: &[DiffOperation]) -> Result<()> {
        for operation in operations {
            match operation {
                // Positional operations (supported by diff calculator)
                DiffOperation::Insert { position, data } => {
                    let pos = *position as usize;
                    if pos <= state.len() {
                        state.splice(pos..pos, data.iter().cloned());
                    }
                }
                DiffOperation::Delete { position, length } => {
                    let start = *position as usize;
                    let end = start + (*length as usize);
                    if start < state.len() && end <= state.len() {
                        state.drain(start..end);
                    }
                }
                DiffOperation::Replace { position, data } => {
                    let start = *position as usize;
                    let end = start + data.len();
                    if start < state.len() {
                        state.splice(start..end.min(state.len()), data.iter().cloned());
                    }
                }
                // Key-value operations (not supported for byte array diffs)
                DiffOperation::Add { .. } | 
                DiffOperation::Update { .. } | 
                DiffOperation::Remove { .. } | 
                DiffOperation::Move { .. } => {
                    msg!("Key-value operations not supported for byte array diffs");
                    // Skip these operations for byte array diffs
                }
            }
        }
        Ok(())
    }
} 