# Build-related apps and packages
{
  pkgs,
  inputs',
  ...
}: rec {
  # Build only on-chain programs
  build-onchain = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-build-onchain" ''
      set -e
      
      echo "=== Building On-Chain Programs (Solana) ==="
      echo "This builds only programs that run on the Solana blockchain"
      echo ""
      
      # Set up environment for SBF builds
      export RUST_BACKTRACE=1
      export MACOSX_DEPLOYMENT_TARGET=11.0
      export PATH="${inputs'.zero-nix.packages.solana-tools}/bin:$PATH"
      
      # Colors for output
      GREEN='\033[0;32m'
      YELLOW='\033[1;33m'
      RED='\033[0;31m'
      NC='\033[0m' # No Color
      
      # Build the gateway program
      echo -e "''${YELLOW}Building valence-gateway...''${NC}"
      if cargo build-sbf --manifest-path programs/gateway/Cargo.toml; then
          echo -e "''${GREEN}valence-gateway built successfully''${NC}"
      else
          echo -e "''${RED}Failed to build valence-gateway''${NC}"
          exit 1
      fi
      
      # Build the registry program
      echo -e "''${YELLOW}Building valence-registry...''${NC}"
      if cargo build-sbf --manifest-path programs/registry/Cargo.toml; then
          echo -e "''${GREEN}valence-registry built successfully''${NC}"
      else
          echo -e "''${RED}Failed to build valence-registry''${NC}"
          exit 1
      fi
      
      # Build the verifier program
      echo -e "''${YELLOW}Building valence-verifier...''${NC}"
      if cargo build-sbf --manifest-path programs/verifier/Cargo.toml; then
          echo -e "''${GREEN}valence-verifier built successfully''${NC}"
      else
          echo -e "''${RED}Failed to build valence-verifier''${NC}"
          exit 1
      fi
      
      # Build the shard program
      echo -e "''${YELLOW}Building valence-shard...''${NC}"
      if cargo build-sbf --manifest-path programs/shard/Cargo.toml; then
          echo -e "''${GREEN}valence-shard built successfully''${NC}"
      else
          echo -e "''${RED}Failed to build valence-shard''${NC}"
          exit 1
      fi
      
      echo ""
      echo -e "''${GREEN}=== On-Chain Build Complete ===''${NC}"
      echo ""
      echo "Generated artifacts:"
      find target/deploy -name "*.so" -type f 2>/dev/null | while read -r file; do
          echo "  - $(basename "$file")"
      done || echo "  No .so files found in target/deploy/"
      
      echo ""
      echo "Next steps:"
      echo "1. Deploy programs using: solana program deploy target/deploy/<program>.so"
      echo "2. Build off-chain components using: nix run .#build-offchain"
    ''}/bin/valence-build-onchain";
  };
  
  # Build only off-chain components using standard cargo for now
  build-offchain = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-build-offchain" ''
      set -e
      
      echo "=== Building Off-Chain Components ==="
      echo "This builds client libraries, SDKs, and services using nightly Rust"
      echo ""
      echo "Note: For faster builds, generate Cargo.nix with 'nix run .#generate-cargo-nix'"
      echo ""
      
      # Set up nightly rust environment for Edition 2024 support
      export PATH="${inputs'.zero-nix.packages.nightly-rust}/bin:$PATH"
      export CARGO="${inputs'.zero-nix.packages.nightly-rust}/bin/cargo"
      export RUSTC="${inputs'.zero-nix.packages.nightly-rust}/bin/rustc"
      
      # Set up environment variables for macOS
      export MACOSX_DEPLOYMENT_TARGET=11.0
      export SOURCE_DATE_EPOCH=1686858254
      export RUST_BACKTRACE=1
      export PROTOC=${pkgs.protobuf}/bin/protoc
      export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig
      export OPENSSL_DIR=${pkgs.openssl.dev}
      export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
      export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
      
      echo "Using nightly Rust: $(cargo --version)"
      echo ""
      
      # Build only off-chain crates (exclude programs/ directory)
      echo "Building valence-sdk..."
      cargo build --release --manifest-path sdk/Cargo.toml
      
      echo "Building lifecycle-manager..."
      cargo build --release --manifest-path services/lifecycle_manager/Cargo.toml
      
      echo "✓ Off-chain components built successfully!"
      echo ""
      echo "Built artifacts available in:"
      echo "  - sdk/target/release/: SDK libraries"
      echo "  - services/session_builder/target/release/: Session builder service"
      echo ""
      echo "To use crate2nix for faster incremental builds:"
      echo "  1. Run 'nix run .#generate-cargo-nix' to generate Cargo.nix"
      echo "  2. Import the generated Cargo.nix in your Nix expressions"
    ''}/bin/valence-build-offchain";
  };
  
  # Build everything (on-chain + off-chain)
  build = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-build" ''
      set -e
      
      echo "=== Building Complete Valence Solana Project ==="
      echo ""
      
      echo "Step 1: Building on-chain programs..."
      nix run .#build-onchain
      echo "✓ On-chain programs built successfully"
      echo ""
      
      echo "Step 2: Building off-chain components..."
      nix run .#build-offchain
      echo "✓ Off-chain components built successfully"
      echo ""
      
      echo "=== Build Complete ==="
      echo "On-chain artifacts: ./target/deploy/"
      echo "Off-chain artifacts: ./sdk/target/release/ and ./services/*/target/release/"
      echo ""
      echo "Next steps:"
      echo "1. Deploy programs using: solana program deploy target/deploy/<program>.so"
      echo "2. Use off-chain libraries from the target directories"
      echo ""
      echo "For faster builds in the future:"
      echo "  - Run 'nix run .#generate-cargo-nix' to set up crate2nix"
      echo "  - This generates Cargo.nix for incremental Nix builds"
    ''}/bin/valence-build";
  };
}