use anchor_lang::prelude::*;
use valence_core::state::{Session, SessionVariant};
use valence_core::guards::Guard;
use valence_functions::guards::builtin::always_true_guard;

#[program]
pub mod hello_world {
    use super::*;

    pub fn initialize_hello_session(ctx: Context<InitializeHelloSession>) -> Result<()> {
        let session_account = &mut ctx.accounts.session;
        let owner_key = ctx.accounts.owner.key();
        let protocol_id = Pubkey::new_unique(); // Mock protocol ID
        let authz_state = Pubkey::new_unique(); // Mock authz state
        
        // Use a simple guard from valence-functions
        let simple_guard = always_true_guard();

        // Create a dynamic session
        let variant = SessionVariant::Dynamic {
            initial_states: vec![],
            expires_at: Clock::get()?.unix_timestamp + 3600, // Expires in 1 hour
            state_read_permissions: 0b1111,
            state_write_permissions: 0b1111,
            state_execute_permissions: 0b1111,
            state_cross_protocol_permissions: 0b1111,
        };

        // Initialize the session account
        *session_account = Session::Dynamic(valence_core::state::DynamicSessionData {
            owner: owner_key,
            protocol: protocol_id,
            authz_state,
            bound_states: vec![],
            state_read_permissions: 0b1111,
            state_write_permissions: 0b1111,
            state_execute_permissions: 0b1111,
            state_cross_protocol_permissions: 0b1111,
            expires_at: Clock::get()?.unix_timestamp + 3600,
            guard_hash: [0u8; 32], // Placeholder
            shared_data: Default::default(),
        });

        msg!("Hello Valence Session Initialized!");
        msg!("Owner: {}", owner_key);
        msg!("Protocol: {}", protocol_id);
        Ok(())
    }

    pub fn execute_hello_operation(ctx: Context<ExecuteHelloOperation>) -> Result<()> {
        let session = &ctx.accounts.session;
        let caller = &ctx.accounts.caller.key();
        let clock = &ctx.accounts.clock;

        // Evaluate the guard (always_true_guard should pass)
        let operation_data = vec![1, 2, 3]; // Mock operation data
        let guard_passed = session.guard.evaluate(session, caller, clock, &operation_data)?;

        if guard_passed {
            msg!("Guard passed! Executing hello operation.");
            // Your application logic here
        } else {
            msg!("Guard failed! Operation denied.");
            return Err(ProgramError::Custom(100).into()); // Example error
        }
        Ok(())
    }

    /// A new instruction to simulate a compute-intensive operation.
    pub fn process_data(ctx: Context<ProcessData>, data_items: Vec<u64>) -> Result<()> {
        msg!("Processing {} data items.", data_items.len());
        let mut sum: u64 = 0;
        for item in data_items {
            sum = sum.checked_add(item).unwrap_or(u64::MAX); // Simulate some work
        }
        msg!("Finished processing. Sum: {}", sum);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeHelloSession<'info> {
    #[account(init, payer = owner, space = 8 + 1000)] // Placeholder space
    pub session: Account<'info, Session>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteHelloOperation<'info> {
    #[account(mut)]
    pub session: Account<'info, Session>,
    pub caller: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ProcessData<'info> {
    // This instruction doesn't directly interact with a session for simplicity,
    // but in a real app, it might be part of a session-controlled flow.
    pub signer: Signer<'info>,
}