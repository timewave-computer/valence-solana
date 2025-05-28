// Comprehensive tests for Processor Program functionality

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use processor::state::*;

    use anchor_lang::prelude::*;

    #[test]
    fn test_processor_state_creation() {
        let processor_state = ProcessorState {
            authorization_program_id: generate_test_pubkey("auth_program"),
            is_paused: false,
            high_priority_queue: QueueState::new(100),
            medium_priority_queue: QueueState::new(200),
            low_priority_queue: QueueState::new(300),
            total_executions: 321,
            successful_executions: 300,
            failed_executions: 4,
            last_execution_time: 1000000100,
            owner: generate_test_pubkey("owner"),
            bump: 255,
        };

        // Test serialization
        let serialized = processor_state.try_to_vec().unwrap();
        let deserialized: ProcessorState = ProcessorState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.authorization_program_id, processor_state.authorization_program_id);
        assert!(!deserialized.is_paused);
        assert_eq!(deserialized.total_executions, 321);
        assert_eq!(deserialized.successful_executions, 300);
        assert_eq!(deserialized.failed_executions, 4);

        // Test success rate calculation
        let success_rate = (processor_state.successful_executions as f64) / (processor_state.total_executions as f64);
        assert!(success_rate > 0.9); // Should have high success rate
    }

    #[test]
    fn test_queue_state_management() {
        let mut queue_state = QueueState::new(10);

        // Test initial state
        assert_eq!(queue_state.capacity, 10);
        assert_eq!(queue_state.count, 0);
        assert!(queue_state.is_empty());
        assert!(!queue_state.is_full());

        // Test enqueue operations
        for i in 0..5 {
            let index = queue_state.enqueue().unwrap();
            assert_eq!(index, i);
        }
        
        assert_eq!(queue_state.count, 5);
        assert!(!queue_state.is_empty());
        assert!(!queue_state.is_full());

        // Test dequeue operations
        for i in 0..3 {
            let index = queue_state.dequeue().unwrap();
            assert_eq!(index, i);
        }
        
        assert_eq!(queue_state.count, 2);

        // Test serialization
        let serialized = queue_state.try_to_vec().unwrap();
        let deserialized: QueueState = QueueState::try_from_slice(&serialized).unwrap();
        assert_eq!(deserialized.capacity, 10);
        assert_eq!(deserialized.count, 2);
    }

    #[test]
    fn test_message_batch_creation() {
        let ctx = TestContext::new();
        
        let message_batch = MessageBatch {
            execution_id: 12345,
            messages: vec![
                ProcessorMessage {
                    program_id: generate_test_pubkey("token_program"),
                    data: vec![1, 2, 3, 4],
                    accounts: vec![
                        AccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        AccountMetaData {
                            pubkey: generate_test_pubkey("token_account"),
                            is_signer: false,
                            is_writable: true,
                        },
                    ],
                },
            ],
            subroutine_type: SubroutineType::Atomic,
            expiration_time: Some(1000000000),
            priority: Priority::High,
            caller: ctx.user,
            callback_address: generate_test_pubkey("callback"),
            created_at: 999999999,
            bump: 254,
        };

        // Test serialization
        let serialized = message_batch.try_to_vec().unwrap();
        let deserialized: MessageBatch = MessageBatch::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.execution_id, 12345);
        assert_eq!(deserialized.messages.len(), 1);
        assert_eq!(deserialized.subroutine_type, SubroutineType::Atomic);
        assert_eq!(deserialized.priority, Priority::High);
        assert_eq!(deserialized.caller, ctx.user);

        // Test message validation
        assert!(!message_batch.messages.is_empty(), "Should have messages");
        
        for message in &message_batch.messages {
            assert!(!message.data.is_empty(), "Message data should not be empty");
            assert!(!message.accounts.is_empty(), "Message should have accounts");
            assert!(message.data.len() <= 1024, "Message data should be reasonable size");
        }
    }

    #[test]
    fn test_priority_levels() {
        // Test priority ordering
        let priorities = vec![
            Priority::Low,
            Priority::Medium,
            Priority::High,
        ];

        // Test serialization
        for priority in priorities {
            let serialized = priority.try_to_vec().unwrap();
            let deserialized: Priority = Priority::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, priority);
        }

        // Test priority conversion from u8
        assert_eq!(Priority::from(0), Priority::Low);
        assert_eq!(Priority::from(1), Priority::Medium);
        assert_eq!(Priority::from(2), Priority::High);
        assert_eq!(Priority::from(255), Priority::High); // Default to high for unknown values
    }

    #[test]
    fn test_subroutine_types() {
        let subroutine_types = vec![
            SubroutineType::Atomic,
            SubroutineType::NonAtomic,
        ];

        for sub_type in subroutine_types {
            // Test serialization
            let serialized = sub_type.try_to_vec().unwrap();
            let deserialized: SubroutineType = SubroutineType::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, sub_type);
        }

        // Test conversion from u8
        assert_eq!(SubroutineType::from(0), SubroutineType::Atomic);
        assert_eq!(SubroutineType::from(1), SubroutineType::NonAtomic);
        assert_eq!(SubroutineType::from(255), SubroutineType::NonAtomic); // Default to non-atomic
    }

    #[test]
    fn test_execution_result_handling() {
        let execution_results = vec![
            ExecutionResult::Success,
            ExecutionResult::Failure,
        ];

        for result in execution_results {
            // Test serialization
            let serialized = result.try_to_vec().unwrap();
            let deserialized: ExecutionResult = ExecutionResult::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, result);
        }
    }

    #[test]
    fn test_pending_callback_tracking() {
        let _ctx = TestContext::new();
        
        let pending_callback = PendingCallback {
            execution_id: 12345,
            callback_address: generate_test_pubkey("callback"),
            result: ExecutionResult::Success,
            executed_count: 5,
            error_data: None,
            created_at: 1000000000,
            bump: 254,
        };

        // Test serialization
        let serialized = pending_callback.try_to_vec().unwrap();
        let deserialized: PendingCallback = PendingCallback::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.execution_id, 12345);
        assert_eq!(deserialized.result, ExecutionResult::Success);
        assert_eq!(deserialized.executed_count, 5);
        assert!(deserialized.error_data.is_none());
    }

    #[test]
    fn test_queue_capacity_management() {
        // Test different queue capacities
        let small_queue = QueueState::new(5);
        let medium_queue = QueueState::new(50);
        let large_queue = QueueState::new(500);

        assert_eq!(small_queue.capacity, 5);
        assert_eq!(medium_queue.capacity, 50);
        assert_eq!(large_queue.capacity, 500);

        // Test queue full condition
        let mut full_queue = QueueState::new(2);
        full_queue.enqueue().unwrap();
        full_queue.enqueue().unwrap();
        
        assert!(full_queue.is_full());
        assert!(full_queue.enqueue().is_err()); // Should fail when full
    }

    #[test]
    fn test_message_size_limits() {
        let ctx = TestContext::new();
        
        // Test different message sizes
        let small_message = ProcessorMessage {
            program_id: generate_test_pubkey("program"),
            data: vec![1, 2, 3],
            accounts: vec![
                AccountMetaData {
                    pubkey: ctx.user,
                    is_signer: true,
                    is_writable: false,
                },
            ],
        };

        let large_message = ProcessorMessage {
            program_id: generate_test_pubkey("program"),
            data: vec![0u8; 1000], // Large data
            accounts: vec![
                AccountMetaData {
                    pubkey: ctx.user,
                    is_signer: true,
                    is_writable: false,
                },
            ],
        };

        // Test serialization of different sizes
        let small_serialized = small_message.try_to_vec().unwrap();
        let large_serialized = large_message.try_to_vec().unwrap();

        assert!(small_serialized.len() < large_serialized.len());
        assert!(large_serialized.len() > 1000); // Should be larger due to data
    }

    #[test]
    fn test_account_meta_data() {
        let ctx = TestContext::new();
        
        let account_meta = AccountMetaData {
            pubkey: ctx.user,
            is_signer: true,
            is_writable: false,
        };

        // Test serialization
        let serialized = account_meta.try_to_vec().unwrap();
        let deserialized: AccountMetaData = AccountMetaData::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.pubkey, ctx.user);
        assert!(deserialized.is_signer);
        assert!(!deserialized.is_writable);
    }
} 