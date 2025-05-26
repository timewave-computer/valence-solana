// Compute budget management utilities for dynamic adjustment and optimization
use anchor_lang::prelude::*;
use solana_program::instruction::Instruction;

/// Default compute budget limits
pub const DEFAULT_COMPUTE_UNITS: u32 = 200_000;
pub const MAX_COMPUTE_UNITS: u32 = 1_400_000;
pub const MIN_COMPUTE_UNITS: u32 = 200;

/// Base compute costs for common operations
pub const BASE_CPI_COST: u32 = 1_000;
pub const ACCOUNT_SERIALIZATION_COST: u32 = 100;
pub const ACCOUNT_DESERIALIZATION_COST: u32 = 150;
pub const PDA_DERIVATION_COST: u32 = 25;
pub const SIGNATURE_VERIFICATION_COST: u32 = 2_000;

/// Compute budget configuration for different operation types
#[derive(Debug, Clone)]
pub struct ComputeBudgetConfig {
    pub base_units: u32,
    pub per_account_units: u32,
    pub per_cpi_units: u32,
    pub buffer_percentage: u8, // Additional buffer (10-50%)
}

impl Default for ComputeBudgetConfig {
    fn default() -> Self {
        Self {
            base_units: 5_000,
            per_account_units: 200,
            per_cpi_units: BASE_CPI_COST,
            buffer_percentage: 20, // 20% buffer
        }
    }
}

/// Compute budget estimator for different operation types
pub struct ComputeBudgetEstimator;

impl ComputeBudgetEstimator {
    /// Estimate compute units for a basic operation
    pub fn estimate_basic_operation(
        account_count: u32,
        cpi_count: u32,
        config: Option<ComputeBudgetConfig>,
    ) -> u32 {
        let config = config.unwrap_or_default();
        
        let base_cost = config.base_units;
        let account_cost = account_count * config.per_account_units;
        let cpi_cost = cpi_count * config.per_cpi_units;
        
        let total = base_cost + account_cost + cpi_cost;
        
        // Add buffer
        let buffer = (total * config.buffer_percentage as u32) / 100;
        
        std::cmp::min(total + buffer, MAX_COMPUTE_UNITS)
    }
    
    /// Estimate compute units for ZK verification operations
    pub fn estimate_zk_verification(
        proof_size: usize,
        public_input_count: u32,
    ) -> u32 {
        // ZK verification is compute-intensive
        let base_zk_cost = 50_000;
        let proof_cost = (proof_size / 32) as u32 * 100; // Cost per 32-byte chunk
        let input_cost = public_input_count * 500;
        
        let total = base_zk_cost + proof_cost + input_cost;
        std::cmp::min(total, MAX_COMPUTE_UNITS)
    }
    
    /// Estimate compute units for batch operations
    pub fn estimate_batch_operation(
        batch_size: u32,
        per_item_cost: u32,
        base_cost: u32,
    ) -> u32 {
        let total = base_cost + (batch_size * per_item_cost);
        
        // Add 30% buffer for batch operations (higher due to complexity)
        let buffer = (total * 30) / 100;
        
        std::cmp::min(total + buffer, MAX_COMPUTE_UNITS)
    }
    
    /// Estimate compute units for SMT operations
    pub fn estimate_smt_operation(
        tree_depth: u32,
        operation_type: SmtOperationType,
    ) -> u32 {
        let base_cost = match operation_type {
            SmtOperationType::Create => 10_000,
            SmtOperationType::Update => 15_000,
            SmtOperationType::Verify => 20_000,
        };
        
        // Cost increases with tree depth
        let depth_cost = tree_depth * 1_000;
        
        let total = base_cost + depth_cost;
        std::cmp::min(total, MAX_COMPUTE_UNITS)
    }
}

/// SMT operation types for compute estimation
#[derive(Debug, Clone, Copy)]
pub enum SmtOperationType {
    Create,
    Update,
    Verify,
}

/// Operation types for dynamic compute budget adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Simple,
    ZKVerification,
    BatchOperation,
    CrossProgramCall,
    ComplexComputation,
}

/// Network congestion levels for fee adjustment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkCongestionLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Transaction urgency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionUrgency {
    Low,
    Normal,
    High,
    Critical,
}

/// Operation metrics for historical analysis
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub operation_type: OperationType,
    pub account_count: u32,
    pub cpi_count: u32,
    pub estimated_compute_units: u32,
    pub actual_compute_units: u32,
    pub timestamp: i64,
}

/// Operation descriptor for compute estimation
#[derive(Debug, Clone)]
pub struct OperationDescriptor {
    pub operation_type: OperationType,
    pub account_count: u32,
    pub cpi_count: u32,
    pub data_size: usize,
}

/// Dynamic compute budget manager with real-time adjustment
pub struct ComputeBudgetManager;

impl ComputeBudgetManager {
    /// Create compute budget instruction with dynamic adjustment
    /// Note: This is a placeholder - actual implementation would use ComputeBudgetInstruction
    pub fn create_compute_budget_instruction(
        estimated_units: u32,
        priority_fee_microlamports: Option<u64>,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        
        // Set compute unit limit if different from default
        if estimated_units != DEFAULT_COMPUTE_UNITS {
            let adjusted_units = std::cmp::max(
                std::cmp::min(estimated_units, MAX_COMPUTE_UNITS),
                MIN_COMPUTE_UNITS,
            );
            
            msg!("Would set compute unit limit to: {}", adjusted_units);
            // TODO: Add actual ComputeBudgetInstruction when available
        }
        
        // Set priority fee if specified
        if let Some(fee) = priority_fee_microlamports {
            if fee > 0 {
                msg!("Would set priority fee to: {} microlamports", fee);
                // TODO: Add actual ComputeBudgetInstruction when available
            }
        }
        
        instructions
    }
    
    /// Dynamically adjust compute budget based on operation complexity
    pub fn adjust_budget_for_operation(
        base_estimate: u32,
        operation_type: OperationType,
        network_congestion: NetworkCongestionLevel,
    ) -> u32 {
        let mut adjusted = base_estimate;
        
        // Adjust based on operation type
        let operation_multiplier = match operation_type {
            OperationType::Simple => 1.0,
            OperationType::ZKVerification => 1.5,
            OperationType::BatchOperation => 1.3,
            OperationType::CrossProgramCall => 1.4,
            OperationType::ComplexComputation => 1.6,
        };
        
        adjusted = (adjusted as f32 * operation_multiplier) as u32;
        
        // Adjust based on network congestion
        let congestion_multiplier = match network_congestion {
            NetworkCongestionLevel::Low => 1.0,
            NetworkCongestionLevel::Medium => 1.1,
            NetworkCongestionLevel::High => 1.2,
            NetworkCongestionLevel::Critical => 1.3,
        };
        
        adjusted = (adjusted as f32 * congestion_multiplier) as u32;
        
        // Ensure within bounds
        std::cmp::max(
            std::cmp::min(adjusted, MAX_COMPUTE_UNITS),
            MIN_COMPUTE_UNITS,
        )
    }
    
    /// Calculate optimal priority fee based on urgency and network conditions
    pub fn calculate_priority_fee(
        urgency: TransactionUrgency,
        network_congestion: NetworkCongestionLevel,
        base_fee: Option<u64>,
    ) -> u64 {
        let base = base_fee.unwrap_or(Self::get_recommended_priority_fee());
        
        let urgency_multiplier = match urgency {
            TransactionUrgency::Low => 0.5,
            TransactionUrgency::Normal => 1.0,
            TransactionUrgency::High => 2.0,
            TransactionUrgency::Critical => 3.0,
        };
        
        let congestion_multiplier = match network_congestion {
            NetworkCongestionLevel::Low => 1.0,
            NetworkCongestionLevel::Medium => 1.5,
            NetworkCongestionLevel::High => 2.0,
            NetworkCongestionLevel::Critical => 3.0,
        };
        
        ((base as f64) * urgency_multiplier * congestion_multiplier) as u64
    }
    
    /// Estimate compute units with machine learning-like adjustment
    pub fn smart_estimate(
        operation_history: &[OperationMetrics],
        current_operation: &OperationDescriptor,
    ) -> u32 {
        if operation_history.is_empty() {
            return Self::fallback_estimate(current_operation);
        }
        
        // Find similar operations in history
        let similar_ops: Vec<&OperationMetrics> = operation_history
            .iter()
            .filter(|op| op.operation_type == current_operation.operation_type)
            .filter(|op| {
                // Similar complexity (within 20% of account/CPI counts)
                let account_diff = (op.account_count as f32 - current_operation.account_count as f32).abs();
                let cpi_diff = (op.cpi_count as f32 - current_operation.cpi_count as f32).abs();
                
                (account_diff / current_operation.account_count as f32) < 0.2 &&
                (cpi_diff / current_operation.cpi_count as f32) < 0.2
            })
            .collect();
        
        if similar_ops.is_empty() {
            return Self::fallback_estimate(current_operation);
        }
        
        // Calculate weighted average based on recency
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        
        for (i, op) in similar_ops.iter().enumerate() {
            let weight = 1.0 / (i as f32 + 1.0); // More recent operations have higher weight
            total_weight += weight;
            weighted_sum += (op.actual_compute_units as f32) * weight;
        }
        
        let estimated = (weighted_sum / total_weight) as u32;
        
        // Add 15% buffer for safety
        let buffered = (estimated as f32 * 1.15) as u32;
        
        std::cmp::max(
            std::cmp::min(buffered, MAX_COMPUTE_UNITS),
            MIN_COMPUTE_UNITS,
        )
    }
    
    /// Fallback estimation when no history is available
    fn fallback_estimate(operation: &OperationDescriptor) -> u32 {
        ComputeBudgetEstimator::estimate_basic_operation(
            operation.account_count,
            operation.cpi_count,
            None,
        )
    }
    
    /// Get recommended priority fee based on network conditions
    /// This would typically query recent prioritization fees
    pub fn get_recommended_priority_fee() -> u64 {
        // In a real implementation, this would query the RPC for recent fees
        // For now, return a conservative default
        1_000 // 1000 microlamports per compute unit
    }
    
    /// Validate compute budget requirements
    pub fn validate_compute_budget(
        estimated_units: u32,
        max_allowed: Option<u32>,
    ) -> Result<()> {
        let max_allowed = max_allowed.unwrap_or(MAX_COMPUTE_UNITS);
        
        if estimated_units > max_allowed {
            msg!(
                "Estimated compute units ({}) exceed maximum allowed ({})",
                estimated_units,
                max_allowed
            );
            return Err(ProgramError::InvalidArgument.into());
        }
        
        if estimated_units < MIN_COMPUTE_UNITS {
            msg!(
                "Estimated compute units ({}) below minimum required ({})",
                estimated_units,
                MIN_COMPUTE_UNITS
            );
            return Err(ProgramError::InvalidArgument.into());
        }
        
        Ok(())
    }
}

/// Compute budget optimization strategies
pub struct ComputeBudgetOptimizer;

impl ComputeBudgetOptimizer {
    /// Optimize account access patterns to reduce compute costs
    pub fn optimize_account_access_pattern(account_count: u32) -> ComputeBudgetConfig {
        let mut config = ComputeBudgetConfig::default();
        
        // Reduce per-account cost for large account sets
        if account_count > 10 {
            config.per_account_units = 150; // Reduced from 200
        }
        
        if account_count > 20 {
            config.per_account_units = 100; // Further reduction
            config.buffer_percentage = 30; // Increase buffer for complexity
        }
        
        config
    }
    
    /// Optimize CPI patterns to reduce nested call overhead
    pub fn optimize_cpi_pattern(cpi_count: u32, max_depth: u32) -> ComputeBudgetConfig {
        let mut config = ComputeBudgetConfig::default();
        
        // Increase cost for nested CPIs (exponential penalty)
        if max_depth > 1 {
            config.per_cpi_units = BASE_CPI_COST * (2_u32.pow(max_depth - 1));
        }
        
        // Add extra buffer for complex CPI patterns
        if cpi_count > 5 {
            config.buffer_percentage = 40;
        }
        
        config
    }
    
    /// Optimize for batch operations
    pub fn optimize_batch_operation(batch_size: u32) -> ComputeBudgetConfig {
        let mut config = ComputeBudgetConfig::default();
        
        // Batch operations have economies of scale
        if batch_size > 5 {
            config.per_account_units = 120; // Reduced per-item cost
        }
        
        if batch_size > 10 {
            config.per_account_units = 80;
            config.buffer_percentage = 25; // Larger buffer for batch complexity
        }
        
        config
    }
}

/// Compute budget monitoring and logging
pub struct ComputeBudgetMonitor;

impl ComputeBudgetMonitor {
    /// Log compute budget usage for monitoring
    pub fn log_compute_usage(
        operation: &str,
        estimated_units: u32,
        actual_units: Option<u32>,
    ) {
        match actual_units {
            Some(actual) => {
                let efficiency = (actual as f64 / estimated_units as f64) * 100.0;
                msg!(
                    "Compute usage - Operation: {}, Estimated: {}, Actual: {}, Efficiency: {:.1}%",
                    operation,
                    estimated_units,
                    actual,
                    efficiency
                );
            }
            None => {
                msg!(
                    "Compute budget - Operation: {}, Estimated: {}",
                    operation,
                    estimated_units
                );
            }
        }
    }
    
    /// Check if compute budget is running low
    pub fn check_compute_budget_health(
        remaining_units: u32,
        total_units: u32,
        threshold_percentage: u8,
    ) -> bool {
        let threshold = (total_units * threshold_percentage as u32) / 100;
        remaining_units > threshold
    }
}

/// Macro for easy compute budget estimation
#[macro_export]
macro_rules! estimate_compute_budget {
    (basic: $accounts:expr, $cpis:expr) => {
        ComputeBudgetEstimator::estimate_basic_operation($accounts, $cpis, None)
    };
    (basic: $accounts:expr, $cpis:expr, $config:expr) => {
        ComputeBudgetEstimator::estimate_basic_operation($accounts, $cpis, Some($config))
    };
    (zk: $proof_size:expr, $inputs:expr) => {
        ComputeBudgetEstimator::estimate_zk_verification($proof_size, $inputs)
    };
    (batch: $size:expr, $per_item:expr, $base:expr) => {
        ComputeBudgetEstimator::estimate_batch_operation($size, $per_item, $base)
    };
    (smt: $depth:expr, $op:expr) => {
        ComputeBudgetEstimator::estimate_smt_operation($depth, $op)
    };
}

/// Macro for creating compute budget instructions
#[macro_export]
macro_rules! create_compute_budget {
    ($units:expr) => {
        ComputeBudgetManager::create_compute_budget_instruction($units, None)
    };
    ($units:expr, $priority:expr) => {
        ComputeBudgetManager::create_compute_budget_instruction($units, Some($priority))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_compute_estimation() {
        let estimated = ComputeBudgetEstimator::estimate_basic_operation(5, 2, None);
        assert!(estimated > 0);
        assert!(estimated <= MAX_COMPUTE_UNITS);
    }
    
    #[test]
    fn test_zk_compute_estimation() {
        let estimated = ComputeBudgetEstimator::estimate_zk_verification(1024, 10);
        assert!(estimated >= 50_000); // Should be at least base cost
        assert!(estimated <= MAX_COMPUTE_UNITS);
    }
    
    #[test]
    fn test_batch_compute_estimation() {
        let estimated = ComputeBudgetEstimator::estimate_batch_operation(10, 500, 5_000);
        assert!(estimated > 5_000); // Should be more than base cost
        assert!(estimated <= MAX_COMPUTE_UNITS);
    }
    
    #[test]
    fn test_compute_budget_validation() {
        assert!(ComputeBudgetManager::validate_compute_budget(100_000, None).is_ok());
        assert!(ComputeBudgetManager::validate_compute_budget(2_000_000, None).is_err());
        assert!(ComputeBudgetManager::validate_compute_budget(100, None).is_err());
    }
} 