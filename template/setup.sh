#!/bin/bash
# Setup script for Valence template

echo "Setting up Valence template project..."

# Copy all example files
echo "Copying example configuration files..."
cp Cargo.toml.example Cargo.toml
cp Anchor.toml.example Anchor.toml
cp shard/Cargo.toml.example shard/Cargo.toml
cp functions/Cargo.toml.example functions/Cargo.toml
cp functions/hello_world/Cargo.toml.example functions/hello_world/Cargo.toml
cp functions/math_ops/Cargo.toml.example functions/math_ops/Cargo.toml
cp tests/Cargo.toml.example tests/Cargo.toml

echo "Configuration files copied"

# Generate keypairs
echo ""
echo "Generating program keypairs..."
mkdir -p target/deploy

# Generate shard keypair
solana-keygen new -o target/deploy/template_shard-keypair.json --no-bip39-passphrase -s

# Generate function keypairs
solana-keygen new -o target/deploy/hello_world_function-keypair.json --no-bip39-passphrase -s
solana-keygen new -o target/deploy/math_ops_function-keypair.json --no-bip39-passphrase -s

# Get program IDs
SHARD_ID=$(solana address -k target/deploy/template_shard-keypair.json)
HELLO_ID=$(solana address -k target/deploy/hello_world_function-keypair.json)
MATH_ID=$(solana address -k target/deploy/math_ops_function-keypair.json)

echo ""
echo "Generated Program IDs:"
echo " Shard: $SHARD_ID"
echo " Hello World: $HELLO_ID"
echo " Math Ops: $MATH_ID"

# Update program IDs in source files
echo ""
echo "Updating program IDs in source files..."

# Update shard ID
sed -i.bak "s/11111111111111111111111111111111/$SHARD_ID/g" shard/src/lib.rs
sed -i.bak "s/template_shard = \"11111111111111111111111111111111\"/template_shard = \"$SHARD_ID\"/g" Anchor.toml

# Update function IDs
sed -i.bak "s/11111111111111111111111111111111/$HELLO_ID/g" functions/hello_world/src/lib.rs
sed -i.bak "s/hello_world_function = \"11111111111111111111111111111111\"/hello_world_function = \"$HELLO_ID\"/g" Anchor.toml

sed -i.bak "s/22222222222222222222222222222222/$MATH_ID/g" functions/math_ops/src/lib.rs
sed -i.bak "s/math_ops_function = \"22222222222222222222222222222222\"/math_ops_function = \"$MATH_ID\"/g" Anchor.toml

# Clean up backup files
rm -f shard/src/lib.rs.bak functions/hello_world/src/lib.rs.bak functions/math_ops/src/lib.rs.bak Anchor.toml.bak

echo "Program IDs updated"

echo "
✅ Setup complete!

Next steps:
1. Start a local validator: 'solana-test-validator'
2. Build your programs:
   - For all programs: 'anchor build'
   - For individual programs: 'cd shard && cargo build-sbf'
3. Deploy to localnet: 'anchor deploy'
4. Initialize your shard: 'anchor run initialize'

Note: If 'anchor build' fails, you can build programs individually with cargo build-sbf"