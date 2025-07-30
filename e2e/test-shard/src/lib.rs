#![allow(unexpected_cfgs)]
#![allow(deprecated)]

//! Test Shard - E2E testing shard that demonstrates kernel integration
//! 
//! This shard demonstrates:
//! 1. Creating sessions with the valence-kernel
//! 2. Using the function registry to call registered functions
//! 3. Perform both direct and batch operations
//! 4. Managing session lifecycle

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use valence_kernel::{
    cpi::accounts as kernel_accounts,
    instructions::batch_operations::{
        KernelOperation, OperationBatch, ACCESS_MODE_READ_WRITE,
    },
    CreateSessionParams, RegisteredAccount, RegisteredProgram,
    MAX_BATCH_ACCOUNTS, MAX_BATCH_OPERATIONS, MAX_CPI_ACCOUNT_INDICES, MAX_OPERATION_DATA_SIZE,
};

declare_id!("TestShard1111111111111111111111111111111111");

#[program]
pub mod test_shard {
    use super::*;

    /// Initialize the test shard
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let shard = &mut ctx.accounts.shard;
        shard.authority = ctx.accounts.authority.key();
        shard.kernel_program = ctx.accounts.kernel_program.key();
        shard.bump = ctx.bumps.shard;
        
        msg!("Test shard initialized");
        msg!("Authority: {}", shard.authority);
        msg!("Kernel program: {}", shard.kernel_program);
        
        Ok(())
    }

    /// Create a session using the kernel
    pub fn create_test_session(
        ctx: Context<CreateTestSession>,
        namespace: String,
    ) -> Result<()> {
        msg!("Creating test session with namespace: {}", namespace);

        // Create guard account first
        let guard_cpi_accounts = kernel_accounts::CreateGuardAccount {
            guard_account: ctx.accounts.guard_account.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        
        let guard_cpi_context = CpiContext::new(
            ctx.accounts.kernel_program.to_account_info(),
            guard_cpi_accounts,
        );
        
        valence_kernel::cpi::create_guard_account(
            guard_cpi_context,
            ctx.accounts.session.key(),
            true, // allow_unregistered_cpi for testing
        )?;

        // Create session parameters
        let mut namespace_path = [0u8; 128];
        let namespace_bytes = namespace.as_bytes();
        if namespace_bytes.len() > 128 {
            return Err(ErrorCode::InvalidNamespace.into());
        }
        namespace_path[..namespace_bytes.len()].copy_from_slice(namespace_bytes);
        
        let params = CreateSessionParams {
            namespace_path,
            namespace_path_len: namespace_bytes.len() as u16,
            metadata: [0u8; 64], // Empty metadata for testing
            parent_session: None, // No parent session
        };

        // Initial accounts to register
        let initial_borrowable = vec![
            RegisteredAccount {
                address: ctx.accounts.token_account_a.key(),
                permissions: ACCESS_MODE_READ_WRITE,
                label: *b"Token Account A                 ",
            },
            RegisteredAccount {
                address: ctx.accounts.token_account_b.key(),
                permissions: ACCESS_MODE_READ_WRITE,
                label: *b"Token Account B                 ",
            },
        ];

        let initial_programs = vec![
            RegisteredProgram {
                address: ctx.accounts.token_program.key(),
                active: true,
                label: *b"SPL Token Program               ",
            },
        ];

        // Create session through CPI
        let cpi_accounts = kernel_accounts::CreateSession {
            session: ctx.accounts.session.to_account_info(),
            account_lookup: ctx.accounts.account_lookup.to_account_info(),
            guard_account: ctx.accounts.guard_account.to_account_info(),
            owner: ctx.accounts.authority.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        
        let cpi_context = CpiContext::new(
            ctx.accounts.kernel_program.to_account_info(),
            cpi_accounts,
        );
        
        valence_kernel::cpi::create_session_account(
            cpi_context,
            ctx.accounts.shard.key(),
            params,
            initial_borrowable,
            initial_programs,
        )?;

        msg!("Test session created successfully");
        Ok(())
    }

    /// Execute a registered function using batch operations
    pub fn execute_registered_function(
        ctx: Context<ExecuteRegisteredFunction>,
        function_id: u64,
        amount: u64,
    ) -> Result<()> {
        msg!("Executing registered function {} with amount {}", function_id, amount);

        // Build the account array for batch operations
        let mut accounts = [Pubkey::default(); MAX_BATCH_ACCOUNTS];
        accounts[0] = ctx.accounts.session.key();
        accounts[1] = ctx.accounts.token_account_a.key();
        accounts[2] = ctx.accounts.token_account_b.key();
        accounts[3] = ctx.accounts.token_program.key();

        // Build the operation sequence
        let mut operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS] = Default::default();
        let mut op_count = 0;

        // Borrow the token accounts
        operations[op_count] = Some(KernelOperation::BorrowAccount {
            account_index: 1, // token_account_a
            mode: ACCESS_MODE_READ_WRITE,
        });
        op_count += 1;

        operations[op_count] = Some(KernelOperation::BorrowAccount {
            account_index: 2, // token_account_b
            mode: ACCESS_MODE_READ_WRITE,
        });
        op_count += 1;

        // Call the registered function
        let mut account_indices = [0u8; MAX_CPI_ACCOUNT_INDICES];
        account_indices[0] = 1; // token_account_a
        account_indices[1] = 2; // token_account_b
        account_indices[2] = 3; // token_program
        
        let mut data = [0u8; MAX_OPERATION_DATA_SIZE];
        let amount_bytes = amount.to_le_bytes();
        data[..8].copy_from_slice(&amount_bytes);
        
        operations[op_count] = Some(KernelOperation::CallRegisteredFunction {
            registry_id: function_id,
            account_indices,
            account_indices_len: 3,
            data,
            data_len: 8,
        });
        op_count += 1;

        // Release the borrowed accounts
        operations[op_count] = Some(KernelOperation::ReleaseAccount { account_index: 1 });
        op_count += 1;
        operations[op_count] = Some(KernelOperation::ReleaseAccount { account_index: 2 });
        op_count += 1;

        // Create the batch
        let batch = OperationBatch {
            accounts,
            accounts_len: 4,
            operations,
            operations_len: op_count as u8,
        };

        // Execute through CPI
        let cpi_accounts = kernel_accounts::ExecuteBatch {
            session: ctx.accounts.session.to_account_info(),
            guard_account: ctx.accounts.guard_account.to_account_info(),
            account_lookup: ctx.accounts.account_lookup.to_account_info(),
            cpi_allowlist: ctx.accounts.cpi_allowlist.to_account_info(),
            caller: ctx.accounts.authority.to_account_info(),
            tx_submitter: ctx.accounts.authority.to_account_info(),
            clock: ctx.accounts.clock.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_context = CpiContext::new(
            ctx.accounts.kernel_program.to_account_info(),
            cpi_accounts,
        );

        valence_kernel::cpi::execute_batch(cpi_context, batch)?;

        msg!("Registered function executed successfully");
        Ok(())
    }

    /// Perform a direct operation (SPL transfer)
    pub fn execute_direct_transfer(
        ctx: Context<ExecuteDirectTransfer>,
        amount: u64,
    ) -> Result<()> {
        msg!("Executing direct SPL transfer of {} tokens", amount);

        // Direct SPL transfer through kernel
        let cpi_accounts = kernel_accounts::SplTransfer {
            session: ctx.accounts.session.to_account_info(),
            guard_account: ctx.accounts.guard_account.to_account_info(),
            from: ctx.accounts.from_token_account.to_account_info(),
            to: ctx.accounts.to_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            clock: ctx.accounts.clock.to_account_info(),
        };

        let cpi_context = CpiContext::new(
            ctx.accounts.kernel_program.to_account_info(),
            cpi_accounts,
        );

        valence_kernel::cpi::spl_transfer(cpi_context, amount)?;

        msg!("Direct transfer completed successfully");
        Ok(())
    }
}

// ================================
// State Accounts
// ================================

#[account]
pub struct ShardState {
    pub authority: Pubkey,
    pub kernel_program: Pubkey,
    pub bump: u8,
}

impl ShardState {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}

// ================================
// Account Contexts
// ================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = ShardState::LEN,
        seeds = [b"shard"],
        bump
    )]
    pub shard: Box<Account<'info, ShardState>>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Kernel program to be used
    pub kernel_program: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTestSession<'info> {
    #[account(seeds = [b"shard"], bump = shard.bump)]
    pub shard: Box<Account<'info, ShardState>>,
    
    /// CHECK: Session account to be created by kernel
    #[account(mut)]
    pub session: AccountInfo<'info>,
    
    /// CHECK: Account lookup table to be created by kernel
    #[account(mut)]
    pub account_lookup: AccountInfo<'info>,
    
    /// CHECK: Guard account to be created by kernel
    #[account(mut)]
    pub guard_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// Token accounts to register in the session
    pub token_account_a: Box<Account<'info, TokenAccount>>,
    pub token_account_b: Box<Account<'info, TokenAccount>>,
    
    /// CHECK: Kernel program
    pub kernel_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteRegisteredFunction<'info> {
    #[account(seeds = [b"shard"], bump = shard.bump)]
    pub shard: Box<Account<'info, ShardState>>,
    
    /// CHECK: Session account managed by kernel
    #[account(mut)]
    pub session: AccountInfo<'info>,
    
    /// CHECK: Account lookup table for session
    pub account_lookup: AccountInfo<'info>,
    
    /// CHECK: CPI allowlist for security checks
    pub cpi_allowlist: AccountInfo<'info>,
    
    /// CHECK: Guard account for authorization
    pub guard_account: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub token_account_a: Box<Account<'info, TokenAccount>>,
    
    #[account(mut)]
    pub token_account_b: Box<Account<'info, TokenAccount>>,
    
    /// CHECK: Kernel program
    pub kernel_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ExecuteDirectTransfer<'info> {
    #[account(seeds = [b"shard"], bump = shard.bump)]
    pub shard: Box<Account<'info, ShardState>>,
    
    /// CHECK: Session account managed by kernel
    pub session: AccountInfo<'info>,
    
    /// CHECK: Guard account for authorization
    pub guard_account: AccountInfo<'info>,
    
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub from_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(mut)]
    pub to_token_account: Box<Account<'info, TokenAccount>>,
    
    /// CHECK: Kernel program
    pub kernel_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

// ================================
// Error Codes
// ================================

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid namespace")]
    InvalidNamespace,
}