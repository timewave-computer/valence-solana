// Per-application execution configuration
// This module provides configuration for execution settings specific to each shard

use anchor_lang::prelude::*;
use crate::error::ExecutionConfigError;

/// Execution configuration for per-app settings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExecutionConfig {
    /// Maximum execution time allowed (seconds)
    pub max_execution_time: Option<u64>,
    /// Maximum compute units allowed per capability
    pub max_compute_units: Option<u64>,
    /// Whether to record execution results
    pub record_execution: bool,
    /// Additional parameters for execution
    pub parameters: Option<Vec<u8>>,
    /// Gas limit for execution
    pub gas_limit: Option<u64>,
    /// Priority level for execution (0-255)
    pub priority: u8,
    /// Whether to enable parallel execution
    pub enable_parallel: bool,
    /// Maximum number of concurrent executions
    pub max_concurrent: u8,
}

impl ExecutionConfig {
    /// Create a new execution config with defaults
    pub fn new() -> Self {
        Self {
            max_execution_time: Some(60), // 60 seconds default
            max_compute_units: Some(200_000), // Default compute units
            record_execution: false,
            parameters: None,
            gas_limit: None,
            priority: 128, // Medium priority
            enable_parallel: false,
            max_concurrent: 1,
        }
    }
    
    /// Set maximum execution time
    #[must_use]
    pub fn with_max_execution_time(mut self, max_time: u64) -> Self {
        self.max_execution_time = Some(max_time);
        self
    }
    
    /// Set maximum compute units
    #[must_use]
    pub fn with_max_compute_units(mut self, max_compute: u64) -> Self {
        self.max_compute_units = Some(max_compute);
        self
    }
    
    /// Enable execution recording
    #[must_use]
    pub fn with_recording(mut self, record: bool) -> Self {
        self.record_execution = record;
        self
    }
    
    /// Set execution parameters
    #[must_use]
    pub fn with_parameters(mut self, params: Vec<u8>) -> Self {
        self.parameters = Some(params);
        self
    }
    
    /// Set gas limit
    pub fn with_gas_limit(mut self, gas: u64) -> Self {
        self.gas_limit = Some(gas);
        self
    }
    
    /// Set execution priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    /// Enable parallel execution
    pub fn with_parallel_execution(mut self, enable: bool, max_concurrent: u8) -> Self {
        self.enable_parallel = enable;
        self.max_concurrent = max_concurrent;
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate execution time
        if let Some(max_time) = self.max_execution_time {
            require!(max_time > 0 && max_time <= 3600, ExecutionConfigError::ExecutionConfigInvalidExecutionTime);
        }
        
        // Validate compute units
        if let Some(max_compute) = self.max_compute_units {
            require!(max_compute > 0 && max_compute <= 1_400_000, ExecutionConfigError::ExecutionConfigInvalidComputeUnits);
        }
        
        // Validate gas limit
        if let Some(gas) = self.gas_limit {
            require!(gas > 0, ExecutionConfigError::ExecutionConfigInvalidGasLimit);
        }
        
        // Validate concurrency settings
        if self.enable_parallel {
            require!(self.max_concurrent > 0 && self.max_concurrent <= 10, ExecutionConfigError::ExecutionConfigInvalidConcurrency);
        }
        
        Ok(())
    }
    
    /// Merge with another configuration (self takes precedence)
    #[must_use]
    pub fn merge_with(&self, other: &ExecutionConfig) -> ExecutionConfig {
        ExecutionConfig {
            max_execution_time: self.max_execution_time.or(other.max_execution_time),
            max_compute_units: self.max_compute_units.or(other.max_compute_units),
            record_execution: self.record_execution || other.record_execution,
            parameters: self.parameters.clone().or_else(|| other.parameters.clone()),
            gas_limit: self.gas_limit.or(other.gas_limit),
            priority: if self.priority != 128 { self.priority } else { other.priority },
            enable_parallel: self.enable_parallel || other.enable_parallel,
            max_concurrent: self.max_concurrent.max(other.max_concurrent),
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for different execution modes
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub enum ExecutionMode {
    /// Synchronous execution - wait for completion
    #[default]
    Synchronous,
    /// Asynchronous execution - return immediately
    Asynchronous,
    /// Batch execution - group multiple executions
    Batch { batch_size: u8 },
    /// Streaming execution - process in chunks
    Streaming { chunk_size: u32 },
}

/// Resource limits for execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory: Option<u64>,
    /// Maximum storage usage (bytes)
    pub max_storage: Option<u64>,
    /// Maximum network requests
    pub max_network_requests: Option<u32>,
    /// Maximum execution depth
    pub max_depth: Option<u8>,
}

impl ResourceLimits {
    pub fn new() -> Self {
        Self {
            max_memory: Some(1_000_000), // 1MB default
            max_storage: Some(10_000_000), // 10MB default
            max_network_requests: Some(10),
            max_depth: Some(5),
        }
    }
    
    pub fn validate(&self) -> Result<()> {
        if let Some(memory) = self.max_memory {
            require!(memory > 0 && memory <= 100_000_000, ExecutionConfigError::ExecutionConfigInvalidMemoryLimit);
        }
        
        if let Some(storage) = self.max_storage {
            require!(storage > 0 && storage <= 1_000_000_000, ExecutionConfigError::ExecutionConfigInvalidStorageLimit);
        }
        
        if let Some(requests) = self.max_network_requests {
            require!(requests > 0 && requests <= 100, ExecutionConfigError::ExecutionConfigInvalidNetworkLimit);
        }
        
        if let Some(depth) = self.max_depth {
            require!(depth > 0 && depth <= 20, ExecutionConfigError::ExecutionConfigInvalidDepthLimit);
        }
        
        Ok(())
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete execution configuration with all settings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CompleteExecutionConfig {
    /// Basic execution configuration
    pub execution: ExecutionConfig,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Resource limits
    pub resources: ResourceLimits,
    /// Custom configuration data
    pub custom_config: Option<Vec<u8>>,
}

impl CompleteExecutionConfig {
    pub fn new() -> Self {
        Self {
            execution: ExecutionConfig::default(),
            mode: ExecutionMode::default(),
            resources: ResourceLimits::default(),
            custom_config: None,
        }
    }
    
    pub fn validate(&self) -> Result<()> {
        self.execution.validate()?;
        self.resources.validate()?;
        Ok(())
    }
}

impl Default for CompleteExecutionConfig {
    fn default() -> Self {
        Self::new()
    }
}

 