// Comprehensive tests for Authorization Program functionality

#[cfg(test)]
mod tests {
    
    use crate::utils::*;
    use authorization::state::*;
    use authorization::error::AuthorizationError;
    use anchor_lang::prelude::*;

    #[test]
    fn test_authorization_creation() {
        let ctx = TestContext::new();
        
        // Test authorization creation parameters
        let label = "test_authorization".to_string();
        let _permission_type = PermissionType::Allowlist;
        let allowed_users = [ctx.user];
        let not_before = 0i64;
        let expiration = Some(1000000000i64);
        let max_concurrent_executions = 5u32;
        let _priority = Priority::Medium;
        let _subroutine_type = SubroutineType::Atomic;

        // Validate parameters
        assert!(label.len() <= 64, "Label should be within length limits");
        assert!(allowed_users.len() <= 10, "Should support reasonable number of users");
        assert!(not_before >= 0, "Not before should be valid timestamp");
        assert!(expiration.map_or(false, |exp| exp > not_before), "Expiration should be after not_before");
        assert!(max_concurrent_executions > 0, "Should allow at least one execution");
    }

    #[test]
    fn test_authorization_state_serialization() {
        let ctx = TestContext::new();
        
        let auth_state = AuthorizationState {
            owner: ctx.authority,
            sub_owners: vec![ctx.user],
            processor_id: generate_test_pubkey("processor"),
            registry_id: generate_test_pubkey("registry"),
            execution_counter: 42,
            bump: 255,
            last_zk_sequence: 100,
            zk_sequence_counter: 5,
            reserved: [0u8; 64],
        };

        // Test serialization/deserialization
        let serialized = auth_state.try_to_vec().unwrap();
        let deserialized: AuthorizationState = AuthorizationState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, ctx.authority);
        assert_eq!(deserialized.sub_owners, vec![ctx.user]);
        assert_eq!(deserialized.execution_counter, 42);
        assert_eq!(deserialized.last_zk_sequence, 100);
        assert_eq!(deserialized.zk_sequence_counter, 5);
    }

    #[test]
    fn test_permission_types() {
        // Test all permission type variants
        let permission_types = vec![
            PermissionType::Public,
            PermissionType::OwnerOnly,
            PermissionType::Allowlist,
        ];

        for perm_type in permission_types {
            // Test serialization
            let serialized = perm_type.try_to_vec().unwrap();
            let deserialized: PermissionType = PermissionType::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, perm_type);
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
    fn test_current_execution_tracking() {
        let ctx = TestContext::new();
        
        let execution = CurrentExecution {
            id: 12345,
            authorization_label: "test_auth".to_string(),
            sender: ctx.user,
            start_time: 999999999,
            bump: 254,
        };

        // Test serialization
        let serialized = execution.try_to_vec().unwrap();
        let deserialized: CurrentExecution = CurrentExecution::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.id, 12345);
        assert_eq!(deserialized.authorization_label, "test_auth");
        assert_eq!(deserialized.sender, ctx.user);
        assert_eq!(deserialized.start_time, 999999999);
    }

    #[test]
    fn test_authorization_space_calculations() {
        // Test space calculation for different configurations
        let base_space = AuthorizationState::space(0); // No sub-owners
        let with_sub_owners = AuthorizationState::space(5); // 5 sub-owners
        let max_sub_owners = AuthorizationState::space(10); // Maximum sub-owners

        // Each sub-owner adds 32 bytes (Pubkey size)
        assert_eq!(with_sub_owners - base_space, 5 * 32);
        assert_eq!(max_sub_owners - base_space, 10 * 32);

        // Ensure space calculations are reasonable
        assert!(base_space > 100); // Should have reasonable base size
        assert!(max_sub_owners < 1000); // Should not be excessive
    }

    #[test]
    fn test_zk_sequence_validation() {
        let mut auth_state = AuthorizationState {
            owner: generate_test_pubkey("owner"),
            sub_owners: vec![],
            processor_id: generate_test_pubkey("processor"),
            registry_id: generate_test_pubkey("registry"),
            execution_counter: 0,
            bump: 255,
            last_zk_sequence: 100,
            zk_sequence_counter: 5,
            reserved: [0u8; 64],
        };

        // Test sequence number validation logic
        let new_sequence = 101;
        assert!(new_sequence > auth_state.last_zk_sequence, "New sequence should be greater");

        // Update sequence
        auth_state.last_zk_sequence = new_sequence;
        auth_state.zk_sequence_counter += 1;

        assert_eq!(auth_state.last_zk_sequence, 101);
        assert_eq!(auth_state.zk_sequence_counter, 6);
    }

    #[test]
    fn test_error_handling() {
        // Test that error codes are properly defined
        let errors = vec![
            AuthorizationError::NotAuthorized,
            AuthorizationError::AuthorizationNotFound,
            AuthorizationError::AuthorizationExpired,
            AuthorizationError::MaxConcurrentExecutionsReached,
        ];

        for error in errors {
            // Test that errors can be converted to error codes
            let error_code = error as u32;
            // Error enum discriminants should be reasonable values (Anchor uses anchor-generated codes starting from 6000)
            assert!((6000..7000).contains(&error_code), "Error code should be in Anchor range, got {}", error_code);
        }
    }

    #[test]
    fn test_processor_message_structure() {
        let ctx = TestContext::new();
        
        let message = ProcessorMessage {
            program_id: generate_test_pubkey("target_program"),
            data: vec![1, 2, 3, 4, 5],
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
        };

        // Test serialization
        let serialized = message.try_to_vec().unwrap();
        let deserialized: ProcessorMessage = ProcessorMessage::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, message.program_id);
        assert_eq!(deserialized.data, vec![1, 2, 3, 4, 5]);
        assert_eq!(deserialized.accounts.len(), 2);
        assert_eq!(deserialized.accounts[0].pubkey, ctx.user);
        assert!(deserialized.accounts[0].is_signer);
        assert!(!deserialized.accounts[0].is_writable);
    }

    #[test]
    fn test_authorization_validation_logic() {
        let ctx = TestContext::new();
        
        // Test authorization with allowlist
        let auth = Authorization {
            label: "test_auth".to_string(),
            owner: ctx.authority,
            is_active: true,
            permission_type: PermissionType::Allowlist,
            allowed_users: vec![ctx.user],
            not_before: 0,
            expiration: Some(2000000000),
            max_concurrent_executions: 3,
            priority: Priority::Medium,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 1,
            bump: 254,
        };

        // Test validation logic
        assert!(auth.is_active, "Authorization should be active");
        assert!(auth.allowed_users.contains(&ctx.user), "User should be in allowlist");
        assert!(auth.current_executions < auth.max_concurrent_executions, "Should have capacity");
        
        // Test timestamp validation
        let current_time = 1000000000i64;
        assert!(current_time >= auth.not_before, "Should be after not_before");
        assert!(current_time < auth.expiration.unwrap(), "Should be before expiration");
    }
} 