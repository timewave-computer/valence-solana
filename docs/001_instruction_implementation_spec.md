# Instruction Implementation Specification

## Overview

The Valence Kernel implements a dual-path instruction architecture that balances architectural clarity with operational flexibility. Direct instructions provide optimized, single-purpose operations for common tasks, while batch instructions enable complex, multi-step operations that require atomic execution across multiple accounts and programs.

## Direct Instruction Architecture

Direct instructions handle the majority of common operations through specialized, optimized handlers that bypass the generic execution engine. Each direct instruction is implemented as a dedicated Anchor instruction handler with specific account contexts and parameter validation.

The `SPL Transfer` instruction exemplifies the direct instruction pattern, accepting a session reference, guard account, source and destination token accounts, authority, token program, and amount parameter. The implementation validates session ownership, checks guard permissions, verifies account registration in the session's Account Lookup Table, and executes the token transfer through Cross-Program Invocation.

Session invalidation instructions provide direct access to session lifecycle management without requiring batch operations. The `InvalidateSession` instruction accepts the target session and validates caller permissions before setting the `active` flag to false and incrementing the `nonce`. Cascading invalidation extends this pattern to invalidate child sessions within depth limits.

Account Lookup Table management instructions enable direct ALT modifications through dedicated handlers. The `ManageAlt` instruction accepts account arrays for registration, permission specifications, and program registrations, validating each entry before updating the ALT state. This direct approach provides clear audit trails and simplified validation logic.

Child account creation uses direct instructions to enable dynamic account generation within namespace boundaries. The `CreateChildAccount` instruction derives child account addresses using the parent session's namespace as a seed component, validates ownership permissions, and initializes the account with specified parameters.

Direct instructions implement comprehensive parameter validation at the instruction level, ensuring that all inputs meet system requirements before execution begins. This front-loading of validation reduces compute unit consumption and provides clear error reporting for client applications.

## Batch Operation Framework

Batch operations provide atomic execution of complex operation sequences through the `ExecuteBatch` instruction, which accepts an `OperationBatch` structure containing account arrays, operation sequences, and metadata for execution control.

The `OperationBatch` structure encapsulates all required information for batch execution, including a fixed-size account array that can reference up to 12 accounts, an operation array supporting up to 5 discrete operations, and length fields that specify the actual number of accounts and operations to process.

Account referencing within batch operations uses index-based addressing where operations specify account indices rather than direct public keys. This approach enables efficient account resolution while maintaining security through pre-registration requirements in the session's Account Lookup Table.

Operation sequencing ensures that operations execute in the specified order with proper dependency management. `BorrowAccount` operations must precede any operations that access the specified accounts, while `ReleaseAccount` operations return borrowed accounts to available status for subsequent operations or other sessions.

```rust
pub struct OperationBatch {
    pub accounts: [Pubkey; MAX_BATCH_ACCOUNTS],
    pub accounts_len: u8,
    pub operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS],
    pub operations_len: u8,
}
```

Batch validation occurs at multiple stages during execution, beginning with structural validation of the batch parameters, account resolution through the session's ALT, and operation-specific validation for each discrete operation. Failed validation at any stage results in complete batch rejection without partial execution.

The batch execution engine maintains operation context throughout the execution sequence, tracking borrowed accounts, CPI depth, and compute unit consumption. This context enables proper resource management and prevents operations from exceeding system limits or conflicting with concurrent operations.

## Operation Type Implementation

The `BorrowAccount` operation enables sessions to gain exclusive or shared access to pre-registered accounts. The operation accepts an account index referencing the session's Account Lookup Table and an access mode specifying read-only or read-write access. Validation ensures that the requested access mode is compatible with the pre-registered permissions.

`BorrowAccount` implementation locates an available slot in the session's `borrowed_accounts` array, validates that the account is registered in the ALT with appropriate permissions, and updates both the `borrowed_accounts` entry and the `borrowed_bitmap` to reflect the new borrow. The operation fails if no slots are available or if the account is already borrowed.

`ReleaseAccount` operations return borrowed accounts to available status, enabling reuse by subsequent operations or other sessions. The operation validates that the specified account index corresponds to a currently borrowed account and updates the session state to reflect the release.

Account release validation ensures that the releasing session currently holds a borrow on the specified account and that no other operations in the current batch depend on continued access to the account. Premature releases that could compromise operation integrity are rejected with appropriate error codes.

`CallRegisteredFunction` operations invoke programs that have been registered in the system's function registry using numeric identifiers rather than full program addresses. These operations accept a registry ID, account index array, data payload, and length specifications for both accounts and data.

Function registry resolution maps the provided registry ID to a concrete program address through a hardcoded mapping maintained within the kernel. This approach provides deterministic function identification while enabling committee-managed updates to registered functions for security-critical operations.

`UnsafeRawCpi` operations enable direct program invocation for programs not in the function registry, subject to guard account permission through the `allow_unregistered_cpi` flag. These operations provide maximum flexibility for innovative protocols while maintaining explicit opt-in security controls.

Raw CPI validation requires that the session's guard account explicitly enables unregistered CPI through the `allow_unregistered_cpi` flag. Operations attempting raw CPI without this permission are rejected immediately, ensuring that arbitrary program invocation remains an explicit choice rather than an accidental capability.

## Account Resolution and Validation

Account resolution begins with index validation to ensure that all operation-specified account indices fall within the valid range of the batch's account array. Invalid indices result in immediate batch rejection with clear error reporting to the calling program.

Account Lookup Table resolution maps each account index to a concrete account address through the session's pre-registered ALT entries. The resolution process validates that the account has been explicitly registered and that the current operation's access requirements are compatible with the pre-registered permissions.

Account existence validation ensures that all referenced accounts exist on-chain and contain expected data structures. Token accounts are validated for proper ownership and mint relationships, while program accounts are verified for executability and appropriate owner programs.

Permission compatibility checking compares the operation's requested access mode with the pre-registered permissions in the ALT. Operations requesting write access to read-only registered accounts are rejected, while read-only operations on read-write accounts are permitted as safe permission restrictions.

Account ownership validation ensures that all account mutations comply with Solana's ownership rules. Token account transfers require appropriate authority signatures, while program account mutations require ownership by the calling program or explicit delegation through program interfaces.

## Cross-Program Invocation Management

CPI operations undergo multi-layer security validation beginning with global allowlist checking for system-wide approved programs. Programs on the global allowlist can be invoked by any session without additional permission requirements, providing a foundation of widely trusted operations.

Session-specific program authorization through the Account Lookup Table enables controlled CPI for programs not on the global allowlist. Sessions must explicitly register programs before invoking them, with registration requiring appropriate authority and validation of program executability.

Guard-controlled CPI through the allow_unregistered_cpi flag enables sessions to invoke arbitrary programs not covered by global or session-specific authorization. This capability requires explicit opt-in through guard account configuration, making the security implications clear to protocol developers.

CPI depth tracking prevents unbounded recursion that could exhaust compute budgets or create attack vectors. The system maintains a depth counter that increments on each CPI operation and rejects operations that would exceed configured depth limits.

CPI account validation ensures that all accounts passed to invoked programs meet the same registration and permission requirements as direct operations. Cross-program account access maintains the same security boundaries established by the session's Account Lookup Table.

## Instruction Parameter Validation

Parameter validation occurs at multiple levels within the instruction processing pipeline. Anchor framework validation provides basic type safety and account relationship verification, while kernel-specific validation ensures compliance with system constraints and security requirements.

Numeric parameter validation ensures that all quantity specifications fall within acceptable ranges. Token transfer amounts are validated for non-zero positive values, while account indices are checked against array bounds and operation counts are verified against system limits.

String parameter validation applies to namespace paths and metadata fields, ensuring proper encoding, length constraints, and character restrictions. Namespace paths undergo hierarchical format validation to ensure proper parent-child relationships and prevent invalid path construction.

Array parameter validation checks that all fixed-size arrays contain valid entries within their specified lengths. Account arrays are validated for unique entries and proper key format, while operation arrays are checked for valid operation types and proper sequencing requirements.

Reference parameter validation ensures that all account references point to valid, accessible accounts. Session references are validated for active status and appropriate ownership, while program references are checked for executability and proper registration status.

## Error Handling and Recovery

Instruction error handling provides comprehensive error reporting with specific error codes and descriptive messages for debugging and client application integration. Each validation stage generates appropriate errors that indicate the specific failure mode and affected parameters.

Batch operation error handling ensures atomic execution properties where any operation failure results in complete batch rollback without partial state changes. Failed operations generate detailed error information including the operation index, failure type, and specific validation errors.

Account validation errors provide specific information about which accounts failed validation and the nature of the validation failure. Permission errors indicate the requested versus available permissions, while existence errors specify which accounts could not be located or accessed.

CPI error propagation ensures that errors from invoked programs are properly captured and reported to the calling session. CPI errors include both the invoked program's error information and additional context about the invocation parameters and account states.

Recovery mechanisms enable clients to retry failed operations after addressing the underlying issues. Nonce-based validation ensures that retry attempts operate against current session state, while detailed error reporting enables targeted fixes for specific failure modes.

## Compute Budget Management

Compute unit tracking occurs throughout instruction execution to ensure operations complete within Solana's compute budget limits. The system monitors compute unit consumption at operation boundaries and provides early termination for operations that would exceed available budget.

Batch operation compute budgeting allocates available compute units across the operation sequence with reservations for account validation, CPI operations, and state updates. Complex batches may require compute budget increases through client-side compute unit price adjustments.

CPI compute budget management ensures that invoked programs receive appropriate compute allocations while preventing excessive consumption that could compromise batch completion. The system reserves compute units for post-CPI validation and state updates.

Operation complexity estimation provides clients with guidance for compute budget planning based on the number of accounts, operations, and expected CPI depth. These estimates enable appropriate fee calculation and reduce the likelihood of compute budget exhaustion during execution.

Performance optimization throughout the instruction implementation minimizes compute unit consumption through efficient algorithms, cached validation results, and streamlined account access patterns. The system prioritizes compute efficiency to maximize the complexity of operations that can be performed within budget constraints.