# Package definitions
{
  pkgs,
  inputs',
  ...
}: let
  crate2nix = inputs'.crate2nix.packages.default;
in {
  # Default package - build everything
  default = pkgs.writeShellScriptBin "valence-build-all" ''
    set -e
    
    echo "=== Building Complete Valence Solana Project ==="
    echo ""
    
    echo "Step 1: Building on-chain programs..."
    nix run .#build-onchain
    echo "On-chain programs built successfully"
    echo ""
    
    echo "Step 2: Building off-chain components..."
    nix run .#build-offchain
    echo "Off-chain components built successfully"
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
  '';

  # Generate initial Cargo.nix if it doesn't exist
  generate-cargo-nix = pkgs.writeShellScriptBin "generate-cargo-nix" ''
    set -e
    echo "Generating Cargo.nix for faster Nix builds..."
    echo ""
    
    if [ ! -f Cargo.toml ]; then
      echo "Error: No Cargo.toml found in current directory"
      exit 1
    fi
    
    # Set up nightly rust environment for Edition 2024 support
    export PATH="${inputs'.zero-nix.packages.nightly-rust}/bin:$PATH"
    export CARGO="${inputs'.zero-nix.packages.nightly-rust}/bin/cargo"
    export RUSTC="${inputs'.zero-nix.packages.nightly-rust}/bin/rustc"
    
    # Set up environment variables for macOS
    export MACOSX_DEPLOYMENT_TARGET=11.0
    export SOURCE_DATE_EPOCH=1686858254
    export RUST_BACKTRACE=1
    
    echo "Using nightly Rust: $(cargo --version)"
    echo "Generating Cargo.nix from workspace..."
    ${crate2nix}/bin/crate2nix generate \
      --output ./Cargo.nix
    
    echo ""
    echo "Cargo.nix generated successfully!"
    echo ""
    echo "This file contains Nix expressions for all Rust dependencies."
    echo "Commit Cargo.nix to version control for reproducible builds."
    echo ""
    echo "Usage:"
    echo "  - Run this command whenever you modify Cargo.toml or add/remove dependencies"
    echo "  - The generated Cargo.nix enables fast incremental builds with Nix caching"
    echo "  - Each dependency becomes a separate Nix derivation for maximum caching efficiency"
    echo ""
    echo "Next steps:"
    echo "  1. After generating Cargo.nix, you can use 'nix build .#build-offchain' for fast builds"
    echo "  2. The generated file contains packages that can be imported and used in Nix expressions"
  '';
  
  # Regenerate Cargo.nix when dependencies change
  regenerate-cargo-nix = pkgs.writeShellScriptBin "regenerate-cargo-nix" ''
    set -e
    echo "Regenerating Cargo.nix for faster nix builds..."
    echo ""
    
    if [ ! -f Cargo.toml ]; then
      echo "Error: No Cargo.toml found in current directory"
      exit 1
    fi
    
    # Set up nightly rust environment for Edition 2024 support
    export PATH="${inputs'.zero-nix.packages.nightly-rust}/bin:$PATH"
    export CARGO="${inputs'.zero-nix.packages.nightly-rust}/bin/cargo"
    export RUSTC="${inputs'.zero-nix.packages.nightly-rust}/bin/rustc"
    
    # Set up environment variables for macOS
    export MACOSX_DEPLOYMENT_TARGET=11.0
    export SOURCE_DATE_EPOCH=1686858254
    export RUST_BACKTRACE=1
    
    echo "Using nightly Rust: $(cargo --version)"
    echo "Regenerating Cargo.nix from workspace..."
    ${crate2nix}/bin/crate2nix generate \
      --output ./Cargo.nix
    
    echo ""
    echo "✓ Cargo.nix regenerated successfully!"
    echo ""
    echo "This file contains Nix expressions for all Rust dependencies."
    echo "Commit Cargo.nix to version control for reproducible builds."
    echo ""
    echo "Usage:"
    echo "  - Run this command whenever you modify Cargo.toml or add/remove dependencies"
    echo "  - The generated Cargo.nix enables fast incremental builds with Nix caching"
    echo "  - Each dependency becomes a separate Nix derivation for maximum caching efficiency"
  '';
}