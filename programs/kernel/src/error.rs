/// Unified error definitions for the Valence system
/// This module consolidates all error types from various modules into a single namespace

use anchor_lang::prelude::*;

#[error_code]
pub enum ValenceError {
    // ==================== Core Errors ====================
    #[msg("Invalid input parameter")]
    InvalidInput,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("System is paused")]
    SystemPaused,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Compute limit exceeded")]
    ComputeLimitExceeded,
    #[msg("Conflicting constraints")]
    ConflictingConstraints,
    #[msg("Resource allocation failed")]
    ResourceAllocationFailed,
    #[msg("Execution failed")]
    ExecutionFailed,
    #[msg("Invalid state transition")]
    InvalidStateTransition,
    #[msg("Queue is full")]
    QueueFull,
    #[msg("Resource not available")]
    ResourceNotAvailable,
    #[msg("Priority conflict")]
    PriorityConflict,
    #[msg("Invalid ordering constraint")]
    InvalidOrderingConstraint,
    #[msg("Diff calculation failed")]
    DiffCalculationFailed,
    #[msg("Atomic operation failed")]
    AtomicOperationFailed,
    #[msg("Batch optimization failed")]
    BatchOptimizationFailed,
    #[msg("Invalid diff operation")]
    InvalidDiffOperation,
    #[msg("Diff size exceeded")]
    DiffSizeExceeded,
    #[msg("Snapshot not found")]
    SnapshotNotFound,
    #[msg("Max capacity reached")]
    MaxCapacityReached,
    #[msg("Processor is paused")]
    ProcessorPaused,
    #[msg("System is already paused")]
    SystemAlreadyPaused,
    #[msg("System is not paused")]
    SystemNotPaused,
    #[msg("Invalid program state")]
    InvalidProgramState,
    #[msg("Unknown error")]
    UnknownError,
    
    // ==================== Capability Errors ====================
    #[msg("Invalid capability configuration")]
    CapabilityInvalidConfig,
    #[msg("Capability not found")]
    CapabilityNotFound,
    #[msg("Unauthorized capability access")]
    CapabilityUnauthorizedAccess,
    #[msg("Namespace validation failed")]
    CapabilityNamespaceValidationFailed,
    #[msg("Object access denied")]
    CapabilityObjectAccessDenied,
    #[msg("Invalid capability ID")]
    CapabilityInvalidId,
    
    // ==================== Namespace Scoping Errors ====================
    #[msg("Invalid namespace capability - hash verification failed")]
    NamespaceInvalidCapability,
    #[msg("Object is not within the namespace capability")]
    NamespaceObjectNotInNamespace,
    #[msg("Object type is not allowed by the namespace capability")]
    NamespaceObjectTypeNotAllowed,
    #[msg("Too many objects accessed - exceeds capability limit")]
    NamespaceTooManyObjects,
    #[msg("Namespace capability is empty")]
    NamespaceEmptyCapability,
    #[msg("Cannot create sub-capability - parent capability insufficient")]
    NamespaceInsufficientParentCapability,
    #[msg("Capability composition failed - conflicting constraints")]
    NamespaceCapabilityCompositionFailed,
    #[msg("Function input does not match namespace capability")]
    NamespaceFunctionInputMismatch,
    
    // ==================== Session Errors ====================
    #[msg("Only the designated Eval can execute calls")]
    SessionUnauthorizedCaller,
    #[msg("Session is not active")]
    SessionNotActive,
    #[msg("Invalid eval program")]
    SessionInvalidEvalProgram,
    #[msg("Invalid owner")]
    SessionInvalidOwner,
    #[msg("Invalid target program")]
    SessionInvalidTargetProgram,
    #[msg("Call data too large - maximum 1024 bytes")]
    SessionCallDataTooLarge,
    #[msg("Namespace too long - maximum 32 characters")]
    SessionNamespaceTooLong,
    #[msg("Too many namespaces - maximum 10")]
    SessionTooManyNamespaces,
    #[msg("Session ID too long - maximum 64 characters")]
    SessionIdTooLong,
    #[msg("Data key too long - maximum 64 characters")]
    SessionDataKeyTooLong,
    #[msg("Data value too large - maximum 1024 bytes")]
    SessionDataValueTooLarge,
    #[msg("Data not found for key")]
    SessionDataNotFound,
    #[msg("Maximum data entries reached")]
    SessionMaxDataEntriesReached,
    #[msg("Cross-program invocation failed")]
    SessionCpiFailed,
    #[msg("Insufficient funds")]
    SessionInsufficientFunds,
    #[msg("Invalid token account")]
    SessionInvalidTokenAccount,
    #[msg("Token transfer failed")]
    SessionTokenTransferFailed,
    #[msg("SOL transfer failed")]
    SessionSolTransferFailed,
    #[msg("Invalid metadata")]
    SessionInvalidMetadata,
    #[msg("Session already closed")]
    SessionAlreadyClosed,
    #[msg("Session not found")]
    SessionNotFound,
    
    // ==================== Function Composition Errors ====================
    #[msg("You are not authorized to perform this operation")]
    FunctionNotAuthorized,
    #[msg("Chain ID cannot be empty")]
    FunctionEmptyChainId,
    #[msg("Chain ID is too long")]
    FunctionChainIdTooLong,
    #[msg("Aggregation ID cannot be empty")]
    FunctionEmptyAggregationId,
    #[msg("Aggregation ID is too long")]
    FunctionAggregationIdTooLong,
    #[msg("Function steps cannot be empty")]
    FunctionEmptyFunctionSteps,
    #[msg("Too many steps in function chain")]
    FunctionTooManySteps,
    #[msg("Input functions cannot be empty")]
    FunctionEmptyInputFunctions,
    #[msg("Too many input functions")]
    FunctionTooManyInputFunctions,
    #[msg("Invalid function hash")]
    FunctionInvalidFunctionHash,
    #[msg("Invalid dependency - step index out of bounds")]
    FunctionInvalidDependency,
    #[msg("Function chain is inactive")]
    FunctionChainInactive,
    #[msg("Function aggregation is inactive")]
    FunctionAggregationInactive,
    #[msg("Step execution failed")]
    FunctionStepExecutionFailed,
    #[msg("Chain execution failed")]
    FunctionChainExecutionFailed,
    #[msg("Aggregation execution failed")]
    FunctionAggregationExecutionFailed,
    #[msg("Input count mismatch")]
    FunctionInputCountMismatch,
    #[msg("Insufficient inputs for aggregation")]
    FunctionInsufficientInputs,
    #[msg("Too many inputs for aggregation")]
    FunctionTooManyInputs,
    #[msg("Insufficient success rate")]
    FunctionInsufficientSuccessRate,
    #[msg("No successful results to aggregate")]
    FunctionNoSuccessfulResults,
    #[msg("Circular dependency detected")]
    FunctionCircularDependency,
    #[msg("Execution timeout")]
    FunctionExecutionTimeout,
    #[msg("Memory limit exceeded")]
    FunctionMemoryLimitExceeded,
    #[msg("Invalid execution mode")]
    FunctionInvalidExecutionMode,
    #[msg("Invalid aggregation mode")]
    FunctionInvalidAggregationMode,
    #[msg("Invalid condition type")]
    FunctionInvalidConditionType,
    #[msg("Condition evaluation failed")]
    FunctionConditionEvaluationFailed,
    #[msg("Step configuration invalid")]
    FunctionInvalidStepConfiguration,
    #[msg("Aggregation configuration invalid")]
    FunctionInvalidAggregationConfiguration,
    #[msg("Consensus threshold not met")]
    FunctionConsensusThresholdNotMet,
    #[msg("Voting failed - no clear winner")]
    FunctionVotingFailed,
    #[msg("Reduction function failed")]
    FunctionReductionFunctionFailed,
    #[msg("Custom logic execution failed")]
    FunctionCustomLogicFailed,
    #[msg("Function step not found")]
    FunctionStepNotFound,
    #[msg("Composition validation failed")]
    FunctionCompositionValidationFailed,
    #[msg("Resource allocation failed")]
    FunctionResourceAllocationFailed,
    #[msg("Parallel execution failed")]
    FunctionParallelExecutionFailed,
    #[msg("Conditional execution failed")]
    FunctionConditionalExecutionFailed,
    #[msg("Output transformation failed")]
    FunctionOutputTransformationFailed,
    #[msg("Input transformation failed")]
    FunctionInputTransformationFailed,
    #[msg("Metadata update failed")]
    FunctionMetadataUpdateFailed,
    #[msg("Performance tracking failed")]
    FunctionPerformanceTrackingFailed,
    #[msg("Function execution failed")]
    FunctionExecutionFailed,
    #[msg("Function not found")]
    FunctionNotFound,
    
    // ==================== Verification Errors ====================
    #[msg("You are not authorized to perform this operation")]
    VerificationUnauthorized,
    #[msg("Invalid verification input")]
    VerificationInvalidInput,
    #[msg("Verification failed")]
    VerificationFailed,
    #[msg("Invalid verifier")]
    VerificationInvalidVerifier,
    #[msg("Verification timeout")]
    VerificationTimeout,
    #[msg("Invalid execution context")]
    VerificationInvalidExecutionContext,
    #[msg("Sender is not authorized to execute this capability")]
    VerificationSenderNotAuthorized,
    #[msg("Permission configuration is not active")]
    VerificationPermissionConfigNotActive,
    #[msg("Auth configuration is not active")]
    VerificationAuthConfigNotActive,
    #[msg("Caller is not authorized for this execution level")]
    VerificationCallerNotAuthorized,
    #[msg("Unauthorized caller")]
    VerificationUnauthorizedCaller,
    #[msg("Invalid execution level")]
    VerificationInvalidExecutionLevel,
    #[msg("Block state verification is not active")]
    VerificationBlockStateNotActive,
    #[msg("Invalid block order - current block must be greater than last execution")]
    VerificationInvalidBlockOrder,
    #[msg("Transaction is too stale - exceeds maximum block staleness")]
    VerificationTransactionTooStale,
    #[msg("Invalid block height")]
    VerificationInvalidBlockHeight,
    #[msg("Block state not found")]
    VerificationBlockStateNotFound,
    #[msg("Verification not authorized")]
    VerificationNotAuthorized,
    #[msg("Verification parameter size too large")]
    VerificationParamSizeTooLarge,
    #[msg("Verification amount exceeds limit")]
    VerificationAmountExceedsLimit,
    #[msg("Verification system auth required")]
    VerificationSystemAuthRequired,
    #[msg("Verification invalid session owner")]
    VerificationInvalidSessionOwner,
    #[msg("Verification invalid params")]
    VerificationInvalidParams,
    #[msg("Verification block height not reached")]
    VerificationBlockHeightNotReached,
    #[msg("Verification block height exceeded")]
    VerificationBlockHeightExceeded,
    #[msg("Verification block height not in range")]
    VerificationBlockHeightNotInRange,
    #[msg("Verification invalid condition type")]
    VerificationInvalidConditionType,
    #[msg("Verification session required")]
    VerificationSessionRequired,
    #[msg("Verification invalid session params")]
    VerificationInvalidSessionParams,
    #[msg("Verification session expired")]
    VerificationSessionExpired,
    #[msg("Verification invalid session type")]
    VerificationInvalidSessionType,
    #[msg("Verification session not initialized")]
    VerificationSessionNotInitialized,
    #[msg("Constraint configuration is not active")]
    VerificationConstraintConfigNotActive,
    #[msg("Amount exceeds maximum allowed")]
    VerificationAmountExceedsMaximum,
    #[msg("Amount is below minimum required")]
    VerificationAmountBelowMinimum,
    #[msg("Recipient is not in allowlist")]
    VerificationRecipientNotAllowed,
    #[msg("Slippage exceeds maximum tolerance")]
    VerificationSlippageExceedsTolerance,
    #[msg("Invalid call parameters")]
    VerificationInvalidCallParameters,
    #[msg("Session creation verification failed")]
    VerificationSessionCreationFailed,
    #[msg("Session is not properly registered")]
    VerificationSessionNotRegistered,
    #[msg("Session is closed or inactive")]
    VerificationSessionInactive,
    #[msg("Session data mismatch with registry")]
    VerificationSessionDataMismatch,
    #[msg("Function code is too large - maximum supported size exceeded")]
    VerificationFunctionCodeTooLarge,
    #[msg("Function name is too long")]
    VerificationFunctionNameTooLong,
    #[msg("Description string is too long")]
    VerificationDescriptionTooLong,
    #[msg("Version string is too long")]
    VerificationVersionTooLong,
    #[msg("Too many tags specified")]
    VerificationTooManyTags,
    #[msg("Too many dependencies specified")]
    VerificationTooManyDependencies,
    #[msg("Function hash mismatch - code does not match stored hash")]
    VerificationFunctionHashMismatch,
    #[msg("Function not found in registry")]
    VerificationFunctionNotFound,
    #[msg("Function is not approved for use")]
    VerificationFunctionNotApproved,
    #[msg("Function code cannot be empty")]
    VerificationFunctionCodeEmpty,
    #[msg("Function already exists with this hash")]
    VerificationFunctionAlreadyExists,
    #[msg("Invalid function type")]
    VerificationInvalidFunctionType,
    #[msg("Function usage count overflow")]
    VerificationFunctionUsageCountOverflow,
    #[msg("Total functions count overflow")]
    VerificationTotalFunctionsCountOverflow,
    #[msg("Search query limit too high")]
    VerificationSearchQueryLimitTooHigh,
    #[msg("Search query cannot be empty")]
    VerificationSearchQueryEmpty,
    #[msg("Search query is too long")]
    VerificationSearchQueryTooLong,
    #[msg("Dependency verification failed")]
    VerificationDependencyVerificationFailed,
    #[msg("Performance metrics calculation failed")]
    VerificationPerformanceMetricsCalculationFailed,
    #[msg("Gateway is not active")]
    VerificationGatewayNotActive,
    #[msg("Registry is not active")]
    VerificationRegistryNotActive,
    #[msg("Registry ID mismatch")]
    VerificationRegistryIdMismatch,
    #[msg("Proof type mismatch")]
    VerificationProofTypeMismatch,
    #[msg("Unsupported proof type")]
    VerificationUnsupportedProofType,
    #[msg("Invalid proof size")]
    VerificationInvalidProofSize,
    #[msg("Invalid verification key")]
    VerificationInvalidVerificationKey,
    #[msg("Empty verification key")]
    VerificationEmptyVerificationKey,
    #[msg("Verification key too large")]
    VerificationVerificationKeyTooLarge,
    #[msg("Proof type too long")]
    VerificationProofTypeTooLong,
    #[msg("Future block number not allowed")]
    VerificationFutureBlockNumberNotAllowed,
    #[msg("Block number too stale")]
    VerificationBlockNumberTooStale,
    #[msg("Invalid domain")]
    VerificationInvalidDomain,
    #[msg("Public inputs do not match")]
    VerificationPublicInputsMismatch,
    #[msg("Insufficient public inputs")]
    VerificationInsufficientPublicInputs,
    #[msg("Proof verification failed")]
    VerificationProofVerificationFailed,
    #[msg("Version mismatch")]
    VersionMismatch,
    #[msg("Constraint check failed")]
    ConstraintCheckFailed,
    
    // ==================== Execution Config Errors ====================
    #[msg("Invalid execution time - must be between 1 and 3600 seconds")]
    ExecutionConfigInvalidExecutionTime,
    #[msg("Invalid compute units - must be between 1 and 1,400,000")]
    ExecutionConfigInvalidComputeUnits,
    #[msg("Invalid gas limit - must be greater than 0")]
    ExecutionConfigInvalidGasLimit,
    #[msg("Invalid concurrency setting - max_concurrent must be between 1 and 10")]
    ExecutionConfigInvalidConcurrency,
    #[msg("Invalid memory limit - must be between 1 and 100MB")]
    ExecutionConfigInvalidMemoryLimit,
    #[msg("Invalid storage limit - must be between 1 and 1GB")]
    ExecutionConfigInvalidStorageLimit,
    #[msg("Invalid network limit - must be between 1 and 100 requests")]
    ExecutionConfigInvalidNetworkLimit,
    #[msg("Invalid depth limit - must be between 1 and 20")]
    ExecutionConfigInvalidDepthLimit,
}

// Type aliases for backward compatibility
pub type ProcessorError = ValenceError;
pub type SchedulerError = ValenceError;
pub type DiffError = ValenceError;
pub type CapabilityError = ValenceError;
pub type NamespaceScopingError = ValenceError;
pub type ValenceSessionError = ValenceError;
pub type FunctionCompositionError = ValenceError;
pub type VerificationError = ValenceError;
pub type ExecutionConfigError = ValenceError;

/// Registry-specific error codes (extending the unified system)
#[error_code]
pub enum RegistryError {
    /// Library registration errors (10000-10099)
    #[msg("Library already exists")]
    LibraryAlreadyExists = 10000,
    
    #[msg("Library not found")]
    LibraryNotFound = 10001,
    
    #[msg("Invalid library version")]
    InvalidLibraryVersion = 10002,
    
    #[msg("Library verification failed")]
    LibraryVerificationFailed = 10003,
    
    #[msg("Library metadata invalid")]
    LibraryMetadataInvalid = 10004,
    
    /// ZK program registration errors (10100-10199)
    #[msg("ZK program already exists")]
    ZkProgramAlreadyExists = 10100,
    
    #[msg("ZK program not found")]
    ZkProgramNotFound = 10101,
    
    #[msg("Invalid verification key")]
    InvalidVerificationKey = 10102,
    
    #[msg("ZK program verification failed")]
    ZkProgramVerificationFailed = 10103,
    
    #[msg("Invalid circuit constraints")]
    InvalidCircuitConstraints = 10104,
    
    /// Dependency management errors (10200-10299)
    #[msg("Dependency already exists")]
    DependencyAlreadyExists = 10200,
    
    #[msg("Dependency not found")]
    DependencyNotFound = 10201,
    
    #[msg("Circular dependency detected")]
    CircularDependency = 10202,
    
    #[msg("Dependency version conflict")]
    DependencyVersionConflict = 10203,
    
    #[msg("Unresolvable dependency")]
    UnresolvableDependency = 10204,
    
    /// Version compatibility errors (10300-10399)
    #[msg("Version incompatible")]
    VersionIncompatible = 10300,
    
    #[msg("Version requirement invalid")]
    VersionRequirementInvalid = 10301,
    
    #[msg("Breaking change detected")]
    BreakingChangeDetected = 10302,
    
    /// Registry management errors (10400-10499)
    #[msg("Registry not initialized")]
    RegistryNotInitialized = 10400,
    
    #[msg("Registry already initialized")]
    RegistryAlreadyInitialized = 10401,
    
    #[msg("Registry version mismatch")]
    RegistryVersionMismatch = 10402,
    
    #[msg("Registry migration required")]
    RegistryMigrationRequired = 10403,
}

/// Convert registry errors to unified Valence errors
/// This function manually converts registry errors to appropriate unified error categories
pub fn convert_registry_error(error: RegistryError) -> ValenceError {
    match error {
        RegistryError::LibraryNotFound | 
        RegistryError::ZkProgramNotFound | 
        RegistryError::DependencyNotFound => ValenceError::FunctionNotFound,
        
        RegistryError::LibraryVerificationFailed | 
        RegistryError::ZkProgramVerificationFailed => ValenceError::VerificationFailed,
        
        RegistryError::VersionIncompatible | 
        RegistryError::VersionRequirementInvalid => ValenceError::VersionMismatch,
        
        RegistryError::CircularDependency | 
        RegistryError::DependencyVersionConflict => ValenceError::ConstraintCheckFailed,
        
        RegistryError::RegistryNotInitialized => ValenceError::InvalidProgramState,
        
        _ => ValenceError::UnknownError,
    }
}

/// Registry-specific error handling macros
#[macro_export]
macro_rules! require_library_exists {
    ($library:expr) => {
        require!(
            $library.is_some(),
            RegistryError::LibraryNotFound
        );
    };
}

#[macro_export]
macro_rules! require_unique_library {
    ($library:expr) => {
        require!(
            $library.is_none(),
            RegistryError::LibraryAlreadyExists
        );
    };
}

#[macro_export]
macro_rules! require_valid_version {
    ($version:expr) => {
        require!(
            !$version.is_empty() && $version.len() <= 32,
            RegistryError::InvalidLibraryVersion
        );
    };
}

#[macro_export]
macro_rules! require_no_circular_dependency {
    ($dependencies:expr, $target:expr) => {
        require!(
            !$dependencies.contains($target),
            RegistryError::CircularDependency
        );
    };
} 