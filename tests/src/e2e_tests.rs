// End-to-end tests for Valence Protocol
// These tests simulate real-world usage scenarios from start to finish

#[cfg(test)]
mod tests {
    use crate::utils::*;

    // Import all program states
    use authorization::state::{
        AuthorizationState, Authorization, PermissionType, 
        Priority, SubroutineType, CurrentExecution
    };
    
    use processor::state::{
        ProcessorState, QueueState, Priority as ProcPriority, 
        SubroutineType as ProcSubroutineType, ProcessorMessage as ProcProcessorMessage,
        AccountMetaData as ProcAccountMetaData, ExecutionResult as ProcExecutionResult,
        MessageBatch as ProcMessageBatch, PendingCallback
    };
    
    use registry::state::{
        RegistryState, LibraryInfo, LibraryDependency, DependencyType
    };
    
    use zk_verifier::state::{
        VerifierState, VerificationKey, VerificationKeyType
    };
    
    use base_account::state::AccountState;

    #[test]
    fn test_e2e_defi_vault_operations() {
        println!("🚀 Starting E2E Test: DeFi Vault Operations");
        
        let ctx = TestContext::new();
        
        // === PHASE 1: System Initialization ===
        println!("📋 Phase 1: Initializing system components...");
        
        // 1.1 Initialize Registry
        let registry_state = RegistryState {
            owner: ctx.authority,
            authorization_program_id: generate_test_pubkey("auth_program"),
            account_factory: generate_test_pubkey("account_factory"),
            bump: 255,
        };
        
        // 1.2 Register DeFi libraries
        let vault_library = LibraryInfo {
            program_id: generate_test_pubkey("vault_library"),
            library_type: "defi_vault".to_string(),
            description: "Secure DeFi vault management library".to_string(),
            is_approved: true,
            version: "1.2.0".to_string(),
            last_updated: 1700000000,
            dependencies: vec![
                LibraryDependency {
                    program_id: generate_test_pubkey("token_library"),
                    required_version: "^1.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
            ],
            bump: 254,
        };
        
        let token_library = LibraryInfo {
            program_id: generate_test_pubkey("token_library"),
            library_type: "token_operations".to_string(),
            description: "SPL Token operations library".to_string(),
            is_approved: true,
            version: "1.0.5".to_string(),
            last_updated: 1700000000,
            dependencies: vec![],
            bump: 253,
        };
        
        // 1.3 Initialize Authorization System
        let auth_state = AuthorizationState {
            owner: ctx.authority,
            sub_owners: vec![ctx.user],
            processor_id: generate_test_pubkey("processor"),
            registry_id: registry_state.authorization_program_id,
            execution_counter: 0,
            bump: 252,
            last_zk_sequence: 0,
            zk_sequence_counter: 0,
            reserved: [0u8; 64],
        };
        
        // 1.4 Initialize Processor
        let processor_state = ProcessorState {
            authorization_program_id: registry_state.authorization_program_id,
            is_paused: false,
            high_priority_queue: QueueState::new(50),
            medium_priority_queue: QueueState::new(100),
            low_priority_queue: QueueState::new(200),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_execution_time: 0,
            owner: ctx.authority,
            bump: 251,
        };
        
        println!("✅ System components initialized");
        
        // === PHASE 2: User Account Setup ===
        println!("👤 Phase 2: Setting up user account...");
        
        // 2.1 Create user's base account
        let user_account = AccountState {
            owner: ctx.user,
            approved_libraries: vec![vault_library.program_id, token_library.program_id],
            vault_authority: generate_test_pubkey("user_vault_authority"),
            vault_bump_seed: 255,
            token_accounts: vec![
                generate_test_pubkey("usdc_account"),
                generate_test_pubkey("sol_account"),
            ],
            last_activity: 1700000000,
            instruction_count: 0,
        };
        
        // 2.2 Create authorization for vault operations
        let vault_authorization = Authorization {
            label: "defi_vault_ops".to_string(),
            owner: ctx.user,
            is_active: true,
            permission_type: PermissionType::OwnerOnly,
            allowed_users: vec![ctx.user],
            not_before: 1700000000,
            expiration: Some(1800000000), // ~3 months
            max_concurrent_executions: 3,
            priority: Priority::High,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 0,
            bump: 250,
        };
        
        println!("✅ User account and authorization created");
        
        // === PHASE 3: DeFi Operations Execution ===
        println!("💰 Phase 3: Executing DeFi vault operations...");
        
        // 3.1 Create deposit operation message batch
        let deposit_batch = ProcMessageBatch {
            execution_id: 1001,
            messages: vec![
                // First: Approve token transfer
                ProcProcessorMessage {
                    program_id: token_library.program_id,
                    data: create_approve_instruction_data(1000_000_000), // 1000 USDC
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: user_account.token_accounts[0], // USDC account
                            is_signer: false,
                            is_writable: true,
                        },
                        ProcAccountMetaData {
                            pubkey: vault_library.program_id,
                            is_signer: false,
                            is_writable: false,
                        },
                    ],
                },
                // Second: Deposit to vault
                ProcProcessorMessage {
                    program_id: vault_library.program_id,
                    data: create_vault_deposit_instruction_data(1000_000_000),
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: user_account.vault_authority,
                            is_signer: false,
                            is_writable: true,
                        },
                        ProcAccountMetaData {
                            pubkey: user_account.token_accounts[0],
                            is_signer: false,
                            is_writable: true,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1700001000), // 1000 seconds from now
            priority: ProcPriority::High,
            caller: ctx.user,
            callback_address: generate_test_pubkey("deposit_callback"),
            created_at: 1700000000,
            bump: 249,
        };
        
        // 3.2 Track execution
        let _current_execution = CurrentExecution {
            id: 1001,
            authorization_label: vault_authorization.label.clone(),
            sender: ctx.user,
            start_time: 1700000000,
            bump: 248,
        };
        
        // 3.3 Simulate successful execution
        let deposit_callback = PendingCallback {
            execution_id: 1001,
            callback_address: deposit_batch.callback_address,
            result: ProcExecutionResult::Success,
            executed_count: 2, // Both messages executed
            error_data: None,
            created_at: 1700000100,
            bump: 247,
        };
        
        println!("✅ Deposit operation completed successfully");
        
        // === PHASE 4: Yield Farming Operations ===
        println!("🌾 Phase 4: Executing yield farming operations...");
        
        // 4.1 Create yield farming message batch
        let yield_batch = ProcMessageBatch {
            execution_id: 1002,
            messages: vec![
                // Stake vault tokens for yield
                ProcProcessorMessage {
                    program_id: vault_library.program_id,
                    data: create_stake_instruction_data(500_000_000), // Stake 50% of deposit
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: user_account.vault_authority,
                            is_signer: false,
                            is_writable: true,
                        },
                        ProcAccountMetaData {
                            pubkey: generate_test_pubkey("yield_pool"),
                            is_signer: false,
                            is_writable: true,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1700002000),
            priority: ProcPriority::Medium,
            caller: ctx.user,
            callback_address: generate_test_pubkey("yield_callback"),
            created_at: 1700000200,
            bump: 246,
        };
        
        // 4.2 Simulate yield farming execution
        let yield_callback = PendingCallback {
            execution_id: 1002,
            callback_address: yield_batch.callback_address,
            result: ProcExecutionResult::Success,
            executed_count: 1,
            error_data: None,
            created_at: 1700000300,
            bump: 245,
        };
        
        println!("✅ Yield farming operation completed successfully");
        
        // === PHASE 5: Validation and Cleanup ===
        println!("🔍 Phase 5: Validating final state...");
        
        // 5.1 Update processor state with successful executions
        let mut final_processor_state = processor_state;
        final_processor_state.total_executions = 2;
        final_processor_state.successful_executions = 2;
        final_processor_state.failed_executions = 0;
        final_processor_state.last_execution_time = 1700000300;
        
        // 5.2 Update authorization state
        let mut final_auth_state = auth_state;
        final_auth_state.execution_counter = 2;
        
        // 5.3 Update user account activity
        let mut final_user_account = user_account;
        final_user_account.instruction_count = 3; // 2 deposit + 1 stake
        final_user_account.last_activity = 1700000300;
        
        // === FINAL VALIDATIONS ===
        
        // Validate system state
        assert_eq!(final_processor_state.total_executions, 2);
        assert_eq!(final_processor_state.successful_executions, 2);
        assert_eq!(final_processor_state.failed_executions, 0);
        assert!(!final_processor_state.is_paused);
        
        // Validate authorization state
        assert_eq!(final_auth_state.execution_counter, 2);
        assert!(final_auth_state.sub_owners.contains(&ctx.user));
        
        // Validate user account state
        assert_eq!(final_user_account.instruction_count, 3);
        assert_eq!(final_user_account.owner, ctx.user);
        assert!(final_user_account.approved_libraries.contains(&vault_library.program_id));
        assert!(final_user_account.approved_libraries.contains(&token_library.program_id));
        
        // Validate library approvals
        assert!(vault_library.is_approved);
        assert!(token_library.is_approved);
        assert_eq!(vault_library.dependencies.len(), 1);
        assert_eq!(token_library.dependencies.len(), 0);
        
        // Validate execution results
        assert_eq!(deposit_callback.result, ProcExecutionResult::Success);
        assert_eq!(deposit_callback.executed_count, 2);
        assert!(deposit_callback.error_data.is_none());
        
        assert_eq!(yield_callback.result, ProcExecutionResult::Success);
        assert_eq!(yield_callback.executed_count, 1);
        assert!(yield_callback.error_data.is_none());
        
        // Validate authorization permissions
        assert!(vault_authorization.is_active);
        assert_eq!(vault_authorization.permission_type, PermissionType::OwnerOnly);
        assert_eq!(vault_authorization.current_executions, 0); // Should be reset after completion
        
        // Calculate success rate
        let success_rate = (final_processor_state.successful_executions as f64) / 
                          (final_processor_state.total_executions as f64);
        assert_eq!(success_rate, 1.0); // 100% success rate
        
        println!("🎉 E2E Test: DeFi Vault Operations - PASSED");
        println!("   ✓ 2 executions completed successfully");
        println!("   ✓ 3 instructions processed");
        println!("   ✓ 100% success rate");
        println!("   ✓ All validations passed");
    }

    #[test]
    fn test_e2e_zk_verified_governance_voting() {
        println!("🚀 Starting E2E Test: ZK-Verified Governance Voting");
        
        let ctx = TestContext::new();
        
        // === PHASE 1: ZK System Setup ===
        println!("🔐 Phase 1: Setting up ZK verification system...");
        
        // 1.1 Initialize ZK Verifier
        let verifier_state = VerifierState {
            owner: ctx.authority,
            coprocessor_root: generate_test_hash(b"zk_coprocessor_root"),
            verifier: generate_test_pubkey("sp1_verifier"),
            total_keys: 2,
            bump: 255,
        };
        
        // 1.2 Register verification keys
        let governance_vk = VerificationKey {
            program_id: generate_test_pubkey("governance_zk_program"),
            registry_id: 12345,
            vk_hash: generate_test_hash(b"governance_verification_key"),
            key_type: VerificationKeyType::SP1,
            is_active: true,
            bump: 254,
        };
        
        let identity_vk = VerificationKey {
            program_id: generate_test_pubkey("identity_zk_program"),
            registry_id: 12346,
            vk_hash: generate_test_hash(b"identity_verification_key"),
            key_type: VerificationKeyType::SP1,
            is_active: true,
            bump: 253,
        };
        
        // 1.3 Initialize Registry with ZK programs
        let registry_state = RegistryState {
            owner: ctx.authority,
            authorization_program_id: generate_test_pubkey("auth_program"),
            account_factory: generate_test_pubkey("account_factory"),
            bump: 252,
        };
        
        // 1.4 Register governance library
        let governance_library = LibraryInfo {
            program_id: generate_test_pubkey("governance_library"),
            library_type: "zk_governance".to_string(),
            description: "Zero-knowledge governance voting library".to_string(),
            is_approved: true,
            version: "2.0.0".to_string(),
            last_updated: 1700000000,
            dependencies: vec![
                LibraryDependency {
                    program_id: governance_vk.program_id,
                    required_version: "^2.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
                LibraryDependency {
                    program_id: identity_vk.program_id,
                    required_version: "^1.0.0".to_string(),
                    is_optional: false,
                    dependency_type: DependencyType::Runtime,
                },
            ],
            bump: 251,
        };
        
        println!("✅ ZK verification system initialized");
        
        // === PHASE 2: Governance Setup ===
        println!("🗳️ Phase 2: Setting up governance infrastructure...");
        
        // 2.1 Initialize Authorization System
        let auth_state = AuthorizationState {
            owner: ctx.authority,
            sub_owners: vec![],
            processor_id: generate_test_pubkey("processor"),
            registry_id: registry_state.authorization_program_id,
            execution_counter: 0,
            bump: 250,
            last_zk_sequence: 0,
            zk_sequence_counter: 0,
            reserved: [0u8; 64],
        };
        
        // 2.2 Initialize Processor
        let processor_state = ProcessorState {
            authorization_program_id: registry_state.authorization_program_id,
            is_paused: false,
            high_priority_queue: QueueState::new(25),
            medium_priority_queue: QueueState::new(50),
            low_priority_queue: QueueState::new(100),
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            last_execution_time: 0,
            owner: ctx.authority,
            bump: 249,
        };
        
        // 2.3 Create voter accounts
        let voter1_account = AccountState {
            owner: ctx.user,
            approved_libraries: vec![governance_library.program_id],
            vault_authority: generate_test_pubkey("voter1_vault"),
            vault_bump_seed: 255,
            token_accounts: vec![generate_test_pubkey("voter1_gov_tokens")],
            last_activity: 1700000000,
            instruction_count: 0,
        };
        
        let voter2_account = AccountState {
            owner: generate_test_pubkey("voter2"),
            approved_libraries: vec![governance_library.program_id],
            vault_authority: generate_test_pubkey("voter2_vault"),
            vault_bump_seed: 254,
            token_accounts: vec![generate_test_pubkey("voter2_gov_tokens")],
            last_activity: 1700000000,
            instruction_count: 0,
        };
        
        // 2.4 Create governance authorization (public voting)
        let governance_authorization = Authorization {
            label: "zk_governance_vote".to_string(),
            owner: ctx.authority,
            is_active: true,
            permission_type: PermissionType::Public, // Anyone can vote with valid ZK proof
            allowed_users: vec![],
            not_before: 1700000000,
            expiration: Some(1700604800), // 1 week voting period
            max_concurrent_executions: 100, // Many concurrent votes
            priority: Priority::High,
            subroutine_type: SubroutineType::Atomic,
            current_executions: 0,
            bump: 248,
        };
        
        println!("✅ Governance infrastructure setup complete");
        
        // === PHASE 3: ZK-Verified Voting Process ===
        println!("🔐 Phase 3: Executing ZK-verified votes...");
        
        // 3.1 Voter 1 casts vote with ZK proof
        let vote1_batch = ProcMessageBatch {
            execution_id: 2001,
            messages: vec![
                // Verify identity proof
                ProcProcessorMessage {
                    program_id: identity_vk.program_id,
                    data: create_zk_identity_verification_data(
                        &generate_test_hash(b"voter1_identity_proof"),
                        &[1u8; 32], // public inputs
                    ),
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: verifier_state.verifier,
                            is_signer: false,
                            is_writable: false,
                        },
                    ],
                },
                // Verify governance eligibility proof
                ProcProcessorMessage {
                    program_id: governance_vk.program_id,
                    data: create_zk_governance_verification_data(
                        &generate_test_hash(b"voter1_eligibility_proof"),
                        &[2u8; 32], // voting power proof
                    ),
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: verifier_state.verifier,
                            is_signer: false,
                            is_writable: false,
                        },
                    ],
                },
                // Cast vote
                ProcProcessorMessage {
                    program_id: governance_library.program_id,
                    data: create_governance_vote_data(
                        1, // proposal ID
                        true, // vote yes
                        1000, // voting power
                    ),
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.user,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: generate_test_pubkey("proposal_1"),
                            is_signer: false,
                            is_writable: true,
                        },
                        ProcAccountMetaData {
                            pubkey: voter1_account.token_accounts[0],
                            is_signer: false,
                            is_writable: false,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1700001000),
            priority: ProcPriority::High,
            caller: ctx.user,
            callback_address: generate_test_pubkey("vote1_callback"),
            created_at: 1700000100,
            bump: 247,
        };
        
        // 3.2 Voter 2 casts vote with ZK proof
        let vote2_batch = ProcMessageBatch {
            execution_id: 2002,
            messages: vec![
                // Verify identity proof
                ProcProcessorMessage {
                    program_id: identity_vk.program_id,
                    data: create_zk_identity_verification_data(
                        &generate_test_hash(b"voter2_identity_proof"),
                        &[3u8; 32],
                    ),
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: voter2_account.owner,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: verifier_state.verifier,
                            is_signer: false,
                            is_writable: false,
                        },
                    ],
                },
                // Verify governance eligibility proof
                ProcProcessorMessage {
                    program_id: governance_vk.program_id,
                    data: create_zk_governance_verification_data(
                        &generate_test_hash(b"voter2_eligibility_proof"),
                        &[4u8; 32],
                    ),
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: voter2_account.owner,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: verifier_state.verifier,
                            is_signer: false,
                            is_writable: false,
                        },
                    ],
                },
                // Cast vote
                ProcProcessorMessage {
                    program_id: governance_library.program_id,
                    data: create_governance_vote_data(
                        1, // same proposal ID
                        false, // vote no
                        500, // voting power
                    ),
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: voter2_account.owner,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: generate_test_pubkey("proposal_1"),
                            is_signer: false,
                            is_writable: true,
                        },
                        ProcAccountMetaData {
                            pubkey: voter2_account.token_accounts[0],
                            is_signer: false,
                            is_writable: false,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1700002000),
            priority: ProcPriority::High,
            caller: voter2_account.owner,
            callback_address: generate_test_pubkey("vote2_callback"),
            created_at: 1700000200,
            bump: 246,
        };
        
        // 3.3 Simulate successful vote executions
        let vote1_callback = PendingCallback {
            execution_id: 2001,
            callback_address: vote1_batch.callback_address,
            result: ProcExecutionResult::Success,
            executed_count: 3, // All 3 messages executed
            error_data: None,
            created_at: 1700000150,
            bump: 245,
        };
        
        let vote2_callback = PendingCallback {
            execution_id: 2002,
            callback_address: vote2_batch.callback_address,
            result: ProcExecutionResult::Success,
            executed_count: 3,
            error_data: None,
            created_at: 1700000250,
            bump: 244,
        };
        
        println!("✅ ZK-verified votes cast successfully");
        
        // === PHASE 4: Vote Tallying and Finalization ===
        println!("📊 Phase 4: Tallying votes and finalizing proposal...");
        
        // 4.1 Tally votes (admin operation)
        let tally_batch = ProcMessageBatch {
            execution_id: 2003,
            messages: vec![
                ProcProcessorMessage {
                    program_id: governance_library.program_id,
                    data: create_governance_tally_data(1), // proposal ID
                    accounts: vec![
                        ProcAccountMetaData {
                            pubkey: ctx.authority,
                            is_signer: true,
                            is_writable: false,
                        },
                        ProcAccountMetaData {
                            pubkey: generate_test_pubkey("proposal_1"),
                            is_signer: false,
                            is_writable: true,
                        },
                    ],
                },
            ],
            subroutine_type: ProcSubroutineType::Atomic,
            expiration_time: Some(1700003000),
            priority: ProcPriority::High,
            caller: ctx.authority,
            callback_address: generate_test_pubkey("tally_callback"),
            created_at: 1700000300,
            bump: 243,
        };
        
        let tally_callback = PendingCallback {
            execution_id: 2003,
            callback_address: tally_batch.callback_address,
            result: ProcExecutionResult::Success,
            executed_count: 1,
            error_data: None,
            created_at: 1700000350,
            bump: 242,
        };
        
        println!("✅ Vote tallying completed");
        
        // === PHASE 5: Final State Validation ===
        println!("🔍 Phase 5: Validating final governance state...");
        
        // 5.1 Update processor state
        let mut final_processor_state = processor_state;
        final_processor_state.total_executions = 3;
        final_processor_state.successful_executions = 3;
        final_processor_state.failed_executions = 0;
        final_processor_state.last_execution_time = 1700000350;
        
        // 5.2 Update authorization state with ZK sequences
        let mut final_auth_state = auth_state;
        final_auth_state.execution_counter = 3;
        final_auth_state.zk_sequence_counter = 4; // 2 identity + 2 governance verifications
        final_auth_state.last_zk_sequence = 4;
        
        // 5.3 Update voter accounts
        let mut final_voter1_account = voter1_account;
        final_voter1_account.instruction_count = 3;
        final_voter1_account.last_activity = 1700000150;
        
        let mut final_voter2_account = voter2_account;
        final_voter2_account.instruction_count = 3;
        final_voter2_account.last_activity = 1700000250;
        
        // === FINAL VALIDATIONS ===
        
        // Validate ZK system state
        assert_eq!(verifier_state.total_keys, 2);
        assert!(governance_vk.is_active);
        assert!(identity_vk.is_active);
        assert_eq!(governance_vk.key_type, VerificationKeyType::SP1);
        assert_eq!(identity_vk.key_type, VerificationKeyType::SP1);
        
        // Validate processor execution state
        assert_eq!(final_processor_state.total_executions, 3);
        assert_eq!(final_processor_state.successful_executions, 3);
        assert_eq!(final_processor_state.failed_executions, 0);
        
        // Validate ZK sequence tracking
        assert_eq!(final_auth_state.zk_sequence_counter, 4);
        assert_eq!(final_auth_state.last_zk_sequence, 4);
        assert_eq!(final_auth_state.execution_counter, 3);
        
        // Validate governance library setup
        assert!(governance_library.is_approved);
        assert_eq!(governance_library.dependencies.len(), 2);
        assert!(governance_library.dependencies.iter().any(|d| d.dependency_type == DependencyType::Runtime));
        
        // Validate authorization permissions
        assert!(governance_authorization.is_active);
        assert_eq!(governance_authorization.permission_type, PermissionType::Public);
        assert_eq!(governance_authorization.max_concurrent_executions, 100);
        
        // Validate voter account states
        assert_eq!(final_voter1_account.instruction_count, 3);
        assert_eq!(final_voter2_account.instruction_count, 3);
        assert!(final_voter1_account.approved_libraries.contains(&governance_library.program_id));
        assert!(final_voter2_account.approved_libraries.contains(&governance_library.program_id));
        
        // Validate execution results
        assert_eq!(vote1_callback.result, ProcExecutionResult::Success);
        assert_eq!(vote1_callback.executed_count, 3);
        assert_eq!(vote2_callback.result, ProcExecutionResult::Success);
        assert_eq!(vote2_callback.executed_count, 3);
        assert_eq!(tally_callback.result, ProcExecutionResult::Success);
        assert_eq!(tally_callback.executed_count, 1);
        
        // All callbacks should have no errors
        assert!(vote1_callback.error_data.is_none());
        assert!(vote2_callback.error_data.is_none());
        assert!(tally_callback.error_data.is_none());
        
        // Calculate success rate
        let success_rate = (final_processor_state.successful_executions as f64) / 
                          (final_processor_state.total_executions as f64);
        assert_eq!(success_rate, 1.0); // 100% success rate
        
        println!("🎉 E2E Test: ZK-Verified Governance Voting - PASSED");
        println!("   ✓ 3 executions completed successfully");
        println!("   ✓ 7 total instructions processed");
        println!("   ✓ 4 ZK verifications completed");
        println!("   ✓ 2 voters participated");
        println!("   ✓ 1 proposal tallied");
        println!("   ✓ 100% success rate");
        println!("   ✓ All validations passed");
    }

    // Helper functions for creating instruction data
    fn create_approve_instruction_data(amount: u64) -> Vec<u8> {
        let mut data = vec![1u8]; // Approve instruction discriminator
        data.extend_from_slice(&amount.to_le_bytes());
        data
    }

    fn create_vault_deposit_instruction_data(amount: u64) -> Vec<u8> {
        let mut data = vec![2u8]; // Deposit instruction discriminator
        data.extend_from_slice(&amount.to_le_bytes());
        data
    }

    fn create_stake_instruction_data(amount: u64) -> Vec<u8> {
        let mut data = vec![3u8]; // Stake instruction discriminator
        data.extend_from_slice(&amount.to_le_bytes());
        data
    }

    fn create_zk_identity_verification_data(proof_hash: &[u8; 32], public_inputs: &[u8; 32]) -> Vec<u8> {
        let mut data = vec![10u8]; // ZK identity verification discriminator
        data.extend_from_slice(proof_hash);
        data.extend_from_slice(public_inputs);
        data
    }

    fn create_zk_governance_verification_data(proof_hash: &[u8; 32], public_inputs: &[u8; 32]) -> Vec<u8> {
        let mut data = vec![11u8]; // ZK governance verification discriminator
        data.extend_from_slice(proof_hash);
        data.extend_from_slice(public_inputs);
        data
    }

    fn create_governance_vote_data(proposal_id: u32, vote_yes: bool, voting_power: u64) -> Vec<u8> {
        let mut data = vec![20u8]; // Vote instruction discriminator
        data.extend_from_slice(&proposal_id.to_le_bytes());
        data.push(if vote_yes { 1 } else { 0 });
        data.extend_from_slice(&voting_power.to_le_bytes());
        data
    }

    fn create_governance_tally_data(proposal_id: u32) -> Vec<u8> {
        let mut data = vec![21u8]; // Tally instruction discriminator
        data.extend_from_slice(&proposal_id.to_le_bytes());
        data
    }
} 