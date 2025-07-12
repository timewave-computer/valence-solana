#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Setting up Valence Lifecycle Manager${NC}"

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! command -v psql &> /dev/null; then
    echo -e "${YELLOW}Warning: PostgreSQL client not found. Database setup may fail.${NC}"
fi

# Create directories
echo "Creating directories..."
mkdir -p wallet
mkdir -p logs
mkdir -p data

# Build the service
echo "Building service..."
cargo build --release

# Setup database
if [ -n "$DATABASE_URL" ]; then
    echo "Running database migrations..."
    cargo install sqlx-cli --no-default-features --features postgres || true
    sqlx migrate run
else
    echo -e "${YELLOW}DATABASE_URL not set, skipping migrations${NC}"
fi

# Generate example configuration
cat > .env.example << EOF
# Solana Configuration
RPC_URL=http://localhost:8899
WS_URL=ws://localhost:8900
WALLET_PATH=./wallet/id.json
SHARD_PROGRAM_ID=YourShardProgramIdHere

# Database Configuration
DATABASE_URL=postgres://localhost/valence_lifecycle

# Service Configuration
API_PORT=8080
POLL_INTERVAL=5
MAX_ACCOUNTS_PER_SESSION=10
CONSUMPTION_TIMEOUT=300
AUTO_PROGRESS=false

# Logging
RUST_LOG=info
EOF

echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Copy .env.example to .env and configure"
echo "2. Copy your Solana wallet to ./wallet/id.json"
echo "3. Run database migrations: sqlx migrate run"
echo "4. Start the service: cargo run"
echo ""
echo "For production deployment:"
echo "- Use systemd service file in systemd/"