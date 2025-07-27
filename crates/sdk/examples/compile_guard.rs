// Example showing how to compile guards client-side
use valence_sdk::{compile_guard, Guard, Result as SdkResult};
use valence_core::guards::{GuardOp, CompiledGuard};

fn main() -> SdkResult<()> {
    // Create a complex guard using the builder pattern
    let guard = Guard::And(
        Box::new(Guard::OwnerOnly),
        Box::new(Guard::UsageLimit { max: 10 }),
    );
    
    // Compile the guard client-side
    let compiled = compile_guard(&guard)?;
    
    println!("Compiled guard has {} opcodes", compiled.opcodes.len());
    for (i, op) in compiled.opcodes.iter().enumerate() {
        println!("  {}: {:?}", i, op);
    }
    
    // The compiled guard can now be sent to the chain
    // via the create_guard_data instruction
    
    Ok(())
}

#[test]
fn test_guard_compilation() {
    // Test simple guard
    let guard = Guard::OwnerOnly;
    let compiled = compile_guard(&guard).unwrap();
    assert!(compiled.opcodes.contains(&GuardOp::CheckOwner));
    
    // Test AND composition
    let and_guard = Guard::And(
        Box::new(Guard::OwnerOnly),
        Box::new(Guard::Expiration { expires_at: 1234567890 }),
    );
    let compiled = compile_guard(&and_guard).unwrap();
    
    // Should contain both checks
    assert!(compiled.opcodes.iter().any(|op| matches!(op, GuardOp::CheckOwner)));
    assert!(compiled.opcodes.iter().any(|op| matches!(op, GuardOp::CheckExpiry { .. })));
    
    // Test OR composition
    let or_guard = Guard::Or(
        Box::new(Guard::AlwaysTrue),
        Box::new(Guard::AlwaysFalse),
    );
    let compiled = compile_guard(&or_guard).unwrap();
    
    // Should have control flow opcodes
    assert!(compiled.opcodes.iter().any(|op| matches!(op, GuardOp::JumpIfFalse { .. })));
}