// Function composition system for Valence Protocol
// Enables pure function chaining and aggregation patterns
use anchor_lang::prelude::*;

// Import from our modules
use crate::functions::execution::{FunctionChain, FunctionAggregation, FunctionStep, ExecutionMode, AggregationMode};
use crate::error::FunctionCompositionError;

/// Simple structures for function composition
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionInput {
    pub data: Vec<u8>,
    pub version: String,
}

impl FunctionInput {
    pub fn new(data: Vec<u8>, version: String) -> Self {
        Self { data, version }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionOutput {
    pub data: Vec<u8>,
    pub version: String,
}

impl FunctionOutput {
    pub fn new(data: Vec<u8>, version: String) -> Self {
        Self { data, version }
    }
}

/// Composition result structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CompositionResult {
    pub success: bool,
    pub step_results: Vec<StepResult>,
    pub final_output: FunctionOutput,
    pub execution_time_us: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StepResult {
    pub step_index: u32,
    pub function_hash: String,
    pub success: bool,
    pub output: FunctionOutput,
    pub execution_time_us: u64,
    pub error_message: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationResult {
    pub success: bool,
    pub input_results: Vec<AggregationInputResult>,
    pub aggregated_output: FunctionOutput,
    pub execution_time_us: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationInputResult {
    pub input_index: u32,
    pub function_hash: String,
    pub success: bool,
    pub output: FunctionOutput,
    pub execution_time_us: u64,
}

/// Account structures
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8 + 8 + 1 + 1 + 64,
        seeds = [b"function_composition"],
        bump
    )]
    pub composition_state: Account<'info, FunctionCompositionState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(chain_id: String)]
pub struct CreateFunctionChain<'info> {
    #[account(mut)]
    pub composition_state: Account<'info, FunctionCompositionState>,
    #[account(
        init,
        payer = payer,
        space = 8 + FunctionChain::get_space(chain_id.len(), 10),
        seeds = [b"function_chain", chain_id.as_bytes()],
        bump
    )]
    pub function_chain: Account<'info, FunctionChain>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteFunctionChain<'info> {
    #[account(mut)]
    pub function_chain: Account<'info, FunctionChain>,
    pub executor: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(aggregation_id: String)]
pub struct CreateFunctionAggregation<'info> {
    #[account(mut)]
    pub composition_state: Account<'info, FunctionCompositionState>,
    #[account(
        init,
        payer = payer,
        space = 8 + FunctionAggregation::get_space(aggregation_id.len(), 10),
        seeds = [b"function_aggregation", aggregation_id.as_bytes()],
        bump
    )]
    pub function_aggregation: Account<'info, FunctionAggregation>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteFunctionAggregation<'info> {
    #[account(mut)]
    pub function_aggregation: Account<'info, FunctionAggregation>,
    pub executor: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateFunctionChain<'info> {
    #[account(mut)]
    pub function_chain: Account<'info, FunctionChain>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeactivateComposition<'info> {
    #[account(mut)]
    pub function_chain: Account<'info, FunctionChain>,
    pub authority: Signer<'info>,
}

/// State account for function composition system
#[account]
pub struct FunctionCompositionState {
    pub authority: Pubkey,
    pub total_compositions: u64,
    pub total_chains: u64,
    pub total_aggregations: u64,
    pub version: u8,
    pub bump: u8,
    pub _reserved: [u8; 64],
}

/// Initialize function composition system
pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let composition_state = &mut ctx.accounts.composition_state;
    
    // Initialize composition state
    composition_state.authority = ctx.accounts.authority.key();
    composition_state.total_compositions = 0;
    composition_state.total_chains = 0;
    composition_state.total_aggregations = 0;
    composition_state.version = 1;
    composition_state.bump = ctx.bumps.composition_state;
    composition_state._reserved = [0u8; 64];
    
    msg!("Function composition system initialized");
    
    Ok(())
}

/// Create a new function composition chain
pub fn create_function_chain(
    ctx: Context<CreateFunctionChain>,
    chain_id: String,
    function_steps: Vec<FunctionStep>,
    execution_mode: ExecutionMode,
) -> Result<()> {
    let composition_state = &mut ctx.accounts.composition_state;
    let function_chain = &mut ctx.accounts.function_chain;
    let clock = Clock::get()?;
    
    // Validate inputs
    require!(!chain_id.is_empty(), FunctionCompositionError::FunctionEmptyChainId);
    require!(chain_id.len() <= 64, FunctionCompositionError::FunctionChainIdTooLong);
    require!(!function_steps.is_empty(), FunctionCompositionError::FunctionEmptyFunctionSteps);
    require!(function_steps.len() <= 50, FunctionCompositionError::FunctionTooManySteps);
    
    // Initialize function chain
    function_chain.chain_id = chain_id.clone();
    function_chain.function_steps = function_steps;
    function_chain.execution_mode = execution_mode;
    function_chain.created_at = clock.unix_timestamp;
    function_chain.last_executed = 0;
    function_chain.execution_count = 0;
    function_chain.is_active = true;
    function_chain.bump = ctx.bumps.function_chain;
    
    // Update composition state
    composition_state.total_compositions = composition_state.total_compositions.saturating_add(1);
    composition_state.total_chains = composition_state.total_chains.saturating_add(1);
    
    msg!("Created function chain: {} with {} steps", chain_id, function_chain.function_steps.len());
    
    Ok(())
}

/// Execute a function composition chain
pub fn execute_function_chain(
    ctx: Context<ExecuteFunctionChain>,
    _input: FunctionInput,
) -> Result<CompositionResult> {
    let function_chain = &mut ctx.accounts.function_chain;
    let clock = Clock::get()?;
    
    // Verify chain is active
    require!(function_chain.is_active, FunctionCompositionError::FunctionChainInactive);
    
    // Simple execution simulation
    let mut step_results = Vec::new();
    let mut total_time = 0u64;
    
    for (i, step) in function_chain.function_steps.iter().enumerate() {
        let step_result = StepResult {
            step_index: i as u32,
            function_hash: step.function_hash.clone(),
            success: true,
            output: FunctionOutput::new(vec![], "1.0.0".to_string()),
            execution_time_us: 1000,
            error_message: None,
        };
        
        total_time += step_result.execution_time_us;
        step_results.push(step_result);
    }
    
    // Update execution stats
    function_chain.execution_count = function_chain.execution_count.saturating_add(1);
    function_chain.last_executed = clock.unix_timestamp;
    
    let result = CompositionResult {
        success: true,
        step_results,
        final_output: FunctionOutput::new(vec![], "1.0.0".to_string()),
        execution_time_us: total_time,
    };
    
    msg!(
        "Chain execution completed: {} - Success: {}, Steps: {}, Time: {}us",
        function_chain.chain_id,
        result.success,
        result.step_results.len(),
        total_time
    );
    
    Ok(result)
}

/// Create a function aggregation pattern
pub fn create_function_aggregation(
    ctx: Context<CreateFunctionAggregation>,
    aggregation_id: String,
    input_functions: Vec<FunctionStep>,
    aggregation_function: FunctionStep,
    aggregation_mode: AggregationMode,
) -> Result<()> {
    let composition_state = &mut ctx.accounts.composition_state;
    let function_aggregation = &mut ctx.accounts.function_aggregation;
    let clock = Clock::get()?;
    
    // Validate inputs
    require!(!aggregation_id.is_empty(), FunctionCompositionError::FunctionEmptyAggregationId);
    require!(aggregation_id.len() <= 64, FunctionCompositionError::FunctionAggregationIdTooLong);
    require!(!input_functions.is_empty(), FunctionCompositionError::FunctionEmptyInputFunctions);
    require!(input_functions.len() <= 20, FunctionCompositionError::FunctionTooManyInputFunctions);
    
    // Initialize function aggregation
    function_aggregation.aggregation_id = aggregation_id.clone();
    function_aggregation.input_functions = input_functions;
    function_aggregation.aggregation_function = aggregation_function;
    function_aggregation.aggregation_mode = aggregation_mode;
    function_aggregation.created_at = clock.unix_timestamp;
    function_aggregation.last_executed = 0;
    function_aggregation.execution_count = 0;
    function_aggregation.is_active = true;
    function_aggregation.bump = ctx.bumps.function_aggregation;
    
    // Update composition state
    composition_state.total_compositions = composition_state.total_compositions.saturating_add(1);
    composition_state.total_aggregations = composition_state.total_aggregations.saturating_add(1);
    
    msg!(
        "Created function aggregation: {} with {} input functions",
        aggregation_id,
        function_aggregation.input_functions.len()
    );
    
    Ok(())
}

/// Execute a function aggregation
pub fn execute_function_aggregation(
    ctx: Context<ExecuteFunctionAggregation>,
    inputs: Vec<FunctionInput>,
) -> Result<AggregationResult> {
    let function_aggregation = &mut ctx.accounts.function_aggregation;
    let clock = Clock::get()?;
    
    // Verify aggregation is active
    require!(function_aggregation.is_active, FunctionCompositionError::FunctionAggregationInactive);
    
    // Simple execution simulation
    let mut input_results = Vec::new();
    let mut total_time = 0u64;
    
    for (i, _input) in inputs.iter().enumerate() {
        let result = AggregationInputResult {
            input_index: i as u32,
            function_hash: function_aggregation.input_functions[i].function_hash.clone(),
            success: true,
            output: FunctionOutput::new(vec![], "1.0.0".to_string()),
            execution_time_us: 1000,
        };
        total_time += result.execution_time_us;
        input_results.push(result);
    }
    
    // Update execution stats
    function_aggregation.execution_count = function_aggregation.execution_count.saturating_add(1);
    function_aggregation.last_executed = clock.unix_timestamp;
    
    let result = AggregationResult {
        success: true,
        input_results,
        aggregated_output: FunctionOutput::new(vec![], "1.0.0".to_string()),
        execution_time_us: total_time,
    };
    
    msg!(
        "Aggregation execution completed: {} - Success: {}",
        function_aggregation.aggregation_id,
        result.success
    );
    
    Ok(result)
} 