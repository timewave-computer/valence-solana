/// Function registry for managing on-chain function execution
/// This module provides registration, discovery, and execution capabilities for functions
use anchor_lang::prelude::*;
use crate::functions::types::FunctionInput;
use crate::functions::execution::{AggregationMode, AggregationInputResult};
use crate::functions::instructions::FunctionOutput;
use crate::error::FunctionCompositionError;

/// Aggregation pattern configurations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationConfig {
    /// Minimum number of inputs required for aggregation
    pub min_inputs: u32,
    /// Maximum number of inputs allowed
    pub max_inputs: u32,
    /// Timeout for individual input functions
    pub input_timeout_ms: u64,
    /// Consensus threshold (percentage for vote/consensus modes)
    pub consensus_threshold: f64,
    /// Whether to continue if some inputs fail
    pub continue_on_partial_failure: bool,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            min_inputs: 1,
            max_inputs: 20,
            input_timeout_ms: 30_000, // 30 seconds
            consensus_threshold: 0.51, // 51% majority
            continue_on_partial_failure: true,
        }
    }
}

/// Aggregation strategies for different use cases
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum AggregationStrategy {
    /// Simple merge: concatenate all outputs
    SimpleMerge,
    /// Weighted merge: merge based on function weights
    WeightedMerge(Vec<f64>),
    /// Majority vote: select most common output
    MajorityVote,
    /// Weighted vote: vote based on function weights
    WeightedVote(Vec<f64>),
    /// Consensus: require agreement above threshold
    Consensus(f64),
    /// Reduce: apply reduction function across outputs
    Reduce(ReductionFunction),
    /// Custom: use custom aggregation logic
    Custom(String),
}

/// Reduction function types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ReductionFunction {
    /// Sum all numeric values
    Sum,
    /// Average all numeric values
    Average,
    /// Find maximum value
    Maximum,
    /// Find minimum value
    Minimum,
    /// Count occurrences
    Count,
    /// Custom reduction logic
    Custom(String),
}

/// Aggregation validation result
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationValidation {
    /// Whether aggregation is valid
    pub is_valid: bool,
    /// Issues found during validation
    pub issues: Vec<String>,
    /// Recommended fixes
    pub recommendations: Vec<String>,
}

/// Aggregation execution context
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationContext {
    /// Aggregation configuration
    pub config: AggregationConfig,
    /// Strategy being used
    pub strategy: AggregationStrategy,
    /// Input function weights (if applicable)
    pub input_weights: Vec<f64>,
    /// Execution constraints
    pub constraints: AggregationConstraints,
}

/// Constraints for aggregation execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationConstraints {
    /// Maximum total execution time
    pub max_total_time_ms: u64,
    /// Maximum memory usage
    pub max_memory_bytes: u64,
    /// Minimum success rate required
    pub min_success_rate: f64,
    /// Resource limits per input
    pub per_input_limits: ResourceLimits,
}

/// Resource limits for individual inputs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ResourceLimits {
    /// Maximum execution time per input
    pub max_time_ms: u64,
    /// Maximum memory per input
    pub max_memory_bytes: u64,
    /// Maximum output size per input
    pub max_output_size_bytes: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_time_ms: 10_000, // 10 seconds
            max_memory_bytes: 1_048_576, // 1MB
            max_output_size_bytes: 65_536, // 64KB
        }
    }
}

impl Default for AggregationConstraints {
    fn default() -> Self {
        Self {
            max_total_time_ms: 60_000, // 1 minute
            max_memory_bytes: 10_485_760, // 10MB
            min_success_rate: 0.7, // 70%
            per_input_limits: ResourceLimits::default(),
        }
    }
}

/// Aggregation utility functions
impl AggregationContext {
    /// Create a new aggregation context with default settings
    pub fn new(mode: AggregationMode) -> Self {
        let strategy = match mode {
            AggregationMode::Merge => AggregationStrategy::SimpleMerge,
            AggregationMode::Reduce => AggregationStrategy::Reduce(ReductionFunction::Sum),
            AggregationMode::Vote => AggregationStrategy::MajorityVote,
            AggregationMode::Consensus => AggregationStrategy::Consensus(0.8),
        };
        
        Self {
            config: AggregationConfig::default(),
            strategy,
            input_weights: vec![],
            constraints: AggregationConstraints::default(),
        }
    }
    
    /// Validate aggregation configuration
    pub fn validate(&self) -> AggregationValidation {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        
        if self.config.min_inputs > self.config.max_inputs {
            issues.push("Minimum inputs cannot be greater than maximum inputs".to_string());
            recommendations.push("Set min_inputs <= max_inputs".to_string());
        }
        
        if self.config.consensus_threshold < 0.0 || self.config.consensus_threshold > 1.0 {
            issues.push("Consensus threshold must be between 0.0 and 1.0".to_string());
            recommendations.push("Set consensus_threshold between 0.0 and 1.0".to_string());
        }
        
        if self.constraints.min_success_rate < 0.0 || self.constraints.min_success_rate > 1.0 {
            issues.push("Minimum success rate must be between 0.0 and 1.0".to_string());
            recommendations.push("Set min_success_rate between 0.0 and 1.0".to_string());
        }
        
        // Check weight consistency
        if !self.input_weights.is_empty() {
            let weight_sum: f64 = self.input_weights.iter().sum();
            if (weight_sum - 1.0).abs() > 0.001 {
                issues.push("Input weights do not sum to 1.0".to_string());
                recommendations.push("Normalize input weights to sum to 1.0".to_string());
            }
        }
        
        AggregationValidation {
            is_valid: issues.is_empty(),
            issues,
            recommendations,
        }
    }
    
    /// Calculate expected execution time
    pub fn estimated_execution_time_ms(&self, input_count: u32) -> u64 {
        let per_input_time = self.constraints.per_input_limits.max_time_ms;
        let total_input_time = per_input_time * input_count as u64;
        
        // Add overhead for aggregation processing
        let aggregation_overhead = match self.strategy {
            AggregationStrategy::SimpleMerge => 100, // 100ms overhead
            AggregationStrategy::WeightedMerge(_) => 200,
            AggregationStrategy::MajorityVote => 500,
            AggregationStrategy::WeightedVote(_) => 600,
            AggregationStrategy::Consensus(_) => 1000,
            AggregationStrategy::Reduce(_) => 300,
            AggregationStrategy::Custom(_) => 1000,
        };
        
        total_input_time + aggregation_overhead
    }
    
    /// Calculate memory requirements
    pub fn estimated_memory_bytes(&self, input_count: u32) -> u64 {
        let per_input_memory = self.constraints.per_input_limits.max_memory_bytes;
        let total_input_memory = per_input_memory * input_count as u64;
        
        // Add overhead for aggregation processing
        let aggregation_overhead = match self.strategy {
            AggregationStrategy::SimpleMerge => per_input_memory / 2,
            AggregationStrategy::WeightedMerge(_) => per_input_memory,
            AggregationStrategy::MajorityVote => per_input_memory * 2,
            AggregationStrategy::WeightedVote(_) => per_input_memory * 2,
            AggregationStrategy::Consensus(_) => per_input_memory * 3,
            AggregationStrategy::Reduce(_) => per_input_memory,
            AggregationStrategy::Custom(_) => per_input_memory * 2,
        };
        
        total_input_memory + aggregation_overhead
    }
}

/// Aggregation execution utilities
pub struct AggregationExecutor;

impl AggregationExecutor {
    /// Execute aggregation with the given context
    pub fn execute(
        context: &AggregationContext,
        inputs: Vec<FunctionInput>,
        input_results: Vec<AggregationInputResult>,
    ) -> Result<FunctionOutput> {
        // Validate input count
        if inputs.len() < context.config.min_inputs as usize {
            return Err(error!(FunctionCompositionError::FunctionInsufficientInputs));
        }
        
        if inputs.len() > context.config.max_inputs as usize {
            return Err(error!(FunctionCompositionError::FunctionTooManyInputs));
        }
        
        // Calculate success rate
        let successful_inputs = input_results.iter().filter(|r| r.success).count();
        let success_rate = successful_inputs as f64 / input_results.len() as f64;
        
        if success_rate < context.constraints.min_success_rate {
            return Err(error!(FunctionCompositionError::FunctionInsufficientSuccessRate));
        }
        
        // Execute aggregation based on strategy
        let aggregated_output = match &context.strategy {
            AggregationStrategy::SimpleMerge => {
                Self::execute_simple_merge(&input_results)?
            }
            AggregationStrategy::WeightedMerge(weights) => {
                Self::execute_weighted_merge(&input_results, weights)?
            }
            AggregationStrategy::MajorityVote => {
                Self::execute_majority_vote(&input_results)?
            }
            AggregationStrategy::WeightedVote(weights) => {
                Self::execute_weighted_vote(&input_results, weights)?
            }
            AggregationStrategy::Consensus(threshold) => {
                Self::execute_consensus(&input_results, *threshold)?
            }
            AggregationStrategy::Reduce(function) => {
                Self::execute_reduce(&input_results, function)?
            }
            AggregationStrategy::Custom(logic) => {
                Self::execute_custom(&input_results, logic)?
            }
        };
        
        Ok(aggregated_output)
    }
    
    fn execute_simple_merge(results: &[AggregationInputResult]) -> Result<FunctionOutput> {
        let mut merged_data = Vec::new();
        
        for result in results {
            if result.success {
                merged_data.extend(result.output.data.clone());
            }
        }
        
        Ok(FunctionOutput {
            data: merged_data,
            version: "1.0.0".to_string(),
        })
    }
    
    fn execute_weighted_merge(results: &[AggregationInputResult], _weights: &[f64]) -> Result<FunctionOutput> {
        // For now, treat as simple merge
        // In a real implementation, this would apply weights to the merge
        Self::execute_simple_merge(results)
    }
    
    fn execute_majority_vote(results: &[AggregationInputResult]) -> Result<FunctionOutput> {
        // For now, return the first successful result
        // In a real implementation, this would find the most common output
        for result in results {
            if result.success {
                return Ok(result.output.clone());
            }
        }
        
        Err(error!(FunctionCompositionError::FunctionNoSuccessfulResults))
    }
    
    fn execute_weighted_vote(results: &[AggregationInputResult], _weights: &[f64]) -> Result<FunctionOutput> {
        // For now, treat as majority vote
        // In a real implementation, this would apply weights to the vote
        Self::execute_majority_vote(results)
    }
    
    fn execute_consensus(results: &[AggregationInputResult], _threshold: f64) -> Result<FunctionOutput> {
        // For now, treat as majority vote
        // In a real implementation, this would check for consensus above threshold
        Self::execute_majority_vote(results)
    }
    
    fn execute_reduce(results: &[AggregationInputResult], _function: &ReductionFunction) -> Result<FunctionOutput> {
        // For now, treat as simple merge
        // In a real implementation, this would apply the reduction function
        Self::execute_simple_merge(results)
    }
    
    fn execute_custom(results: &[AggregationInputResult], _logic: &str) -> Result<FunctionOutput> {
        // For now, treat as simple merge
        // In a real implementation, this would execute custom logic
        Self::execute_simple_merge(results)
    }
} 