# Kernel Program Architecture

The Valence Protocol kernel program implements a hosted microkernel that provides a unified execution environment for capability-based operations on Solana.

## Module Organization

```mermaid
graph TB
    subgraph "Kernel Program Entry Point"
        Program[#program kernel]
    end
    
    subgraph "Primary Modules"
        Capabilities[capabilities::*<br/>Capability Definitions & Embedded Eval]
        Sessions[sessions::*<br/>Session Management]
        Functions[functions::*<br/>Function Registry]
        Verification[verification::*<br/>Verification System]
    end
    
    subgraph "Supporting Systems"
        Events[events::*<br/>Event Coordination]
        Config[config::*<br/>Configuration]
        Optimization[optimization::*<br/>Performance]
        State[state::*<br/>Shared State]
        Error[error::*<br/>Error Handling]
    end
    
    Program --> Capabilities
    Capabilities --> Sessions
    Capabilities --> Functions
    Capabilities --> Verification
    Capabilities --> Processor[Processor Singleton]
    
    Events -.-> Capabilities
    Events -.-> Sessions
    Events -.-> Functions
    
    Config -.-> All[All Modules]
    Optimization -.-> All
    State -.-> All
    Error -.-> All
```

## Module Responsibilities

### capabilities:: Capability Definitions & Embedded Eval
The capabilities module defines capabilities and manages execution with embedded eval logic. It includes capability definition and namespace scoping in state.rs, execution rules and ordering constraints in eval_rules.rs, app-specific execution configuration in execution_config.rs, and capability-related instructions in instructions.rs. The module manages capability definition and validation, embedded execution logic, ordering rules for multi-shard coordination, and permission boundary enforcement.

### sessions:: Session Lifecycle Management
The sessions module manages session creation, activation, and lifecycle. It includes session configuration and permissions in state.rs, session factory and management in lifecycle.rs, session isolation and security in isolation.rs, and session-related instructions in instructions.rs. The module handles session creation and activation, permission and configuration management, session isolation and security boundaries, and optimistic session handling.


### functions:: Function Registry & Composition
The functions module manages the function registry and enables composition. It includes function discovery and registration in registry.rs, function execution coordination in execution.rs, function composition management in metadata.rs, and function-related operations in instructions.rs. The module handles function registration and discovery, function chain composition, function aggregation patterns, and metadata management.

### verification:: Verification Function System
The verification module executes verification functions for capability validation. It contains core verification predicates in predicates.rs, permission verification in basic_permission.rs, system-level authorization in system_auth.rs, session validation in session_creation.rs, parameter validation in parameter_constraint.rs, block height verification in block_height.rs, and zero-knowledge proof verification in zk_proof.rs. The module manages pure verification function execution, multi-layered verification composition, verification result aggregation, and pluggable verification architecture.

## Inter-Module Communication

### Direct Function Calls
Most module interactions use direct function calls for maximum performance:

```mermaid
sequenceDiagram
    participant Client
    participant Kernel as kernel::program
    participant Caps as capabilities::*
    participant Verify as verification::*
    participant Proc as Processor Singleton
    
    Client->>Kernel: execute_capability()
    Kernel->>Caps: process_execution()
    Caps->>Verify: run_verifications()
    Verify-->>Caps: verification_results
    Caps->>Proc: CPI execute()
    Proc-->>Caps: execution_result
    Caps-->>Kernel: execution_result
    Kernel-->>Client: Result<()>
```

### Event-Based Coordination
For cross-cutting concerns and loose coupling:

```mermaid
graph LR
    A[Module A] -->|emit| Events[Event System]
    Events -->|notify| B[Module B]
    Events -->|notify| C[Module C]
    Events -->|notify| D[Module D]
```

### Shared State Access
Modules access shared state through well-defined interfaces. For example, the capabilities module can access session state from the session account and build execution context using the loaded session configuration.

### Singleton Program Integration
The kernel program integrates with three singleton programs via CPI:

```mermaid
graph LR
    Kernel[Kernel Program] -->|CPI| Proc[Processor Singleton]
    Kernel -->|CPI| Sched[Scheduler Singleton]
    Kernel -->|CPI| Diff[Diff Singleton]
    
    Proc -->|Stateless Execution| Kernel
    Sched -->|Queue Management| Kernel
    Diff -->|State Diffs| Kernel
```

- **Processor**: Handles stateless execution orchestration
- **Scheduler**: Manages multi-shard scheduling and partial order composition
- **Diff**: Calculates and optimizes state diffs

## Instruction Flow

### Main Program Instructions
The kernel program exposes four primary instructions:

```mermaid
graph TD
    A[initialize] --> B{System Ready?}
    B -->|No| C[Setup Core State]
    B -->|Yes| D[Return Success]
    
    E[execute_capability] --> F[Build Context]
    F --> G[Validate Capability]
    G --> H[Run Verifications]
    H --> I[Execute Functions]
    I --> J[Emit Events]
    
    K[pause] --> L[Set Paused State]
    M[resume] --> N[Clear Paused State]
```

### Module Instruction Patterns
Each module follows consistent patterns for instruction handling. They verify accounts and permissions during context validation, load required state from accounts, execute module-specific business logic, update accounts with new state, emit relevant events, and return success or error results.

## Security & Isolation

### Permission Boundaries
```mermaid
graph TB
    subgraph "Permission Layer"
        P1[Session Permissions]
        P2[Capability Permissions]  
        P3[Function Permissions]
        P4[Namespace Permissions]
    end
    
    subgraph "Validation Layer"
        V1[Session Validation]
        V2[Capability Validation]
        V3[Parameter Validation]
        V4[Access Control]
    end
    
    P1 --> V1
    P2 --> V2
    P3 --> V3
    P4 --> V4
```

### Isolation Mechanisms
The system provides namespace scoping where objects are accessible only within defined namespaces. Session boundaries constrain operations to session permissions. Capability limits restrict functions to capability definitions. Verification gates provide multi-layer verification before execution.

## Performance Optimizations

### Direct Call Optimization
The architecture eliminates inter-process communication overhead through direct function calls. Modules use shared memory access patterns and benefit from compile-time optimization across modules.

### State Access Patterns
The system employs efficient account loading and caching strategies. It minimizes state transitions and uses batch operations where possible.

### Execution Optimization
The architecture supports inline verification execution, optimistic execution patterns, and lazy evaluation strategies to maximize performance. 