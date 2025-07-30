# Package definitions
{
  pkgs,
  inputs',
  ...
}: let
  crate2nix = inputs'.crate2nix.packages.default;
  bpfBuilder = import ./bpf-builder.nix {inherit pkgs inputs';};
in {
  # Default package - build everything
  default = pkgs.writeShellScriptBin "valence-build-all" ''
    set -e
    
    echo "=== Building Valence Solana Project ==="
    echo ""
    
    # Set up environment
    export PATH="${inputs'.zero-nix.packages.solana-tools}/bin:$PATH"
    export RUST_BACKTRACE=1
    export MACOSX_DEPLOYMENT_TARGET=11.0
    
    echo "Building Valence programs..."
    if (cd programs/valence-kernel && cargo build-sbf) && (cd programs/valence-functions && cargo build-sbf); then
      echo ""
      echo "=== Build Complete ==="
      echo "Built artifacts available in: ./target/deploy/"
      echo ""
      echo "To deploy programs:"
      echo "  solana program deploy target/deploy/<program>.so"
    else
      echo "Build failed!"
      exit 1
    fi
  '';

  # Generate or update Cargo.nix for optimized Nix builds
  generate-cargo-nix = pkgs.writeShellScriptBin "generate-cargo-nix" ''
    set -e
    
    if [ -f Cargo.nix ]; then
      echo "Updating existing Cargo.nix for faster Nix builds..."
    else
      echo "Generating Cargo.nix for faster Nix builds..."
    fi
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
    echo "Processing Cargo workspace..."
    ${crate2nix}/bin/crate2nix generate \
      --output ./Cargo.nix
    
    echo ""
    echo "Cargo.nix updated successfully!"
    echo ""
    echo "This file contains Nix expressions for all Rust dependencies."
    echo "Commit Cargo.nix to version control for reproducible builds."
    echo ""
    echo "Usage:"
    echo "  - Run this command whenever you modify Cargo.toml or add/remove dependencies"
    echo "  - The generated Cargo.nix enables fast incremental builds with Nix caching"
    echo "  - Each dependency becomes a separate Nix derivation for maximum caching efficiency"
  '';
  
  # BPF program packages
  valence-kernel = bpfBuilder.buildBPFProgram {
    name = "valence-kernel";
    src = ./..;
    cargoToml = "programs/valence-kernel/Cargo.toml";
  };
  
  valence-functions = bpfBuilder.buildBPFProgram {
    name = "valence-functions";
    src = ./..;
    cargoToml = "programs/valence-functions/Cargo.toml";
  };
  
  # Build all BPF programs
  bpf-programs = pkgs.symlinkJoin {
    name = "valence-bpf-programs";
    paths = [
      (bpfBuilder.buildBPFProgram {
        name = "valence-kernel";
        src = ./..;
        cargoToml = "programs/valence-kernel/Cargo.toml";
      })
      (bpfBuilder.buildBPFProgram {
        name = "valence-functions";
        src = ./..;
        cargoToml = "programs/valence-functions/Cargo.toml";
      })
    ];
  };
}