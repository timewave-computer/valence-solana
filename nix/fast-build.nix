# Fast build apps using generated Cargo.nix
{
  pkgs,
  inputs',
  ...
}: let
  # Import the generated Cargo.nix if it exists
  cargoNix = if builtins.pathExists ../Cargo.nix
    then import ../Cargo.nix {
      inherit pkgs;
      release = true;
    }
    else null;
    
  # Check if crate2nix is available
  hasCrate2nix = cargoNix != null;
  
in {
  # Fast build using crate2nix (if available)
  build-fast = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-build-fast" ''
      set -e
      
      if [ ! -f Cargo.nix ]; then
        echo "=== Cargo.nix not found ==="
        echo "To use fast builds, first generate Cargo.nix:"
        echo "  nix run .#generate-cargo-nix"
        echo ""
        echo "Then you can use:"
        echo "  nix run .#build-fast"
        echo ""
        echo "Falling back to regular build..."
        nix run .#build-offchain
        exit 0
      fi
      
      echo "=== Fast Incremental Build (crate2nix) ==="
      echo "Building off-chain components with maximum caching..."
      echo ""
      
      # Build SDK
      echo "Building valence-sdk..."
      nix build --impure --expr '
        let
          pkgs = import <nixpkgs> {};
          cargoNix = import ./Cargo.nix { inherit pkgs; release = true; };
        in
        cargoNix.workspaceMembers."valence-sdk".build
      ' --out-link result-sdk
      
      # Build session builder
      echo "Building session-builder..."
      nix build --impure --expr '
        let
          pkgs = import <nixpkgs> {};
          cargoNix = import ./Cargo.nix { inherit pkgs; release = true; };
        in
        cargoNix.workspaceMembers."session_builder".build
      ' --out-link result-session-builder
      
      echo ""
      echo "=== Fast Build Complete ==="
      echo "Results:"
      echo "  SDK: ./result-sdk"
      echo "  Session Builder: ./result-session-builder"
      echo ""
      echo "Each dependency was cached individually for maximum build speed!"
      echo "Future builds will be much faster as unchanged dependencies are reused."
    ''}/bin/valence-build-fast";
  };
  
  # Build specific crate using crate2nix
  build-crate = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-build-crate" ''
      set -e
      
      if [ $# -eq 0 ]; then
        echo "Usage: nix run .#build-crate <crate-name>"
        echo ""
        echo "Available crates:"
        if [ -f Cargo.nix ]; then
          echo "  - valence-sdk"
          echo "  - session_builder"
          echo "  - valence-gateway"
          echo "  - valence-registry"
          echo "  - valence-verifier"
          echo "  - valence-shard"
        else
          echo "  Run 'nix run .#generate-cargo-nix' first"
        fi
        exit 1
      fi
      
      CRATE_NAME="$1"
      
      if [ ! -f Cargo.nix ]; then
        echo "Error: Cargo.nix not found"
        echo "Run 'nix run .#generate-cargo-nix' first"
        exit 1
      fi
      
      echo "Building $CRATE_NAME with crate2nix..."
      
      nix build --impure --expr "
        let
          pkgs = import <nixpkgs> {};
          cargoNix = import ./Cargo.nix { inherit pkgs; release = true; };
        in
        cargoNix.workspaceMembers.\"$CRATE_NAME\".build
      " --out-link "result-$CRATE_NAME"
      
      echo "Built $CRATE_NAME -> ./result-$CRATE_NAME"
    ''}/bin/valence-build-crate";
  };
  
  # Show crate2nix status and usage
  crate2nix-status = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-crate2nix-status" ''
      set -e
      
      echo "=== Crate2nix Status ==="
      echo ""
      
      if [ -f Cargo.nix ]; then
        echo "✓ Cargo.nix found"
        echo "  File size: $(wc -l < Cargo.nix) lines"
        echo "  Generated: $(stat -f %Sm Cargo.nix)"
        echo ""
        echo "Available workspace members:"
        grep -A 1 'workspaceMembers = {' Cargo.nix | tail -n +2 | grep '".*"' | head -10 | sed 's/^/  - /'
        echo ""
        echo "Fast build commands available:"
        echo "  nix run .#build-fast           # Build all off-chain components"
        echo "  nix run .#build-crate <name>   # Build specific crate"
        echo ""
        echo "To regenerate:"
        echo "  nix run .#regenerate-cargo-nix"
      else
        echo "✗ Cargo.nix not found"
        echo ""
        echo "To enable fast builds:"
        echo "  nix run .#generate-cargo-nix"
        echo ""
        echo "Benefits of crate2nix:"
        echo "  - Each dependency becomes a separate Nix derivation"
        echo "  - Unchanged dependencies are reused from cache"
        echo "  - Parallel building of independent crates"
        echo "  - Reproducible builds with exact dependency versions"
      fi
    ''}/bin/valence-crate2nix-status";
  };
} 