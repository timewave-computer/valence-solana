# capability_enforcement_test

A Valence e2e test project that tests capability enforcement in the complete architecture.

## Structure

- `src/lib.rs` - Main program entrypoint
- `src/shard.rs` - Shard processor handling capabilities and control flow
- `src/functions/` - Directory containing all registered functions
  - `echo.rs` - Echo function implementation
- `src/client.rs` - Client to interact with the shard
- `run.sh` - Complete deployment and execution script

## Quick Start

1. Make sure you have a local Solana node running:
   ```bash
   nix run ..#valence-local
   ```

2. Run the complete flow:
   ```bash
   ./run.sh
   ```

Note: The project uses `.valence-env` to store deployment information. This file is created automatically during deployment.

This will:
- Build the shard program and client
- Deploy the shard program
- Initialize the shard
- Register the echo function
- Create a session with echo capability
- Execute the echo function via the client

## Manual Steps

### Build
```bash
nix run ..#valence-template-build
```

### Deploy
```bash
nix run ..#valence-template-deploy
```

### Initialize Shard
```bash
nix run ..#valence-template-init
```

### Register Functions
```bash
nix run ..#valence-template-register
```

### Create Session
```bash
nix run ..#valence-template-session
```

### Run Client
```bash
./target/release/capability_enforcement_test_client
```
