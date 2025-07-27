// Comprehensive unit tests for valence-core
#[cfg(test)]
mod tests {
    use crate::{
        guards::*,
        state::*,
        MODE_READ, MODE_WRITE, MODE_READ_WRITE,
    };
    use anchor_lang::prelude::*;
    
    // ================================
    // Guard Tests
    // ================================
    
    #[test]
    fn test_guard_op_direct_construction() {
        // Test direct construction of compiled guards (as would be done client-side)
        let compiled = CompiledGuard {
            opcodes: vec![GuardOp::CheckOwner],
            cpi_manifest: vec![],
        };
        assert_eq!(compiled.opcodes.len(), 1);
        assert!(matches!(compiled.opcodes[0], GuardOp::CheckOwner));
        
        // Test expiration guard
        let compiled = CompiledGuard {
            opcodes: vec![GuardOp::CheckExpiry { timestamp: 1234567890 }],
            cpi_manifest: vec![],
        };
        assert_eq!(compiled.opcodes.len(), 1);
        assert!(matches!(
            compiled.opcodes[0], 
            GuardOp::CheckExpiry { timestamp: 1234567890 }
        ));
    }
    
    #[test]
    fn test_guard_op_and_construction() {
        // Test AND logic construction (as would be done client-side)
        let compiled = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckOwner,
                GuardOp::JumpIfFalse { offset: 2 },
                GuardOp::CheckUsageLimit { limit: 10 },
                GuardOp::Terminate,
            ],
            cpi_manifest: vec![],
        };
        // Should be: CheckOwner, JumpIfFalse, CheckUsageLimit, Terminate
        assert_eq!(compiled.opcodes.len(), 4);
        assert!(matches!(compiled.opcodes[0], GuardOp::CheckOwner));
        assert!(matches!(compiled.opcodes[1], GuardOp::JumpIfFalse { .. }));
        assert!(matches!(compiled.opcodes[2], GuardOp::CheckUsageLimit { limit: 10 }));
        assert!(matches!(compiled.opcodes[3], GuardOp::Terminate));
    }
    
    #[test]
    fn test_guard_op_validation() {
        let mut compiled = CompiledGuard {
            opcodes: vec![GuardOp::JumpIfFalse { offset: 10 }],
            cpi_manifest: vec![],
        };
        
        // Should fail - jump out of bounds
        assert!(compiled.validate().is_err());
        
        // Fix it
        compiled.opcodes = vec![
            GuardOp::CheckOwner,
            GuardOp::JumpIfFalse { offset: 1 },
            GuardOp::Terminate,
        ];
        assert!(compiled.validate().is_ok());
    }
    
    // ================================
    // Session Tests
    // ================================
    
    #[test]
    fn test_session_bitmap_borrowing() {
        let clock = Clock {
            slot: 100,
            epoch_start_timestamp: 1000,
            epoch: 1,
            leader_schedule_epoch: 1,
            unix_timestamp: 1234567890,
        };
        
        let mut session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &clock,
        );
        
        // Test borrowing
        let account1 = Pubkey::new_unique();
        let account2 = Pubkey::new_unique();
        
        assert!(session.borrow_account(account1, 1, &clock).is_ok());
        assert_eq!(session.borrowed_bitmap, 0b00000001);
        
        assert!(session.borrow_account(account2, 2, &clock).is_ok());
        assert_eq!(session.borrowed_bitmap, 0b00000011);
        
        // Can't borrow same account twice
        assert!(session.borrow_account(account1, 1, &clock).is_err());
        
        // Test release
        assert!(session.release_account(&account1, &clock).is_ok());
        assert_eq!(session.borrowed_bitmap, 0b00000010);
        
        // Can borrow again after release
        assert!(session.borrow_account(account1, 1, &clock).is_ok());
        assert_eq!(session.borrowed_bitmap, 0b00000011);
    }
    
    #[test]
    fn test_session_bitmap_capacity() {
        let clock = Clock::default();
        let mut session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &clock,
        );
        
        // Fill all 8 slots
        for i in 0..8 {
            let account = Pubkey::new_from_array([i; 32]);
            assert!(session.borrow_account(account, 1, &clock).is_ok());
        }
        assert_eq!(session.borrowed_bitmap, 0b11111111);
        
        // 9th should fail
        let account9 = Pubkey::new_from_array([9; 32]);
        assert!(session.borrow_account(account9, 1, &clock).is_err());
        
        // Release one and try again
        let account0 = Pubkey::new_from_array([0; 32]);
        assert!(session.release_account(&account0, &clock).is_ok());
        assert_eq!(session.borrowed_bitmap, 0b11111110);
        
        // Now can borrow again
        assert!(session.borrow_account(account9, 1, &clock).is_ok());
        assert_eq!(session.borrowed_bitmap, 0b11111111);
    }
    
    // ================================
    // Shared Data Tests
    // ================================
    
    #[test]
    fn test_reentrancy_protection() {
        let mut shared_data = SessionSharedData::default();
        assert!(!shared_data.is_entered());
        
        assert!(shared_data.enter_protected_section().is_ok());
        assert!(shared_data.is_entered());
        
        // Can't enter twice
        assert!(shared_data.enter_protected_section().is_err());
        
        shared_data.exit_protected_section();
        assert!(!shared_data.is_entered());
        
        // Can enter again
        assert!(shared_data.enter_protected_section().is_ok());
    }
    
    #[test]
    fn test_cpi_depth_management() {
        let mut shared_data = SessionSharedData::default();
        assert_eq!(shared_data.current_cpi_depth(), 0);
        
        // Increment up to max
        for i in 1..=SessionSharedData::MAX_CPI_DEPTH {
            assert!(shared_data.check_and_increment_cpi_depth().is_ok());
            assert_eq!(shared_data.current_cpi_depth(), i);
        }
        
        // Next increment should fail
        assert!(shared_data.check_and_increment_cpi_depth().is_err());
        assert!(shared_data.is_at_max_cpi_depth());
        
        // Decrement
        shared_data.decrement_cpi_depth();
        assert_eq!(shared_data.current_cpi_depth(), SessionSharedData::MAX_CPI_DEPTH - 1);
        assert!(!shared_data.is_at_max_cpi_depth());
    }
    
    #[test]
    fn test_feature_flags() {
        let mut shared_data = SessionSharedData::default();
        
        assert!(!shared_data.is_paused());
        shared_data.set_paused(true);
        assert!(shared_data.is_paused());
        
        assert!(!shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
        shared_data.set_flag(SessionSharedData::FLAG_DEBUG);
        assert!(shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
        
        shared_data.toggle_flag(SessionSharedData::FLAG_DEBUG);
        assert!(!shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
        
        shared_data.toggle_flag(SessionSharedData::FLAG_DEBUG);
        assert!(shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
    }
    
    // ================================
    // Guard Evaluation Tests
    // ================================
    
    #[test]
    fn test_guard_op_evaluation() {
        let session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &Clock::default(),
        );
        
        let caller = session.owner;
        let clock = Clock::default();
        let operation = &[0u8];
        let remaining_accounts = &[];
        
        let ctx = EvaluationContext {
            session: &session,
            caller: &caller,
            clock: &clock,
            operation,
            remaining_accounts,
        };
        
        // Test always true (immediate terminate)
        let guard = CompiledGuard {
            opcodes: vec![GuardOp::Terminate],
            cpi_manifest: vec![],
        };
        assert!(evaluate_compiled_guard(&guard, &ctx).unwrap());
        
        // Test always false (immediate abort)
        let guard = CompiledGuard {
            opcodes: vec![GuardOp::Abort],
            cpi_manifest: vec![],
        };
        assert!(!evaluate_compiled_guard(&guard, &ctx).unwrap());
        
        // Test owner check (should pass and fall off end)
        let guard = CompiledGuard {
            opcodes: vec![GuardOp::CheckOwner],
            cpi_manifest: vec![],
        };
        assert!(!evaluate_compiled_guard(&guard, &ctx).unwrap()); // Falls off end = false
        
        // Test owner check with terminate (should pass)
        let guard = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckOwner,
                GuardOp::JumpIfFalse { offset: 1 },
                GuardOp::Terminate,
            ],
            cpi_manifest: vec![],
        };
        assert!(evaluate_compiled_guard(&guard, &ctx).unwrap());
        
        // Test owner check with wrong caller  
        let wrong_caller = Pubkey::new_unique();
        let ctx_wrong = EvaluationContext {
            session: &session,
            caller: &wrong_caller,
            clock: &clock,
            operation,
            remaining_accounts,
        };
        
        // Use a guard that explicitly handles the failure case
        let guard_with_fail = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckOwner,           // 0: Check if caller is owner
                GuardOp::JumpIfFalse { offset: 2 }, // 1: If false, jump to position 3 (Abort)
                GuardOp::Terminate,            // 2: Success case - caller is owner
                GuardOp::Abort,                // 3: Failure case - wrong caller jumps here
            ],
            cpi_manifest: vec![],
        };
        let result = evaluate_compiled_guard(&guard_with_fail, &ctx_wrong);
        assert!(!result.unwrap()); // CheckOwner fails, jumps to Abort
    }
    
    #[test]
    fn test_guard_expiry_checks() {
        let session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &Clock::default(),
        );
        
        let caller = session.owner;
        let operation = &[0u8];
        let remaining_accounts = &[];
        
        // Test CheckExpiry - should pass when not expired
        let future_time = 2000000000i64;
        let current_time = 1000000000i64;
        let clock = Clock {
            unix_timestamp: current_time,
            ..Clock::default()
        };
        
        let ctx = EvaluationContext {
            session: &session,
            caller: &caller,
            clock: &clock,
            operation,
            remaining_accounts,
        };
        
        let guard = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckExpiry { timestamp: future_time },
                GuardOp::JumpIfFalse { offset: 1 },
                GuardOp::Terminate,
            ],
            cpi_manifest: vec![],
        };
        assert!(evaluate_compiled_guard(&guard, &ctx).unwrap());
        
        // Test when expired
        let past_time = 500000000i64;
        let guard_expired = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckExpiry { timestamp: past_time },
                GuardOp::JumpIfFalse { offset: 2 },
                GuardOp::Terminate,
                GuardOp::Abort,
            ],
            cpi_manifest: vec![],
        };
        assert!(!evaluate_compiled_guard(&guard_expired, &ctx).unwrap());
    }
    
    #[test]
    fn test_guard_not_before() {
        let session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &Clock::default(),
        );
        
        let current_time = 1000000000i64;
        let clock = Clock {
            unix_timestamp: current_time,
            ..Clock::default()
        };
        
        let ctx = EvaluationContext {
            session: &session,
            caller: &session.owner,
            clock: &clock,
            operation: &[0u8],
            remaining_accounts: &[],
        };
        
        // Test CheckNotBefore - should pass when time has passed
        let past_time = 500000000i64;
        let guard = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckNotBefore { timestamp: past_time },
                GuardOp::JumpIfFalse { offset: 1 },
                GuardOp::Terminate,
            ],
            cpi_manifest: vec![],
        };
        assert!(evaluate_compiled_guard(&guard, &ctx).unwrap());
        
        // Test when time hasn't passed yet
        let future_time = 2000000000i64;
        let guard_too_early = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckNotBefore { timestamp: future_time },
                GuardOp::JumpIfFalse { offset: 2 },
                GuardOp::Terminate,
                GuardOp::Abort,
            ],
            cpi_manifest: vec![],
        };
        assert!(!evaluate_compiled_guard(&guard_too_early, &ctx).unwrap());
    }
    
    #[test]
    fn test_guard_usage_limit() {
        let mut session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &Clock::default(),
        );
        
        // Set usage count
        session.usage_count = 5;
        
        let ctx = EvaluationContext {
            session: &session,
            caller: &session.owner,
            clock: &Clock::default(),
            operation: &[0u8],
            remaining_accounts: &[],
        };
        
        // Test under limit
        let guard = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckUsageLimit { limit: 10 },
                GuardOp::JumpIfFalse { offset: 1 },
                GuardOp::Terminate,
            ],
            cpi_manifest: vec![],
        };
        assert!(evaluate_compiled_guard(&guard, &ctx).unwrap());
        
        // Test at limit
        let guard_at_limit = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckUsageLimit { limit: 5 },
                GuardOp::JumpIfFalse { offset: 2 },
                GuardOp::Terminate,
                GuardOp::Abort,
            ],
            cpi_manifest: vec![],
        };
        assert!(!evaluate_compiled_guard(&guard_at_limit, &ctx).unwrap());
    }
    
    #[test]
    fn test_guard_complex_logic() {
        let session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &Clock::default(),
        );
        
        let ctx = EvaluationContext {
            session: &session,
            caller: &session.owner,
            clock: &Clock::default(),
            operation: &[0u8],
            remaining_accounts: &[],
        };
        
        // Test AND logic: Owner AND NotExpired
        let guard_and = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckOwner,                      // 0
                GuardOp::JumpIfFalse { offset: 4 },       // 1: jump to 5 (Abort)
                GuardOp::CheckExpiry { timestamp: 2000000000 }, // 2
                GuardOp::JumpIfFalse { offset: 2 },       // 3: jump to 5 (Abort)
                GuardOp::Terminate,                       // 4: both passed
                GuardOp::Abort,                           // 5: either failed
            ],
            cpi_manifest: vec![],
        };
        assert!(evaluate_compiled_guard(&guard_and, &ctx).unwrap());
        
        // Test OR logic: Owner OR UsageLimit
        let mut session_high_usage = session.clone();
        session_high_usage.usage_count = 100;
        
        let ctx_high_usage = EvaluationContext {
            session: &session_high_usage,
            caller: &session_high_usage.owner,
            clock: &Clock::default(),
            operation: &[0u8],
            remaining_accounts: &[],
        };
        
        let guard_or = CompiledGuard {
            opcodes: vec![
                GuardOp::CheckOwner,                      // 0
                GuardOp::JumpIfFalse { offset: 2 },       // 1: if false, check next condition
                GuardOp::Terminate,                       // 2: owner check passed
                GuardOp::CheckUsageLimit { limit: 200 },  // 3: check usage
                GuardOp::JumpIfFalse { offset: 1 },       // 4: if false, abort
                GuardOp::Terminate,                       // 5: usage check passed
                GuardOp::Abort,                           // 6: both failed
            ],
            cpi_manifest: vec![],
        };
        assert!(evaluate_compiled_guard(&guard_or, &ctx_high_usage).unwrap());
    }
    
    // ================================
    // CPI Allowlist Tests
    // ================================
    
    #[test]
    fn test_cpi_allowlist() {
        use crate::state::CPIAllowlist;
        
        let mut allowlist = CPIAllowlist {
            authority: Pubkey::new_unique(),
            allowed_programs: vec![],
            version: 1,
        };
        
        let program1 = Pubkey::new_unique();
        let program2 = Pubkey::new_unique();
        let system_program = anchor_lang::system_program::ID;
        
        // Test system programs are always allowed
        assert!(allowlist.is_allowed(&system_program));
        assert!(allowlist.is_allowed(&anchor_lang::solana_program::sysvar::rent::ID));
        
        // Test adding programs
        assert!(allowlist.add_program(program1).is_ok());
        assert!(allowlist.is_allowed(&program1));
        assert!(!allowlist.is_allowed(&program2));
        
        // Test duplicate add
        assert!(allowlist.add_program(program1).is_err());
        
        // Test removing programs
        assert!(allowlist.remove_program(&program1).is_ok());
        assert!(!allowlist.is_allowed(&program1));
        
        // Test removing non-existent
        assert!(allowlist.remove_program(&program2).is_err());
        
        // Test capacity
        for i in 0..CPIAllowlist::MAX_PROGRAMS {
            let program = Pubkey::new_from_array([i as u8; 32]);
            assert!(allowlist.add_program(program).is_ok());
        }
        
        // Should fail when full
        let overflow_program = Pubkey::new_unique();
        assert!(allowlist.add_program(overflow_program).is_err());
    }
    
    // ================================
    // Session Optimized Methods Tests
    // ================================
    
    #[test]
    fn test_session_bitmap_edge_cases() {
        let clock = Clock::default();
        let mut session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &clock,
        );
        
        // Test borrowing with different modes
        let account1 = Pubkey::new_unique();
        assert!(session.borrow_account(account1, MODE_READ, &clock).is_ok());
        assert_eq!(session.borrowed_accounts[0].mode, MODE_READ);
        assert!(session.borrowed_accounts[0].can_read());
        assert!(!session.borrowed_accounts[0].can_write());
        
        // Release and reborrow with write
        assert!(session.release_account(&account1, &clock).is_ok());
        assert!(session.borrow_account(account1, MODE_WRITE, &clock).is_ok());
        assert!(session.borrowed_accounts[0].can_write());
        assert!(!session.borrowed_accounts[0].can_read());
        
        // Test read-write mode
        let account2 = Pubkey::new_unique();
        assert!(session.borrow_account(account2, MODE_READ_WRITE, &clock).is_ok());
        assert!(session.borrowed_accounts[1].can_read());
        assert!(session.borrowed_accounts[1].can_write());
        
        // Test release_all
        session.release_all(&clock);
        assert_eq!(session.borrowed_bitmap, 0);
        for i in 0..8 {
            assert!(session.borrowed_accounts[i].is_empty());
        }
    }
    
    #[test]
    fn test_session_has_borrowed_performance() {
        let clock = Clock::default();
        let mut session = Session::new(
            CreateSessionParams {
                scope: SessionScope::User,
                guard_data: Pubkey::new_unique(),
                bound_to: None,
                shared_data: SessionSharedData::default(),
                metadata: [0u8; 64],
            },
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            &clock,
        );
        
        // Test fast path with empty bitmap
        let account = Pubkey::new_unique();
        assert!(session.has_borrowed(&account).is_none());
        
        // Borrow multiple accounts
        let accounts: Vec<Pubkey> = (0..8).map(|i| Pubkey::new_from_array([i; 32])).collect();
        
        // Borrow accounts[0], accounts[2], accounts[4], accounts[6]
        // They will be placed in slots 0, 1, 2, 3 respectively
        for i in (0..8).step_by(2) {
            assert!(session.borrow_account(accounts[i], MODE_READ, &clock).is_ok());
        }
        
        // Verify bitmap is correct (0b00001111 = 15 in decimal)
        assert_eq!(session.borrowed_bitmap, 15);
        
        // Test has_borrowed finds correct accounts
        // accounts[0], accounts[2], accounts[4], accounts[6] were borrowed
        for i in (0..8).step_by(2) {
            assert!(session.has_borrowed(&accounts[i]).is_some());
        }
        
        // Test has_borrowed returns None for non-borrowed
        // accounts[1], accounts[3], accounts[5], accounts[7] were not borrowed
        for i in (1..8).step_by(2) {
            assert!(session.has_borrowed(&accounts[i]).is_none());
        }
    }
    
    // ================================
    // SessionSharedData Tests
    // ================================
    
    #[test]
    fn test_session_shared_data_methods() {
        let mut shared_data = SessionSharedData::default();
        
        // Test reentrancy protection
        assert!(!shared_data.is_entered());
        assert!(shared_data.enter_protected_section().is_ok());
        assert!(shared_data.is_entered());
        assert!(shared_data.enter_protected_section().is_err()); // Can't reenter
        shared_data.exit_protected_section();
        assert!(!shared_data.is_entered());
        
        // Test CPI depth management
        assert_eq!(shared_data.current_cpi_depth(), 0);
        for i in 1..=SessionSharedData::MAX_CPI_DEPTH {
            assert!(shared_data.check_and_increment_cpi_depth().is_ok());
            assert_eq!(shared_data.current_cpi_depth(), i);
        }
        assert!(shared_data.is_at_max_cpi_depth());
        assert!(shared_data.check_and_increment_cpi_depth().is_err());
        
        // Test decrement
        shared_data.decrement_cpi_depth();
        assert_eq!(shared_data.current_cpi_depth(), SessionSharedData::MAX_CPI_DEPTH - 1);
        
        // Test underflow protection
        for _ in 0..10 {
            shared_data.decrement_cpi_depth();
        }
        assert_eq!(shared_data.current_cpi_depth(), 0); // Should stop at 0
        
        // Test feature flags
        assert!(!shared_data.is_paused());
        shared_data.set_paused(true);
        assert!(shared_data.is_paused());
        assert!(shared_data.has_flag(SessionSharedData::FLAG_PAUSED));
        
        // Test other flags
        shared_data.set_flag(SessionSharedData::FLAG_DEBUG | SessionSharedData::FLAG_ATOMIC);
        assert!(shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
        assert!(shared_data.has_flag(SessionSharedData::FLAG_ATOMIC));
        assert!(!shared_data.has_flag(SessionSharedData::FLAG_CROSS_PROTOCOL));
        
        // Test toggle
        shared_data.toggle_flag(SessionSharedData::FLAG_DEBUG);
        assert!(!shared_data.has_flag(SessionSharedData::FLAG_DEBUG));
        assert!(shared_data.has_flag(SessionSharedData::FLAG_ATOMIC)); // Others unchanged
        
        // Test custom data access
        let custom_data = shared_data.custom_data_mut();
        custom_data[0] = 42;
        custom_data[31] = 255;
        
        assert_eq!(shared_data.custom_data()[0], 42);
        assert_eq!(shared_data.custom_data()[31], 255);
    }
}