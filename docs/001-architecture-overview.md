# Architecture Overview

Valence Solana implements a microkernel architecture for cross-chain applications, emphasizing security through capability-based access control and verification through content-addressed functions. This document provides a high-level overview of the system architecture and its design principles.

## System Architecture

The Valence architecture consists of four on-chain programs and supporting off-chain infrastructure:

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Gateway   │────▶│   Registry   │     │  Verifier   │
│  (Router)   │     │  (Functions) │     │ (Predicates)│
└──────┬──────┘     └──────────────┘     └─────────────┘
       │                    ▲                     ▲
       │                    │                     │
       ▼                    └─────────────────────┘
┌─────────────┐                    │
│    Shard    │────────────────────┘
│ (App Logic) │
└─────────────┘
```
*Figure 1: Core component relationships showing the gateway as entry point, with shards accessing registry and verifier services*

The gateway serves as the single entry point for all protocol operations, routing requests to the appropriate singleton services or user-deployed shards. This design ensures consistent access patterns and enables future protocol upgrades without breaking existing integrations.

## Design Principles

### Capability-Based Security

Every operation in Valence requires explicit capabilities. Sessions are created with specific capabilities that define what operations they can perform. Capabilities are pre-aggregated at session creation time, enabling O(1) permission checks during execution.

Capabilities are represented as efficient bitmaps (e.g., `Capability::Transfer`, `Capability::Mint`, `Capability::Admin`) and are checked automatically during operation execution. The capability model enables fine-grained permission control and makes security policies explicit and auditable.

### Content-Addressed Functions

Functions in Valence are identified by their content hash. When a function is registered, its bytecode is hashed to produce a unique 32-byte identifier. This approach provides several benefits:

- Deterministic function resolution
- Immutable function references
- Simplified dependency management
- Natural deduplication

The registry maintains a mapping from function hashes to their implementing programs, but functions themselves remain immutable once registered.

### Diff-Based State Transitions

State changes in Valence are tracked through a hash chain, where each operation produces a new state hash based on the previous hash and the operation result. This creates an auditable history of state transitions and enables verification of execution paths.

The diff model supports both synchronous execution (all operations in one transaction) and asynchronous execution (operations spread across multiple transactions with checkpoints). This flexibility allows complex operations to span multiple blocks while maintaining consistency.

### Session-Based Operations

Valence provides a clean Session API that abstracts away infrastructure complexity:

**Sessions** are the primary abstraction developers work with. Each session contains:
- Pre-aggregated capabilities (what operations can be performed)
- Application state (managed automatically)
- Linear consumption semantics (UTXO-like guarantees)

Sessions implement linear type semantics - they can be consumed exactly once to create new sessions, ensuring atomic state transitions. All account management and capability aggregation happens automatically behind the scenes, so developers only need to understand Sessions.

This model enables complex operations while maintaining clear ownership and preventing double-spending of state. Session consumption creates an audit trail with transaction signatures, providing complete visibility into state evolution.

### Microkernel Architecture

The system follows microkernel design principles by keeping core functionality minimal and pushing complexity to user space. The kernel components (gateway, registry, verifier) provide only essential services:

- Routing and dispatch
- Function registration and lookup
- Verification predicate routing

All application logic resides in shards, which are user-deployed programs that compose kernel services. This separation enables independent evolution of components and reduces the attack surface of core infrastructure.

## Component Overview

### Gateway

The gateway is a thin routing layer with no business logic. It examines incoming requests and forwards them to the appropriate destination using Cross-Program Invocation (CPI). The gateway maintains minimal state - only configuration needed for routing decisions.

### Registry

The registry is a singleton program managing the global function registry. It stores mappings from function hashes to program addresses and enforces registration policies. Functions can be registered by anyone but can only be deregistered by their original registrant.

### Verifier

The verifier routes verification predicates to specialized verifier programs. It maintains a registry of verifier labels (e.g., "balance_check") mapped to programs that implement the verification logic. This design allows pluggable verification strategies without modifying core infrastructure.

### Shards

Shards are user-deployed programs that implement application logic. They import functions from the registry and execute operations within sessions. Developers work with a simple Session API:

- Create sessions with capabilities (`create_session_v2`)
- Execute single operations (`execute_on_session`) 
- Execute atomic bundles (`execute_bundle_v2`)

Shards automatically handle all account management, capability aggregation, and state synchronization. Sessions support linear consumption semantics - they can be consumed to create new sessions, providing UTXO-like guarantees. Shards can choose whether to respect function deregistration, providing flexibility in handling dependency updates.

## Security Model

Security in Valence builds on several layers:

1. **Entry Point Control**: All operations flow through the gateway, enabling consistent security policies

2. **Capability Isolation**: Sessions contain pre-aggregated capabilities that limit operations. O(1) bitmap checks enforce permissions automatically

3. **Content Verification**: Function hashes ensure code integrity and prevent substitution attacks

4. **Authority Checks**: Each component validates that callers have appropriate permissions

5. **State Verification**: Hash chains enable detection of unauthorized state modifications

The combination of these mechanisms creates defense in depth, where compromise of one layer doesn't immediately lead to system compromise.

## Comparison with Traditional Architectures

Traditional smart contract platforms typically feature monolithic programs with ambient authority. Any code can attempt any operation, with security enforced through runtime checks. This model leads to complex security analysis and frequent vulnerabilities.

Valence's architecture inverts this model. Capabilities must be explicitly granted, functions are immutable and content-addressed, and core infrastructure is minimal and auditable. This design trades some convenience for significantly improved security properties and clearer reasoning about system behavior.

The microkernel approach also enables better modularity. Components can be upgraded independently, new verification strategies can be added without modifying existing code, and applications can compose functionality without tight coupling.

## Future Extensibility

The architecture supports several extension points:

- New verification predicates can be added by registering additional verifiers
- Alternative function registries could implement different policies
- The gateway could route to additional services as the protocol evolves
- Shards can implement arbitrary application logic while leveraging core services

This extensibility ensures the protocol can adapt to new requirements without requiring wholesale rewrites or breaking changes.