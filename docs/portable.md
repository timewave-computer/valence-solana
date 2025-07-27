# Valence Microkernel: Portable Design Specification

## 1. Introduction

The Valence Microkernel is designed as a minimal, secure, and high-performance authorization layer for decentralized applications. Its core philosophy separates fundamental mechanisms (provided by the kernel) from application-specific policies (implemented by external verifiers or guard logic).

While the initial implementation targets Solana, the underlying architectural principles—session-based authorization, a deterministic virtual machine for guard evaluation, and a clear separation of concerns—are inherently portable. This document specifies a design for Valence that enables its core logic to be implemented across diverse blockchain execution environments, including EVM, CosmWasm, and Move, without compromising security, performance, or functionality.

## 2. Core Portable Components

The following components constitute the blockchain-agnostic core of the Valence Microkernel. These elements are defined conceptually and can be re-implemented in any deterministic smart contract environment.

### 2.1. Authorization Processing Unit (APU) / Guard VM

At the heart of Valence is a small, deterministic virtual machine responsible for evaluating guard logic. This VM operates on a flattened instruction set, ensuring auditability and predictable execution.

#### 2.1.1. `GuardOp` Instruction Set

The `GuardOp` enum defines the minimal set of opcodes for authorization logic. These opcodes are designed to be primitive, efficient, and universally applicable.

```rust
pub enum GuardOp {
    // State Assertion Opcodes (set result_flag based on context)
    CheckOwner,             // Is transaction signer the session owner?
    CheckExpiry { timestamp: i66 }, // Is current time < timestamp?
    CheckNotBefore { timestamp: i66 }, // Is current time >= timestamp?
    CheckUsageLimit { limit: u64 }, // Is session.usage_count < limit?

    // Control Flow Opcodes (manipulate instruction_pointer)
    JumpIfFalse { offset: u8 }, // If result_flag is false, jump IP by offset
    Terminate,              // Halt execution, return true (success)
    Abort,                  // Halt execution, return false (failure)

    // External Interaction Opcode (bridge to blockchain-specific CPI)
    Invoke { manifest_index: u8 }, // Execute CPI defined in manifest
}
```

#### 2.1.2. `APUEvaluator` Logic

The `APUEvaluator` is the core interpreter loop that executes `GuardOp` sequences. It maintains an `instruction_pointer` and a `result_flag`. This logic is purely functional and does not depend on blockchain-specific APIs, only on the `EvaluationContext` provided.

### 2.2. Session State Management

The `Session` account manages the lifecycle and state of an authorization session. Its structure and associated logic are conceptually portable.

*   **`Session` Data Structure:** Contains fields like `owner`, `protocol`, `usage_count`, `metadata`, `created_at`, `updated_at`, `borrowed_accounts` (with bitmap for efficient tracking), and a reference to its associated guard logic.
*   **Core Logic:** Methods for `increment_usage`, `set_metadata`, `borrow_account`, `release_account`, `release_all`, and `has_borrowed` are state transitions that can be implemented in any environment.

### 2.3. CPI Allowlisting

The concept of a program-controlled allowlist for external calls is a critical security pattern.

*   **`CPIAllowlist` State:** A state object (e.g., a singleton contract/module) that stores a list of `program_id`s (or equivalent contract/module addresses) that the kernel is permitted to invoke.
*   **`is_allowed` Logic:** A function to check if a given target program/contract is in the allowlist. This logic is portable, though the representation of `program_id`s will vary.

### 2.4. Shared Data Components

General-purpose security and state management patterns embedded within the session's shared data.

*   **`ReentrancyGuard`:** Logic to prevent re-entrant calls within a single execution context.
*   **`CPIDepthGuard`:** Logic to limit the depth of inter-contract/module calls.
*   **`FeatureFlags`:** A simple bitmask for enabling/disabling features.

## 3. Blockchain Abstraction Layer (BAL)

To achieve portability, a thin Blockchain Abstraction Layer (BAL) is defined. The core portable components interact solely with this BAL, which then translates calls to the native blockchain runtime.

### 3.1. `BlockchainContext` Interface

This interface defines the minimal set of blockchain-specific operations required by the portable core.

```rust
// Conceptual Rust Trait / Interface
pub trait BlockchainContext {
    // Get current execution context information
    fn get_caller_address() -> Address; // Address of the entity initiating the current call
    fn get_program_address() -> Address; // Address of the currently executing program/contract
    fn get_timestamp() -> i64; // Current block timestamp
    fn get_block_height() -> u64; // Current block height/slot
    fn get_return_data() -> Option<(Address, Vec<u8>)>; // Return data from last CPI
    fn set_return_data(program: Address, data: &[u8]); // Set return data for current call

    // Account/State Interaction
    fn get_account_info(address: Address) -> AccountInfo; // Get metadata about an account/contract
    fn get_account_data(address: Address) -> Vec<u8>; // Get raw data of an account/contract
    fn set_account_data(address: Address, data: &[u8]); // Write raw data to an account/contract
    fn get_account_balance(address: Address) -> u64; // Get native token balance
    fn is_account_writable(address: Address) -> bool; // Check if account is writable in current context
    fn is_account_signer(address: Address) -> bool; // Check if account is a signer in current context

    // Inter-Contract/Module Calls (CPI)
    fn invoke_contract(
        program_id: Address,
        instruction_data: &[u8],
        account_infos: &[AccountInfo],
    ) -> Result<()>;

    // Logging/Debugging
    fn log(message: &str);

    // Error Handling
    fn abort(error_code: u32, message: &str) -> !; // Terminate execution with an error
}

// Conceptual AccountInfo struct (simplified)
pub struct AccountInfo {
    pub address: Address,
    pub is_writable: bool,
    pub is_signer: bool,
    pub is_executable: bool,
    // ... other relevant metadata
}

// Conceptual Address type (e.g., Pubkey, H160, etc.)
pub type Address = Vec<u8>; // Or fixed-size array like [u8; 32]
```

### 3.2. `EvaluationContext` Adaptation

The `EvaluationContext` for the APU would be constructed using data retrieved via the `BlockchainContext`.

```rust
pub struct EvaluationContext {
    pub session: SessionState; // Portable Session struct
    pub caller: Address;
    pub clock: ClockInfo; // Portable clock info
    pub operation: Vec<u8>;
    pub remaining_accounts: Vec<AccountInfo>; // List of accounts passed to the current instruction
}

pub struct ClockInfo {
    pub timestamp: i66,
    pub block_height: u64,
}
```

## 4. Environment-Specific Implementations

For each target blockchain, a concrete implementation of the `BlockchainContext` trait (or equivalent) would be provided, along with necessary serialization and data mapping layers.

### 4.1. Solana Implementation (`valence-core-solana`)

*   **Mapping:** Direct mapping of `Pubkey` to `Address`, `AccountInfo` to `AccountInfo`, `Clock::get()` to `get_timestamp`/`get_block_height`, `solana_program::program::invoke` to `invoke_contract`.
*   **Serialization:** Continue using Borsh for on-chain state.
*   **Current Codebase:** The existing `valence-core` codebase would be refactored to explicitly use the `BlockchainContext` interface, rather than direct Solana APIs.

### 4.2. EVM Implementation (`valence-core-evm`)

*   **Language:** Solidity or Rust with `solang` for compilation to EVM bytecode.
*   **Mapping:** `Address` to `address` (H160), `timestamp` to `block.timestamp`, `block_height` to `block.number`, `invoke_contract` to `call`/`delegatecall`.
*   **State:** Session and GuardData would be stored as contract state variables or in storage slots.
*   **Serialization:** ABI encoding/decoding for instruction data, potentially RLP or custom serialization for internal state.

### 4.3. CosmWasm Implementation (`valence-core-cosmwasm`)

*   **Language:** Rust.
*   **Mapping:** `Address` to `Addr` (CosmWasm address type), `timestamp` to `env.block.time.seconds()`, `block_height` to `env.block.height`, `invoke_contract` to `CosmosMsg::Wasm(Execute { ... })`.
*   **State:** Session and GuardData would be stored in CosmWasm's `Storage` (e.g., using `cw_storage_plus`).
*   **Serialization:** JSON for messages, potentially Borsh for internal state.

### 4.4. Move Implementation (`valence-core-move`)

*   **Language:** Move.
*   **Mapping:** `Address` to `address` (Move address type), `timestamp` to `signer::timestamp()`, `block_height` to `signer::block_height()`, `invoke_contract` to `call_module_function`.
*   **State:** Session and GuardData would be Move resources or modules.
*   **Serialization:** Move's native serialization.

## 5. Serialization Strategy

To maintain portability while optimizing for each environment:

*   **Core Portable Data Structures:** `GuardOp`, `CPIManifestEntry`, `CompiledGuard`, `Session` (and its sub-structs like `BorrowedAccount`), `SessionSharedData`, `CPIAllowlist` should be defined in a blockchain-agnostic manner.
*   **Environment-Specific Serialization:** Each environment's adapter would handle the serialization/deserialization of these core structures to and from the native byte format of that blockchain (e.g., Borsh for Solana, ABI for EVM, JSON/Borsh for CosmWasm, Move's native serialization).
*   **Client-Side Compilation:** The client-side SDK remains responsible for compiling developer-friendly guard logic into the `CompiledGuard` bytecode, which is then serialized and sent to the appropriate blockchain program.

## 6. Compilation Strategy

*   **Client-Side Compiler:** The `Guard` enum (the developer-friendly, recursive representation) and its compiler (`guards/compiler.rs`) would reside in a client-side SDK (e.g., `valence-sdk-js`, `valence-sdk-rust`). This compiler generates the `CompiledGuard` bytecode.
*   **On-Chain Validation & Execution:** The on-chain kernel program for each environment receives the pre-compiled `CompiledGuard`, validates it, and executes it using its `APUEvaluator`.

## 7. Benefits of this Approach

*   **Maximum Portability:** The core logic is decoupled from any specific blockchain runtime.
*   **Enhanced Security:** The minimal APU and strict separation of concerns are maintained across all implementations.
*   **Optimized Performance:** Each implementation can leverage the native primitives and optimizations of its target blockchain.
*   **Maintainability:** Changes to the core logic can be propagated across environments more easily.
*   **Auditability:** The small, deterministic APU remains the primary focus for security audits.

This portable design ensures that the Valence Microkernel can serve as a foundational authorization layer across the multi-chain ecosystem, providing consistent security guarantees regardless of the underlying execution environment.
