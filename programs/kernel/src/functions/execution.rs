// Core function execution with verification
use anchor_lang::prelude::*;
use crate::functions::instructions::{FunctionOutput};
use crate::sessions::isolation::Diff;

/// Main state for the function composition system
#[account]
pub struct FunctionCompositionState {
    /// Authority that manages the composition system
    pub authority: Pubkey,
    /// Total number of compositions created
    pub total_compositions: u64,
    /// Total number of chains created
    pub total_chains: u64,
    /// Total number of aggregations created
    pub total_aggregations: u64,
    /// Version for future upgrades
    pub version: u8,
    /// PDA bump seed
    pub bump: u8,
    /// Reserved space for future use
    pub _reserved: [u8; 64],
}

impl FunctionCompositionState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        8 + // total_compositions
        8 + // total_chains
        8 + // total_aggregations
        1 + // version
        1 + // bump
        64; // _reserved
}

/// A function composition chain
#[account]
pub struct FunctionChain {
    /// Unique identifier for this chain
    pub chain_id: String,
    /// Function steps in the chain
    pub function_steps: Vec<FunctionStep>,
    /// Execution mode for the chain
    pub execution_mode: ExecutionMode,
    /// When this chain was created
    pub created_at: i64,
    /// Last execution timestamp
    pub last_executed: i64,
    /// Total number of executions
    pub execution_count: u64,
    /// Whether this chain is active
    pub is_active: bool,
    /// Chain metadata and statistics
    pub metadata: ChainMetadata,
    /// PDA bump seed
    pub bump: u8,
}

impl FunctionChain {
    pub fn get_space(chain_id_len: usize, step_count: usize) -> usize {
        8 + // discriminator
        4 + chain_id_len + // chain_id string
        4 + (step_count * std::mem::size_of::<FunctionStep>()) + // function_steps vec
        std::mem::size_of::<ExecutionMode>() + // execution_mode
        8 + // created_at
        8 + // last_executed
        8 + // execution_count
        1 + // is_active
        std::mem::size_of::<ChainMetadata>() + // metadata
        1 // bump
    }
}

/// A function aggregation pattern
#[account]
pub struct FunctionAggregation {
    /// Unique identifier for this aggregation
    pub aggregation_id: String,
    /// Input functions that provide data
    pub input_functions: Vec<FunctionStep>,
    /// Function that aggregates the results
    pub aggregation_function: FunctionStep,
    /// Aggregation mode
    pub aggregation_mode: AggregationMode,
    /// When this aggregation was created
    pub created_at: i64,
    /// Last execution timestamp
    pub last_executed: i64,
    /// Total number of executions
    pub execution_count: u64,
    /// Whether this aggregation is active
    pub is_active: bool,
    /// Aggregation metadata and statistics
    pub metadata: AggregationMetadata,
    /// PDA bump seed
    pub bump: u8,
}

impl FunctionAggregation {
    pub fn get_space(aggregation_id_len: usize, input_count: usize) -> usize {
        8 + // discriminator
        4 + aggregation_id_len + // aggregation_id string
        4 + (input_count * std::mem::size_of::<FunctionStep>()) + // input_functions vec
        std::mem::size_of::<FunctionStep>() + // aggregation_function
        std::mem::size_of::<AggregationMode>() + // aggregation_mode
        8 + // created_at
        8 + // last_executed
        8 + // execution_count
        1 + // is_active
        std::mem::size_of::<AggregationMetadata>() + // metadata
        1 // bump
    }
}

/// A step in a function composition
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionStep {
    /// Hash/ID of the function to execute
    pub function_hash: String,
    /// Input transformation for this step
    pub input_transformation: InputTransformation,
    /// Output transformation for this step
    pub output_transformation: OutputTransformation,
    /// Dependencies on other steps (indices)
    pub dependencies: Vec<u32>,
    /// Conditional execution criteria
    pub condition: Option<StepCondition>,
    /// Step-specific configuration
    pub config: StepConfig,
}

/// Input transformation options
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum InputTransformation {
    /// Use input as-is
    PassThrough,
    /// Map previous step output to this step's input
    MapFromStep(u32),
    /// Merge outputs from multiple steps
    MergeFromSteps(Vec<u32>),
    /// Transform with custom logic
    Custom(String),
}

/// Output transformation options
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum OutputTransformation {
    /// Use output as-is
    PassThrough,
    /// Extract specific fields
    Extract(Vec<String>),
    /// Transform with custom logic
    Transform(String),
}

/// Execution mode for function chains
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ExecutionMode {
    /// Execute steps sequentially
    Sequential,
    /// Execute independent steps in parallel
    Parallel,
    /// Execute steps conditionally based on results
    Conditional,
}

/// Aggregation mode for function aggregations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum AggregationMode {
    /// Merge all outputs into a single result
    Merge,
    /// Apply reduction function across outputs
    Reduce,
    /// Vote on outputs and select winner
    Vote,
    /// Require consensus across all outputs
    Consensus,
}

/// Conditional execution criteria
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StepCondition {
    /// Type of condition check
    pub condition_type: ConditionType,
    /// Value to compare against
    pub expected_value: String,
    /// Whether to execute if condition is true or false
    pub execute_if_true: bool,
}

/// Types of conditions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ConditionType {
    /// Check if previous step succeeded
    PreviousStepSuccess,
    /// Check if previous step output equals value
    PreviousStepOutput,
    /// Check if diff count meets threshold
    DiffCountThreshold,
    /// Custom condition logic
    Custom(String),
}

/// Step-specific configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StepConfig {
    /// Maximum execution time in microseconds
    pub max_execution_time_us: u64,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Number of retry attempts on failure
    pub retry_attempts: u32,
    /// Whether to continue chain execution if this step fails
    pub continue_on_failure: bool,
}

impl Default for StepConfig {
    fn default() -> Self {
        Self {
            max_execution_time_us: 1_000_000, // 1 second default
            max_memory_bytes: 1_048_576, // 1MB default
            retry_attempts: 0,
            continue_on_failure: false,
        }
    }
}

/// Metadata for function chains
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ChainMetadata {
    /// Average execution time in microseconds
    pub avg_execution_time_us: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Most common failure reason
    pub most_common_failure: String,
    /// Performance statistics
    pub performance_stats: PerformanceStats,
}

impl Default for ChainMetadata {
    fn default() -> Self {
        Self {
            avg_execution_time_us: 0,
            successful_executions: 0,
            failed_executions: 0,
            most_common_failure: String::new(),
            performance_stats: PerformanceStats::default(),
        }
    }
}

/// Metadata for function aggregations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationMetadata {
    /// Average execution time in microseconds
    pub avg_execution_time_us: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Consensus success rate
    pub consensus_success_rate: f64,
    /// Performance statistics
    pub performance_stats: PerformanceStats,
}

impl Default for AggregationMetadata {
    fn default() -> Self {
        Self {
            avg_execution_time_us: 0,
            successful_executions: 0,
            failed_executions: 0,
            consensus_success_rate: 0.0,
            performance_stats: PerformanceStats::default(),
        }
    }
}

/// Performance statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceStats {
    /// Minimum execution time
    pub min_execution_time_us: u64,
    /// Maximum execution time
    pub max_execution_time_us: u64,
    /// Average memory usage
    pub avg_memory_bytes: u64,
    /// Peak memory usage
    pub peak_memory_bytes: u64,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            min_execution_time_us: 0,
            max_execution_time_us: 0,
            avg_memory_bytes: 0,
            peak_memory_bytes: 0,
        }
    }
}

/// Result of a function composition execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CompositionResult {
    /// Whether the composition succeeded
    pub success: bool,
    /// Results from individual steps
    pub step_results: Vec<StepResult>,
    /// Final output of the composition
    pub final_output: FunctionOutput,
    /// Diffs generated during execution
    pub generated_diffs: Vec<Diff>,
    /// Execution metadata
    pub execution_metadata: CompositionExecutionMetadata,
}

/// Result of a single step in a composition
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StepResult {
    /// Index of the step
    pub step_index: u32,
    /// Function hash that was executed
    pub function_hash: String,
    /// Whether the step succeeded
    pub success: bool,
    /// Output of the step
    pub output: FunctionOutput,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Result of a function aggregation execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationResult {
    /// Whether the aggregation succeeded
    pub success: bool,
    /// Results from input functions
    pub input_results: Vec<AggregationInputResult>,
    /// Aggregated final output
    pub aggregated_output: FunctionOutput,
    /// Aggregation execution metadata
    pub aggregation_metadata: AggregationExecutionMetadata,
}

/// Result of a single input function in an aggregation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationInputResult {
    /// Index of the input function
    pub input_index: u32,
    /// Function hash that was executed
    pub function_hash: String,
    /// Whether the input function succeeded
    pub success: bool,
    /// Output of the input function
    pub output: FunctionOutput,
    /// Execution time in microseconds
    pub execution_time_us: u64,
}

/// Execution metadata for compositions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CompositionExecutionMetadata {
    /// Total execution time in microseconds
    pub total_execution_time_us: u64,
    /// Number of steps executed
    pub steps_executed: u32,
    /// Number of parallel branches
    pub parallel_branches: u32,
    /// Memory used during execution
    pub memory_used: u64,
}

/// Execution metadata for aggregations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationExecutionMetadata {
    /// Total execution time in microseconds
    pub total_execution_time_us: u64,
    /// Number of inputs processed
    pub inputs_processed: u32,
    /// Whether consensus was reached
    pub consensus_reached: bool,
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f64,
} 