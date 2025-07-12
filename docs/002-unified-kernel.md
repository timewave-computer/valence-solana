# Unified Kernel Architecture

This document details the core kernel components of Valence Solana: the Gateway, Registry, and Verifier programs. These singleton programs provide essential services that user-deployed shards compose to build applications.

## Kernel Design Philosophy

The Valence kernel follows minimalist design principles. Each component has a single, well-defined responsibility and exposes a simple interface. Complex behavior emerges from the composition of these simple primitives rather than from complex individual components.

The kernel enforces no application-level policies. It provides mechanisms for routing, registration, and verification, but leaves policy decisions to user space. This separation enables diverse applications to share common infrastructure while implementing their own specific requirements.

## Gateway Program

The gateway serves as the protocol's entry point, routing all operations to their appropriate destinations. Its design prioritizes simplicity and reliability over feature richness.

### Routing Mechanism

The gateway accepts a single instruction type with a routing target and payload:

```
route(target: RouteTarget, data: Vec<u8>)
```

The RouteTarget enum specifies three possible destinations:
- Registry operations for function management
- Verifier operations for predicate evaluation  
- Shard operations for application logic

The gateway performs no validation beyond basic target verification. It constructs the appropriate Cross-Program Invocation (CPI) and forwards the request with the original signer permissions.

### State Management

Gateway state consists solely of configuration data:
- Authority pubkey for administrative operations
- Pause flag for emergency stops
- Reserved space for future extensions

This minimal state reduces attack surface and simplifies security analysis. The gateway cannot be corrupted by malformed requests since it maintains no request-specific state.

### CPI Pattern

The gateway uses a consistent CPI pattern for all routing:

1. Validate the target program exists
2. Construct instruction data from the payload
3. Build account metas from remaining accounts
4. Invoke the target program
5. Return the result directly to caller

This pattern ensures the gateway adds minimal overhead while preserving security properties of the underlying programs.

## Registry Program

The registry manages a global, content-addressed function store. Its design emphasizes immutability and deterministic resolution.

### Content Addressing

Functions are identified by the SHA256 hash of their program ID. This creates a 32-byte identifier that uniquely represents the function's implementation. The hash serves as both an identifier and an integrity check.

When registering a function, the registry:
1. Accepts the function hash and program ID
2. Derives a Program Derived Address (PDA) from the hash
3. Stores the mapping in the PDA account
4. Records the registering authority

The PDA derivation uses the seeds `[b"function", hash]`, ensuring each function has a unique, deterministic address.

### Registration Policy

Any account can register new functions, but several constraints apply:
- Functions cannot be re-registered (first-come, first-served)
- Only the original registrant can deregister
- Deregistration closes the account and recovers rent

This policy balances openness with accountability. Anyone can contribute functions to the ecosystem, but they remain responsible for their registrations.

### Lookup Mechanism

Function lookup is a simple PDA derivation and account read. Given a function hash, clients can:
1. Derive the PDA address
2. Check if the account exists
3. Read the program ID if present

This mechanism requires no RPC calls to the registry program itself, reducing load and improving performance.

## Verifier Program

The verifier provides a pluggable system for evaluation predicates. It routes verification requests to specialized programs based on semantic labels.

### Label-Based Routing

Verifiers are identified by string labels rather than addresses or hashes. This enables semantic verification patterns:

- "balance_check" routes to a balance verification program
- "signature_verify" routes to a signature verification program  
- "merkle_proof" routes to a Merkle proof verification program

Labels provide meaningful names while maintaining flexibility. New verification strategies can be added by registering new labels.

### Verifier Registration

Verifier registration follows a similar pattern to function registration:
1. Derive PDA from label: `[b"verifier", label.as_bytes()]`
2. Store verifier program ID in PDA account
3. Record registering authority

Unlike functions, verifiers can be updated by their authority. This allows bug fixes and improvements while maintaining stable labels.

### Verification Flow

When verify_predicate is called:
1. Look up verifier program by label
2. Forward predicate data and context via CPI
3. Return success/failure result

The verifier program itself defines the predicate format and evaluation rules. The kernel only provides the routing mechanism.

## Account Structure Design

All kernel programs use Program Derived Addresses (PDAs) extensively. This design choice provides several benefits:

- Deterministic addressing without keypair management
- Natural uniqueness constraints
- Simplified client interactions
- Rent recovery on deregistration

PDA seeds follow consistent patterns:
- Function entries: `[b"function", hash]`
- Verifier entries: `[b"verifier", label.as_bytes()]`
- Config accounts: `[b"config", authority.as_ref()]`

This consistency simplifies client code and reduces cognitive overhead.

## Inter-Component Communication

Kernel components communicate exclusively through CPI. They share no state and make no assumptions about each other's internals. This loose coupling enables independent evolution and reduces system-wide failure modes.

Communication patterns follow these principles:
- Pass through original signer permissions
- Forward all provided accounts
- Return results unmodified
- Propagate errors with context

The gateway exemplifies this pattern - it knows nothing about registry or verifier internals, only their instruction formats.

## Error Handling

Each kernel component defines its own error types:
- Gateway: InvalidTarget, Unauthorized
- Registry: FunctionAlreadyRegistered, FunctionNotFound
- Verifier: VerifierNotFound, VerificationFailed

Errors bubble up through CPI calls, preserving the original error source. This aids debugging while maintaining abstraction boundaries.

## Performance Considerations

The kernel design prioritizes correctness over performance, but several optimizations apply:

- PDA lookups can be cached client-side
- Gateway routing adds minimal overhead (one CPI)
- Registry lookups require no program execution
- Verifier routing is pay-for-what-you-use

Most performance costs come from application logic in shards, not kernel operations.

## Security Properties

The kernel provides several security guarantees:

1. **No Ambient Authority**: All operations require explicit authorization
2. **Immutable Functions**: Registered functions cannot be modified
3. **Isolated Components**: Compromise of one component doesn't affect others
4. **Minimal State**: Reduced attack surface from minimal stored data
5. **Deterministic Addresses**: PDAs prevent address substitution attacks

These properties compose to create a secure foundation for applications.

## Future Evolution

The kernel design accommodates evolution through several mechanisms:

- Gateway can route to new services
- Registry can implement new function types
- Verifier can support new predicate categories
- Reserved space in accounts enables upgrades

However, the core interfaces should remain stable. Breaking changes would require careful migration strategies to preserve ecosystem compatibility.