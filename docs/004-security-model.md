# Security Model and Capability System

## Overview

Valence implements a comprehensive security model built on three foundational concepts: **capability-based security**, **shard encapsulation**, and **linear sessions**. This model ensures that shards operate within strict permission boundaries while maintaining composability, auditability, and UTXO-like execution semantics.

The security model enforces that:
- Shards cannot directly access external Solana programs
- All operations require explicit capabilities granted at account creation
- Functions serve as controlled interfaces with declared capability requirements
- Sessions are linear types that can only be consumed once (UTXO-like semantics)
- Runtime enforcement prevents unauthorized operations

## Core Principles

### 1. Capability-Based Security

Every operation in Valence requires explicit capabilities. Accounts are created with specific capabilities that define what operations can be performed. This approach moves away from ambient authority patterns common in traditional smart contracts.

Capabilities are represented as strings (e.g., "transfer", "mint", "admin") and are checked during bundle execution. The capability model enables fine-grained permission control and makes security policies explicit and auditable.

### 2. Shard Encapsulation

Shards are isolated execution environments that:
- Cannot make direct Cross-Program Invocations (CPIs) to arbitrary programs
- Must use registered functions from the registry for all external operations
- Have their execution confined to the functions they've explicitly imported

### 3. Linear Sessions (UTXO-like Semantics)

Sessions are linear types that simulate UTXO semantics on top of Solana's account model:
- Sessions contain multiple accounts in a shared namespace
- Sessions can only be consumed once (linear consumption)
- Consuming a session can create new sessions (transaction outputs)
- This provides atomicity and prevents double-spending at the session level

### 4. Explicit Permission Model

All permissions are explicit and auditable:
- Accounts request specific capabilities at creation time
- Capabilities cannot be elevated during account lifetime
- Every external operation requires a matching capability from session accounts

## Architecture

### Two-Level Security Model

```
┌─────────────────────────────────────────────────────────────────┐
│                        Shard Boundary                           │
│                                                                 │
│  ┌─────────────────────┐     ┌──────────────┐  ┌────────────┐  │
│  │      Session        │────▶│Bundle Executor│─▶│  Function  │  │
│  │  (Linear Type)      │     │              │  │   Import   │  │
│  │                     │     │ Aggregates   │  │            │  │
│  │ ┌─────────────────┐ │     │ capabilities │  └──────┬─────┘  │
│  │ │ Account A       │ │     │ from accounts│         │        │
│  │ │ caps: [transfer]│ │     │ before CPI   │         │        │
│  │ └─────────────────┘ │     └──────────────┘         │        │
│  │ ┌─────────────────┐ │                               │        │
│  │ │ Account B       │ │                               │        │
│  │ │ caps: [mint]    │ │                               │        │
│  │ └─────────────────┘ │                               │        │
│  │ namespace: "my-app" │                               │        │
│  │ is_consumed: false  │                               │        │
│  └─────────────────────┘                               │        │
└─────────────────────────────────────────────────────────┼────────┘
                                                          │
                                                          ▼
                                                  ┌──────────────┐
                                                  │   Registry   │
                                                  │              │
                                                  │ Function:    │
                                                  │ - hash       │
                                                  │ - program    │
                                                  │ - caps req'd │
                                                  └──────┬───────┘
                                                         │
                                                         ▼
                                                  ┌──────────────┐
                                                  │  External    │
                                                  │   Program    │
                                                  └──────────────┘
```

### Linear Session Lifecycle

```
Session A ──consume──> [Bundle Execution] ──create──> Session B + Session C
(accounts:           (aggregates caps      (new account    (new account
 [acc1, acc2])        from acc1, acc2)      groupings)      groupings)
is_consumed: false                        is_consumed: false is_consumed: false
                   is_consumed: true
```

## Standard Capabilities

The following standard capabilities are defined:

- **`transfer`** - Transfer tokens or SOL
- **`mint`** - Mint new tokens  
- **`burn`** - Burn tokens
- **`admin`** - Administrative operations
- **`read`** - Read-only data access
- **`write`** - Write data access
- **`create_account`** - Create new accounts
- **`close_account`** - Close accounts
- **`cpi`** - Execute cross-program invocations
- **`upgrade`** - Manage program upgrades

## How the Security Model Works

### 1. Function Registration

When registering a function, specify its required capabilities:

```rust
// Register a transfer function that requires TRANSFER capability
let register_ix = valence_registry::instruction::Register {
    hash: function_hash,
    program: function_program_id,
    required_capabilities: vec!["transfer".to_string()],
};
```

Functions serve as the controlled interface to external resources:
- Each function declares its required capabilities
- Functions can only be executed if session accounts have all required capabilities
- The runtime enforces capability checks before function execution

### 2. Account Creation

First, create accounts with specific capabilities:

```rust
// Request an account with transfer capabilities
let request_ix = valence_shard::instruction::RequestAccount {
    capabilities: vec!["transfer".to_string()],
    init_state_hash: [0u8; 32],
};

// Initialize the account (called by off-chain service)
let initialize_ix = valence_shard::instruction::InitializeAccount {
    request_id: account_request_id,
    init_state_data: state_data,
};
```

### 3. Session Creation

Create sessions by grouping multiple accounts:

```rust
// Create a session containing multiple accounts
let create_session_ix = valence_shard::instruction::CreateSession {
    accounts: vec![account_a_id, account_b_id], // Accounts with different capabilities
    namespace: "my-trading-session".to_string(),
    nonce: 1,
    metadata: vec![],
};
```

### 4. Runtime Enforcement

During bundle execution, the runtime aggregates capabilities from all accounts in the session:

```rust
// This happens automatically during bundle execution
let mut available_capabilities = HashSet::new();
for account_id in session.accounts {
    let account = load_account(account_id)?;
    available_capabilities.extend(account.capabilities);
}

for required_cap in function.required_capabilities {
    if !available_capabilities.contains(&required_cap) {
        return Err(ShardError::InsufficientCapabilities);
    }
}
```

### 5. Linear Consumption

Sessions follow UTXO-like semantics:

```rust
// Consume session (marks it as consumed)
let consume_ix = valence_shard::instruction::ConsumeSession {
    new_sessions_data: vec![
        (vec![new_account_1], "output-session-1".to_string(), 2, vec![]),
        (vec![new_account_2], "output-session-2".to_string(), 3, vec![]),
    ],
};
```

## Examples

### Example 1: Multi-Account Session

```rust
// Create accounts with different capabilities
let account_a = create_account(vec!["read".to_string()]);
let account_b = create_account(vec!["transfer".to_string()]);

// Group accounts into a session
let session = create_session(
    accounts: vec![account_a.id, account_b.id],
    namespace: "trading-session",
    nonce: 1,
);

// Execute bundle (has aggregated capabilities: read + transfer)
execute_bundle(session, [
    Operation {
        function_hash: "read_token_balance", // requires "read"
        target_account: Some(account_a.id),
        ...
    },
    Operation {
        function_hash: "transfer_tokens", // requires "transfer"  
        target_account: Some(account_b.id),
        ...
    }
])
// ✅ Succeeds because session has both capabilities
```

### Example 2: Linear Session Consumption

```rust
// Start with session containing accounts
let session_1 = Session {
    accounts: vec![account_a, account_b],
    namespace: "input-session",
    is_consumed: false,
    ...
};

// Execute bundle that consumes session
execute_bundle(session_1, operations); // Session marked as consumed

// Create new sessions from the consumption
let session_2 = create_session(
    accounts: vec![new_account_c],
    namespace: "output-session-1",
    ...
);

let session_3 = create_session(
    accounts: vec![new_account_d],
    namespace: "output-session-2", 
    ...
);

// Original session cannot be used again
execute_bundle(session_1, operations); // ❌ Fails: SessionAlreadyConsumed
```

### Example 3: Insufficient Capabilities

```rust
// Account with only READ capability
let account = create_account(vec!["read".to_string()]);
let session = create_session(accounts: vec![account.id], ...);

// Function requires WRITE capability
register_function(
    hash: "update_oracle_price",
    required_capabilities: ["write"]
);

// Bundle execution
execute_bundle(session, [
    Operation {
        function_hash: "update_oracle_price",
        ...
    }
])
// ❌ Fails with InsufficientCapabilities error
```

## Security Benefits

1. **Principle of Least Privilege**: Accounts only get the specific capabilities they need
2. **Audit Trail**: All capability grants and session lineage are on-chain and auditable
3. **No Ambient Authority**: No implicit permissions based on signer or account ownership
4. **Composable Security**: Accounts can be composed into sessions while maintaining security
5. **Defense in Depth**: Capability checks add an additional layer beyond Solana's account permissions
6. **Controlled External Access**: All external operations must go through registered functions
7. **UTXO-like Atomicity**: Linear sessions prevent double-spending and provide transaction atomicity
8. **Namespace Isolation**: Sessions provide isolated execution contexts with shared namespaces

## Best Practices

1. **Minimal Capabilities**: Only grant capabilities that are actually needed for each account
2. **Capability Aggregation**: Design sessions to aggregate minimal sets of capabilities
3. **Explicit Requirements**: Functions should explicitly declare all required capabilities
4. **Linear Consumption**: Plan session lifecycles to take advantage of UTXO-like semantics
5. **Namespace Design**: Use meaningful namespaces to organize related accounts
6. **Session Sizing**: Keep sessions small and focused for better composability

## Security Considerations

- Capabilities are granted at account creation and cannot be changed during account lifetime
- Sessions are linear types and can only be consumed once
- Functions without declared capabilities are treated as requiring no special permissions
- The capability model provides defense-in-depth alongside Solana's native account permissions
- Capability checks happen before CPI execution, preventing unauthorized operations
- Shard encapsulation prevents direct CPI calls, enforcing capability boundaries
- Session consumption creates an immutable audit trail of state transitions

## Testing

The security model is validated through comprehensive tests that verify:
- Shards cannot make direct CPIs to external programs
- Functions without proper capabilities are rejected
- Capability requirements are enforced at runtime across session accounts
- Multiple capabilities are properly aggregated from session accounts
- Linear session consumption prevents double-spending
- Malicious functions cannot bypass the capability system

See `/tests/encapsulation/` for the complete test suite.

## Future Enhancements

While not yet implemented, future versions may support:
- Hierarchical capabilities (e.g., `admin` implies `read` and `write`)
- Time-bound capabilities that expire
- Capability delegation between accounts
- Dynamic capability policies based on runtime conditions
- Cross-session capability sharing within namespaces
- Automated session splitting and merging based on capability requirements 