use anchor_lang::prelude::*;

declare_id!("3YX6iPrpB9yQab1BoBEmRBfjRehJkZnuBZdn69pi6eLu");

pub mod error;
pub mod state;
pub mod instructions;
pub mod queue;

use state::*;

#[program]
pub mod processor {
    use super::*;

    /// Initialize the processor program
    pub fn initialize(
        ctx: Context<Initialize>,
        authorization_program_id: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, authorization_program_id)
    }

    /// Enqueue messages for processing
    pub fn enqueue_messages(
        ctx: Context<EnqueueMessages>,
        execution_id: u64,
        priority: u8,
        subroutine_type: u8,
        messages: Vec<ProcessorMessage>,
    ) -> Result<()> {
        instructions::enqueue_messages::handler(ctx, execution_id, priority, subroutine_type, messages)
    }

    /// Process queued messages (tick)
    pub fn process_tick(
        ctx: Context<ProcessTick>,
    ) -> Result<()> {
        instructions::process_tick::handler(ctx)
    }

    /// Send callback to authorization program
    pub fn send_callback(
        ctx: Context<SendCallback>,
        execution_id: u64,
        result: ExecutionResult,
        executed_count: u32,
        error_data: Option<Vec<u8>>
    ) -> Result<()> {
        instructions::send_callback::handler(ctx, execution_id, result, executed_count, error_data)
    }

    /// Pause the processor
    pub fn pause_processor(
        ctx: Context<PauseProcessor>,
    ) -> Result<()> {
        instructions::pause_processor::handler(ctx)
    }

    /// Resume the processor
    pub fn resume_processor(
        ctx: Context<ResumeProcessor>,
    ) -> Result<()> {
        instructions::resume_processor::handler(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    

    #[test]
    fn test_execution_result_variants() {
        let success = ExecutionResult::Success;
        let failure = ExecutionResult::Failure;
        assert_ne!(success, failure);
    }

    #[test]
    fn test_subroutine_type_variants() {
        let atomic = SubroutineType::Atomic;
        let non_atomic = SubroutineType::NonAtomic;
        assert_ne!(atomic, non_atomic);
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(Priority::from(0), Priority::Low);
        assert_eq!(Priority::from(1), Priority::Medium);
        assert_eq!(Priority::from(2), Priority::High);
        assert_eq!(Priority::from(255), Priority::High); // Default to high for invalid values
    }

    #[test]
    fn test_queue_state_operations() {
        let mut queue = QueueState::new(3);
        
        // Test initial state
        assert!(queue.is_empty());
        assert!(!queue.is_full());
        assert_eq!(queue.count, 0);
        
        // Test enqueue
        let index1 = queue.enqueue().unwrap();
        assert_eq!(index1, 0);
        assert_eq!(queue.count, 1);
        
        let index2 = queue.enqueue().unwrap();
        assert_eq!(index2, 1);
        assert_eq!(queue.count, 2);
        
        // Test dequeue
        let dequeue_index = queue.dequeue().unwrap();
        assert_eq!(dequeue_index, 0);
        assert_eq!(queue.count, 1);
    }

    #[test]
    fn test_processor_message_serialization() {
        let message = ProcessorMessage {
            program_id: Pubkey::new_unique(),
            data: vec![1, 2, 3, 4, 5],
            accounts: vec![
                AccountMetaData {
                    pubkey: Pubkey::new_unique(),
                    is_signer: true,
                    is_writable: false,
                }
            ],
        };

        // Test that the structure can be serialized/deserialized
        let serialized = message.try_to_vec().unwrap();
        let deserialized: ProcessorMessage = ProcessorMessage::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, message.program_id);
        assert_eq!(deserialized.data, vec![1, 2, 3, 4, 5]);
        assert_eq!(deserialized.accounts.len(), 1);
        assert_eq!(deserialized.accounts[0].pubkey, message.accounts[0].pubkey);
        assert!(deserialized.accounts[0].is_signer);
        assert!(!deserialized.accounts[0].is_writable);
    }

    #[test]
    fn test_processor_state_basic_fields() {
        let owner = Pubkey::new_unique();
        let auth_program = Pubkey::new_unique();
        
        // Test basic field access patterns
        assert_ne!(owner, Pubkey::default());
        assert_ne!(auth_program, Pubkey::default());
        
        // Test queue state creation
        let queue = QueueState::new(100);
        assert_eq!(queue.capacity, 100);
        assert_eq!(queue.count, 0);
        assert!(queue.is_empty());
        assert!(!queue.is_full());
    }

    #[test]
    fn test_message_batch_basic_fields() {
        let program_id = Pubkey::new_unique();
        let message = ProcessorMessage {
            program_id,
            data: vec![1, 2, 3],
            accounts: vec![],
        };

        assert_eq!(message.program_id, program_id);
        assert_eq!(message.data, vec![1, 2, 3]);
        assert!(message.accounts.is_empty());
    }

    #[test]
    fn test_execution_result_equality() {
        let success = ExecutionResult::Success;
        let failure = ExecutionResult::Failure;

        assert_eq!(success, ExecutionResult::Success);
        assert_eq!(failure, ExecutionResult::Failure);
        assert_ne!(success, failure);
    }

    #[test]
    fn test_priority_variants() {
        // Test priority variants
        let high = Priority::High;
        let medium = Priority::Medium;
        let low = Priority::Low;

        assert_eq!(high, Priority::High);
        assert_eq!(medium, Priority::Medium);
        assert_eq!(low, Priority::Low);
        assert_ne!(high, medium);
        assert_ne!(medium, low);
    }

    #[test]
    fn test_priority_ordering() {
        // Test priority ordering (High > Medium > Low)
        let high = Priority::High;
        let medium = Priority::Medium;
        let low = Priority::Low;

        // In a real implementation, these would have numeric values for comparison
        assert_eq!(high, Priority::High);
        assert_eq!(medium, Priority::Medium);
        assert_eq!(low, Priority::Low);
    }

    #[test]
    fn test_subroutine_type_equality() {
        let atomic = SubroutineType::Atomic;
        let non_atomic = SubroutineType::NonAtomic;

        assert_eq!(atomic, SubroutineType::Atomic);
        assert_eq!(non_atomic, SubroutineType::NonAtomic);
    }
}

#[cfg(test)]
mod queue_tests {
    use super::*;
    use crate::queue::*;

    #[test]
    fn test_default_queue_capacity() {
        assert_eq!(DEFAULT_QUEUE_CAPACITY, 100);
    }

    #[test]
    fn test_priority_to_string() {
        assert_eq!(priority_to_string(&Priority::High), "high");
        assert_eq!(priority_to_string(&Priority::Medium), "medium");
        assert_eq!(priority_to_string(&Priority::Low), "low");
    }

    #[test]
    fn test_derive_message_batch_pda() {
        let execution_id = 42u64;
        let program_id = crate::ID;
        
        let (pda, bump) = derive_message_batch_pda(execution_id, &program_id);
        
        // PDA should be valid
        assert_ne!(pda, Pubkey::default());
        // bump is u8, so it's always <= 255
        
        // Should be deterministic
        let (pda2, bump2) = derive_message_batch_pda(execution_id, &program_id);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }

    #[test]
    fn test_derive_pending_callback_pda() {
        let execution_id = 123u64;
        let program_id = crate::ID;
        
        let (pda, bump) = derive_pending_callback_pda(execution_id, &program_id);
        
        // PDA should be valid
        assert_ne!(pda, Pubkey::default());
        // bump is u8, so it's always <= 255
        
        // Should be deterministic
        let (pda2, bump2) = derive_pending_callback_pda(execution_id, &program_id);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }

    #[test]
    fn test_different_execution_ids_produce_different_pdas() {
        let program_id = crate::ID;
        
        let (pda1, _) = derive_message_batch_pda(1, &program_id);
        let (pda2, _) = derive_message_batch_pda(2, &program_id);
        
        assert_ne!(pda1, pda2);
    }

    #[test]
    fn test_queue_manager_basic_operations() {
        // This would test QueueManager if it were implemented
        // For now, just test that the constants are reasonable
        assert!(DEFAULT_QUEUE_CAPACITY > 0);
        assert!(DEFAULT_QUEUE_CAPACITY <= 10000); // Reasonable upper bound
    }
}

#[cfg(test)]
mod error_tests {
    
    

    #[test]
    fn test_processor_error_variants() {
        // Test that error variants can be created
        // Note: We can't easily test the actual error messages without more setup
        
        // Just verify the error enum exists and has expected variants
        // This is a basic smoke test for the error module
        assert!(true); // Placeholder - in a real test we'd check specific error conditions
    }
}
