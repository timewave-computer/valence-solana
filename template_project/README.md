# template_project

A Valence shard project template demonstrating the Session V2 API.

## Structure

- `functions/echo.rs` - Echo function implementation  
- `src/lib.rs` - Shard program using Session V2 API
- `src/client.rs` - Client demonstrating Session V2 patterns
- `run.sh` - Complete deployment and execution script

## Session V2 Features Demonstrated

- **Direct session creation** with capability specification
- **Bitmap-based capabilities** for O(1) permission checks
- **Single operation execution** with `execute_on_session`
- **Bundle execution** with `execute_bundle_v2`
- **Clean error handling** for capability violations

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
- Initialize the shard with Session V2 support
- Register the echo function
- Create a session using Session V2 API with capabilities
- Execute echo operations directly on the session
- Demonstrate bitmap capability checking

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
./target/release/template_project_client
```
