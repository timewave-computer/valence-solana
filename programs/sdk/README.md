# Valence Protocol Rust SDK

A comprehensive Rust SDK for interacting with the Valence Protocol, a capability-based execution framework on Solana.

## Overview

The Valence Protocol SDK provides a unified client library for interacting with the Valence Protocol programs:
- **Kernel Program**: Manages capabilities, namespaces, and execution tracking (with embedded eval)
- **Processor Singleton**: Stateless execution orchestration
- **Scheduler Singleton**: Multi-shard scheduling and queue management
- **Diff Singleton**: State diff calculation and optimization
- **Registry Program**: Manages library and ZK program registration

## Features

- **Unified Client**: Single client for all Valence Protocol operations
- **Type-Safe**: Comprehensive type definitions for all protocol interactions
- **Error Handling**: Hierarchical error system with detailed error codes
- **Validation**: Built-in validation for all inputs and parameters
- **Utilities**: Helper functions for common operations
- **Async/Await**: Full async support with tokio
- **Builder Patterns**: Fluent interfaces for complex operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
valence-sdk = { path = "../sdk" }
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use valence_sdk::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = ValenceConfig::localhost();
    
    // Create client
    let client = ValenceClient::new(config)?;
    
    // Initialize singleton programs
    let authority = generate_keypair();
    client.initialize_processor(&authority.pubkey()).await?;
    client.initialize_scheduler(&authority.pubkey(), 100, 1000).await?;
    client.initialize_diff(&authority.pubkey(), 50, OptimizationLevel::Aggressive).await?;
    client.initialize_registry(&authority.pubkey()).await?;
    
    // Grant a capability
    let capability_id = "my_capability";
    let verification_functions = vec![[1; 32], [2; 32]];
    let description = "My custom capability";
    
    client.grant_capability(
        &authority.pubkey(),
        &client.program_ids.shard,
        capability_id,
        verification_functions,
        description,
    ).await?;
    
    // Execute the capability
    let session = generate_keypair();
    let caller = generate_keypair();
    
    let context = ValenceExecutionContext::new(
        capability_id.to_string(),
        session.pubkey(),
        caller.pubkey(),
    ).with_input_data(vec![1, 2, 3, 4, 5]);
    
    let config = ExecutionConfig::default();
    let result = client.execute_capability(&context, &config).await?;
    
    println!("Capability executed: {}", result.transaction_result.signature);
    
    Ok(())
}
```

## Configuration

### Environment Variables

The SDK can be configured using environment variables:

```bash
export VALENCE_CLUSTER=devnet  # mainnet-beta, testnet, devnet, or localnet
export VALENCE_PAYER_KEYPAIR=~/.config/solana/id.json
```

### Programmatic Configuration

```rust
use valence_sdk::*;

// For localhost/testing
let config = ValenceConfig::localhost();

// For devnet
let config = ValenceConfig::devnet();

// Custom configuration
let config = ValenceConfig {
    program_ids: ProgramIds {
        kernel: "your_kernel_program_id".parse().unwrap(),
        processor: "your_processor_program_id".parse().unwrap(),
        scheduler: "your_scheduler_program_id".parse().unwrap(),
        diff: "your_diff_program_id".parse().unwrap(),
        registry: "your_registry_program_id".parse().unwrap(),
    },
    cluster: anchor_client::Cluster::Devnet,
    payer: load_keypair_from_file("~/.config/solana/id.json").unwrap(),
    commitment: Some(CommitmentConfig::confirmed()),
};
```

## Core Operations

### 1. Program Initialization

```rust
// Initialize processor singleton
client.initialize_processor(&authority.pubkey()).await?;

// Initialize scheduler singleton
client.initialize_scheduler(
    &authority.pubkey(),
    100,  // max_shards
    1000, // max_queue_size
).await?;

// Initialize diff singleton
client.initialize_diff(
    &authority.pubkey(),
    50,                            // max_batch_size
    OptimizationLevel::Aggressive,
).await?;

// Initialize registry program
client.initialize_registry(&authority.pubkey()).await?;
```

### 2. Capability Management

```rust
// Grant a capability
let capability_id = "token_transfer";
let verification_functions = vec![
    calculate_hash(b"basic_permission"),
    calculate_hash(b"parameter_constraint"),
];
let description = "Capability for token transfers";

client.grant_capability(
    &authority.pubkey(),
    &shard_state,
    capability_id,
    verification_functions,
    description,
).await?;

// Update a capability
client.update_capability(
    &authority.pubkey(),
    &shard_state,
    &capability_address,
    new_verification_functions,
    Some("Updated description".to_string()),
).await?;

// Revoke a capability
client.revoke_capability(
    &authority.pubkey(),
    &shard_state,
    &capability_address,
).await?;
```

### 3. Singleton Operations

#### Processor Operations
```rust
// Process a capability through processor singleton
client.process_capability(
    "my_capability".to_string(),
    vec![1, 2, 3, 4, 5],     // input_data
    Some(session_pubkey),
).await?;

// Pause/resume processor
client.pause_processor(&authority.pubkey()).await?;
client.resume_processor(&authority.pubkey()).await?;
```

#### Scheduler Operations
```rust
// Schedule an execution
client.schedule_execution(
    shard_pubkey,
    "operation_123".to_string(),
    5,                         // priority
    input_data,
).await?;

// Process the queue
client.process_queue(10).await?;  // max_operations

// Register a shard with ordering rules
let ordering_rules = vec![
    OrderingConstraint {
        constraint_type: ConstraintType::Before,
        operations: vec!["op1".to_string(), "op2".to_string()],
        priority: 10,
    },
];
client.register_shard(shard_pubkey, ordering_rules).await?;
```

#### Diff Operations
```rust
// Calculate diff between states
client.calculate_diff(old_state, new_state).await?;

// Process a batch of diffs
let diffs = vec![
    DiffOperation::Add { key: "key1".to_string(), value: vec![1, 2, 3] },
    DiffOperation::Update { 
        key: "key2".to_string(), 
        old_value: vec![4, 5], 
        new_value: vec![6, 7] 
    },
];
client.process_diff_batch(diffs, true).await?;  // atomic=true

// Optimize a batch
client.optimize_batch("batch_123".to_string()).await?;
```

### 4. Capability Execution

```rust
// Create execution context
let context = ValenceExecutionContext::new(
    "token_transfer".to_string(),
    session_pubkey,
    caller_pubkey,
)
.with_input_data(transfer_data)
.with_compute_limit(100_000)
.with_labels(vec!["finance".to_string(), "transfer".to_string()]);

// Configure execution
let config = ExecutionConfig {
    max_execution_time: Some(60),
    max_compute_units: Some(100_000),
    record_execution: true,
    parameters: Some(encoded_parameters),
};

// Execute capability
let result = client.execute_capability(&context, &config).await?;
println!("Execution completed: {}", result.transaction_result.signature);
```

### 4. Library Registry

```rust
// Register a library
let library_entry = LibraryEntry {
    library_id: "my_library".to_string(),
    name: "My Library".to_string(),
    version: "1.0.0".to_string(),
    author: authority.pubkey(),
    metadata_hash: calculate_metadata_hash(&name, &version, &description, &tags),
    program_id: library_program_id,
    status: LibraryStatus::Published,
    dependencies: vec![],
    tags: vec!["utility".to_string(), "math".to_string()],
    is_verified: false,
    usage_count: 0,
};

client.register_library(&authority.pubkey(), &library_entry).await?;

// Query a library
let library = client.query_library("my_library").await?;
if let Some(lib) = library {
    println!("Found library: {} v{}", lib.name, lib.version);
}

// List libraries with pagination
let pagination = PaginationOptions {
    page: Some(1),
    page_size: Some(10),
    sort_by: Some("name".to_string()),
    sort_order: Some("asc".to_string()),
};

let libraries = client.list_libraries(&pagination).await?;
for library in libraries.items {
    println!("Library: {} v{}", library.name, library.version);
}
```

## Builder Patterns

The SDK provides fluent builder interfaces for complex operations:

```rust
// Grant capability with builder
let instruction = GrantCapabilityBuilder::new("my_capability".to_string())
    .with_description("My custom capability".to_string())
    .add_verification_function([1; 32])
    .add_verification_function([2; 32])
    .build();

// Execute capability with builder
let instruction = ExecuteCapabilityBuilder::new(
    "my_capability".to_string(),
    session_pubkey,
    caller_pubkey,
)
.with_input_data(vec![1, 2, 3, 4, 5])
.with_compute_limit(100_000)
.with_max_execution_time(60)
.with_recording_disabled()
.build();

// Register library with builder
let instruction = RegisterLibraryBuilder::new(
    "my_library".to_string(),
    "My Library".to_string(),
    "1.0.0".to_string(),
    program_id,
)
.with_metadata_hash(metadata_hash)
.add_tag("utility".to_string())
.add_tag("math".to_string())
.build();
```

## Validation and Utilities

The SDK includes comprehensive validation and utility functions:

```rust
// Validation
validate_capability_id("my_capability")?;
validate_version("1.0.0")?;
validate_namespace("my_namespace")?;
validate_library_entry(&library_entry)?;

// Hash calculations
let metadata_hash = calculate_metadata_hash("name", "1.0.0", "description", &tags);
let settlement_hash = calculate_settlement_hash(&capability_id, &session, &input_data, timestamp);

// Key utilities
let keypair = generate_keypair();
let session_id = generate_session_id();
let pubkey_str = pubkey_to_string(&pubkey);
let pubkey = string_to_pubkey(&pubkey_str)?;

// Time utilities
let timestamp = current_timestamp();
let timestamp_str = timestamp_to_string(timestamp);
let is_recent = is_timestamp_recent(timestamp, 3600);

// Serialization
let json = serialize_json(&data)?;
let data: MyStruct = deserialize_json(&json)?;
let bytes = serialize_bincode(&data)?;
let data: MyStruct = deserialize_bincode(&bytes)?;
```

## Error Handling

The SDK provides comprehensive error handling with hierarchical error codes:

```rust
use valence_sdk::*;

match client.execute_capability(&context, &config).await {
    Ok(result) => {
        println!("Success: {}", result.transaction_result.signature);
    }
    Err(e) => {
        match e {
            ValenceError::Unauthorized => {
                println!("Authorization failed");
            }
            ValenceError::CapabilityNotFound(id) => {
                println!("Capability not found: {}", id);
            }
            ValenceError::NetworkError(msg) => {
                println!("Network error: {}", msg);
                if is_recoverable_error(&e) {
                    // Retry logic
                }
            }
            _ => {
                println!("Other error: {} (code: {})", e, e.code());
            }
        }
    }
}
```

## Error Codes

- **1000-1099**: Context Errors
- **2000-2999**: Authentication and Authorization Errors
- **3000-3099**: Validation Errors
- **4000-4099**: Capability & Session Errors
- **6000-6099**: System & Resource Errors
- **8000-8099**: SDK Errors
- **9000+**: Wrapped Errors

## Testing

Run the example:

```bash
cargo run --example basic_usage
```

Run tests:

```bash
cargo test
```

## Advanced Usage

### Custom Program IDs

```rust
let program_ids = ProgramIds {
    kernel: "YourKernelProgramId111111111111111111111111".parse().unwrap(),
    processor: "YourProcessorId111111111111111111111111".parse().unwrap(),
    scheduler: "YourSchedulerId111111111111111111111111".parse().unwrap(),
    diff: "YourDiffId11111111111111111".parse().unwrap(),
    registry: "YourRegistryProgramId1111111111111111111111".parse().unwrap(),
};

let config = ValenceConfig {
    program_ids,
    cluster: anchor_client::Cluster::Devnet,
    payer: keypair,
    commitment: Some(CommitmentConfig::confirmed()),
};
```

### Custom RPC Endpoint

```rust
let cluster = anchor_client::Cluster::Custom("https://api.devnet.solana.com".to_string(), "wss://api.devnet.solana.com/".to_string());

let config = ValenceConfig {
    program_ids: ProgramIds::default(),
    cluster,
    payer: keypair,
    commitment: Some(CommitmentConfig::confirmed()),
};
```

### Environment Configuration

```rust
// Load configuration from environment
let config = load_config_from_env()?;
let client = ValenceClient::new(config)?;
```

## Testing

The SDK provides comprehensive testing utilities to help you develop and test your Valence Protocol applications.

### Testing Utilities

```rust
use valence_sdk::utils::testing::*;

// Generate mock data for testing
let mock_capability = mock_data::mock_capability_id();
let mock_session = mock_data::mock_session();
let mock_context = mock_data::mock_execution_context();

// Create test contexts
let capability_context = context_builders::build_capability_context();
let session_context = context_builders::build_session_context();

// Setup test environment
let test_env = integration_helpers::setup_test_environment();
let test_accounts = integration_helpers::create_test_accounts(5);

// Verify test results
let expected = TestResult::success().with_log("Test completed".to_string());
let actual = TestResult::success().with_log("Test completed".to_string());
assert!(integration_helpers::verify_test_results(&expected, &actual));
```

### PDA Testing

```rust
use valence_sdk::utils::testing::pda_utils::*;

// Test PDA generation
let seeds = create_test_seeds("capability", "test_123");
let (pda, bump) = find_test_pda(&[&seeds[0], &seeds[1]], &program_id);
```

### Mock Data Generation

The SDK provides convenient mock data generators for all major types:

```rust
// Mock execution context with realistic test data
let context = mock_data::mock_execution_context();
assert_eq!(context.capability_id, "test_capability_12345");
assert_eq!(context.input_data, vec![1, 2, 3, 4, 5]);

// Mock session with default test configuration
let session = mock_data::mock_session();
assert!(session.is_active);
assert_eq!(session.capabilities, vec!["test_capability".to_string()]);
```
