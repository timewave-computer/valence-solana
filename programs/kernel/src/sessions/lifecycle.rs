// Session factory for coordinated session creation with lifecycle management
use anchor_lang::prelude::*;
use crate::sessions::state::{SessionReservation, SessionConfiguration};
use crate::scheduler::SessionOperationQueue;

// Known session program ID - this should match the actual session program ID
const SESSION_PROGRAM_ID: Pubkey = pubkey!("Sess1on111111111111111111111111111111111111");

/// Session Factory State - tracks all sessions and their metadata
#[account]
pub struct SessionFactoryState {
    /// Authority that can update the factory
    pub authority: Pubkey,
    /// Total number of sessions created
    pub total_sessions: u64,
    /// Total number of active sessions
    pub active_sessions: u64,
    /// Program version
    pub version: u8,
    /// PDA bump seed
    pub bump: u8,
    /// Reserved space for future use
    pub _reserved: [u8; 32],
}

impl SessionFactoryState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        8 + // total_sessions
        8 + // active_sessions
        1 + // version
        1 + // bump
        32; // reserved
}

/// Session Registry Entry - tracks individual session metadata
#[account]
pub struct SessionEntry {
    /// The session's PDA address
    pub session_address: Pubkey,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to this session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Namespaces this session has access to
    pub namespaces: Vec<String>,
    /// Session status (on-chain state)
    pub status: SessionStatus,
    /// Optimistic state for UX (can be pending while status is requested)
    pub optimistic_state: SessionState,
    /// When session was requested
    pub created_at: i64,
    /// When session was last updated
    pub last_updated: i64,
    /// When session was activated (account created and initialized)
    pub activated_at: Option<i64>,
    /// When optimistic state was set
    pub optimistic_state_set_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl SessionEntry {
    pub fn get_space(session_id: &str, namespaces: &[String]) -> usize {
        8 + // discriminator
        32 + // session_address
        32 + // owner
        32 + // eval_program
        32 + // shard_address
        4 + session_id.len() + // session_id
        4 + namespaces.iter().map(|ns| 4 + ns.len()).sum::<usize>() + // namespaces
        1 + // status
        1 + // optimistic_state
        8 + // created_at
        8 + // last_updated
        1 + 8 + // activated_at (Option<i64>)
        8 + // optimistic_state_set_at
        1 + // bump
        64 // padding
    }
}

/// Session status tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum SessionStatus {
    /// Session creation requested but account not yet created
    Requested,
    /// Account created but not yet initialized
    Created,
    /// Session fully initialized and active
    Active,
    /// Session has been closed/deactivated
    Closed,
}

/// Optimistic session state for UX improvement
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum SessionState {
    /// Session is pending activation (optimistic)
    Pending,
    /// Session is fully active and initialized
    Active,
    /// Session has been closed
    Closed,
}

/// Session-specific settings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionSettings {
    /// Auto-expire session after max_duration
    pub auto_expire: bool,
    /// Log all session activity
    pub enable_logging: bool,
    /// Performance monitoring
    pub enable_monitoring: bool,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            auto_expire: true,
            enable_logging: false,
            enable_monitoring: false,
        }
    }
}

// Queue management has been moved to scheduler singleton
// See scheduler/session_queue.rs for queue implementation

/// Initialize the session factory
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The factory state account
    #[account(
        init,
        payer = authority,
        space = SessionFactoryState::SIZE,
        seeds = [b"session_factory"],
        bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
    
    /// The authority initializing the factory
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

/// Request session creation by computing PDA and emitting event for off-chain service
#[derive(Accounts)]
#[instruction(owner: Pubkey, eval_program: Pubkey, session_id: String, namespaces: Vec<String>)]
pub struct RequestSessionCreation<'info> {
    /// The user requesting session creation
    #[account(mut)]
    pub requester: Signer<'info>,
    
    /// The factory state
    #[account(
        mut,
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
    
    /// The session entry to create
    #[account(
        init,
        payer = requester,
        space = SessionEntry::get_space(&session_id, &namespaces),
        seeds = [b"session_entry", owner.as_ref(), session_id.as_bytes()],
        bump
    )]
    pub session_entry: Account<'info, SessionEntry>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

/// Update session status (called by off-chain service or session program)
#[derive(Accounts)]
pub struct UpdateSessionStatus<'info> {
    /// The session entry to update
    #[account(
        mut,
        seeds = [b"session_entry", session_entry.owner.as_ref(), session_entry.session_id.as_bytes()],
        bump = session_entry.bump
    )]
    pub session_entry: Account<'info, SessionEntry>,
    
    /// The factory state
    #[account(
        mut,
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
    
    /// Authority or session program (for CPI)
    pub authority: Signer<'info>,
}

/// Close session and clean up registry
#[derive(Accounts)]
pub struct CloseSession<'info> {
    /// The session entry to close
    #[account(
        mut,
        close = refund_receiver,
        seeds = [b"session_entry", session_entry.owner.as_ref(), session_entry.session_id.as_bytes()],
        bump = session_entry.bump
    )]
    pub session_entry: Account<'info, SessionEntry>,
    
    /// The factory state
    #[account(
        mut,
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
    
    /// Session owner or authority
    pub authority: Signer<'info>,
    
    /// Account to receive refunded rent
    /// CHECK: This is safe as we're just transferring lamports
    #[account(mut)]
    pub refund_receiver: AccountInfo<'info>,
}

/// Reserve session ID for two-phase creation
#[derive(Accounts)]
#[instruction(reservation_id: String, session_id: String, session_owner: Pubkey, template_id: Option<String>)]
pub struct ReserveSession<'info> {
    /// The user making the reservation
    #[account(mut)]
    pub reserver: Signer<'info>,
    
    /// The factory state
    #[account(
        mut,
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
    
    /// The session reservation account
    #[account(
        init,
        payer = reserver,
        space = SessionReservation::get_space(
            reservation_id.len(),
            session_id.len(),
            template_id.as_ref().map(|t| t.len())
        ),
        seeds = [b"session_reservation", reserver.key().as_ref(), reservation_id.as_bytes()],
        bump
    )]
    pub session_reservation: Account<'info, SessionReservation>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

/// Create optimistic session (pending state for better UX)
#[derive(Accounts)]
#[instruction(owner: Pubkey, session_id: String)]
pub struct CreateOptimisticSession<'info> {
    /// The user creating the optimistic session
    #[account(mut)]
    pub creator: Signer<'info>,
    
    /// The factory state
    #[account(
        mut,
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
    
    /// The session entry to create optimistically
    #[account(
        init,
        payer = creator,
        space = SessionEntry::get_space(&session_id, &[]), // Start with empty namespaces
        seeds = [b"session_entry", owner.as_ref(), session_id.as_bytes()],
        bump
    )]
    pub session_entry: Account<'info, SessionEntry>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

// Queue-related operations have been moved to the scheduler singleton
// For batch processing and queue management, use the scheduler program via CPI
// See scheduler/session_queue.rs for the new queue implementation
//
// The following account contexts have been deprecated:
// - BatchActivateOptimisticSessions: Use scheduler's batch processing
// - InitializeSessionQueue: Use scheduler's queue initialization
// - QueueSessionInit: Use scheduler's enqueue_operation
// - ExecuteQueuedInit: Use scheduler's process_batch
// - BatchProcessQueue: Use scheduler's batch processing
//
// Manual operations remain in this module:

/// Manual session initialization (direct path without queue)
#[derive(Accounts)]
#[instruction(session_id: String)]
pub struct ManualInitializeSession<'info> {
    /// The session owner or authorized party
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The session entry to initialize
    #[account(
        mut,
        seeds = [b"session_entry", session_entry.owner.as_ref(), session_id.as_bytes()],
        bump = session_entry.bump,
        constraint = session_entry.owner == authority.key() || session_entry.owner == session_entry.owner
    )]
    pub session_entry: Account<'info, SessionEntry>,
    
    /// Factory state for statistics
    #[account(
        mut,
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
}

/// Emergency reset session state (for failed operations)
#[derive(Accounts)]
#[instruction(session_id: String)]
pub struct EmergencyResetSession<'info> {
    /// The session owner or factory authority
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// The session entry to reset
    #[account(
        mut,
        seeds = [b"session_entry", session_entry.owner.as_ref(), session_id.as_bytes()],
        bump = session_entry.bump,
        constraint = session_entry.owner == authority.key() || authority.key() == factory_state.authority
    )]
    pub session_entry: Account<'info, SessionEntry>,
    
    /// Factory state for verification
    #[account(
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
}

// The following context has been moved to the scheduler:
// BatchProcessQueue - use scheduler singleton instead

#[derive(Accounts)]
pub struct ProcessViaScheduler<'info> {
    /// The processor (can be anyone)
    
    /// The session initialization queue
    #[account(
        mut,
        seeds = [b"session_init_queue"],
        bump = init_queue.bump
    )]
    pub init_queue: Account<'info, SessionOperationQueue>,
    
    /// Factory state for statistics
    #[account(
        mut,
        seeds = [b"session_factory"],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, SessionFactoryState>,
}

// Convert from #[program] to regular module
pub mod session_factory {
    use super::*;

    /// Initialize the session factory
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let factory_state = &mut ctx.accounts.factory_state;
        
        factory_state.authority = ctx.accounts.authority.key();
        factory_state.total_sessions = 0;
        factory_state.active_sessions = 0;
        factory_state.version = 1;
        factory_state.bump = ctx.bumps.factory_state;
        factory_state._reserved = [0u8; 32];
        
        msg!("Session factory initialized with authority: {}", factory_state.authority);
        Ok(())
    }

    /// Request off-chain session creation by emitting PDA computation event
    pub fn request_session_creation(
        ctx: Context<RequestSessionCreation>,
        owner: Pubkey,
        eval_program: Pubkey,
        session_id: String,
        namespaces: Vec<String>,
        shard_address: Pubkey, // Add shard tracking
    ) -> Result<()> {
        // Validate inputs
        require!(
            owner != Pubkey::default(),
            SessionFactoryError::InvalidOwner
        );
        require!(
            eval_program != Pubkey::default(),
            SessionFactoryError::InvalidEvalProgram
        );
        require!(
            shard_address != Pubkey::default(),
            SessionFactoryError::InvalidShard
        );
        require!(
            !session_id.is_empty() && session_id.len() <= 64,
            SessionFactoryError::InvalidSessionId
        );
        require!(
            namespaces.len() <= 10,
            SessionFactoryError::TooManyNamespaces
        );
        
        // Compute the session PDA (same as session program uses)
        let (session_pda, _bump) = Pubkey::find_program_address(
            &[b"session", owner.as_ref(), session_id.as_bytes()],
            &SESSION_PROGRAM_ID,
        );
        
        let clock = Clock::get()?;
        
        // Get factory address before any mutable borrows
        let factory_address = ctx.accounts.factory_state.key();
        
        let factory_state = &mut ctx.accounts.factory_state;
        let session_entry = &mut ctx.accounts.session_entry;
        
        // Initialize session entry
        session_entry.session_address = session_pda;
        session_entry.owner = owner;
        session_entry.eval_program = eval_program;
        session_entry.shard_address = shard_address;
        session_entry.session_id = session_id.clone();
        session_entry.namespaces = namespaces.clone();
        session_entry.status = SessionStatus::Requested;
        session_entry.created_at = clock.unix_timestamp;
        session_entry.last_updated = clock.unix_timestamp;
        session_entry.activated_at = None;
        session_entry.optimistic_state = SessionState::Pending;
        session_entry.optimistic_state_set_at = clock.unix_timestamp;
        session_entry.bump = ctx.bumps.session_entry;
        
        // Update factory state
        factory_state.total_sessions = factory_state.total_sessions
            .checked_add(1)
            .unwrap_or(u64::MAX);
        
        // Estimate session size (this is a rough estimate - the actual session program will determine final size)
        let estimated_size = calculate_estimated_session_size(&session_id, &namespaces);
        
        // Generate attestation for this session creation
        let attestation_id = format!("att_{}_{}", clock.slot, session_id);
        let attestation = SessionCreationAttestation {
            factory_address,
            session_address: session_pda,
            session_id: session_id.clone(),
            eval_address: eval_program,
            attestation_id: attestation_id.clone(),
            creation_parameters: CreationParameters {
                template_id: None, // TODO: Add template support
                custom_config: None, // TODO: Add custom config support
                requested_namespaces: namespaces.clone(),
                metadata: vec![
                    ("factory_version".to_string(), factory_state.version.to_string()),
                    ("creation_method".to_string(), "direct_request".to_string()),
                ],
            },
            verification_signature: [0u8; 64], // TODO: Implement actual signature
            attested_at_height: clock.slot,
        };
        
        // Emit comprehensive session creation event with full attestation details
        emit!(SessionCreatedEvent {
            session_address: session_pda,
            owner,
            eval_program,
            shard_address,
            session_id: session_id.clone(),
            namespaces: namespaces.clone(),
            capability_used: Some("session_factory.create_session".to_string()), // Default capability
            verification_functions_executed: vec![], // TODO: Track actual verification functions
            attestation: attestation.clone(),
            created_at: clock.unix_timestamp,
            account_size: estimated_size as u64,
            creation_slot: clock.slot,
        });
        
        // Still emit PDA event for backward compatibility with off-chain service
        emit!(PDAComputedEvent {
            expected_pda: session_pda,
            expected_size: estimated_size as u64,
            expected_owner: SESSION_PROGRAM_ID,
            session_id: session_id.clone(),
            owner,
            eval_program,
            shard_address,
            namespaces: namespaces.clone(),
            compute_slot: clock.slot,
        });
        
        msg!(
            "Session creation requested: PDA={}, owner={}, eval={}, shard={}, session_id={}",
            session_pda,
            owner,
            eval_program,
            shard_address,
            session_id
        );
        
        Ok(())
    }
    
    /// Update session status (called by off-chain service or session program)
    pub fn update_session_status(
        ctx: Context<UpdateSessionStatus>,
        new_status: SessionStatus,
    ) -> Result<()> {
        let session_entry = &mut ctx.accounts.session_entry;
        let factory_state = &mut ctx.accounts.factory_state;
        let clock = Clock::get()?;
        
        let old_status = session_entry.status;
        session_entry.status = new_status;
        session_entry.last_updated = clock.unix_timestamp;
        
        // Update activated_at when session becomes active
        if new_status == SessionStatus::Active && old_status != SessionStatus::Active {
            session_entry.activated_at = Some(clock.unix_timestamp);
            factory_state.active_sessions = factory_state.active_sessions
                .checked_add(1)
                .unwrap_or(u64::MAX);
            
            // Emit session activation event with comprehensive details
            emit!(SessionActivatedEvent {
                session_address: session_entry.session_address,
                session_id: session_entry.session_id.clone(),
                owner: session_entry.owner,
                eval_program: session_entry.eval_program,
                shard_address: session_entry.shard_address,
                namespaces: session_entry.namespaces.clone(),
                activated_at: clock.unix_timestamp,
                activated_by: ctx.accounts.authority.key(),
                activation_slot: clock.slot,
            });
        }
        
        // Update active count when session becomes inactive
        if old_status == SessionStatus::Active && new_status != SessionStatus::Active {
            factory_state.active_sessions = factory_state.active_sessions
                .checked_sub(1)
                .unwrap_or(0);
        }
        
        msg!(
            "Session status updated: {} -> {:?}",
            session_entry.session_address,
            new_status
        );
        
        Ok(())
    }
    
    /// Close session and clean up registry
    pub fn close_session(ctx: Context<CloseSession>) -> Result<()> {
        let session_entry = &ctx.accounts.session_entry;
        let factory_state = &mut ctx.accounts.factory_state;
        
        // Verify authority
        require!(
            ctx.accounts.authority.key() == session_entry.owner ||
            ctx.accounts.authority.key() == factory_state.authority,
            SessionFactoryError::NotAuthorized
        );
        
        // Update active count if session was active
        if session_entry.status == SessionStatus::Active {
            factory_state.active_sessions = factory_state.active_sessions
                .checked_sub(1)
                .unwrap_or(0);
        }
        
        msg!("Session closed: {}", session_entry.session_address);
        
        // Account will be closed automatically due to close constraint
        Ok(())
    }

    /// Reserve session ID for two-phase creation (prevents conflicts)
    pub fn reserve_session(
        ctx: Context<ReserveSession>,
        reservation_id: String,
        session_id: String,
        session_owner: Pubkey,
        template_id: Option<String>,
        custom_config: Option<SessionConfiguration>,
        reservation_duration_hours: u64,
    ) -> Result<()> {
        // Validate inputs
        require!(
            !reservation_id.is_empty() && reservation_id.len() <= 64,
            SessionFactoryError::InvalidReservationId
        );
        require!(
            !session_id.is_empty() && session_id.len() <= 64,
            SessionFactoryError::InvalidSessionId
        );
        require!(
            session_owner != Pubkey::default(),
            SessionFactoryError::InvalidOwner
        );
        require!(
            reservation_duration_hours > 0 && reservation_duration_hours <= 168, // Max 1 week
            SessionFactoryError::InvalidReservationDuration
        );
        
        // Ensure either template_id or custom_config is provided, but not both
        require!(
            template_id.is_some() ^ custom_config.is_some(),
            SessionFactoryError::InvalidReservationConfig
        );
        
        let clock = Clock::get()?;
        let reservation = &mut ctx.accounts.session_reservation;
        
        // Calculate expiry timestamp
        let expires_at = clock.unix_timestamp
            .checked_add((reservation_duration_hours * 3600) as i64)
            .ok_or(SessionFactoryError::InvalidReservationDuration)?;
        
        // Initialize reservation
        reservation.reservation_id = reservation_id.clone();
        reservation.session_id = session_id.clone();
        reservation.reserved_by = ctx.accounts.reserver.key();
        reservation.session_owner = session_owner;
        reservation.template_id = template_id.clone();
        reservation.session_config = custom_config;
        reservation.expires_at = expires_at;
        reservation.reserved_at = clock.unix_timestamp;
        reservation.is_used = false;
        reservation.bump = ctx.bumps.session_reservation;
        
        emit!(SessionReservedEvent {
            reservation_id: reservation_id.clone(),
            session_id: session_id.clone(),
            reserved_by: ctx.accounts.reserver.key(),
            session_owner,
            template_id: template_id.clone(),
            expires_at,
            reserved_at: clock.unix_timestamp,
        });
        
        msg!(
            "Session reserved: reservation_id={}, session_id={}, owner={}, expires_at={}",
            reservation_id,
            session_id,
            session_owner,
            expires_at
        );
        
        Ok(())
    }

    /// Create optimistic session (pending state for better UX)
    pub fn create_optimistic_session(
        ctx: Context<CreateOptimisticSession>,
        owner: Pubkey,
        session_id: String,
    ) -> Result<()> {
        // Validate inputs
        require!(
            !session_id.is_empty() && session_id.len() <= 64,
            SessionFactoryError::InvalidSessionId
        );
        
        let clock = Clock::get()?;
        let factory_state = &mut ctx.accounts.factory_state;
        let session_entry = &mut ctx.accounts.session_entry;
        
        // Initialize session entry
        session_entry.session_address = Pubkey::default();
        session_entry.owner = owner;
        session_entry.eval_program = Pubkey::default();
        session_entry.shard_address = Pubkey::default();
        session_entry.session_id = session_id.clone();
        session_entry.namespaces = vec![];
        session_entry.status = SessionStatus::Requested;
        session_entry.created_at = clock.unix_timestamp;
        session_entry.last_updated = clock.unix_timestamp;
        session_entry.activated_at = None;
        session_entry.optimistic_state = SessionState::Pending;
        session_entry.optimistic_state_set_at = clock.unix_timestamp;
        session_entry.bump = ctx.bumps.session_entry;
        
        // Update factory state
        factory_state.total_sessions = factory_state.total_sessions
            .checked_add(1)
            .unwrap_or(u64::MAX);
        
        // Estimate session size (this is a rough estimate - the actual session program will determine final size)
        let estimated_size = calculate_estimated_session_size(&session_id, &[]);
        
        // Generate attestation for this session creation
        let attestation_id = format!("att_{}_{}", clock.slot, session_id);
        let attestation = SessionCreationAttestation {
            factory_address: Pubkey::default(),
            session_address: Pubkey::default(),
            session_id: session_id.clone(),
            eval_address: Pubkey::default(),
            attestation_id: attestation_id.clone(),
            creation_parameters: CreationParameters {
                template_id: None,
                custom_config: None,
                requested_namespaces: vec![],
                metadata: vec![
                    ("factory_version".to_string(), factory_state.version.to_string()),
                    ("creation_method".to_string(), "optimistic_session".to_string()),
                ],
            },
            verification_signature: [0u8; 64],
            attested_at_height: clock.slot,
        };
        
        // Emit comprehensive session creation event with full attestation details
        emit!(SessionCreatedEvent {
            session_address: Pubkey::default(),
            owner,
            eval_program: Pubkey::default(),
            shard_address: Pubkey::default(),
            session_id: session_id.clone(),
            namespaces: vec![],
            capability_used: Some("session_factory.create_optimistic_session".to_string()),
            verification_functions_executed: vec![],
            attestation: attestation.clone(),
            created_at: clock.unix_timestamp,
            account_size: estimated_size as u64,
            creation_slot: clock.slot,
        });
        
        // Still emit PDA event for backward compatibility with off-chain service
        emit!(PDAComputedEvent {
            expected_pda: Pubkey::default(),
            expected_size: estimated_size as u64,
            expected_owner: SESSION_PROGRAM_ID,
            session_id: session_id.clone(),
            owner,
            eval_program: Pubkey::default(),
            shard_address: Pubkey::default(),
            namespaces: vec![],
            compute_slot: clock.slot,
        });
        
        msg!(
            "Optimistic session created: session_id={}",
            session_id
        );
        
        Ok(())
    }

    /// Batch activate sessions from pending to active state
//     pub fn batch_activate_optimistic_sessions(ctx: Context<BatchActivateOptimisticSessions>) -> Result<()> {
//         let factory_state = &mut ctx.accounts.factory_state;
//         
//         // Update active count when sessions become active
//         factory_state.active_sessions = factory_state.active_sessions
//             .checked_add(factory_state.total_sessions)
//             .unwrap_or(u64::MAX);
//         
//         msg!("All sessions activated");
//         
//         Ok(())
//     }

    /// Initialize session initialization queue
//     pub fn initialize_session_queue(ctx: Context<InitializeSessionQueue>) -> Result<()> {
//         let init_queue = &mut ctx.accounts.init_queue;
//         let clock = Clock::get()?;
//         
//         init_queue.authority = ctx.accounts.authority.key();
//         init_queue.pending_inits = vec![];
//         init_queue.max_queue_size = 50; // Maximum 50 pending initializations
//         init_queue.total_processed = 0;
//         init_queue.total_failed = 0;
//         init_queue.created_at = clock.unix_timestamp;
//         init_queue.last_processed_at = 0;
//         init_queue.bump = ctx.bumps.init_queue;
//         
//         msg!("Session initialization queue initialized");
//         
//         Ok(())
//     }

    // Queue-related functions have been moved to scheduler singleton
    // The following functions are deprecated and should use scheduler CPI instead:
    // - queue_session_init
    // - execute_queued_init  
    // - batch_process_queue
    
    /*
    /// Queue session for initialization (FIFO ordering) - DEPRECATED
    pub fn queue_session_init(
        ctx: Context<QueueSessionInit>,
        session_pda: Pubkey,
        session_id: String,
        owner: Pubkey,
        eval_program: Pubkey,
        shard_address: Pubkey,
        namespaces: Vec<String>,
        deadline_hours: u64,
    ) -> Result<()> {
        let init_queue = &mut ctx.accounts.init_queue;
        let session_entry = &mut ctx.accounts.session_entry;
        let clock = Clock::get()?;
        
        // Check queue capacity (FIFO ordering maintained by Vec)
        require!(
            init_queue.pending_inits.len() < init_queue.max_queue_size as usize,
            SessionFactoryError::QueueFull
        );
        
        // Validate deadline
        require!(
            deadline_hours > 0 && deadline_hours <= 168, // Max 1 week
            SessionFactoryError::InvalidDeadline
        );
        
        let deadline = clock.unix_timestamp
            .checked_add((deadline_hours * 3600) as i64)
            .ok_or(SessionFactoryError::InvalidDeadline)?;
        
        // Create pending initialization (FIFO - added to end of Vec)
        let pending_init = PendingInit {
            session_pda,
            owner,
            eval_program,
            shard_address,
            session_id: session_id.clone(),
            namespaces: namespaces.clone(),
            queued_by: ctx.accounts.queuer.key(),
            queued_at: clock.unix_timestamp,
            deadline,
        };
        
        init_queue.pending_inits.push(pending_init);
        
        // Update session entry status
        session_entry.status = SessionStatus::Created; // Account exists, waiting for init
        session_entry.last_updated = clock.unix_timestamp;
        
        emit!(SessionQueuedEvent {
            session_id: session_id.clone(),
            queue_position: (init_queue.pending_inits.len() - 1) as u64,
            request_id: session_id.clone(),
            timestamp: Clock::get()?.unix_timestamp as u64,
            estimated_processing_time: 30,
            queue_size: init_queue.pending_inits.len() as u64,
            factory_address: "session_factory".to_string(), // Placeholder since factory_state not available in this context
            creator: ctx.accounts.queuer.key().to_string(),
        });
        
        msg!(
            "Session queued for initialization: session_id={}, deadline={}",
            session_id,
            deadline
        );
        
        Ok(())
    }

    /// Execute queued session initialization (permissionless - anyone can execute)
    pub fn execute_queued_init(
        ctx: Context<ExecuteQueuedInit>,
        queue_index: u32,
    ) -> Result<()> {
        let init_queue = &mut ctx.accounts.init_queue;
        let session_entry = &mut ctx.accounts.session_entry;
        let factory_state = &mut ctx.accounts.factory_state;
        let clock = Clock::get()?;
        let processing_start = clock.unix_timestamp;
        
        // Store initial queue state for analytics
        let initial_queue_size = init_queue.pending_inits.len();
        
        // Validate queue index (FIFO - should execute from front)
        require!(
            (queue_index as usize) < init_queue.pending_inits.len(),
            SessionFactoryError::InvalidQueueIndex
        );
        
        // Get pending initialization (prefer FIFO but allow any index for failed retries)
        let pending = init_queue.pending_inits[queue_index as usize].clone();
        
        // Check deadline hasn't passed
        require!(
            clock.unix_timestamp <= pending.deadline,
            SessionFactoryError::InitDeadlinePassed
        );
        
        // Verify session matches
        require!(
            session_entry.session_address == pending.session_pda,
            SessionFactoryError::SessionMismatch
        );
        
        // Execute initialization (simulate successful init)
        session_entry.status = SessionStatus::Active;
        session_entry.optimistic_state = SessionState::Active;
        session_entry.activated_at = Some(clock.unix_timestamp);
        session_entry.last_updated = clock.unix_timestamp;
        session_entry.optimistic_state_set_at = clock.unix_timestamp;
        
        // Remove from queue (maintain FIFO by removing processed item)
        init_queue.pending_inits.remove(queue_index as usize);
        init_queue.total_processed = init_queue.total_processed
            .checked_add(1)
            .unwrap_or(u64::MAX);
        init_queue.last_processed_at = clock.unix_timestamp;
        
        // Update factory stats
        factory_state.active_sessions = factory_state.active_sessions
            .checked_add(1)
            .unwrap_or(u64::MAX);
        
        emit!(SessionInitializedFromQueueEvent {
            session_pda: pending.session_pda,
            session_id: pending.session_id.clone(),
            owner: pending.owner,
            executor: ctx.accounts.executor.key(),
            queued_by: pending.queued_by,
            initialized_at: clock.unix_timestamp,
            queue_position: queue_index as u64,
        });
        
        // Emit queue processing analytics event
        emit!(QueueProcessedEvent {
            processor: ctx.accounts.executor.key(),
            sessions_processed: 1,
            failed_initializations: 0,
            queue_size_before: initial_queue_size as u64,
            queue_size_after: init_queue.pending_inits.len() as u64,
            processing_duration_ms: ((clock.unix_timestamp - processing_start) * 1000) as u64,
            processing_started_at: processing_start,
            processing_completed_at: clock.unix_timestamp,
            total_processed: init_queue.total_processed,
            total_failed: init_queue.total_failed,
        });
        
        msg!(
            "Session initialized from queue: session_id={}, executor={}",
            pending.session_id,
            ctx.accounts.executor.key()
        );
        
        Ok(())
    }

    /// Batch process queue items (permissionless - for efficiency)
    pub fn batch_process_queue(
        ctx: Context<BatchProcessQueue>,
        max_items: u8,
    ) -> Result<()> {
        let init_queue = &mut ctx.accounts.init_queue;
        let factory_state = &mut ctx.accounts.factory_state;
        let clock = Clock::get()?;
        let processing_start = clock.unix_timestamp;
        
        // Store initial queue state for analytics
        let initial_queue_size = init_queue.pending_inits.len();
        let max_to_process = std::cmp::min(max_items as usize, initial_queue_size);
        let mut processed_count = 0u32;
        let mut failed_count = 0u32;
        
        // Process items from the front of the queue (FIFO)
        let mut items_to_remove = Vec::new();
        
        for i in 0..max_to_process {
            if i >= init_queue.pending_inits.len() {
                break;
            }
            
            let pending = &init_queue.pending_inits[i];
            
            // Check if deadline hasn't passed
            if clock.unix_timestamp <= pending.deadline {
                // Would normally check session state here, but for simulation just mark as processed
                items_to_remove.push(i);
                processed_count += 1;
                
                emit!(SessionInitializedFromQueueEvent {
                    session_pda: pending.session_pda,
                    session_id: pending.session_id.clone(),
                    owner: pending.owner,
                    executor: ctx.accounts.processor.key(),
                    queued_by: pending.queued_by,
                    initialized_at: clock.unix_timestamp,
                    queue_position: i as u64,
                });
            } else {
                // Deadline passed - mark as failed and remove
                items_to_remove.push(i);
                failed_count += 1;
            }
        }
        
        // Remove processed items from queue (reverse order to maintain indices)
        for &index in items_to_remove.iter().rev() {
            init_queue.pending_inits.remove(index);
        }
        
        // Update queue statistics
        init_queue.total_processed = init_queue.total_processed
            .checked_add(processed_count as u64)
            .unwrap_or(u64::MAX);
        init_queue.total_failed = init_queue.total_failed
            .checked_add(failed_count as u64)
            .unwrap_or(u64::MAX);
        init_queue.last_processed_at = clock.unix_timestamp;
        
        // Update factory stats
        factory_state.active_sessions = factory_state.active_sessions
            .checked_add(processed_count as u64)
            .unwrap_or(u64::MAX);
        
        // Emit comprehensive queue processing analytics event
        emit!(QueueProcessedEvent {
            processor: ctx.accounts.processor.key(),
            sessions_processed: processed_count,
            failed_initializations: failed_count,
            queue_size_before: initial_queue_size as u64,
            queue_size_after: init_queue.pending_inits.len() as u64,
            processing_duration_ms: ((clock.unix_timestamp - processing_start) * 1000) as u64,
            processing_started_at: processing_start,
            processing_completed_at: clock.unix_timestamp,
            total_processed: init_queue.total_processed,
            total_failed: init_queue.total_failed,
        });
        
        msg!(
            "Batch processed queue: processed={}, failed={}, remaining={}",
            processed_count,
            failed_count,
            init_queue.pending_inits.len()
        );
        
        Ok(())
    }
    */

    /// Manual session initialization (direct path without queue)
    pub fn manual_initialize_session(
        ctx: Context<ManualInitializeSession>,
        session_id: String,
        force_activation: bool,
    ) -> Result<()> {
        let session_entry = &mut ctx.accounts.session_entry;
        let factory_state = &mut ctx.accounts.factory_state;
        let clock = Clock::get()?;
        
        // Only allow manual initialization if session is not already active
        require!(
            session_entry.status != SessionStatus::Active,
            SessionFactoryError::SessionAlreadyActive
        );
        
        // Manual initialization bypasses queue - directly activate
        session_entry.status = SessionStatus::Active;
        session_entry.optimistic_state = SessionState::Active;
        session_entry.activated_at = Some(clock.unix_timestamp);
        session_entry.last_updated = clock.unix_timestamp;
        session_entry.optimistic_state_set_at = clock.unix_timestamp;
        
        // Update factory stats
        factory_state.active_sessions = factory_state.active_sessions
            .checked_add(1)
            .unwrap_or(u64::MAX);
        
        emit!(ManualSessionInitializedEvent {
            session_pda: session_entry.session_address,
            session_id: session_id.clone(),
            owner: session_entry.owner,
            authority: ctx.accounts.authority.key(),
            force_activation,
            initialized_at: clock.unix_timestamp,
        });
        
        msg!(
            "Session manually initialized: session_id={}, force_activation={}",
            session_id,
            force_activation
        );
        
        Ok(())
    }
    
    /// Emergency reset session state (for failed initializations)
    pub fn emergency_reset_session(
        ctx: Context<EmergencyResetSession>,
        session_id: String,
        reset_to_status: SessionStatus,
    ) -> Result<()> {
        let session_entry = &mut ctx.accounts.session_entry;
        let clock = Clock::get()?;
        
        // Store old status for event
        let old_status = session_entry.status;
        let old_optimistic_state = session_entry.optimistic_state;
        
        // Reset session to specified status
        session_entry.status = reset_to_status;
        session_entry.optimistic_state = match reset_to_status {
            SessionStatus::Requested => SessionState::Pending,
            SessionStatus::Created => SessionState::Pending,
            SessionStatus::Active => SessionState::Active,
            SessionStatus::Closed => SessionState::Closed,
        };
        session_entry.last_updated = clock.unix_timestamp;
        session_entry.optimistic_state_set_at = clock.unix_timestamp;
        
        // Clear activation time if resetting to non-active state
        if reset_to_status != SessionStatus::Active {
            session_entry.activated_at = None;
        }
        
        emit!(EmergencySessionResetEvent {
            session_pda: session_entry.session_address,
            session_id: session_id.clone(),
            owner: session_entry.owner,
            authority: ctx.accounts.authority.key(),
            old_status,
            new_status: reset_to_status,
            old_optimistic_state,
            new_optimistic_state: session_entry.optimistic_state,
            reset_at: clock.unix_timestamp,
        });
        
        msg!(
            "Session emergency reset: session_id={}, old_status={:?}, new_status={:?}",
            session_id,
            old_status,
            reset_to_status
        );
        
        Ok(())
    }
}

/// Calculate estimated size for a session account
fn calculate_estimated_session_size(session_id: &str, namespaces: &[String]) -> usize {
    // Base size for account discriminator and basic fields
    let base_size = 8 + // discriminator
        32 + // owner
        32 + // eval_program
        4 + session_id.len() + // session_id (String)
        4 + namespaces.iter().map(|ns| 4 + ns.len()).sum::<usize>() + // namespaces (Vec<String>)
        8 + // created_at
        8 + // last_updated
        1 + // is_active
        1 + // bump
        64; // some padding for metadata and other fields
    
    // Round up to next multiple of 8 for alignment
    (base_size + 7) & !7
}

/// Event emitted for off-chain session creation service
#[event]
pub struct PDAComputedEvent {
    /// The computed session PDA where account should be created
    pub expected_pda: Pubkey,
    /// Expected size of the session account
    pub expected_size: u64,
    /// Expected owner (session program)
    pub expected_owner: Pubkey,
    /// Session ID for initialization
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Namespaces for the session
    pub namespaces: Vec<String>,
    /// Block slot when computed
    pub compute_slot: u64,
}

/// Event emitted when session is reserved
#[event]
pub struct SessionReservedEvent {
    /// Unique reservation identifier
    pub reservation_id: String,
    /// Reserved session ID
    pub session_id: String,
    /// User who made the reservation
    pub reserved_by: Pubkey,
    /// Intended session owner
    pub session_owner: Pubkey,
    /// Template to use (if any)
    pub template_id: Option<String>,
    /// When the reservation expires
    pub expires_at: i64,
    /// When the reservation was made
    pub reserved_at: i64,
}

/// Comprehensive session creation event with full attestation details
#[event]
pub struct SessionCreatedEvent {
    /// The session PDA address
    pub session_address: Pubkey,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Namespaces assigned to session
    pub namespaces: Vec<String>,
    /// Capability that authorized this creation
    pub capability_used: Option<String>,
    /// Verification functions that were executed
    pub verification_functions_executed: Vec<String>,
    /// Full attestation details
    pub attestation: SessionCreationAttestation,
    /// Creation timestamp
    pub created_at: i64,
    /// Account size
    pub account_size: u64,
    /// Block slot when created
    pub creation_slot: u64,
}

/// Session creation attestation embedded in events
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionCreationAttestation {
    /// Factory that created this session
    pub factory_address: Pubkey,
    /// Session address being attested
    pub session_address: Pubkey,
    /// Session ID being attested  
    pub session_id: String,
    /// Eval program bound to the session
    pub eval_address: Pubkey,
    /// Unique attestation ID
    pub attestation_id: String,
    /// Parameters used for creation
    pub creation_parameters: CreationParameters,
    /// Verification signature
    pub verification_signature: [u8; 64],
    /// Block height when attested
    pub attested_at_height: u64,
}

impl SessionCreationAttestation {
    /// Create a new session creation attestation
    pub fn new(
        factory_address: Pubkey,
        session_address: Pubkey,
        session_id: String,
        eval_address: Pubkey,
        creation_parameters: CreationParameters,
        attested_at_height: u64,
    ) -> Self {
        Self {
            factory_address,
            session_address,
            session_id: session_id.clone(),
            eval_address,
            // Generate deterministic attestation ID from session data
            attestation_id: format!("{}-{}-{}", 
                factory_address.to_string()[..8].to_string(),
                session_id,
                attested_at_height
            ),
            creation_parameters,
            verification_signature: [0; 64], // TODO: Implement proper signature
            attested_at_height,
        }
    }

    /// Verify the integrity of this attestation
    pub fn verify_integrity(&self) -> bool {
        // For now, just check that required fields are not empty/default
        !self.session_id.is_empty() 
            && self.session_address != Pubkey::default()
            && self.factory_address != Pubkey::default()
            && self.eval_address != Pubkey::default()
    }
}

/// Parameters used for session creation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreationParameters {
    /// Template used (if any)
    pub template_id: Option<String>,
    /// Custom configuration (if used)
    pub custom_config: Option<SessionConfiguration>,
    /// Requested namespaces
    pub requested_namespaces: Vec<String>,
    /// Additional metadata
    pub metadata: Vec<(String, String)>, // key-value pairs
}

/// Session activated event with comprehensive details
#[event]
pub struct SessionActivatedEvent {
    /// The session PDA address
    pub session_address: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Namespaces assigned to session
    pub namespaces: Vec<String>,
    /// When session was activated
    pub activated_at: i64,
    /// Activated by
    pub activated_by: Pubkey,
    /// Block slot when activated
    pub activation_slot: u64,
}

/// Session queued event with comprehensive details
#[event]
pub struct SessionQueuedEvent {
    /// The session PDA address
    pub session_id: String,
    /// Queue position
    pub queue_position: u64,
    /// Request ID (session_id)
    pub request_id: String,
    /// Timestamp when queued
    pub timestamp: u64,
    /// Estimated processing time in seconds
    pub estimated_processing_time: u64,
    /// Current queue size
    pub queue_size: u64,
    /// Factory address
    pub factory_address: String,
    /// Creator of the request
    pub creator: String,
}

/// Session initialized from queue event with comprehensive details
#[event]
pub struct SessionInitializedFromQueueEvent {
    /// The session PDA address
    pub session_pda: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Executor
    pub executor: Pubkey,
    /// Queued by
    pub queued_by: Pubkey,
    /// When session was initialized
    pub initialized_at: i64,
    /// Queue position
    pub queue_position: u64,
}

/// Manual session initialization event with comprehensive details
#[event]
pub struct ManualSessionInitializedEvent {
    /// The session PDA address
    pub session_pda: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// Force activation
    pub force_activation: bool,
    /// When session was initialized
    pub initialized_at: i64,
}

/// Emergency session reset event with comprehensive details
#[event]
pub struct EmergencySessionResetEvent {
    /// The session PDA address
    pub session_pda: Pubkey,
    /// Session ID
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// Old session status
    pub old_status: SessionStatus,
    /// New session status
    pub new_status: SessionStatus,
    /// Old optimistic state
    pub old_optimistic_state: SessionState,
    /// New optimistic state
    pub new_optimistic_state: SessionState,
    /// When session was reset
    pub reset_at: i64,
}

/// Queue processed event for analytics and monitoring
#[event]
pub struct QueueProcessedEvent {
    /// Queue processor (executor)
    pub processor: Pubkey,
    /// Number of sessions processed in this batch
    pub sessions_processed: u32,
    /// Number of failed initializations
    pub failed_initializations: u32,
    /// Total queue size before processing
    pub queue_size_before: u64,
    /// Total queue size after processing
    pub queue_size_after: u64,
    /// Processing duration (approximate)
    pub processing_duration_ms: u64,
    /// Timestamp when processing started
    pub processing_started_at: i64,
    /// Timestamp when processing completed
    pub processing_completed_at: i64,
    /// Total sessions processed by this queue (cumulative)
    pub total_processed: u64,
    /// Total failed sessions by this queue (cumulative)
    pub total_failed: u64,
}

#[error_code]
pub enum SessionFactoryError {
    #[msg("Invalid owner address")]
    InvalidOwner,
    #[msg("Invalid eval program address")]
    InvalidEvalProgram,
    #[msg("Invalid session ID")]
    InvalidSessionId,
    #[msg("Too many namespaces")]
    TooManyNamespaces,
    #[msg("Invalid shard address")]
    InvalidShard,
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    #[msg("Invalid reservation ID")]
    InvalidReservationId,
    #[msg("Invalid reservation duration")]
    InvalidReservationDuration,
    #[msg("Invalid reservation configuration")]
    InvalidReservationConfig,
    #[msg("Reservation has expired")]
    ReservationExpired,
    #[msg("Reservation already used")]
    ReservationAlreadyUsed,
    #[msg("Queue full")]
    QueueFull,
    #[msg("Invalid deadline")]
    InvalidDeadline,
    #[msg("Invalid queue index")]
    InvalidQueueIndex,
    #[msg("Initialization deadline passed")]
    InitDeadlinePassed,
    #[msg("Session mismatch")]
    SessionMismatch,
    #[msg("Session already active")]
    SessionAlreadyActive,
} 