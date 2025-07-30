# Valence Registry

Client-side registry for managing Valence functions and shard deployments. Provides utilities for working with the valence-kernel's hardcoded function registry and shard metadata.

## Features

- **Function Registry**: Client-side interface for kernel's hardcoded function registry
- **Shard Management**: Track and manage shard deployments and metadata  
- **IDL Integration**: Generate and validate IDL files for shard interfaces
- **Caching**: LRU caching for efficient function and shard lookups
- **Audit Support**: Built-in audit logging and deployment tracking

## Quick Start

```rust
use valence_registry::*;

// Create function registry
let mut registry = FunctionRegistry::new();

// Register a function (matches kernel's hardcoded entries)
let function = FunctionInfo {
    registry_id: 1,
    program_id: my_program_id,
    name: "transfer".to_string(),
    version: 1,
    is_active: true,
    estimated_compute_units: 5000,
    // ...
};
registry.register_function(function)?;

// Query functions
let transfer_fn = registry.get_function_by_name("transfer")?;
let all_functions = registry.get_active_functions();

// Shard registry
let mut shard_registry = ShardRegistry::new();
let shard = ShardMetadata {
    program_id: shard_program_id,
    name: "escrow-shard".to_string(),
    version: "1.0.0".to_string(),
    // ...
};
shard_registry.register_shard(shard)?;
```

## Modules

- `functions` - Function registry and metadata management
- `shards` - Shard deployment tracking and interface management
- `idl` - IDL generation and validation utilities
- `error` - Registry-specific error types

## Use Cases

- Client applications querying available functions
- Shard deployment automation and tracking
- IDL generation for cross-program integration
- Function discovery and capability queries
- Audit trail for deployments and changes

Built to align with valence-kernel's simplified hardcoded registry approach.