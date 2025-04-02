use anchor_lang::prelude::*;
use crate::error::ProcessorError;

/// Priority level for messages
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum Priority {
    /// Low priority, processed last
    Low,
    /// Medium priority, processed after high
    Medium,
    /// High priority, processed first
    High,
}

impl<'info> Initialize
ProcessTick
PauseProcessor
ResumeProcessor<'info> {
    pub fn try_accounts(
        ctx: &Context<'_, '_, '_, 'info, Initialize
ProcessTick
PauseProcessor
ResumeProcessor<'info>>,
        _bumps: &anchor_lang::prelude::BTreeMap<String, u8>,
    ) -> Result<()> {
        // Additional validation logic can be added here if needed
        Ok(())
    }
}


impl From<u8> for Priority {
    fn from(val: u8) -> Self {
        match val {
            0 => Priority::Low,
            1 => Priority::Medium,
            _ => Priority::High,
        }
    }
}

/// Subroutine execution type
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum SubroutineType {
    /// Atomic execution - all messages must succeed
    Atomic,
    /// Non-atomic execution - messages can fail individually
    NonAtomic,
}

impl From<u8> for SubroutineType {
    fn from(val: u8) -> Self {
        match val {
            0 => SubroutineType::Atomic,
            _ => SubroutineType::NonAtomic,
        }
    }
}

/// Result of execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum ExecutionResult {
    /// Execution succeeded
    Success,
    /// Execution failed
    Failure,
}

/// Message to be processed
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProcessorMessage {
    /// Program ID to call
    pub program_id: Pubkey,
    /// Instruction data
    pub data: Vec<u8>,
    /// Account metas
    pub accounts: Vec<AccountMetaData>,
}

/// Account meta data for cross-program invocations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AccountMetaData {
    /// Account pubkey
    pub pubkey: Pubkey,
    /// Is signer
    pub is_signer: bool,
    /// Is writable
    pub is_writable: bool,
}

/// Queue state for message ordering
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QueueState {
    /// Head index (front of queue)
    pub head: u64,
    /// Tail index (back of queue)
    pub tail: u64,
    /// Number of items in the queue
    pub count: u64,
    /// Maximum capacity of the queue
    pub capacity: u64,
}

impl QueueState {
    /// Create a new queue with the given capacity
    pub fn new(capacity: u64) -> Self {
        Self {
            head: 0,
            tail: 0,
            count: 0,
            capacity,
        }
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if the queue is full
    pub fn is_full(&self) -> bool {
        self.count == self.capacity
    }

    /// Get the next index to enqueue at
    pub fn next_enqueue_index(&self) -> u64 {
        if self.is_full() {
            return self.tail;
        }
        self.tail
    }

    /// Get the next index to dequeue from
    pub fn next_dequeue_index(&self) -> Option<u64> {
        if self.is_empty() {
            return None;
        }
        Some(self.head)
    }

    /// Enqueue an item
    pub fn enqueue(&mut self) -> Result<u64> {
        require!(!self.is_full(), ProcessorError::QueueFull);
        let index = self.tail;
        self.tail = (self.tail + 1) % self.capacity;
        self.count += 1;
        Ok(index)
    }

    /// Dequeue an item
    pub fn dequeue(&mut self) -> Result<u64> {
        require!(!self.is_empty(), ProcessorError::QueueEmpty);
        let index = self.head;
        self.head = (self.head + 1) % self.capacity;
        self.count -= 1;
        Ok(index)
    }
}

/// Message batch for execution
#[account]
pub struct MessageBatch {
    /// Execution ID
    pub execution_id: u64,
    /// Messages to execute
    pub messages: Vec<ProcessorMessage>,
    /// Subroutine type
    pub subroutine_type: SubroutineType,
    /// Expiration timestamp (optional)
    pub expiration_time: Option<i64>,
    /// Priority level
    pub priority: Priority,
    /// Calling account
    pub caller: Pubkey,
    /// Callback address
    pub callback_address: Pubkey,
    /// Timestamp when the batch was created
    pub created_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Pending callback tracking
#[account]
pub struct PendingCallback {
    /// Execution ID
    pub execution_id: u64,
    /// Callback recipient address
    pub callback_address: Pubkey,
    /// Execution result
    pub result: ExecutionResult,
    /// Number of executed messages
    pub executed_count: u32,
    /// Error data if any
    pub error_data: Option<Vec<u8>>,
    /// Timestamp when the callback was created
    pub created_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Processor Program state
#[account]
pub struct ProcessorState {
    /// Authorization program ID
    pub authorization_program_id: Pubkey,
    /// Whether the processor is paused
    pub is_paused: bool,
    /// High priority queue
    pub high_priority_queue: QueueState,
    /// Medium priority queue
    pub medium_priority_queue: QueueState,
    /// Low priority queue
    pub low_priority_queue: QueueState,
    /// Total executions processed
    pub total_executions: u64,
    /// Total successful executions
    pub successful_executions: u64,
    /// Total failed executions
    pub failed_executions: u64,
    /// Last execution timestamp
    pub last_execution_time: i64,
    /// Owner authority
    pub owner: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Instruction context for initializing the processor program
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The processor state account
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<ProcessorState>(),
        seeds = [b"processor_state".as_ref()],
        bump
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    /// The account paying for the initialization
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for enqueuing messages
#[derive(Accounts)]
#[instruction(execution_id: u64, priority: u8, subroutine_type: u8, messages: Vec<ProcessorMessage>)]
pub struct EnqueueMessages<'info> {
    /// The processor state account
    #[account(
        mut,
        seeds = [b"processor_state".as_ref()],
        bump = processor_state.bump,
        constraint = !processor_state.is_paused @ ProcessorError::ProcessorPaused,
        constraint = processor_state.authorization_program_id == caller.key() @ ProcessorError::UnauthorizedCaller,
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    /// The message batch account
    #[account(
        init,
        payer = fee_payer,
        space = 8 + std::mem::size_of::<MessageBatch>() + 
                messages.iter().map(|m| 
                    m.data.len() + 
                    m.accounts.len() * std::mem::size_of::<AccountMetaData>()
                ).sum::<usize>() + 
                1024, // Extra space for future growth
        seeds = [b"message_batch".as_ref(), execution_id.to_le_bytes().as_ref()],
        bump
    )]
    pub message_batch: Account<'info, MessageBatch>,
    
    /// The calling program (must be the authorization program)
    pub caller: Signer<'info>,
    
    /// The callback address (where to send execution results)
    /// #[account()]
    pub callback_address: AccountInfo<'info>,
    
    /// The account paying for the message batch account
    #[account(mut)]
    pub fee_payer: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for processing a tick
#[derive(Accounts)]
pub struct ProcessTick<'info> {
    /// The processor state account
    #[account(
        mut,
        seeds = [b"processor_state".as_ref()],
        bump = processor_state.bump,
        constraint = !processor_state.is_paused @ ProcessorError::ProcessorPaused,
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    /// The account paying for transaction fees
    /// Anyone can call this to process messages
    #[account(mut)]
    pub fee_payer: Signer<'info>,
    
    /// The current message batch being processed (if any)
    /// This is optional as we'll look it up based on queue state
    #[account(mut)]
    pub message_batch: Option<Account<'info, MessageBatch>>,
    
    /// The pending callback to create
    #[account(
        init_if_needed,
        payer = fee_payer,
        space = 8 + std::mem::size_of::<PendingCallback>() + 1024, // Extra space for error data
        seeds = [b"pending_callback".as_ref(), &[0]], // We'll use a proper seed from execution_id
        bump
    )]
    pub pending_callback: Account<'info, PendingCallback>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
}

/// Instruction context for sending a callback
#[derive(Accounts)]
#[instruction(execution_id: u64)]
pub struct SendCallback<'info> {
    /// The processor state account
    #[account(
        seeds = [b"processor_state".as_ref()],
        bump = processor_state.bump,
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    /// The pending callback to process
    #[account(
        mut,
        seeds = [b"pending_callback".as_ref(), execution_id.to_le_bytes().as_ref()],
        bump = pending_callback.bump,
        close = fee_payer
    )]
    pub pending_callback: Account<'info, PendingCallback>,
    
    /// The authorization program to send the callback to
    /// #[account(
    ///     constraint = authorization_program.key() == processor_state.authorization_program_id 
    ///                   @ ProcessorError::InvalidAuthorizationProgram
    /// )]
    pub authorization_program: AccountInfo<'info>,
    
    /// The callback recipient
    pub callback_recipient: AccountInfo<'info>,
    
    /// The account paying for transaction fees
    #[account(mut)]
    pub fee_payer: Signer<'info>,
}

/// Instruction context for pausing the processor
#[derive(Accounts)]
pub struct PauseProcessor<'info> {
    /// The processor state account
    #[account(
        mut,
        seeds = [b"processor_state".as_ref()],
        bump = processor_state.bump,
        constraint = processor_state.owner == owner.key() @ ProcessorError::UnauthorizedOwner,
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    /// The owner of the processor
    pub owner: Signer<'info>,
}

/// Instruction context for resuming the processor
#[derive(Accounts)]
pub struct ResumeProcessor<'info> {
    /// The processor state account
    #[account(
        mut,
        seeds = [b"processor_state".as_ref()],
        bump = processor_state.bump,
        constraint = processor_state.owner == owner.key() @ ProcessorError::UnauthorizedOwner,
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    /// The owner of the processor
    pub owner: Signer<'info>,
} 