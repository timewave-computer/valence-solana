# Guard Compilation Guide

## Overview

The valence-kernel guard system uses a flattened instruction set (APU model) for efficient on-chain evaluation. Guards must be compiled from the high-level recursive `Guard` enum to linear `GuardOp` opcodes **CLIENT-SIDE ONLY**.

## Critical Security Note

**The guard compiler MUST NEVER be used on-chain.** Compilation is a complex, recursive process that:
- Consumes unpredictable amounts of gas
- Creates DoS attack vectors
- Exposes unnecessary attack surface

## Correct Usage Pattern

```rust
// CLIENT-SIDE (in SDK or application code)
use valence_sdk::{compile_guard, Guard, SerializedGuard};

// Create high-level guard
let guard = Guard::And(
    Box::new(Guard::OwnerOnly),
    Box::new(Guard::UsageLimit { max: 10 })
);

// Compile to opcodes CLIENT-SIDE
let compiled = compile_guard(&guard)?;

// Send compiled guard to chain
let signature = program
    .request()
    .accounts(accounts)
    .args(instruction::CreateGuardData {
        session,
        serialized_guard: compiled, // Pre-compiled!
    })
    .send()?;
```

## Architecture

1. **Client-Side**: High-level `Guard` enum with recursive structure
2. **Compilation**: Convert to `SerializedGuard` with linear `GuardOp` instructions
3. **On-Chain**: Only store and evaluate pre-compiled guards

This separation ensures:
- Predictable gas costs
- No recursion on-chain
- Minimal attack surface
- Maximum performance

## Guard Types

### Simple Guards
- `AlwaysTrue` → `Terminate`
- `AlwaysFalse` → `Abort`
- `OwnerOnly` → `CheckOwner`
- `Expiration` → `CheckExpiry`
- `UsageLimit` → `CheckUsageLimit`

### Composite Guards
- `And(a, b)` → Check A, jump if false, check B, terminate
- `Or(a, b)` → Check A, jump if true to success, check B
- `Not(a)` → Check A, invert result with control flow

### External Guards
- `External` → `Invoke` with CPI manifest entry

## Examples

See `examples/compile_guard.rs` for complete working examples.