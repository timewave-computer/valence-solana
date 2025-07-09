# System Data Flow

This document describes how data flows through the Valence Protocol system, from initial requests through execution completion.

## Request Lifecycle Overview

```mermaid
flowchart TD
    A[Client Request] --> B{Request Type}
    
    B -->|Session Creation| C[Session Flow]
    B -->|Capability Execution| D[Execution Flow]
    B -->|System Management| E[Management Flow]
    
    C --> F[Session Factory]
    F --> G[Session State]
    G --> H[Session Events]
    
    D --> I[Capability Validation]
    I --> J[Verification Chain]
    J --> K[Function Execution]
    K --> L[Result & Events]
    
    E --> M[System State Updates]
    M --> N[Configuration Changes]
```

The Valence Protocol processes three main types of requests through distinct flows. Session requests go through the session factory to create session entries and emit events for off-chain services. Capability execution requests are validated against shard state, run through verification chains, and execute functions with result tracking. System management requests update configuration and pause/resume system components.

## Session Creation Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant SF as Session Factory
    participant S as Session State
    participant E as Event System
    
    C->>SF: request_session_creation()
    SF->>SF: validate_request()
    SF->>S: create_session_entry()
    S->>S: initialize_session_state()
    SF->>E: emit(SessionCreated)
    SF->>C: session_address
    
    Note over SF: Optimistic creation pattern
    C->>SF: activate_session()
    SF->>S: update_status(Active)
    SF->>E: emit(SessionActivated)
```

Session creation uses an optimistic pattern where the session factory immediately creates a session entry with a computed PDA, validates the request parameters, and emits events for off-chain services to handle the actual session program initialization. This provides immediate feedback to clients while deferring expensive operations.

## Capability Execution Flow

### Primary Execution Path

```mermaid
flowchart TD
    A[execute_capability] --> B[Build Execution Context]
    B --> C[Load Session State]
    C --> D[Validate Capability Access]
    
    D --> E{Capability Valid?}
    E -->|No| F[Return Error]
    E -->|Yes| G[Load Verification Functions]
    
    G --> H[Execute Verification Chain]
    H --> I{All Verifications Pass?}
    I -->|No| J[Return Verification Error]
    I -->|Yes| K[Execute Function Chain]
    
    K --> L[Update State]
    L --> M[Emit Events]
    M --> N[Return Success]
```

The capability execution flow follows a strict validation-then-execution pattern. The shard state processes the request using embedded eval logic, building an execution context, loading and validating session state, executing verification functions in sequence, and only proceeding to function execution if all verifications pass. State updates and event emission happen atomically.

### Detailed Execution Sequence

```mermaid
sequenceDiagram
    participant Client
    participant Kernel as kernel::execute_capability
    participant Cap as capabilities::
    participant Ver as verification::
    participant Func as functions::
    participant Proc as processor::
    participant Events as events::
    
    Client->>Kernel: execute_capability(cap_id, input)
    Kernel->>Cap: build_execution_context()
    Cap->>Cap: validate_capability_access()
    
    Cap->>Ver: run_verification_chain()
    Ver->>Ver: execute_verifications()
    Ver-->>Cap: verification_results
    
    Cap->>Proc: process_capability()
    Proc->>Func: execute_function_chain()
    Func->>Func: process_functions()
    Func-->>Proc: execution_results
    Proc-->>Cap: execution_results
    
    Cap->>Events: emit_execution_events()
    Events-->>Cap: events_emitted
    Cap-->>Kernel: execution_complete
    Kernel-->>Client: Result<()>
```

The detailed execution sequence shows how the kernel delegates to the capabilities module, which uses the processor singleton for stateless execution orchestration. The processor handles verification chain execution, function execution, and result aggregation. Events are emitted at key points to coordinate between modules and notify external systems.

## Singleton Module Coordination Flow

### Multi-Shard Scheduling

```mermaid
flowchart LR
    A[Shard 1] --> S[scheduler::]
    B[Shard 2] --> S
    C[Shard 3] --> S
    
    S --> D[Partial Order Composition]
    D --> E[Topological Sort]
    E --> F[Execute in Order]
    
    F --> P[processor::]
```

The scheduler singleton coordinates execution across multiple shards by collecting partial orders from each shard, composing them using dependency analysis, and performing topological sorting to determine execution order. This ensures capabilities with dependencies execute in the correct sequence while maximizing parallelism for independent operations.

### Diff Processing Flow

```mermaid
sequenceDiagram
    participant Shard
    participant Diff as diff::
    participant Proc as processor::
    
    Shard->>Diff: calculate_diff(old, new)
    Diff->>Diff: compute_changes()
    Diff-->>Shard: diff_operations
    
    Shard->>Diff: process_batch(diffs)
    Diff->>Diff: optimize_batch()
    Diff->>Proc: apply_diffs()
    Proc-->>Diff: applied
    Diff-->>Shard: batch_result
```

The diff module processes state changes by calculating differences between old and new state, optimizing batches of operations for efficiency, and applying changes atomically. This supports both key-value operations (Add/Update/Remove/Move) and positional operations (Insert/Delete/Replace) for different data structures.

## State Transitions

### Session State Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Requested : request_session_creation()
    Requested --> Created : create_session_entry()
    Created --> Active : activate_session()
    Active --> Paused : pause_session()
    Paused --> Active : resume_session()
    Active --> Closed : close_session()
    Closed --> [*]
    
    Requested --> [*] : timeout/cancel
    Created --> [*] : timeout/cancel
```

Session lifecycle management tracks sessions through discrete states from request to closure. The session factory creates entries optimistically in the "Requested" state, transitions to "Created" when the session program initializes the account, and becomes "Active" when fully operational. Sessions can be paused and resumed as needed, with proper cleanup when closed.

### Capability Execution States

```mermaid
stateDiagram-v2
    [*] --> Validating : execute_capability()
    Validating --> Verified : validations_pass()
    Validating --> Failed : validation_error()
    Verified --> Executing : begin_execution()
    Executing --> Completed : execution_success()
    Executing --> Failed : execution_error()
    Completed --> [*]
    Failed --> [*]
```

Capability execution follows a strict state machine where validation must complete successfully before execution begins. The shard state tracks execution counters and applies eval rules to ensure compliance with configured limits and constraints. Failed executions are logged for debugging and monitoring.

## Data Flow Patterns

### Context Building Pattern

```mermaid
graph LR
    A[Raw Input] --> B[Context Builder]
    B --> C[Session Context]
    B --> D[Capability Context]
    B --> E[Execution Context]
    
    C --> F[Unified Context]
    D --> F
    E --> F
    
    F --> G[Execution Engine]
```

The context builder pattern consolidates information from multiple sources into a unified execution context. This includes session state, capability definitions, and execution parameters. The context builder validates inputs, resolves dependencies, and creates an immutable context object passed through the execution pipeline.

### Verification Chain Pattern

```mermaid
flowchart TD
    A[Verification Request] --> B[Load Verification Functions]
    B --> C{More Verifications?}
    C -->|Yes| D[Execute Next Verification]
    D --> E{Verification Pass?}
    E -->|No| F[Return Failure]
    E -->|Yes| C
    C -->|No| G[All Verifications Complete]
    G --> H[Return Success]
```

The verification chain executes a sequence of pure verification functions, each returning a boolean result. The chain stops immediately on the first failure, providing fast feedback. Verification functions are loaded by hash from the registry and executed with the current execution context and parameters.

### Function Composition Pattern

```mermaid
graph TD
    A[Function Chain] --> B[Function 1]
    B --> C[Function 2]
    C --> D[Function N]
    
    E[Function Aggregation] --> F[Parallel Function A]
    E --> G[Parallel Function B]
    E --> H[Parallel Function C]
    
    F --> I[Aggregation Function]
    G --> I
    H --> I
```

Function composition supports both sequential chains and parallel aggregations. Chains execute functions in order, passing output from one function as input to the next. Aggregations execute functions in parallel and combine results using configurable aggregation strategies (merge, vote, consensus).

## Event Propagation

### Event Flow Architecture

```mermaid
graph TB
    subgraph "Event Sources"
        S1[Session Events]
        S2[Execution Events]
        S3[Verification Events]
        S4[Function Events]
    end
    
    subgraph "Event System"
        E[Event Coordinator]
        F[Event Filters]
        R[Event Routing]
    end
    
    subgraph "Event Consumers"
        C1[On-Chain Listeners]
        C2[Off-Chain Services]
        C3[Analytics Systems]
        C4[Monitoring Systems]
    end
    
    S1 --> E
    S2 --> E
    S3 --> E
    S4 --> E
    
    E --> F
    F --> R
    
    R --> C1
    R --> C2
    R --> C3
    R --> C4
```

The event system coordinates communication between modules and external systems. Events are emitted from various sources, processed through filters and routing logic, and delivered to registered consumers. This enables loose coupling between components and supports real-time monitoring and analytics.

### Event Types & Timing

```mermaid
timeline
    title Capability Execution Events
    
    section Initiation
        CapabilityExecutionStarted : Context Built
                                   : Session Loaded
    
    section Validation  
        CapabilityValidated : Access Confirmed
                           : Permissions Checked
        
        VerificationStarted : Verification Chain Begins
        VerificationCompleted : All Verifications Pass
    
    section Execution
        FunctionExecutionStarted : Function Chain Begins
        FunctionCompleted : Individual Function Done
        
    section Completion
        CapabilityExecutionCompleted : All Functions Complete
                                    : State Updated
                                    : Results Available
```

Event timing follows the execution lifecycle, providing visibility into each stage of capability processing. Events include detailed context about the operation, timing information, and success/failure status. This enables comprehensive monitoring and debugging of system behavior.

## Data Persistence Patterns

### Account State Management

```mermaid
graph LR
    A[Request] --> B[Load Accounts]
    B --> C[Validate State]
    C --> D[Execute Logic]
    D --> E[Update State]
    E --> F[Persist Changes]
    F --> G[Emit Events]
```

Account state management follows Solana's ownership model where programs own and modify their account data. The system loads relevant accounts, validates current state, executes business logic, updates state atomically, and emits events. Account constraints ensure data integrity and prevent unauthorized modifications.

### Optimistic Update Pattern

```mermaid
sequenceDiagram
    participant Client
    participant State as Account State
    participant Logic as Business Logic
    participant Events as Event System
    
    Client->>State: load_current_state()
    State-->>Client: current_state
    
    Client->>Logic: execute_optimistically()
    Logic->>State: update_optimistic_state()
    Logic->>Events: emit_optimistic_event()
    
    Note over Logic: Async validation/confirmation
    
    Logic->>State: confirm_or_rollback()
    Logic->>Events: emit_final_event()
```

The optimistic update pattern provides immediate feedback to clients while deferring expensive operations. Used primarily in session creation, the system creates entries immediately with "pending" status, emits events for off-chain processing, and later confirms or rolls back the operation based on validation results.

## Performance Considerations

### Efficient Data Access

```mermaid
graph TD
    A[Request] --> B{Cached?}
    B -->|Yes| C[Use Cache]
    B -->|No| D[Load from Account]
    D --> E[Update Cache]
    C --> F[Process Request]
    E --> F
```

Performance optimization uses caching strategies for frequently accessed data like function definitions, verification results, and session state. Cache keys are constructed from relevant parameters, with LRU eviction policies to manage memory usage. Cache hits provide significant performance improvements for repeated operations.

### Batch Processing Pattern

```mermaid
flowchart LR
    A[Multiple Requests] --> B[Request Batching]
    B --> C[Batch Validation]
    C --> D[Batch Execution]
    D --> E[Batch State Updates]
    E --> F[Batch Event Emission]
```

Batch processing improves throughput by grouping related operations and processing them together. The scheduler singleton coordinates batching across shards, while the diff module optimizes state changes. Batching reduces transaction costs and improves resource utilization.

## Security in Data Flow

### Permission Validation Points

```mermaid
graph TD
    A[Request Entry] --> B[Session Permission Check]
    B --> C[Capability Permission Check]
    C --> D[Namespace Permission Check]
    D --> E[Function Permission Check]
    E --> F[Parameter Validation]
    F --> G[Execute if All Pass]
    
    B -->|Fail| H[Reject Request]
    C -->|Fail| H
    D -->|Fail| H
    E -->|Fail| H
    F -->|Fail| H
```

Security validation occurs at multiple layers with fail-fast behavior. Each validation point checks specific permissions and constraints, immediately rejecting requests that don't meet requirements. This defense-in-depth approach ensures comprehensive security coverage throughout the execution flow.

### Data Validation Flow

```mermaid
sequenceDiagram
    participant Input as Raw Input
    participant Val as Validators
    participant Ctx as Context
    participant Exec as Executor
    
    Input->>Val: validate_input_format()
    Val->>Val: validate_signatures()
    Val->>Val: validate_permissions()
    Val->>Ctx: build_validated_context()
    Ctx->>Exec: execute_with_context()
```

Data validation follows a strict pipeline where input format is validated first, followed by cryptographic signature verification, permission checking, and context building. Only validated data proceeds to execution, ensuring system integrity and preventing malicious or malformed inputs from affecting system state. 