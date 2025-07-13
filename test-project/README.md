# test-project

A Valence shard project with an echo function.

## Structure

- `functions/echo.rs` - Echo function implementation
- `src/lib.rs` - Shard program with session management
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
./target/release/test-project_client
```
