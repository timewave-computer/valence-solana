/// Function metadata management for the Valence Protocol
/// This module handles function discovery, categorization, and information retrieval
use anchor_lang::prelude::*;
use crate::functions::instructions::FunctionInput;
use crate::functions::execution::{
    FunctionCompositionState, FunctionChain, FunctionStep, ExecutionMode, 
    FunctionAggregation, AggregationMode
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The authority initializing the composition system
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The composition state account
    #[account(
        init,
        payer = authority,
        space = FunctionCompositionState::SIZE,
        seeds = [b"function_composition"],
        bump
    )]
    pub composition_state: Account<'info, FunctionCompositionState>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(
    chain_id: String,
    function_steps: Vec<FunctionStep>,
    execution_mode: ExecutionMode
)]
pub struct CreateFunctionChain<'info> {
    /// The authority creating the chain
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The composition state account
    #[account(
        mut,
        seeds = [b"function_composition"],
        bump = composition_state.bump,
        has_one = authority
    )]
    pub composition_state: Account<'info, FunctionCompositionState>,
    
    /// The function chain being created
    #[account(
        init,
        payer = authority,
        space = FunctionChain::get_space(chain_id.len(), function_steps.len()),
        seeds = [
            b"function_chain",
            chain_id.as_bytes()
        ],
        bump
    )]
    pub function_chain: Account<'info, FunctionChain>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(input: FunctionInput)]
pub struct ExecuteFunctionChain<'info> {
    /// The caller executing the chain
    pub caller: Signer<'info>,
    
    /// The function chain to execute
    #[account(
        mut,
        seeds = [
            b"function_chain",
            function_chain.chain_id.as_bytes()
        ],
        bump = function_chain.bump
    )]
    pub function_chain: Account<'info, FunctionChain>,
}

#[derive(Accounts)]
#[instruction(
    aggregation_id: String,
    input_functions: Vec<FunctionStep>,
    aggregation_function: FunctionStep,
    aggregation_mode: AggregationMode
)]
pub struct CreateFunctionAggregation<'info> {
    /// The authority creating the aggregation
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The composition state account
    #[account(
        mut,
        seeds = [b"function_composition"],
        bump = composition_state.bump,
        has_one = authority
    )]
    pub composition_state: Account<'info, FunctionCompositionState>,
    
    /// The function aggregation being created
    #[account(
        init,
        payer = authority,
        space = FunctionAggregation::get_space(aggregation_id.len(), input_functions.len()),
        seeds = [
            b"function_aggregation",
            aggregation_id.as_bytes()
        ],
        bump
    )]
    pub function_aggregation: Account<'info, FunctionAggregation>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(inputs: Vec<FunctionInput>)]
pub struct ExecuteFunctionAggregation<'info> {
    /// The caller executing the aggregation
    pub caller: Signer<'info>,
    
    /// The function aggregation to execute
    #[account(
        mut,
        seeds = [
            b"function_aggregation",
            function_aggregation.aggregation_id.as_bytes()
        ],
        bump = function_aggregation.bump
    )]
    pub function_aggregation: Account<'info, FunctionAggregation>,
}

#[derive(Accounts)]
pub struct UpdateFunctionChain<'info> {
    /// The authority updating the chain
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The composition state account
    #[account(
        seeds = [b"function_composition"],
        bump = composition_state.bump,
        has_one = authority
    )]
    pub composition_state: Account<'info, FunctionCompositionState>,
    
    /// The function chain to update
    #[account(
        mut,
        seeds = [
            b"function_chain",
            function_chain.chain_id.as_bytes()
        ],
        bump = function_chain.bump
    )]
    pub function_chain: Account<'info, FunctionChain>,
}

#[derive(Accounts)]
pub struct DeactivateComposition<'info> {
    /// The authority deactivating the composition
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The composition state account
    #[account(
        seeds = [b"function_composition"],
        bump = composition_state.bump,
        has_one = authority
    )]
    pub composition_state: Account<'info, FunctionCompositionState>,
    
    /// The function chain to deactivate
    #[account(
        mut,
        seeds = [
            b"function_chain",
            function_chain.chain_id.as_bytes()
        ],
        bump = function_chain.bump
    )]
    pub function_chain: Account<'info, FunctionChain>,
} 