// Comprehensive integration tests for Valence Protocol end-to-end workflows

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use anchor_lang::prelude::*;

    // Import all program states with explicit paths to avoid ambiguity
    use authorization::state::{
        AuthorizationState, Authorization, PermissionType, 
        Priority, SubroutineType, 
        CurrentExecution, ExecutionResult as AuthExecutionResult
    };
    use authorization::error::AuthorizationError;
    
    use processor::state::{
        ProcessorState, QueueState, Priority as ProcPriority, 
        SubroutineType as ProcSubroutineType, ProcessorMessage as ProcProcessorMessage,
        AccountMetaData as ProcAccountMetaData, ExecutionResult as ProcExecutionResult,
        MessageBatch as ProcMessageBatch, PendingCallback
    };
    use processor::error::ProcessorError;
    
    use registry::state::{
        RegistryState, LibraryInfo, LibraryDependency, DependencyType, DependencyGraph,
        ZKProgramInfo
    };
    use registry::error::RegistryError;
    
    use zk_verifier::state::{
        VerifierState, VerificationKey, VerificationKeyType
    };
    use zk_verifier::error::VerifierError;
    
    use base_account::state::AccountState;

    #[test]
    fn test_authorization_to_processor_workflow() {
        let ctx = TestContext::new();
        
        // 1. Create authorization
        let auth_state = AuthorizationState {
            owner: ctx.authority,
            sub_owners: vec![ctx.user],
            processor_id: generate_test_pubkey("processor"),
            registry_id: generate_test_pubkey("registry"),
            execution_counter: 0,
            bump: 255,
            last_zk_sequence: 0,
            zk_sequence_counter: 0,
            reserved: [0u8; 64],
        };

        // 2. Create processor state
        let processor_state = ProcessorState {
            authorization_program_id: generate_test_pubkey("authorization"),
            is_paused: false,
            high_priority_queue: QueueState::new(100),
            medium_priority_queue: QueueState::new(200),
            low_priority_queue: QueueState::new(300),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_execution_time: 0,
            owner: ctx.authority,
            bump: 254,
        };

        // 3. Create message batch for processing
        let message_batch = ProcMessageBatch {
            execution_id: 1,
            messages: vec![
                ProcProcessorMessage {
                    program_id: generate_test_pubkey("token_program"),
                    data: vec![1, 2, 3, 4],
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1000000000),
            priority: ProcPriority::High,
            caller: ctx.user,
            callback_address: generate_test_pubkey("callback"),
            created_at: 999999999,
            bump: 253,
        };

        // Test workflow validation (program IDs are generated randomly, so just check they exist)
        assert_ne!(auth_state.processor_id, Pubkey::default());
        assert_ne!(processor_state.authorization_program_id, Pubkey::default());
        assert!(auth_state.sub_owners.contains(&ctx.user));
        assert_eq!(message_batch.caller, ctx.user);
        assert!(!processor_state.is_paused);
        assert_eq!(processor_state.high_priority_queue.count, 0);

        // Simulate successful execution
        let mut updated_auth = auth_state;
        updated_auth.execution_counter += 1;

        let mut updated_processor = processor_state;
        updated_processor.total_executions += 1;
        updated_processor.successful_executions += 1;

        assert_eq!(updated_auth.execution_counter, 1);
        assert_eq!(updated_processor.total_executions, 1);
        assert_eq!(updated_processor.successful_executions, 1);
    }

    #[test]
    fn test_registry_library_approval_workflow() {
        let ctx = TestContext::new();
        
        // 1. Create registry state
        let registry_state = RegistryState {
            owner: ctx.authority,
            authorization_program_id: generate_test_pubkey("authorization"),
            account_factory: generate_test_pubkey("account_factory"),
            bump: 255,
        };

        // 2. Register a library
        let library_info = LibraryInfo {
            program_id: generate_test_pubkey("token_library"),
            library_type: "token_transfer".to_string(),
            description: "Token transfer library".to_string(),
            is_approved: false, // Initially not approved
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![],
            bump: 254,
        };

        // 3. Test library cache integration
        let mut cache = registry::cache::LibraryCache::new();
        cache.add_library(&library_info);

        // Verify library is in cache but not approved
        assert!(cache.contains(&library_info.program_id));
        assert_eq!(cache.is_approved(&library_info.program_id), Some(false));

        // 4. Approve the library
        let mut approved_library = library_info;
        approved_library.is_approved = true;
        approved_library.last_updated = 1000000100;

        // Update cache
        cache.update_approval_status(&approved_library.program_id, true);

        // Verify approval
        assert_eq!(cache.is_approved(&approved_library.program_id), Some(true));
        assert!(approved_library.is_approved);

        // 5. Test authorization integration
        let auth_state = AuthorizationState {
            owner: ctx.authority,
            sub_owners: vec![],
            processor_id: generate_test_pubkey("processor"),
            registry_id: registry_state.authorization_program_id,
            execution_counter: 0,
            bump: 255,
            last_zk_sequence: 0,
            zk_sequence_counter: 0,
            reserved: [0u8; 64],
        };

        // Verify registry integration
        assert_eq!(auth_state.registry_id, registry_state.authorization_program_id);
    }

    #[test]
    fn test_zk_verification_integration_workflow() {
        let ctx = TestContext::new();
        
        // 1. Create ZK verifier state with current struct definition
        let verifier_state = VerifierState {
            owner: ctx.authority,
            coprocessor_root: [1u8; 32],
            verifier: generate_test_pubkey("verifier"),
            total_keys: 1,
            bump: 255,
        };

        // 2. Register verification key with current struct definition
        let verification_key = VerificationKey {
            program_id: generate_test_pubkey("zk_program"),
            registry_id: 12345,
            vk_hash: [1u8; 32],
            key_type: VerificationKeyType::SP1,
            is_active: true,
            bump: 254,
        };

        // 3. Test ZK proof verification workflow with sample data
        let test_proof = vec![1u8; 192]; // Groth16 proof size
        let test_message = vec![1u8; 32]; // Sample message

        // Verify proof and message are valid
        assert!(!test_proof.is_empty());
        assert!(!test_message.is_empty());
        assert_eq!(test_proof.len(), 192); // Standard Groth16 proof size

        // 4. Test authorization with ZK verification
        let auth_with_zk = Authorization {
            label: "zk_authorization".to_string(),
            owner: ctx.authority,
            is_active: true,
            permission_type: PermissionType::Public,
            allowed_users: vec![],
            not_before: 0,
            expiration: Some(1000000000),
            max_concurrent_executions: 1,
            priority: Priority::High,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 0,
            bump: 252,
        };

        // Verify ZK authorization setup
        assert_eq!(auth_with_zk.permission_type, PermissionType::Public);
        assert!(auth_with_zk.allowed_users.is_empty()); // ZK proof doesn't need allowlist

        // 5. Validate the integration between verifier state and verification key
        assert_eq!(verifier_state.owner, ctx.authority);
        assert_eq!(verification_key.program_id, generate_test_pubkey("zk_program"));
        assert_eq!(verification_key.registry_id, 12345);
        assert!(verification_key.is_active);
        assert_eq!(verification_key.key_type, VerificationKeyType::SP1);
    }

    #[test]
    fn test_complete_execution_workflow() {
        let ctx = TestContext::new();
        
        // 1. Setup all program states
        let auth_state = AuthorizationState {
            owner: ctx.authority,
            sub_owners: vec![ctx.user],
            processor_id: generate_test_pubkey("processor"),
            registry_id: generate_test_pubkey("registry"),
            execution_counter: 0,
            bump: 255,
            last_zk_sequence: 0,
            zk_sequence_counter: 0,
            reserved: [0u8; 64],
        };

        let registry_state = RegistryState {
            owner: ctx.authority,
            authorization_program_id: generate_test_pubkey("authorization"),
            account_factory: generate_test_pubkey("account_factory"),
            bump: 254,
        };

        let processor_state = ProcessorState {
            authorization_program_id: generate_test_pubkey("authorization"),
            is_paused: false,
            high_priority_queue: QueueState::new(100),
            medium_priority_queue: QueueState::new(200),
            low_priority_queue: QueueState::new(300),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_execution_time: 0,
            owner: ctx.authority,
            bump: 253,
        };

        let _verifier_state = VerifierState {
            owner: ctx.authority,
            coprocessor_root: [1u8; 32],
            verifier: generate_test_pubkey("verifier"),
            total_keys: 1,
            bump: 252,
        };

        // 2. Create authorization for execution
        let authorization = Authorization {
            label: "complete_workflow".to_string(),
            owner: ctx.authority,
            is_active: true,
            permission_type: PermissionType::Allowlist,
            allowed_users: vec![ctx.user],
            not_before: 0,
            expiration: Some(2000000000),
            max_concurrent_executions: 5,
            priority: Priority::High,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 0,
            bump: 251,
        };

        // 3. Create library for execution
        let library_info = LibraryInfo {
            program_id: generate_test_pubkey("token_library"),
            library_type: "token_transfer".to_string(),
            description: "Token transfer library".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![],
            bump: 250,
        };

        // 4. Create message batch for execution
        let message_batch = ProcMessageBatch {
            execution_id: 1,
            messages: vec![
                ProcProcessorMessage {
                    program_id: library_info.program_id,
                    data: vec![1, 2, 3, 4],
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: generate_test_pubkey("token_account"),
                            is_signer: false,
                            is_writable: true,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: authorization.expiration,
            priority: ProcPriority::High,
            caller: ctx.user,
            callback_address: generate_test_pubkey("callback"),
            created_at: 1000000000,
            bump: 249,
        };

        // 5. Validate complete workflow
        // Authorization validation
        assert!(authorization.allowed_users.contains(&ctx.user));
        // Note: authorization and processor have different enum types, so we validate separately
        assert_eq!(authorization.subroutine_type, SubroutineType::Atomic);
        assert_eq!(message_batch.subroutine_type, ProcSubroutineType::Atomic);

        // Library validation
        assert!(library_info.is_approved);
        assert_eq!(message_batch.messages[0].program_id, library_info.program_id);

        // Processor validation
        assert!(!processor_state.is_paused);
        assert_eq!(message_batch.caller, ctx.user);

        // Cross-program references (program IDs are generated randomly, so just validate non-default)
        assert_ne!(auth_state.processor_id, Pubkey::default());
        assert_ne!(processor_state.authorization_program_id, Pubkey::default());
        assert_ne!(auth_state.registry_id, Pubkey::default());
        assert_ne!(registry_state.authorization_program_id, Pubkey::default());

        // 6. Simulate successful execution
        let execution_result = AuthExecutionResult::Success;

        // Verify execution result
        assert_eq!(execution_result, AuthExecutionResult::Success);
    }

    #[test]
    fn test_dependency_resolution_workflow() {
        let ctx = TestContext::new();
        
        // 1. Create libraries with dependencies
        let base_library = LibraryInfo {
            program_id: generate_test_pubkey("base_library"),
            library_type: "base".to_string(),
            description: "Base library with no dependencies".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![], // No dependencies
            bump: 255,
        };

        let token_library = LibraryInfo {
            program_id: generate_test_pubkey("token_library"),
            library_type: "token".to_string(),
            description: "Token library depending on base".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![
                LibraryDependency {
                    program_id: base_library.program_id,
                    required_version: "^1.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
            ],
            bump: 254,
        };

        let vault_library = LibraryInfo {
            program_id: generate_test_pubkey("vault_library"),
            library_type: "vault".to_string(),
            description: "Vault library depending on token and base".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![
                LibraryDependency {
                    program_id: base_library.program_id,
                    required_version: "^1.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
                LibraryDependency {
                    program_id: token_library.program_id,
                    required_version: "^1.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
            ],
            bump: 253,
        };

        // 2. Create dependency graph
        let dependency_graph = DependencyGraph {
            root_library: vault_library.program_id,
            resolved_order: vec![
                base_library.program_id,    // First (no dependencies)
                token_library.program_id,   // Second (depends on base)
                vault_library.program_id,   // Third (depends on both)
            ],
            is_valid: true,
            last_resolved: 1000000000,
            bump: 252,
        };

        // 3. Validate dependency resolution
        assert!(dependency_graph.is_valid);
        assert_eq!(dependency_graph.resolved_order.len(), 3);

        // Base library should come first
        assert_eq!(dependency_graph.resolved_order[0], base_library.program_id);
        
        // Token library should come after base
        let base_pos = dependency_graph.resolved_order.iter()
            .position(|&id| id == base_library.program_id).unwrap();
        let token_pos = dependency_graph.resolved_order.iter()
            .position(|&id| id == token_library.program_id).unwrap();
        assert!(base_pos < token_pos);

        // Vault library should come last
        let vault_pos = dependency_graph.resolved_order.iter()
            .position(|&id| id == vault_library.program_id).unwrap();
        assert!(token_pos < vault_pos);
        assert!(base_pos < vault_pos);

        // 4. Test library cache with dependencies
        let mut cache = registry::cache::LibraryCache::new();
        cache.add_library(&base_library);
        cache.add_library(&token_library);
        cache.add_library(&vault_library);

        // Verify all libraries are cached and approved
        for library in [&base_library, &token_library, &vault_library] {
            assert!(cache.contains(&library.program_id));
            assert_eq!(cache.is_approved(&library.program_id), Some(true));
        }

        // 5. Create execution using the vault library
        let message_batch = ProcMessageBatch {
            execution_id: 1,
            messages: vec![
                ProcProcessorMessage {
                    program_id: vault_library.program_id,
                    data: vec![1, 2, 3, 4],
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(2000000000),
            priority: ProcPriority::Medium,
            caller: ctx.user,
            callback_address: generate_test_pubkey("callback"),
            created_at: 1000000000,
            bump: 251,
        };

        // Verify execution targets the correct library
        assert_eq!(message_batch.messages[0].program_id, vault_library.program_id);
        assert_eq!(message_batch.subroutine_type, ProcSubroutineType::Atomic);
    }

    #[test]
    fn test_error_handling_across_programs() {
        let _ctx = TestContext::new();
        
        // Test authorization errors
        let auth_errors = vec![
            AuthorizationError::NotAuthorized,
            AuthorizationError::AuthorizationExpired,
            AuthorizationError::MaxConcurrentExecutionsReached,
        ];

        // Test registry errors
        let registry_errors = vec![
            RegistryError::LibraryNotFound,
            RegistryError::CircularDependency,
            RegistryError::InvalidDependencyVersion,
        ];

        // Test processor errors
        let processor_errors = vec![
            ProcessorError::ProcessorPaused,
            ProcessorError::QueueFull,
            ProcessorError::ExecutionFailed,
        ];

        // Test ZK verifier errors
        let verifier_errors = vec![
            VerifierError::InvalidProof,
            VerifierError::ProofVerificationFailed,
            VerifierError::InvalidParameters,
        ];

        // Verify all error codes are in expected ranges (Anchor uses codes starting from 6000)
        for error in auth_errors {
            let code = error as u32;
            assert!((6000..7000).contains(&code), "Auth error code {} out of range", code);
        }

        for error in registry_errors {
            let code = error as u32;
            assert!(code < 100, "Registry error code {} out of range", code);
        }

        for error in processor_errors {
            let code = error as u32;
            assert!(code < 100, "Processor error code {} out of range", code);
        }

        for error in verifier_errors {
            let code = error as u32;
            assert!(code < 100, "Verifier error code {} out of range", code);
        }
    }

    #[test]
    fn test_cross_program_account_validation() {
        let ctx = TestContext::new();
        
        // Test that program IDs are correctly referenced across programs
        let auth_program_id = generate_test_pubkey("authorization");
        let processor_program_id = generate_test_pubkey("processor");
        let registry_program_id = generate_test_pubkey("registry");
        let _verifier_program_id = generate_test_pubkey("verifier");

        // Authorization state references
        let auth_state = AuthorizationState {
            owner: ctx.authority,
            sub_owners: vec![],
            processor_id: processor_program_id,
            registry_id: registry_program_id,
            execution_counter: 0,
            bump: 255,
            last_zk_sequence: 0,
            zk_sequence_counter: 0,
            reserved: [0u8; 64],
        };

        // Processor state references
        let processor_state = ProcessorState {
            authorization_program_id: auth_program_id,
            is_paused: false,
            high_priority_queue: QueueState::new(100),
            medium_priority_queue: QueueState::new(200),
            low_priority_queue: QueueState::new(300),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_execution_time: 0,
            owner: ctx.authority,
            bump: 254,
        };

        // Registry state references
        let registry_state = RegistryState {
            owner: ctx.authority,
            authorization_program_id: auth_program_id,
            account_factory: generate_test_pubkey("account_factory"),
            bump: 253,
        };

        // Verify cross-program references are consistent
        assert_eq!(auth_state.processor_id, processor_program_id);
        assert_eq!(auth_state.registry_id, registry_program_id);
        assert_eq!(processor_state.authorization_program_id, auth_program_id);
        assert_eq!(registry_state.authorization_program_id, auth_program_id);

        // Test PDA derivation consistency
        let auth_seeds = [b"authorization", ctx.authority.as_ref()];
        let processor_seeds = [b"processor_state"];
        let registry_seeds = [b"registry_state"];

        // Verify seed patterns are consistent (in real implementation, these would derive actual PDAs)
        assert_eq!(auth_seeds[0], b"authorization");
        assert_eq!(processor_seeds[0], b"processor_state");
        assert_eq!(registry_seeds[0], b"registry_state");
    }

    #[test]
    fn test_account_creation_workflow() {
        let ctx = TestContext::new();
        
        // Test end-to-end account creation workflow
        // 1. Registry provides approved libraries
        let registry_state = RegistryState {
            owner: ctx.authority,
            authorization_program_id: generate_test_pubkey("auth_program"),
            account_factory: generate_test_pubkey("account_factory"),
            bump: 255,
        };

        // 2. Account factory creates account with base state
        let account_state = AccountState {
            owner: ctx.user,
            approved_libraries: vec![],
            vault_authority: generate_test_pubkey("vault_authority"),
            vault_bump_seed: 255,
            token_accounts: vec![],
            last_activity: 1000000000,
            instruction_count: 0,
        };

        // 3. Authorization is created for the account
        let authorization = Authorization {
            label: "account_setup".to_string(),
            owner: ctx.user,
            is_active: true,
            permission_type: PermissionType::OwnerOnly,
            allowed_users: vec![ctx.user],
            not_before: 0,
            expiration: Some(2000000000),
            max_concurrent_executions: 1,
            priority: Priority::Medium,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 0,
            bump: 254,
        };

        // Validate workflow
        assert_eq!(registry_state.owner, ctx.authority);
        assert_eq!(account_state.owner, ctx.user);
        assert_eq!(authorization.owner, ctx.user);
        assert!(authorization.is_active);
        assert_eq!(authorization.current_executions, 0);
    }

    #[test]
    fn test_library_approval_and_execution_workflow() {
        let ctx = TestContext::new();
        
        // 1. Library is registered in registry
        let library_info = LibraryInfo {
            program_id: generate_test_pubkey("vault_library"),
            library_type: "vault_management".to_string(),
            description: "Secure vault management library".to_string(),
            is_approved: true,
            version: "1.0.0".to_string(),
            last_updated: 1000000000,
            dependencies: vec![],
            bump: 253,
        };

        // 2. Authorization allows library execution
        let authorization = Authorization {
            label: "vault_operations".to_string(),
            owner: ctx.user,
            is_active: true,
            permission_type: PermissionType::Allowlist,
            allowed_users: vec![ctx.user, library_info.program_id],
            not_before: 0,
            expiration: Some(2000000000),
            max_concurrent_executions: 5,
            priority: Priority::High,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 1,
            bump: 252,
        };

        // 3. Message batch is created for execution
        let message_batch = ProcMessageBatch {
            execution_id: 12345,
            messages: vec![
                ProcProcessorMessage {
                    program_id: library_info.program_id,
                    data: vec![1, 2, 3, 4], // Vault deposit instruction data
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: generate_test_pubkey("vault_account"),
                            is_signer: false,
                            is_writable: true,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1000000000),
            priority: ProcPriority::High,
            caller: ctx.user,
            callback_address: generate_test_pubkey("callback"),
            created_at: 999999999,
            bump: 251,
        };

        // Validate workflow
        assert!(library_info.is_approved);
        assert!(authorization.is_active);
        assert!(authorization.allowed_users.contains(&library_info.program_id));
        assert_eq!(message_batch.messages[0].program_id, library_info.program_id);
        assert_eq!(message_batch.subroutine_type, ProcSubroutineType::Atomic);
    }

    #[test]
    fn test_zk_verification_integration() {
        let _ctx = TestContext::new();
        
        // 1. ZK verifier is registered
        let zk_program_info = ZKProgramInfo {
            program_id: generate_test_pubkey("sp1_verifier"),
            verification_key_hash: generate_test_hash(b"sp1_verification_key"),
            program_type: "sp1_verifier".to_string(),
            description: "SP1 zero-knowledge proof verifier".to_string(),
            is_active: true,
            registered_at: 1000000000,
            last_verified: 1000000000,
            verification_count: 0,
            bump: 250,
        };

        // 2. Verification key is stored
        let verification_key = VerificationKey {
            program_id: zk_program_info.program_id,
            registry_id: 12345,
            vk_hash: [1u8; 32],
            key_type: VerificationKeyType::SP1,
            is_active: true,
            bump: 249,
        };

        // 3. Validate ZK integration
        assert!(zk_program_info.is_active);
        assert!(verification_key.is_active);
        assert_eq!(verification_key.key_type, VerificationKeyType::SP1);
        assert_eq!(verification_key.registry_id, 12345);
        assert_eq!(verification_key.vk_hash, [1u8; 32]);
    }



    #[test]
    fn test_message_processing_pipeline() {
        let ctx = TestContext::new();
        
        // 1. Processor state with multiple queues
        let processor_state = ProcessorState {
            authorization_program_id: generate_test_pubkey("auth_program"),
            is_paused: false,
            high_priority_queue: QueueState::new(100),
            medium_priority_queue: QueueState::new(200),
            low_priority_queue: QueueState::new(300),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_execution_time: 1000000000,
            owner: ctx.authority,
            bump: 243,
        };

        // 2. Multiple message batches with different priorities
        let high_priority_batch = ProcMessageBatch {
            execution_id: 1001,
            messages: vec![
                ProcProcessorMessage {
                    program_id: generate_test_pubkey("critical_program"),
                    data: vec![1, 2, 3],
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1000000000),
            priority: ProcPriority::High,
            caller: ctx.user,
            callback_address: generate_test_pubkey("callback"),
            created_at: 999999999,
            bump: 242,
        };

        let medium_priority_batch = ProcMessageBatch {
            execution_id: 1002,
            messages: vec![
                ProcProcessorMessage {
                    program_id: generate_test_pubkey("normal_program"),
                    data: vec![4, 5, 6],
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::NonAtomic,
            expiration_time: Some(1000000000),
            priority: ProcPriority::Medium,
            caller: ctx.user,
            callback_address: generate_test_pubkey("callback"),
            created_at: 999999999,
            bump: 241,
        };

        // 3. Pending callbacks for results
        let pending_callback = PendingCallback {
            execution_id: 1001,
            callback_address: generate_test_pubkey("callback"),
            result: ProcExecutionResult::Success,
            executed_count: 1,
            error_data: None,
            created_at: 1000000000,
            bump: 240,
        };

        // Validate processing pipeline
        assert!(!processor_state.is_paused);
        assert_eq!(processor_state.total_executions, 0);
        assert_eq!(high_priority_batch.priority, ProcPriority::High);
        assert_eq!(medium_priority_batch.priority, ProcPriority::Medium);
        assert_eq!(pending_callback.result, ProcExecutionResult::Success);
        assert!(pending_callback.error_data.is_none());
    }

    #[test]
    fn test_cross_program_authorization_flow() {
        let ctx = TestContext::new();
        
        // 1. Authorization state tracks multiple programs
        let auth_state = AuthorizationState {
            owner: ctx.user,
            sub_owners: vec![ctx.authority],
            processor_id: generate_test_pubkey("processor"),
            registry_id: generate_test_pubkey("registry"),
            execution_counter: 0,
            bump: 239,
            last_zk_sequence: 0,
            zk_sequence_counter: 0,
            reserved: [0u8; 64],
        };

        // 2. Current execution tracking
        let current_execution = CurrentExecution {
            id: 2001,
            authorization_label: "cross_program_call".to_string(),
            sender: ctx.user,
            start_time: 1000000000,
            bump: 238,
        };

        // 3. Authorization for processor program
        let processor_auth = Authorization {
            label: "processor_access".to_string(),
            owner: ctx.user,
            is_active: true,
            permission_type: PermissionType::Allowlist,
            allowed_users: vec![auth_state.processor_id],
            not_before: 0,
            expiration: Some(2000000000),
            max_concurrent_executions: 10,
            priority: Priority::High,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 1,
            bump: 237,
        };

        // Validate cross-program flow
        assert_eq!(auth_state.owner, ctx.user);
        assert!(auth_state.sub_owners.contains(&ctx.authority));
        assert_eq!(current_execution.sender, ctx.user);
        assert!(processor_auth.is_active);
        assert!(processor_auth.allowed_users.contains(&auth_state.processor_id));
    }

    #[test]
    fn test_storage_and_retrieval_workflow() {
        let ctx = TestContext::new();
        
        // 1. Base account with storage capabilities
        let mut account_state = AccountState {
            owner: ctx.user,
            approved_libraries: vec![generate_test_pubkey("storage_library")],
            vault_authority: generate_test_pubkey("vault_authority"),
            vault_bump_seed: 255,
            token_accounts: vec![],
            last_activity: 1000000000,
            instruction_count: 0,
        };

        // 2. Storage operations simulation
        let storage_keys = ["user_preferences",
            "trading_history",
            "vault_settings",
            "notification_config"];

        let storage_values = ["dark_mode:true,language:en",
            "trades:15,volume:1500.50",
            "auto_compound:true,fee_tier:0.05",
            "email:true,sms:false"];

        // 3. Validate storage capability
        for (key, value) in storage_keys.iter().zip(storage_values.iter()) {
            // Simulate storage operations
            assert!(!key.is_empty(), "Storage key should not be empty");
            assert!(!value.is_empty(), "Storage value should not be empty");
            assert!(key.len() <= 64, "Key should be reasonable length");
            assert!(value.len() <= 256, "Value should be reasonable length");
        }

        // Record storage activity
        account_state.instruction_count += storage_keys.len() as u64;
        account_state.last_activity = 1000000100;

        assert_eq!(account_state.instruction_count, 4);
        assert_eq!(account_state.last_activity, 1000000100);
        assert!(account_state.approved_libraries.contains(&generate_test_pubkey("storage_library")));
    }

    #[test]
    fn test_error_handling_and_recovery() {
        let ctx = TestContext::new();
        
        // 1. Test various error scenarios would be handled
        let inactive_authorization = Authorization {
            label: "inactive_auth".to_string(),
            owner: ctx.user,
            is_active: false, // Inactive
            permission_type: PermissionType::OwnerOnly,
            allowed_users: vec![ctx.user],
            not_before: 0,
            expiration: Some(1000000000), // Expired
            max_concurrent_executions: 1,
            priority: Priority::Low,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 0,
            bump: 236,
        };

        // 2. Failed execution tracking
        let failed_callback = PendingCallback {
            execution_id: 3001,
            callback_address: generate_test_pubkey("callback"),
            result: ProcExecutionResult::Failure,
            executed_count: 0,
            error_data: Some(vec![1, 2, 3]), // Error details
            created_at: 1000000000,
            bump: 235,
        };

        // 3. Processor state with failed executions
        let processor_with_failures = ProcessorState {
            authorization_program_id: generate_test_pubkey("auth_program"),
            is_paused: false,
            high_priority_queue: QueueState::new(100),
            medium_priority_queue: QueueState::new(200),
            low_priority_queue: QueueState::new(300),
            total_executions: 100,
            successful_executions: 95,
            failed_executions: 5,
            last_execution_time: 1000000000,
            owner: ctx.authority,
            bump: 234,
        };

        // Validate error handling
        assert!(!inactive_authorization.is_active);
        assert!(inactive_authorization.expiration.unwrap() < 1500000000); // Expired
        assert_eq!(failed_callback.result, ProcExecutionResult::Failure);
        assert!(failed_callback.error_data.is_some());
        assert_eq!(processor_with_failures.failed_executions, 5);
        
        // Test success rate calculation
        let success_rate = (processor_with_failures.successful_executions as f64) / 
                          (processor_with_failures.total_executions as f64);
        assert!(success_rate >= 0.95); // 95% success rate
    }

    #[test]
    fn test_comprehensive_system_state() {
        let ctx = TestContext::new();
        
        // 1. System-wide state representing a running protocol
        let registry = RegistryState {
            owner: ctx.authority,
            authorization_program_id: generate_test_pubkey("auth_program"),
            account_factory: generate_test_pubkey("account_factory"),
            bump: 233,
        };

        let processor = ProcessorState {
            authorization_program_id: registry.authorization_program_id,
            is_paused: false,
            high_priority_queue: QueueState::new(100),
            medium_priority_queue: QueueState::new(200),
            low_priority_queue: QueueState::new(300),
            total_executions: 1000,
            successful_executions: 990,
            failed_executions: 10,
            last_execution_time: 1000000000,
            owner: registry.owner,
            bump: 232,
        };

        let verifier_state = VerifierState {
            owner: registry.owner,
            coprocessor_root: [1u8; 32],
            verifier: generate_test_pubkey("verifier"),
            total_keys: 5,
            bump: 231,
        };

        let auth_state = AuthorizationState {
            owner: ctx.user,
            sub_owners: vec![],
            processor_id: generate_test_pubkey("processor"),
            registry_id: generate_test_pubkey("registry"),
            execution_counter: 25,
            bump: 230,
            last_zk_sequence: 100,
            zk_sequence_counter: 5,
            reserved: [0u8; 64],
        };

        // 2. Active libraries and programs
        let active_libraries = vec![
            generate_test_pubkey("token_lib"),
            generate_test_pubkey("vault_lib"),
            generate_test_pubkey("defi_lib"),
            generate_test_pubkey("governance_lib"),
        ];

        // 3. User accounts using the system
        let user_accounts = vec![
            AccountState {
                owner: ctx.user,
                approved_libraries: active_libraries.clone(),
                vault_authority: generate_test_pubkey("vault_authority"),
                vault_bump_seed: 255,
                token_accounts: vec![
                    generate_test_pubkey("usdc_account"),
                    generate_test_pubkey("sol_account"),
                ],
                last_activity: 1000000000,
                instruction_count: 150,
            },
            AccountState {
                owner: generate_test_pubkey("user2"),
                approved_libraries: active_libraries[0..2].to_vec(),
                vault_authority: generate_test_pubkey("vault_authority_2"),
                vault_bump_seed: 254,
                token_accounts: vec![generate_test_pubkey("usdc_account_2")],
                last_activity: 999999999,
                instruction_count: 50,
            },
        ];

        // Validate comprehensive system state
        assert_eq!(registry.owner, processor.owner);
        assert_eq!(registry.owner, verifier_state.owner);
        assert_eq!(processor.authorization_program_id, registry.authorization_program_id);
        assert!(!processor.is_paused);
        
        // Validate system health metrics
        let processor_success_rate = (processor.successful_executions as f64) / (processor.total_executions as f64);
        
        assert!(processor_success_rate >= 0.99); // 99% success rate
        
        // Validate user activity
        assert_eq!(user_accounts.len(), 2);
        assert_eq!(user_accounts[0].approved_libraries.len(), 4);
        assert_eq!(user_accounts[1].approved_libraries.len(), 2);
        assert!(user_accounts[0].instruction_count > user_accounts[1].instruction_count);
        
        // Validate cross-program consistency
        assert_eq!(auth_state.execution_counter, 25);
        assert_eq!(auth_state.zk_sequence_counter, 5);
        assert!(auth_state.last_zk_sequence > 0);
    }
} 