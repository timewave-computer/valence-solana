// Common types for session runtime
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Session operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub session: Pubkey,
    pub operation_index: usize,
    pub success: bool,
    pub compute_units_used: u64,
    pub logs: Vec<String>,
}

/// Account type for off-chain tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Token { mint: Pubkey },
    TokenAccount { mint: Pubkey, owner: Pubkey },
    Program { executable: bool },
    Data { discriminator: [u8; 8] },
}

/// Session execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub session: Pubkey,
    pub operations: Vec<PlannedOperation>,
    pub estimated_compute: u64,
    pub required_accounts: Vec<Pubkey>,
}

/// Planned operation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedOperation {
    pub operation: String, // Serialized operation
    pub dependencies: Vec<usize>,
    pub estimated_compute: u64,
    pub can_fail: bool,
}

/// Session metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_compute_units: u64,
    pub average_compute_per_operation: u64,
    pub total_accounts_borrowed: u64,
    pub peak_concurrent_borrows: u64,
}

impl SessionMetrics {
    /// Update metrics with operation result
    pub fn record_operation(&mut self, success: bool, compute_units: u64) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }
        self.total_compute_units += compute_units;
        self.average_compute_per_operation = 
            self.total_compute_units / self.total_operations.max(1);
    }
    
    /// Update borrow metrics
    pub fn record_borrow(&mut self, current_borrows: u64) {
        self.total_accounts_borrowed += 1;
        self.peak_concurrent_borrows = self.peak_concurrent_borrows.max(current_borrows);
    }
}