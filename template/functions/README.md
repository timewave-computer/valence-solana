# Functions Directory

This directory contains function implementations that can be executed through the Valence shard.

## Structure

Each function is its own independent Anchor program:
```
functions/
├── hello_world/          # Simple greeting function
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── math_ops/             # Math operations function
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── README.md
```

## Example Functions

### hello_world/
A simple greeting function that demonstrates:
- Basic input/output handling with Anchor
- Using Solana sysvars (Clock)
- Standard Anchor program structure
- Processing compute units

### math_ops/
A math operations function that shows:
- Multiple operation types (add, subtract, multiply, divide)
- Error handling with custom error types
- Input validation
- Safe arithmetic operations

## Creating New Functions

To add a new function to your shard:

1. **Create a new directory** for your function:
   ```bash
   mkdir functions/my_function
   mkdir functions/my_function/src
   ```

2. **Set up Cargo.toml** (copy from an example and modify):
   ```toml
   [package]
   name = "my-function"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   anchor-lang = "0.31.1"
   ```

3. **Implement your function** in `src/lib.rs`:
   - Use `declare_id!` with a unique program ID
   - Define input/output structs with Borsh serialization
   - Implement the main program logic
   - Add proper error handling

4. **Build your function**:
   ```bash
   cd functions/my_function
   anchor build
   ```

5. **Deploy and register** your function with the shard

## Function Requirements

All functions must:
- Be valid Anchor programs
- Have unique program IDs
- Define clear input/output types
- Handle errors appropriately
- Not exceed compute unit limits

## Best Practices

- **Single Responsibility**: Each function should do one thing well
- **Clear I/O**: Use descriptive names for input/output structs
- **Error Handling**: Define custom errors for different failure cases
- **Documentation**: Document capability requirements and behavior
- **Testing**: Include unit tests for serialization and logic
- **Efficiency**: Optimize for low compute unit usage
- **Security**: Validate all inputs and handle edge cases