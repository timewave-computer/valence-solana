# Shard Program

The shard is your main on-chain program that manages function registration and execution in the Valence Protocol.

## Overview

This shard program provides:
- **Initialization**: Set up the shard with an authority
- **Function Registration**: Register functions with their content hashes
- **Function Execution**: Execute registered functions through Cross-Program Invocation (CPI)

## Setup Instructions

### 1. Generate Program Keypair

First, generate a keypair for your shard program:

```bash
solana-keygen new -o target/deploy/template_shard-keypair.json
```

Get the program ID:
```bash
solana address -k target/deploy/template_shard-keypair.json
```

### 2. Update Program ID

Replace the placeholder ID in `src/lib.rs`:
```rust
declare_id!("YOUR_PROGRAM_ID_HERE");
```

Also update it in `../Anchor.toml`:
```toml
[programs.localnet]
template_shard = "YOUR_PROGRAM_ID_HERE"
```

### 3. Build the Program

From the template root directory:
```bash
anchor build -p template-shard
```

Or using cargo directly:
```bash
cargo build-sbf
```

### 4. Deploy to Localnet

Make sure you have a local validator running:
```bash
solana-test-validator
```

Deploy the program:
```bash
anchor deploy -p template-shard
```

Or manually:
```bash
solana program deploy target/deploy/template_shard.so
```

### 5. Initialize the Shard

After deployment, initialize the shard with your authority:

Using Anchor client:
```typescript
await program.methods
  .initialize()
  .accounts({
    shard: shardPDA,
    authority: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

## Shard Architecture

### Accounts

- **Shard Account**: Main state account storing authority and function count
  - PDA: `["shard"]`
  - Stores: authority pubkey, function count

### Instructions

1. **initialize**
   - Sets up the shard with an authority
   - Can only be called once

2. **register_function**
   - Registers a new function with name and content hash
   - Only callable by the shard authority
   - In production, would create separate function registry entries

3. **execute_function**
   - Executes a registered function through CPI
   - Validates function hash
   - Passes input data to the function

## Extending the Shard

### Adding Capability Checks

```rust
pub struct Shard {
    pub authority: Pubkey,
    pub function_count: u32,
    pub required_capabilities: u64,  // Add capability bitmap
}
```

### Implementing Function Registry

Instead of just counting, store function details:
```rust
#[account]
pub struct FunctionEntry {
    pub name: String,
    pub program_id: Pubkey,
    pub content_hash: [u8; 32],
    pub capabilities_required: u64,
}
```

### Adding Session Management

Integrate session support for stateful operations:
```rust
#[account]
pub struct Session {
    pub owner: Pubkey,
    pub capabilities: u64,
    pub nonce: u64,
    pub expires_at: i64,
}
```

## Security Considerations

1. **Authority Management**: Only the authority can register functions
2. **Function Validation**: Verify function hashes before execution
3. **Input Sanitization**: Validate input data size and format
4. **CPI Security**: Ensure only registered programs can be called

## Integration with Valence

To integrate with the full Valence Protocol:

1. **Use Valence Registry**: Replace local registration with registry CPI
2. **Implement Capabilities**: Add bitmap-based permission system
3. **Add Session Support**: Enable stateful function execution
4. **Content Addressing**: Implement proper content hash verification

## Testing

See the `../tests/` directory for integration test examples.