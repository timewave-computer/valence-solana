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

### Detailed Execution Sequence

```mermaid
sequenceDiagram
    participant Client
    participant Kernel as kernel::execute_capability
    participant Cap as capabilities::processor
    participant Ver as verification::engine
    participant Func as functions::executor
    participant Proc as Processor Singleton
    participant Events as event::emitter
    
    Client->>Kernel: execute_capability(cap_id, input)
    Kernel->>Cap: build_execution_context()
    Cap->>Cap: validate_capability_access()
    
    Cap->>Ver: run_verification_chain()
    Ver->>Ver: execute_verifications()
    Ver-->>Cap: verification_results
    
    Cap->>Proc: CPI: process_capability()
    Proc->>Func: execute_function_chain()
    Func->>Func: process_functions()
    Func-->>Proc: execution_results
    Proc-->>Cap: execution_results
    
    Cap->>Events: emit_execution_events()
    Events-->>Cap: events_emitted
    Cap-->>Core: execution_complete
    Core-->>Client: Result<()>
```

## Singleton Coordination Flow

### Multi-Shard Scheduling

```mermaid
flowchart LR
    A[Shard 1] --> S[Scheduler Singleton]
    B[Shard 2] --> S
    C[Shard 3] --> S
    
    S --> D[Partial Order Composition]
    D --> E[Topological Sort]
    E --> F[Execute in Order]
    
    F --> P[Processor Singleton]
```

### Diff Processing Flow

```mermaid
sequenceDiagram
    participant Shard
    participant Diff as Diff
    participant Proc as Processor
    
    Shard->>Diff: calculate_diff(old, new)
    Diff->>Diff: compute_changes()
    Diff-->>Shard: diff_operations
    
    Shard->>Diff: process_batch(diffs)
    Diff->>Diff: optimize_batch()
    Diff->>Proc: apply_diffs()
    Proc-->>Diff: applied
    Diff-->>Shard: batch_result
```

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

### Batch Processing Pattern

```mermaid
flowchart LR
    A[Multiple Requests] --> B[Request Batching]
    B --> C[Batch Validation]
    C --> D[Batch Execution]
    D --> E[Batch State Updates]
    E --> F[Batch Event Emission]
```

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