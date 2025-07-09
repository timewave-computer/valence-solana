# Version Compatibility Strategy for Solana and Valence-Coprocessor

## Problem Statement
- **Solana 2.0** requires Rust dependencies that use Edition 2021 or earlier
- **valence-coprocessor** depends on `buf-fs 0.1.2` which requires Edition 2024
- We need both to work together in the same project

## Recommended Solution: Replace buf-fs

### Implementation Plan

1. **Create a compatibility module** in valence-coprocessor-domain-prover:

```rust
// src/filesystem_compat.rs
use std::collections::HashMap;
use anyhow::{Result, anyhow};

pub struct FileSystem {
    files: HashMap<String, Vec<u8>>,
    capacity: usize,
    used: usize,
}

pub struct File {
    pub path: String,
    pub contents: Vec<u8>,
}

impl FileSystem {
    pub fn new(capacity: usize) -> Result<Self> {
        Ok(Self {
            files: HashMap::new(),
            capacity,
            used: 0,
        })
    }

    pub fn open(&mut self, path: &str) -> Result<File> {
        self.files.get(path)
            .map(|contents| File {
                path: path.to_string(),
                contents: contents.clone(),
            })
            .ok_or_else(|| anyhow!("File not found: {}", path))
    }

    pub fn save(&mut self, file: File) -> Result<()> {
        let new_size = self.used - self.files.get(&file.path)
            .map(|v| v.len()).unwrap_or(0) + file.contents.len();
        
        if new_size > self.capacity {
            return Err(anyhow!("Filesystem capacity exceeded"));
        }

        self.files.insert(file.path, file.contents);
        self.used = new_size;
        Ok(())
    }
}
```

2. **Update valence-coprocessor** to use conditional compilation:

```toml
[dependencies]
buf-fs = { version = "0.1.2", optional = true }

[features]
default = ["filesystem-compat"]
filesystem-compat = []
buf-fs = ["dep:buf-fs"]
```

## Alternative Solutions

### 2. Process Isolation Architecture
Run valence-coprocessor in a separate process and communicate via IPC:

```rust
// Define a trait for cross-process communication
#[async_trait]
pub trait CoprocessorService {
    async fn execute(&self, request: ExecuteRequest) -> Result<ExecuteResponse>;
    async fn get_storage(&self, controller: &[u8; 32]) -> Result<Vec<u8>>;
    async fn set_storage(&self, controller: &[u8; 32], data: &[u8]) -> Result<()>;
}

// Implement for local (same process) and remote (IPC) variants
pub struct LocalCoprocessor { /* ... */ }
pub struct RemoteCoprocessor { /* ... */ }
```

### 3. Docker/Container-based Solution
Run each component in separate containers:

```yaml
# docker-compose.yml
version: '3.8'
services:
  solana-node:
    image: solana:2.0
    environment:
      - RUST_LOG=info
    
  valence-coprocessor:
    build:
      context: ./valence-coprocessor
      dockerfile: Dockerfile.edition2024
    environment:
      - RUST_EDITION=2024
    
  valence-app:
    build: .
    depends_on:
      - solana-node
      - valence-coprocessor
    environment:
      - SOLANA_RPC_URL=http://solana-node:8899
      - COPROCESSOR_URL=http://valence-coprocessor:8080
```

### 4. Fork and Patch Strategy
Fork buf-fs and downgrade it to Edition 2021:

```toml
# In workspace Cargo.toml
[patch.crates-io]
buf-fs = { git = "https://github.com/your-org/buf-fs-compat", branch = "edition-2021" }
```

### 5. Feature Flag Strategy
Make coprocessor optional at runtime:

```rust
pub enum ExecutionMode {
    SolanaOnly,
    CoprocessorEnabled,
}

impl ValenceClient {
    pub fn new(mode: ExecutionMode) -> Self {
        match mode {
            ExecutionMode::SolanaOnly => Self::without_coprocessor(),
            ExecutionMode::CoprocessorEnabled => Self::with_coprocessor(),
        }
    }
}
```

## Recommendation

The best approach is to **replace buf-fs with a simple implementation** because:

1. **Minimal changes required** - Only affects valence-coprocessor internals
2. **No runtime overhead** - Runs in the same process
3. **Full compatibility** - Works with all Rust editions
4. **Maintainable** - Simple code that we control
5. **No external dependencies** - Reduces supply chain risks

## Implementation Steps

1. Submit PR to valence-coprocessor-domain-prover with filesystem compatibility module
2. Add feature flags to make buf-fs optional
3. Update valence-domain-clients to use the compatibility feature by default
4. Test both configurations (with and without buf-fs)
5. Document the compatibility requirements

This strategy ensures both Solana and valence-coprocessor can work together seamlessly without edition conflicts.