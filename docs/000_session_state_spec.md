# Session and State Management Specification

## Overview

The Valence Kernel implements session-based execution contexts that provide isolated, secure environments for program operations on Solana. Each session maintains its own account registry, security configuration, and hierarchical namespace position while enforcing strict borrowing semantics and ownership transfer protocols.

## Session Account Structure

The Session Account serves as the primary execution context within the kernel. Each session maintains a position within the namespace hierarchy through a NamespacePath structure that stores up to 256 bytes of hierarchical path information with an accompanying length field for efficient operations.

The session tracks its owner through a standard `Pubkey` field and references both its `GuardAccount` for security configuration and its Account Lookup Table for managing accessible accounts. Sessions support hierarchical organization through arrays that can store up to 8 child accounts and 8 child sessions, respecting Solana's stack constraints.

Account borrowing is managed through an array of 4 `SessionBorrowedAccount` slots paired with an efficient bitmap for O(1) slot tracking operations. The `borrowed_bitmap` field uses bit manipulation to quickly determine which slots are occupied, while the individual slot entries track the specific account, access permissions, and borrowing context.

```rust
pub struct SessionBorrowedAccount {
    pub address: Pubkey,
    pub borrowed_at: i64,
    pub mode: u8,
}

pub struct CreateSessionParams {
    pub namespace_path: [u8; 128],
    pub namespace_path_len: u16,
    pub metadata: [u8; 32],
    pub parent_session: Option<Pubkey>,
}

pub struct Session {
    pub namespace: NamespacePath,
    pub guard_account: Pubkey,
    pub account_lookup: Pubkey,
    pub owner: Pubkey,
    pub shard: Pubkey,
    pub parent_session: Option<Pubkey>,
    pub usage_count: u64,
    pub metadata: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
    pub borrowed_accounts: [SessionBorrowedAccount; 4],
    pub borrowed_bitmap: u8,
    pub cpi_depth: u8,
    pub active: bool,
    pub nonce: u64,
    pub child_accounts: [Pubkey; 8],
    pub child_count: u8,
    pub child_sessions: [Pubkey; 8],
    pub child_session_count: u8,
}
```

The `active` flag enables clean session invalidation while the `nonce` field increments on ownership changes to support versioned ownership tracking. Usage tracking through `usage_count`, `created_at`, and `updated_at` fields provides operational metrics and lifecycle management capabilities.

## Borrowing Semantics Implementation

Sessions implement a two-phase borrowing system that provides both security and efficiency. The first phase involves pre-registration through the Account Lookup Table, which explicitly declares all accounts the session may access along with their required permissions. This pre-registration requirement ensures sessions cannot access arbitrary accounts, establishing a strong security boundary.

The second phase involves explicit borrowing during operation execution. When a session needs to operate on an account, it must first issue a `BorrowAccount` operation that specifies the account index (referencing the pre-registered ALT entry) and the desired access mode. The system validates that the requested access mode matches or is more restrictive than the pre-registered permissions.

Borrowed accounts are tracked in the session's `borrowed_accounts` array using one of four available slots. The `borrowed_bitmap` field uses individual bits to track which slots are occupied, enabling O(1) slot availability checks. Each borrowed account entry stores the account's public key, the granted access mode, the ALT index for validation, and additional metadata for operation tracking.

Account release is managed through explicit `ReleaseAccount` operations that specify the account index to be released. The system validates that the account is currently borrowed by the session and updates both the `borrowed_accounts` array and the `borrowed_bitmap` to reflect the release. This explicit release requirement prevents resource leaks and ensures predictable account lifecycle management.

The borrowing system enforces access mode validation at both borrow time and during actual account usage. Read-only borrows prevent any mutations to the account data, while read-write borrows allow full access subject to Solana's account ownership rules. The system tracks the cumulative access patterns to detect potential conflicts and enforce proper sequencing.

## Namespace System Architecture

The namespace system provides hierarchical organization through NamespacePath structures that can represent arbitrary depth hierarchies within a 256-byte storage limit. Each namespace path consists of a fixed-size byte array and a length field that indicates the actual path length, enabling efficient prefix operations and parent-child relationship determination.

Namespace paths use forward slash delimiters to create hierarchical structures like `protocol/subprotocol/user` or `defi/lending/alice/temp1`. The system implements one-way trust semantics where parent namespaces can access their children's state but children cannot access parent state, creating natural administrative boundaries and privilege escalation prevention.

Path validation ensures that namespace strings contain only valid characters and follow proper hierarchical formatting. The system prevents empty path components, double slashes, and paths that exceed the storage limit. Special characters are restricted to alphanumeric characters, hyphens, underscores, and forward slashes for delimiter purposes.

Parent-child relationships are determined through prefix matching operations on the namespace paths. A session with namespace `defi/lending` is considered a parent of `defi/lending/alice` through direct prefix comparison. This relationship affects access permissions, invalidation cascading, and child account creation capabilities.

The namespace system integrates with Program Derived Address (PDA) generation by using the namespace path as a seed component. This provides deterministic addresses for each namespace while maintaining the security properties of PDA derivation. The fixed-size storage approach prevents griefing attacks that could exploit variable-length data structures.

## Session Lifecycle Management

Session creation begins with guard account establishment through the `CreateGuardAccount` instruction. The guard account contains the session's security configuration, particularly the `allow_unregistered_cpi` flag that controls whether the session can invoke arbitrary programs beyond those explicitly registered.

The session account itself is created through `CreateSessionAccount`, which initializes all required fields including the namespace path, ownership information, and associated account references. The creation process establishes the session's Account Lookup Table and populates it with any initially registered accounts and programs.

Session initialization accepts `CreateSessionParams` that specify the namespace path (limited to 128 bytes in the creation parameter, expanded to 256 bytes in the final `NamespacePath` structure), initial metadata, and optional parent session reference. The system validates that the namespace path is properly formatted and that the caller has appropriate permissions for the requested namespace. If a parent session is specified, the system verifies the parent-child relationship and updates the parent's child tracking arrays.

The creation process registers initial accounts and programs through the Account Lookup Table, ensuring the session has immediate access to required resources. Initial account registration follows the same validation rules as subsequent ALT modifications, including permission verification and capacity limits.

Session invalidation supports both individual and cascading modes. Individual invalidation sets the `active` flag to false and increments the `nonce` for ownership tracking. Cascading invalidation propagates through the child hierarchy up to a configurable depth limit, ensuring that orphaned sessions do not persist.

The invalidation process emits structured events that external systems can monitor for state synchronization purposes. These events include the invalidated session address, the invalidation type, and any child sessions affected by cascading operations.

## Move Semantics and Ownership Transfer

The kernel implements sophisticated ownership transfer mechanisms that avoid shared mutable state through session invalidation and recreation patterns. The `InvalidationMove` pattern transfers ownership by invalidating the current session and creating a new session under different ownership, with the `nonce` increment providing version tracking.

The `CloseAndRecreateMove` pattern provides complete account closure followed by recreation, ensuring clean state transitions without residual data. This approach is particularly useful when ownership transfer requires fundamental changes to the session configuration or associated accounts.

`OwnershipVersion` tracking through the `nonce` field enables versioned ownership validation, preventing operations from executing against stale session references. When a session's ownership changes, the `nonce` increments and all existing references become invalid, forcing clients to refresh their session state.

The system supports deferred ownership transfer for complex scenarios where immediate transfer is not feasible. Deferred transfers create pending ownership records that can be claimed by the target owner through subsequent operations, enabling atomic multi-step ownership changes.

Batch invalidation capabilities allow multiple sessions to be invalidated in a single transaction, with configurable cascade depth limits to prevent compute budget exhaustion. The system tracks invalidation operations through event emission and maintains consistency across all affected sessions.

## Account Lookup Table Integration

Each session maintains a dedicated Account Lookup Table that serves as a secure registry of pre-approved accounts and programs. The ALT can store up to 4 borrowable accounts with specified read/write permissions, 4 programs authorized for Cross-Program Invocation, and 4 guard configurations for compatibility purposes.

ALT registration requires explicit permission specification for each account, with validation that the registering session has appropriate access to the target account. The registration process creates `RegisteredAccount` entries that include the account address, permission flags, and descriptive labels for operational clarity.

Program registration through `RegisteredProgram` entries enables controlled CPI operations by explicitly declaring which programs the session can invoke. Each registered program includes activation status and labeling for management purposes, with the system validating program executability and ownership before registration.

The ALT capacity limits (reduced from original specifications) optimize for Solana's 4KB stack constraints while maintaining essential functionality. These limits ensure that complex operations can execute without stack overflow while preserving security boundaries through explicit account declaration.

ALT modifications use dedicated instructions that validate caller permissions and maintain consistency across all registered entries. The system prevents unauthorized modifications and ensures that all changes preserve the security properties of the registration system.

## Stack Optimization Strategies

The session system implements comprehensive stack optimization to operate within Solana's 4KB execution limit. All data structures use compile-time known sizes to prevent heap allocation and enable precise stack frame calculations during development and testing.

Account labels are optimized to 8 bytes from the original 32-byte specification, and metadata fields are reduced to 32 bytes to minimize memory usage. These reductions maintain functional requirements while significantly reducing stack pressure during complex operations.

Helper function patterns break complex operations into minimal stack frame functions that handle specific aspects of session management. The `init_account_lookup_minimal` and `register_accounts_minimal` functions exemplify this approach by minimizing local variable usage and recursive call depth.

Boxed account contexts move large Anchor account structures to heap storage while keeping stack frames small. This approach enables the use of complex account validation while maintaining stack efficiency during program execution.

The stack optimization extends to cascading operations, where depth limits and deferred processing prevent unbounded recursion that could exhaust the stack. Complex invalidation cascades are broken into manageable chunks with event emission for external processing of deeper hierarchies.