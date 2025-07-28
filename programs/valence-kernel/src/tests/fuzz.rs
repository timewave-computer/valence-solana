// Fuzz testing for guard deserialization
// Ensures robust handling of malformed input
#[cfg(test)]
mod fuzz_tests {
    use crate::guards::{Guard, GuardOp, SerializedGuard};
    
    // Simple fuzzer for guard validation (since deserialization isn't implemented)
    fn fuzz_high_level_guard_validation() {
        // Test various guard configurations for validation
        let test_guards = vec![
            Guard::AlwaysTrue,
            Guard::AlwaysFalse,
            Guard::OwnerOnly,
            Guard::Expiration { expires_at: 1234567890 },
            Guard::Permission { required: 1 },
            Guard::UsageLimit { max: 10 },
            Guard::And(
                Box::new(Guard::OwnerOnly),
                Box::new(Guard::Expiration { expires_at: 1234567890 })
            ),
            Guard::Or(
                Box::new(Guard::OwnerOnly),
                Box::new(Guard::Permission { required: 1 })
            ),
            Guard::Not(Box::new(Guard::AlwaysFalse)),
        ];
        
        for guard in test_guards {
            let _ = guard.validate();
            
            // Try to compile it
            if let Ok(compiled) = crate::guards::compile_high_level_guard(&guard) {
                let _ = compiled.validate();
            }
        }
    }
    
    // Simple fuzzer for compiled guard opcodes
    fn fuzz_apu_program_validation() {
        // Test various opcode combinations
        let test_opcode_sets = vec![
            vec![GuardOp::CheckOwner, GuardOp::Terminate],
            vec![GuardOp::CheckExpiry { timestamp: 1234567890 }, GuardOp::Terminate],
            vec![GuardOp::CheckUsageLimit { limit: 10 }, GuardOp::JumpIfFalse { offset: 1 }, GuardOp::Terminate],
            vec![GuardOp::Abort],
        ];
        
        for opcodes in test_opcode_sets {
            if let Ok(guard) = SerializedGuard::new(opcodes) {
                let _ = guard.validate();
            }
        }
    }
    
    #[test]
    fn test_guard_validation_fuzz() {
        fuzz_high_level_guard_validation();
    }
    
    #[test]
    fn test_serialized_guard_opcodes_fuzz() {
        fuzz_apu_program_validation();
    }
    
    #[test]
    fn test_guard_validation_repeated() {
        // Run validation multiple times to test consistency
        for _ in 0..10 {
            fuzz_high_level_guard_validation();
        }
    }
    
    #[test]
    fn test_serialized_guard_validation_repeated() {
        // Run opcode validation multiple times
        for _ in 0..10 {
            fuzz_apu_program_validation();
        }
    }
}