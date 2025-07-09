# Capabilities Module - Capability Definitions & Shards

The capabilities module manages capability definitions and implements shard-based execution with embedded evaluation logic within the Valence Protocol.

## Module Purpose

The capabilities module is responsible for defining what capabilities are and their properties, managing shard state with embedded evaluation logic, implementing namespace-based access control through scoping, managing capability-level permissions, and coordinating with singleton modules for execution.

## Module Architecture

```mermaid
graph TB
    subgraph "Capabilities Module"
        A[eval_rules.rs] --> B[Shard State & Eval Config]
        C[scoping.rs] --> D[Namespace Scoping]
        E[instructions.rs] --> F[Capability Instructions]
        G[ordering_rules.rs] --> H[Partial Order Management]
        I[execution_config.rs] --> J[Execution Configuration]
        K[mod.rs] --> L[Module Exports]
    end
    
    subgraph "External Interfaces"
        M[Capability Validation]
        N[Shard Management]
        O[Access Control]
    end
    
    subgraph "Dependencies"
        P[Processor Module]
        Q[Scheduler Module]
        R[Verification Module]
    end
    
    E --> M
    A --> N
    C --> O
    
    B --> P
    G --> Q
    C --> R
```

## Components

### eval_rules.rs - Shard State & Evaluation Configuration

This component manages shard state with embedded evaluation logic. The ShardState account contains authority that manages the shard, processor program address for execution, pause state, total executions counter, shard version, PDA bump seed, and evaluation configuration. The EvalConfig structure defines maximum execution time allowed, maximum compute units allowed, execution recording settings, and default verification function requirements. Shards embed evaluation logic by default, providing simple deployment, better performance through reduced CPI calls, and tighter coupling between state and evaluation rules.

### scoping.rs - Namespace Scoping & Access Control

This component implements namespace-based access control and capability scoping. The CapabilityDefinition struct contains unique capability identifier, capability type (Function, DataAccess, SystemAdmin, etc.), capability scope with namespace restrictions, required permissions, and verification requirements. The NamespaceManager validates object access within namespaces, verifies capability composition rules, and enforces permission boundaries. Scoping ensures capabilities only access objects within their authorized namespaces.

### ordering_rules.rs - Partial Order Management

This component manages ordering constraints for multi-shard coordination. The PartialOrder structure defines execution ordering with unique identifier, ordering constraints list, and priority level. OrderingConstraint types include FIFO for first-in-first-out ordering, Priority-based ordering, and Dependency-based ordering with before/after relationships. The OrderingRuleRegistry maintains available ordering rules and applies them to compose partial orders from multiple shards.

### execution_config.rs - Execution Configuration

This component defines execution configuration for capabilities. The ExecutionConfig includes execution mode (Sequential, Parallel, Conditional), resource limits with compute units and memory allocation, timeout settings, and retry policies. CompleteExecutionConfig combines base configuration with app-specific settings for comprehensive execution control.

## Shard-Based Execution Flow

### Capability Execution with Shards

```mermaid
sequenceDiagram
    participant Client
    participant Shard as ShardState
    participant Eval as EvalConfig
    participant Proc as processor::
    participant Sched as scheduler::
    
    Client->>Shard: execute_capability(cap_def, input)
    Shard->>Eval: validate_execution_rules()
    Eval-->>Shard: validation_result
    Shard->>Shard: increment_execution_counter()
    Shard->>Proc: process_capability()
    Proc->>Proc: build_execution_context()
    Proc->>Proc: orchestrate_verification()
    Proc-->>Shard: execution_result
    Shard-->>Client: Result<ExecutionResult>
```

### Multi-Shard Coordination

```mermaid
graph TD
    A[Multiple Shards] --> B[scheduler::]
    B --> C[Collect Ordering Constraints]
    C --> D[Compose Partial Order]
    D --> E[Topological Sort]
    E --> F[Execute in Order]
    
    F --> G[processor::]
    G --> H[Execute Capabilities]
    H --> I[Update Shard States]
```

## Capability Validation Flow

### Capability Access Validation

```mermaid
sequenceDiagram
    participant Client
    participant CV as Capability Validator
    participant NS as Namespace Manager
    participant PM as Permission Manager
    
    Client->>CV: validate_capability_access(cap_id, context)
    CV->>CV: load_capability_definition(cap_id)
    CV->>NS: validate_namespace_access(namespace_id)
    NS-->>CV: namespace_access_result
    CV->>PM: check_required_permissions(permissions)
    PM-->>CV: permission_check_result
    CV->>CV: aggregate_validation_results()
    CV-->>Client: ValidationResult
```

### Namespace Scoping Flow

```mermaid
flowchart TD
    A[Access Request] --> B[Load Namespace]
    B --> C[Check Direct Permissions]
    C --> D{Permission Granted?}
    D -->|Yes| E[Allow Access]
    D -->|No| F[Check Parent Namespace]
    F --> G{Has Parent?}
    G -->|Yes| H[Check Parent Permissions]
    G -->|No| I[Deny Access]
    H --> J{Parent Allows?}
    J -->|Yes| E
    J -->|No| I
```

## Integration Points

### Processor Module Integration

The processor:: module handles stateless execution orchestration for capabilities. When a shard executes a capability, it delegates to the processor for context building through the ContextBuilder, verification orchestration via the VerificationOrchestrator, and execution engine operations. The processor maintains no state between executions, ensuring clean execution semantics.

### Scheduler Module Integration

The scheduler:: module manages multi-shard coordination. When multiple shards need coordinated execution, the scheduler collects ordering constraints from each shard's PartialOrder, composes them using the PartialOrderComposer, performs topological sorting to determine execution order, and manages the execution queue. This ensures proper ordering across shard boundaries.

### Verification Module Integration

Capability validation integrates with the verification:: module to execute verification chains. The shard's EvalConfig specifies default verification functions, which are combined with capability-specific verifications. The verification module executes these functions as pure predicates, returning aggregated results for access control decisions.

## Account Structures

### Shard State Account

The ShardState account is the primary account for capability execution, containing authority for shard management, processor program address, pause state, execution counter, version and bump seed, and embedded EvalConfig. The account uses PDA derivation with seeds ["shard_state"] and requires ShardState::SPACE allocation.

### Capability Definition Structure

The CapabilityDefinition is passed as instruction data and includes capability identifier, type, and scope, namespace restrictions for access control, required permissions and verifications, associated function identifiers, and execution configuration overrides.

## Error Handling

### Capability Errors

Capability errors include CapabilityNotFound, InvalidCapabilityType, NamespaceAccessDenied, PermissionNotSatisfied, ObjectNotInNamespace, OperationNotAllowed, ResourceQuotaExceeded, and InvalidNamespaceHierarchy.

### Namespace Errors

Namespace errors include NamespaceNotFound, ParentNamespaceNotFound, NamespaceAlreadyExists, CircularDependency, and NamespaceScopingError.

## Events

### Capability Events

Capability events include CapabilityRegistered with capability ID, type, namespace ID, and registration timestamp. CapabilityExecuted includes capability ID, session ID, caller, execution time, and success status. NamespaceCreated includes namespace ID, optional parent namespace, and creation timestamp. NamespaceAccessGranted includes namespace ID, requester, operation, and grant timestamp.

## Performance Optimizations

### Capability Caching

Optimized capability management uses caching for capability definitions, namespace access results, permission check results, and performance metrics. The OptimizedCapabilityManager validates capabilities with caching by checking capability cache, namespace cache with keys combining namespace ID and caller, permission cache with keys combining capability ID and caller, and returns capability validation results with cached data when available.

## Testing Patterns

### Capability Testing

Capability testing includes capability validation testing to verify proper access validation with test capabilities and execution contexts. Namespace scoping testing creates parent and child namespaces, tests access validation with different operation types and requesters, and verifies proper inheritance and restriction enforcement. 