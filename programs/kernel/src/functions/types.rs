/// Core function type definitions for the Valence Protocol
use anchor_lang::prelude::*;

/// Function result type alias
pub type FunctionResult<T> = Result<T>;

/// Function error enumeration
#[error_code]
pub enum FunctionError {
    #[msg("Invalid function input")]
    InvalidInput,
    #[msg("Function execution failed")]
    ExecutionFailed,
    #[msg("Invalid function output")]
    InvalidOutput,
    #[msg("Function not found")]
    FunctionNotFound,
    #[msg("Insufficient permissions")]
    InsufficientPermissions,
    #[msg("Function timeout")]
    Timeout,
}

/// Function input structure (this should be the same as in instructions.rs)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionInput {
    /// Function identifier
    pub function_id: String,
    /// Input parameters
    pub parameters: Vec<u8>,
    /// Execution context
    pub eval_context: EvalContext,
    /// Available namespaces
    pub namespaces: Vec<String>,
}

/// Function output structure (this should be the same as in instructions.rs)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionOutput {
    /// Function identifier
    pub function_id: String,
    /// Output data
    pub data: Vec<u8>,
    /// Execution success flag
    pub success: bool,
    /// Error message if any
    pub error_message: Option<String>,
}

/// Evaluation context for function execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EvalContext {
    /// Current block height
    pub block_height: u64,
    /// Current timestamp
    pub timestamp: i64,
    /// Transaction signers
    pub signers: Vec<Pubkey>,
    /// Transaction fees
    pub fees: TransactionFees,
}

/// Transaction fee structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TransactionFees {
    pub base_fee: u64,
    pub priority_fee: u64,
}

impl EvalContext {
    /// Get transaction signers
    pub fn transaction_signers(&self) -> &[Pubkey] {
        &self.signers
    }
    
    /// Get transaction fees
    pub fn transaction_fees(&self) -> &TransactionFees {
        &self.fees
    }
}

impl FunctionInput {
    /// Deserialize parameters as a specific type
    pub fn deserialize_parameters<T: borsh::BorshDeserialize>(&self) -> Result<T> {
        T::try_from_slice(&self.parameters)
            .map_err(|_| FunctionError::InvalidInput.into())
    }
    
    /// Check if input has access to a namespace
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.namespaces.contains(&namespace.to_string())
    }
    
    /// Check if input can access an object (simplified implementation)
    pub fn can_access_object(&self, _object_id: &str) -> bool {
        // Simplified - would implement proper access control
        true
    }
    
    /// Get current block height
    pub fn block_height(&self) -> u64 {
        self.eval_context.block_height
    }
    
    /// Get current timestamp
    pub fn timestamp(&self) -> i64 {
        self.eval_context.timestamp
    }
}

impl FunctionOutput {
    /// Deserialize output data as a specific type
    pub fn deserialize_data<T: borsh::BorshDeserialize>(&self) -> Result<T> {
        T::try_from_slice(&self.data)
            .map_err(|_| FunctionError::InvalidOutput.into())
    }
}

/// Pure function trait for verification functions
pub trait PureFunction {
    /// Get the function type identifier
    fn function_type(&self) -> String;
    
    /// Get the schema version
    fn schema_version(&self) -> String;
    
    /// Validate input parameters
    fn validate_input(&self, input: &FunctionInput) -> Result<()>;
    
    /// Validate output data
    fn validate_output(&self, output: &FunctionOutput) -> Result<()>;
} 