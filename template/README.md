# Valence Protocol Template

This template provides a starting point for building with the Valence Protocol on Solana.

## Structure

```
template/
├── shard/                      # Your shard program
│   ├── Cargo.toml.example
│   ├── README.md
│   └── src/
│       └── lib.rs              # Shard implementation
├── functions/                  # Function implementations
│   ├── hello_world/            # Example greeting function
│   │   ├── Cargo.toml.example
│   │   └── src/
│   │       └── lib.rs
│   ├── math_ops/               # Example math operations
│   │   ├── Cargo.toml.example
│   │   └── src/
│   │       └── lib.rs
│   └── README.md
├── tests/                      # Integration tests
│   ├── Cargo.toml.example
│   ├── integration_test.rs
│   ├── shard_test.rs
│   └── README.md
├── .gitignore
├── Anchor.toml.example         # Anchor configuration
├── Cargo.toml.example          # Workspace configuration
├── setup.sh                    # Quick setup script
└── README.md                   # This file
```

## Components

### Shard Program (`shard/`)

The shard program is your main on-chain program that:
- Manages function registration
- Handles function execution through CPI
- Manages permissions and state

See [`shard/README.md`](shard/README.md) for detailed setup and architecture information.

### Functions (`functions/`)

Functions are independent Anchor programs that can be executed through the shard. The template includes:
- **hello_world**: A greeting function demonstrating basic I/O and sysvar access
- **math_ops**: Math operations showing error handling and input validation

Each function is its own deployable program. See [`functions/README.md`](functions/README.md) for how to create new functions.

### Tests (`tests/`)

Contains test examples for:
- Unit tests for individual components
- Integration tests for shard-function interaction
- End-to-end test scenarios

See [`tests/README.md`](tests/README.md) for testing setup and best practices.

## Getting Started

### Prerequisites

Before starting, ensure you have:
- Anchor CLI installed (v0.31.1 or later)
- Solana CLI installed
- A local validator running (`solana-test-validator`)

### Quick Setup (Recommended)

Run the setup script to automatically configure your project:
```bash
./setup.sh
```

This will:
- Copy all example configuration files
- Generate program keypairs
- Update program IDs in source files
- Prepare your project for building

### Manual Setup

1. **Copy example files**:
   ```bash
   cp Cargo.toml.example Cargo.toml
   cp Anchor.toml.example Anchor.toml
   cp shard/Cargo.toml.example shard/Cargo.toml
   cp functions/hello_world/Cargo.toml.example functions/hello_world/Cargo.toml
   cp functions/math_ops/Cargo.toml.example functions/math_ops/Cargo.toml
   cp tests/Cargo.toml.example tests/Cargo.toml
   ```

2. **Generate keypairs and update program IDs**:
   ```bash
   # Generate shard keypair
   solana-keygen new -o target/deploy/template_shard-keypair.json
   
   # Generate function keypairs
   solana-keygen new -o target/deploy/hello_world_function-keypair.json
   solana-keygen new -o target/deploy/math_ops_function-keypair.json
   ```
   
   Update the program IDs in:
   - `shard/src/lib.rs`
   - `functions/hello_world/src/lib.rs`
   - `functions/math_ops/src/lib.rs`
   - `Anchor.toml`

3. **Build all programs**:
   ```bash
   anchor build
   ```

4. **Deploy to localnet**:
   ```bash
   anchor deploy
   ```

5. **Initialize the shard**:
   ```bash
   anchor run initialize
   ```

## Integrating with Valence

To integrate with the full Valence Protocol:

1. **Import Valence SDK**:
   ```toml
   valence-sdk = { version = "0.1.0" }
   ```

2. **Use Valence Registry** for function registration
3. **Implement proper capability checking**
4. **Use session management for stateful operations**

## Creating New Functions

To create a new function:

1. **Create a new directory** in `functions/`:
   ```bash
   mkdir -p functions/my_function/src
   ```

2. **Create Cargo.toml** for your function:
   ```toml
   [package]
   name = "my-function"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   anchor-lang = "0.31.1"

   [lib]
   crate-type = ["cdylib", "lib"]
   ```

3. **Implement your function** in `src/lib.rs`:
   ```rust
   use anchor_lang::prelude::*;

   declare_id!("YOUR_PROGRAM_ID_HERE");

   #[program]
   pub mod my_function {
       use super::*;

       pub fn execute(ctx: Context<Execute>, input: Vec<u8>) -> Result<()> {
           // Your function logic here
           Ok(())
       }
   }

   #[derive(Accounts)]
   pub struct Execute<'info> {
       // Define required accounts
   }
   ```

4. **Build and deploy** your function:
   ```bash
   anchor build
   anchor deploy
   ```

5. **Register** your function with the shard:
   ```bash
   # Get your function's bytecode hash
   anchor idl parse -f functions/my_function/src/lib.rs | sha256sum
   
   # Register via CLI (if available) or client code:
   ```
   ```rust
   // In your client code
   let function_id = /* your deployed function program ID */;
   let bytecode_hash = /* computed hash from above */;
   
   // Call shard's register_function instruction
   shard_program.register_function(
       function_id,
       bytecode_hash,
       capabilities_required, // e.g., READ | WRITE | EXECUTE
   )?;
   ```

## Testing

Run the test suite:
```bash
# Unit tests
cargo test

# Integration tests (requires deployed programs)
anchor test
```

## Next Steps

- Implement proper function registration with content hashes
- Add capability-based access control
- Integrate with Valence Registry for function discovery
- Implement state management for functions
- Add comprehensive error handling
- Deploy to devnet/mainnet

## Resources

- [Valence Protocol Documentation](https://docs.valence.xyz)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)

## License

This template is provided under the same license as the Valence Protocol.