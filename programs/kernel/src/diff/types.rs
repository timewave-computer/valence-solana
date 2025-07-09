// Diff types and data structures

use anchor_lang::prelude::*;

/// Represents a diff operation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DiffOperation {
    /// Add a new value
    Add { key: String, value: Vec<u8> },
    /// Update an existing value
    Update { key: String, old_value: Vec<u8>, new_value: Vec<u8> },
    /// Remove a value
    Remove { key: String, value: Vec<u8> },
    /// Move/rename a key
    Move { old_key: String, new_key: String, value: Vec<u8> },
}

/// A batch of diff operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DiffBatch {
    /// Unique identifier for this batch
    pub batch_id: [u8; 32],
    /// Operations in this batch
    pub operations: Vec<DiffOperation>,
    /// Timestamp when batch was created
    pub created_at: i64,
    /// Source of the diff
    pub source: Pubkey,
    /// Target for the diff
    pub target: Pubkey,
}

/// Result of diff processing
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DiffResult {
    /// Whether diff was applied successfully
    pub success: bool,
    /// Number of operations processed
    pub operations_processed: u32,
    /// Number of operations failed
    pub operations_failed: u32,
    /// Gas consumed
    pub gas_used: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Diff validation result
#[derive(Debug)]
pub struct DiffValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl DiffOperation {
    /// Calculate the size of this operation
    pub fn size(&self) -> usize {
        match self {
            DiffOperation::Add { key, value } => key.len() + value.len(),
            DiffOperation::Update { key, old_value, new_value } => {
                key.len() + old_value.len() + new_value.len()
            }
            DiffOperation::Remove { key, value } => key.len() + value.len(),
            DiffOperation::Move { old_key, new_key, value } => {
                old_key.len() + new_key.len() + value.len()
            }
        }
    }

    /// Check if this operation conflicts with another
    pub fn conflicts_with(&self, other: &DiffOperation) -> bool {
        match (self, other) {
            (DiffOperation::Add { key: k1, .. }, DiffOperation::Add { key: k2, .. }) => k1 == k2,
            (DiffOperation::Update { key: k1, .. }, DiffOperation::Update { key: k2, .. }) => k1 == k2,
            (DiffOperation::Remove { key: k1, .. }, DiffOperation::Update { key: k2, .. }) => k1 == k2,
            (DiffOperation::Update { key: k1, .. }, DiffOperation::Remove { key: k2, .. }) => k1 == k2,
            (DiffOperation::Move { new_key: k1, .. }, DiffOperation::Add { key: k2, .. }) => k1 == k2,
            _ => false,
        }
    }
}

impl DiffBatch {
    /// Create a new diff batch
    pub fn new(source: Pubkey, target: Pubkey) -> Self {
        Self {
            batch_id: [0; 32], // Should be generated properly
            operations: Vec::new(),
            created_at: Clock::get().map(|c| c.unix_timestamp).unwrap_or(0),
            source,
            target,
        }
    }

    /// Add an operation to the batch
    pub fn add_operation(&mut self, operation: DiffOperation) {
        self.operations.push(operation);
    }

    /// Get total size of the batch
    pub fn total_size(&self) -> usize {
        self.operations.iter().map(|op| op.size()).sum()
    }

    /// Validate the batch
    pub fn validate(&self) -> DiffValidation {
        let mut validation = DiffValidation {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check for empty batch
        if self.operations.is_empty() {
            validation.warnings.push("Batch contains no operations".to_string());
        }

        // Check for conflicts within the batch
        for i in 0..self.operations.len() {
            for j in (i + 1)..self.operations.len() {
                if self.operations[i].conflicts_with(&self.operations[j]) {
                    validation.is_valid = false;
                    validation.errors.push(format!(
                        "Operation {} conflicts with operation {}", i, j
                    ));
                }
            }
        }

        validation
    }
}