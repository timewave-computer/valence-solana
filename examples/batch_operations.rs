//! Example: Batch Operations for Complex DeFi Flows
//! 
//! This example demonstrates when and how to use batch operations for
//! complex, dynamic scenarios that require atomic execution of multiple
//! steps with runtime-determined parameters.

use anchor_lang::prelude::*;
use valence_kernel::{
    instructions::batch_operations::{
        KernelOperation, OperationBatch, ExecuteBatch, 
        ACCESS_MODE_READ, ACCESS_MODE_READ_WRITE
    },
    state::Session,
    MAX_BATCH_ACCOUNTS, MAX_BATCH_OPERATIONS, MAX_CPI_ACCOUNT_INDICES, MAX_OPERATION_DATA_SIZE,
};

/// Example: Complex DeFi Operation - Liquidation with Dynamic Paths
/// 
/// Use Case: Liquidate an undercollateralized position by:
/// 1. Checking collateral value (oracle call)
/// 2. Calculating liquidation amount
/// 3. Repaying debt
/// 4. Seizing collateral
/// 5. Swapping to stable token
/// 
/// Why Batch Operation: 
/// - Multiple programs involved (oracle, lending, DEX)
/// - Account set determined at runtime based on user's positions
/// - All steps must succeed atomically
pub fn liquidate_position(
    ctx: Context<LiquidatePosition>,
    position_owner: Pubkey,
) -> Result<()> {
    msg!("Starting complex liquidation for position owner: {}", position_owner);
    
    // Build the account array - order matters for indices!
    let mut accounts = [Pubkey::default(); MAX_BATCH_ACCOUNTS];
    let mut account_count = 0;
    
    // Add oracle accounts
    accounts[0] = ctx.accounts.price_oracle.key();
    accounts[1] = ctx.accounts.collateral_price_feed.key();
    account_count = 2;
    
    // Add lending protocol accounts
    accounts[2] = ctx.accounts.lending_market.key();
    accounts[3] = ctx.accounts.debt_reserve.key();
    accounts[4] = ctx.accounts.collateral_reserve.key();
    accounts[5] = position_owner; // User's position account
    account_count = 6;
    
    // Add DEX accounts (determined at runtime based on best route)
    let (swap_program, pool_account) = find_best_swap_route(
        &ctx.accounts.collateral_mint.key(),
        &ctx.accounts.stable_mint.key(),
    )?;
    accounts[6] = swap_program;
    accounts[7] = pool_account;
    accounts[8] = ctx.accounts.liquidator_collateral_account.key();
    accounts[9] = ctx.accounts.liquidator_stable_account.key();
    account_count = 10;
    
    // Build the operation sequence
    let mut operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS] = Default::default();
    let mut op_count = 0;
    
    // Step 1: Borrow required accounts
    operations[0] = Some(KernelOperation::BorrowAccount {
        account_index: 1,  // collateral_price_feed
        mode: ACCESS_MODE_READ,
    });
    operations[1] = Some(KernelOperation::BorrowAccount {
        account_index: 5,  // user position
        mode: ACCESS_MODE_READ_WRITE,
    });
    operations[2] = Some(KernelOperation::BorrowAccount {
        account_index: 8,  // liquidator collateral account
        mode: ACCESS_MODE_READ_WRITE,
    });
    op_count = 3;
    
    // Step 2: Check collateral value via oracle
    operations[3] = Some(KernelOperation::CallRegisteredFunction {
        registry_id: 1001,  // ORACLE_FUNCTION_ID
        account_indices: [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        account_indices_len: 2,
        data: build_oracle_query_data()?,
        data_len: 32,
    });
    op_count = 4;
    
    // Step 3: Execute liquidation on lending protocol
    operations[4] = Some(KernelOperation::CallRegisteredFunction {
        registry_id: 2001,  // LENDING_LIQUIDATE_FUNCTION_ID
        account_indices: [2, 3, 4, 5, 8, 0, 0, 0, 0, 0, 0, 0],
        account_indices_len: 5,
        data: build_liquidation_data(position_owner)?,
        data_len: MAX_OPERATION_DATA_SIZE as u16,
    });
    op_count = 5;
    
    // Note: Simplified to fit within MAX_BATCH_OPERATIONS limit
    // In practice, you might split complex flows across multiple batches
    // Account release would happen in a separate batch or automatically
    
    // Execute the batch atomically
    let batch = OperationBatch {
        accounts,
        accounts_len: account_count as u8,
        operations,
        operations_len: op_count as u8,
    };
    
    // This example shows the concept - in practice, you'd call execute_batch
    // with a properly constructed ExecuteBatch context
    msg!("Would execute batch with {} operations on {} accounts", 
        batch.operations_len, batch.accounts_len);
    msg!("Liquidation batch prepared successfully");
    Ok(())
}

/// Example: Account Management Pattern
/// 
/// Use Case: Manage multiple accounts in a batch
/// Why Batch Operation: Multiple related operations need to be atomic
pub fn create_token_accounts(
    ctx: Context<CreateTokenAccounts>,
    token_configs: Vec<TokenConfig>,
) -> Result<()> {
    msg!("Managing {} accounts in batch", token_configs.len().min(3));
    
    let mut accounts = [Pubkey::default(); MAX_BATCH_ACCOUNTS];
    accounts[0] = ctx.accounts.session.key();
    accounts[1] = ctx.accounts.token_program.key();
    // Add other accounts as needed...
    
    let mut operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS] = Default::default();
    let mut op_count = 0;
    
    // Example: Borrow accounts for management
    operations[0] = Some(KernelOperation::BorrowAccount {
        account_index: 0,  // session
        mode: ACCESS_MODE_READ,
    });
    op_count = 1;
    
    if op_count < MAX_BATCH_OPERATIONS {
        operations[op_count] = Some(KernelOperation::BorrowAccount {
            account_index: 1,  // token program
            mode: ACCESS_MODE_READ,
        });
        op_count += 1;
    }
    
    // Note: CreateChildAccount operation was removed from the kernel
    // Use direct operations for account creation instead
    
    let batch = OperationBatch {
        accounts,
        accounts_len: 2,
        operations,
        operations_len: op_count as u8,
    };
    
    msg!("Would execute batch with {} operations", op_count);
    msg!("Note: This is a simplified example for demonstration");
    
    Ok(())
}

/// Example: Handling Async Messages
/// 
/// Use Case: Process governance proposal execution with dynamic operations
/// Why Batch Operation: Operations determined by governance vote results
pub fn execute_governance_proposal(
    ctx: Context<ExecuteProposal>,
    proposal: Proposal,
) -> Result<()> {
    msg!("Executing governance proposal: {}", proposal.id);
    
    let mut accounts = [Pubkey::default(); MAX_BATCH_ACCOUNTS];
    let mut operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS] = Default::default();
    let mut account_count = 0;
    let mut op_count = 0;
    
    // Parse proposal actions and build operations dynamically
    for action in proposal.actions.iter() {
        match action {
            ProposalAction::TransferFunds { from, to, amount } => {
                // Add accounts if not already present
                let from_idx = add_or_find_account(&mut accounts, &mut account_count, from)?;
                let to_idx = add_or_find_account(&mut accounts, &mut account_count, to)?;
                
                // Borrow accounts
                operations[op_count] = Some(KernelOperation::BorrowAccount {
                    account_index: from_idx,
                    mode: ACCESS_MODE_READ_WRITE,
                });
                op_count += 1;
                
                // Execute transfer
                operations[op_count] = Some(KernelOperation::CallRegisteredFunction {
                    registry_id: 4001,  // TREASURY_TRANSFER_FUNCTION
                    account_indices: build_account_indices(&[from_idx, to_idx]),
                    account_indices_len: 2,
                    data: amount.to_le_bytes().to_vec().try_into().unwrap(),
                    data_len: 8,
                });
                op_count += 1;
            }
            
            ProposalAction::UpdateParameter { param_id, value } => {
                // Dynamic parameter updates based on governance
                operations[op_count] = Some(KernelOperation::CallRegisteredFunction {
                    registry_id: 5001,  // PARAM_UPDATE_FUNCTION
                    account_indices: [0; MAX_CPI_ACCOUNT_INDICES],
                    account_indices_len: 1,
                    data: build_param_update_data(*param_id, *value)?,
                    data_len: MAX_OPERATION_DATA_SIZE as u16,
                });
                op_count += 1;
            }
            
            ProposalAction::EnableFeature { feature_flag } => {
                // Conditional feature activation
                if should_enable_feature(feature_flag) {
                    operations[op_count] = Some(KernelOperation::UnsafeRawCpi {
                        program_index: 0,  // Governance program
                        account_indices: [0; MAX_CPI_ACCOUNT_INDICES],
                        account_indices_len: 1,
                        data: build_feature_enable_data(*feature_flag)?,
                        data_len: MAX_OPERATION_DATA_SIZE as u16,
                    });
                    op_count += 1;
                }
            }
        }
    }
    
    // Execute all governance actions atomically
    let batch = OperationBatch {
        accounts,
        accounts_len: account_count as u8,
        operations,
        operations_len: op_count as u8,
    };
    
    // This example shows the concept - in practice, you'd call execute_batch
    // with a properly constructed ExecuteBatch context
    msg!("Would execute governance batch with {} operations", batch.operations_len);
    msg!("Governance proposal prepared successfully");
    Ok(())
}

// Helper structures and functions
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TokenConfig {
    pub name: [u8; 32],
    pub name_len: u8,
    pub decimals: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Proposal {
    pub id: u64,
    pub actions: Vec<ProposalAction>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum ProposalAction {
    TransferFunds { from: Pubkey, to: Pubkey, amount: u64 },
    UpdateParameter { param_id: u32, value: u64 },
    EnableFeature { feature_flag: u32 },
}

// Context structures
#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    pub session: Account<'info, Session>,
    pub authority: Signer<'info>,
    pub price_oracle: AccountInfo<'info>,
    pub collateral_price_feed: AccountInfo<'info>,
    pub lending_market: AccountInfo<'info>,
    pub debt_reserve: AccountInfo<'info>,
    pub collateral_reserve: AccountInfo<'info>,
    pub collateral_mint: AccountInfo<'info>,
    pub stable_mint: AccountInfo<'info>,
    pub liquidator_collateral_account: AccountInfo<'info>,
    pub liquidator_stable_account: AccountInfo<'info>,
    // ... other required accounts
}

#[derive(Accounts)]
pub struct CreateTokenAccounts<'info> {
    #[account(mut)]
    pub session: Account<'info, Session>,
    pub authority: Signer<'info>,
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    // ... other required accounts
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    pub session: Account<'info, Session>,
    pub authority: Signer<'info>,
    // ... governance accounts
}

// Mock helper functions
fn find_best_swap_route(_from: &Pubkey, _to: &Pubkey) -> Result<(Pubkey, Pubkey)> {
    // In reality, this would query multiple DEXs
    Ok((Pubkey::new_unique(), Pubkey::new_unique()))
}

fn build_oracle_query_data() -> Result<[u8; MAX_OPERATION_DATA_SIZE]> {
    let mut data = [0u8; MAX_OPERATION_DATA_SIZE];
    // Add oracle query parameters here
    data[0] = 1; // query type
    Ok(data)
}

fn build_liquidation_data(_user: Pubkey) -> Result<[u8; MAX_OPERATION_DATA_SIZE]> {
    let mut data = [0u8; MAX_OPERATION_DATA_SIZE];
    // Add liquidation parameters here
    data[0] = 2; // liquidation type
    Ok(data)
}

fn build_swap_data() -> Result<[u8; MAX_OPERATION_DATA_SIZE]> {
    let mut data = [0u8; MAX_OPERATION_DATA_SIZE];
    // Add swap parameters here
    data[0] = 3; // swap type
    Ok(data)
}

fn build_param_update_data(_id: u32, _value: u64) -> Result<[u8; MAX_OPERATION_DATA_SIZE]> {
    let mut data = [0u8; MAX_OPERATION_DATA_SIZE];
    // Add parameter update data here
    data[0] = 4; // update type
    Ok(data)
}

fn build_feature_enable_data(_flag: u32) -> Result<[u8; MAX_OPERATION_DATA_SIZE]> {
    let mut data = [0u8; MAX_OPERATION_DATA_SIZE];
    // Add feature flag data here
    data[0] = 5; // feature type
    Ok(data)
}

fn should_enable_feature(_flag: &u32) -> bool {
    true
}

fn add_or_find_account(accounts: &mut [Pubkey; MAX_BATCH_ACCOUNTS], count: &mut usize, account: &Pubkey) -> Result<u8> {
    // Check if already present
    for i in 0..*count {
        if accounts[i] == *account {
            return Ok(i as u8);
        }
    }
    
    // Add new account
    if *count >= MAX_BATCH_ACCOUNTS {
        return Err(ErrorCode::TooManyAccounts.into());
    }
    
    accounts[*count] = *account;
    let index = *count as u8;
    *count += 1;
    Ok(index)
}

fn build_account_indices(indices: &[u8]) -> [u8; MAX_CPI_ACCOUNT_INDICES] {
    let mut result = [0u8; MAX_CPI_ACCOUNT_INDICES];
    for (i, &idx) in indices.iter().enumerate().take(MAX_CPI_ACCOUNT_INDICES) {
        result[i] = idx;
    }
    result
}

// Note: In a real implementation, you would properly construct ExecuteBatch contexts
// The examples above use simplified validation instead of actual batch execution

#[error_code]
pub enum ErrorCode {
    #[msg("Too many accounts in batch")]
    TooManyAccounts,
}

// Summary: When to Use Batch Operations
// 
// Batch operations are essential for:
// 1. Complex multi-step operations (DeFi strategies, liquidations)
// 2. Dynamic account resolution (runtime-determined accounts)
// 3. Async message handling (governance, cross-program events)
// 4. Conditional execution flows (if-then-else logic)
// 5. Factory patterns (creating multiple accounts)
// 
// Benefits:
// - Atomic execution (all-or-nothing)
// - Dynamic flexibility (runtime composition)
// - Complex flows (multi-program interactions)
// - Async capability (handle external events)

fn main() {
    println!("Valence Batch Operations Example");
    println!("================================");
    println!();
    println!("This example demonstrates batch operations in Valence:");
    println!("1. Complex DeFi liquidation flows with multiple steps");
    println!("2. Account management patterns with simplified operations");
    println!("3. Governance proposal execution with dynamic operations");
    println!();
    println!("Key concepts:");
    println!("- Batch operations enable atomic multi-step execution");
    println!("- Operations use indices to reference a flat account array");
    println!("- Current limits: {} accounts, {} operations per batch", MAX_BATCH_ACCOUNTS, MAX_BATCH_OPERATIONS);
    println!("- CPI operations limited to {} bytes data, {} account indices", MAX_OPERATION_DATA_SIZE, MAX_CPI_ACCOUNT_INDICES);
    println!();
    println!("Note: Examples are simplified for demonstration and would need");
    println!("proper ExecuteBatch context construction for real execution.");
}