# IDL generation environment using rust-overlay for nightly rust
{
  pkgs,
  inputs',
  lib ? pkgs.lib,
  stdenv ? pkgs.stdenv,
  ...
}: let
  # Use rust-overlay to get nightly rust toolchain
  rust-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgsWithRust = import pkgs.path { overlays = [ rust-overlay ]; };
  
  # Get nightly rust with required components
  nightlyRust = pkgsWithRust.rust-bin.nightly."2024-12-01".default.override {
    extensions = [ "rust-src" "llvm-tools-preview" ];
  };
in {
  # IDL builder using rust-overlay nightly - accepts crate name as argument
  idl-build = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "idl-build" ''
      set -e
      
      # Get the crate name from command line argument
      CRATE_NAME="''${1:-valence_kernel}"
      
      echo "=== Building IDL for $CRATE_NAME with nightly rust ==="
      echo ""
      
      # Set up environment with nightly rust
      export PATH="${nightlyRust}/bin:${inputs'.zero-nix.packages.solana-tools}/bin:$PATH"
      export RUSTC="${nightlyRust}/bin/rustc"
      export CARGO="${nightlyRust}/bin/cargo"
      
      echo "Using Rust toolchain:"
      rustc --version
      cargo --version
      echo "Building IDL..."
      
      # Create cargo wrapper to handle +nightly syntax
      TEMP_DIR=$(mktemp -d)
      cat > "$TEMP_DIR/cargo" << 'EOF'
#!/bin/bash
# Process arguments to handle +nightly
ARGS=()
for arg in "$@"; do
    if [[ "$arg" == "+nightly" ]]; then
        continue
    fi
    ARGS+=("$arg")
done
EOF
      echo "exec ${nightlyRust}/bin/cargo \"\''${ARGS[@]}\"" >> "$TEMP_DIR/cargo"
      chmod +x "$TEMP_DIR/cargo"
      
      # Put wrapper first in PATH
      export PATH="$TEMP_DIR:$PATH"
      
      # Store original workspace location and change to program directory
      export ORIGINAL_PWD="$(pwd)"
      cd "programs/valence-kernel" || exit 1
      
      # Ensure target/idl directory exists
      mkdir -p target/idl
      
      # Generate IDL using anchor's built-in test
      echo "Running: cargo test __anchor_private_print_idl --features idl-build --lib"
      TEST_OUTPUT=$("${nightlyRust}/bin/cargo" test __anchor_private_print_idl --features idl-build --lib -- --show-output --quiet 2>&1)
      
      # Extract IDL components from test output
      echo "Extracting IDL from test output..."
      
      PROGRAM_IDL=$(echo "$TEST_OUTPUT" | sed -n '/--- IDL begin program ---/,/--- IDL end program ---/p' | sed '/--- IDL/d')
      ADDRESS=$(echo "$TEST_OUTPUT" | sed -n '/--- IDL begin address ---/,/--- IDL end address ---/p' | sed '/--- IDL/d' | tr -d '"' | xargs)
      ERRORS=$(echo "$TEST_OUTPUT" | sed -n '/--- IDL begin errors ---/,/--- IDL end errors ---/p' | sed '/--- IDL/d')
      
      # Combine into complete IDL
      if [ -n "$PROGRAM_IDL" ] && [ -n "$ADDRESS" ]; then
        # Update the address in the program IDL
        COMPLETE_IDL=$(echo "$PROGRAM_IDL" | ${pkgs.jq}/bin/jq --arg addr "$ADDRESS" '.address = $addr')
        
        # Add errors if they exist
        if [ -n "$ERRORS" ]; then
          COMPLETE_IDL=$(echo "$COMPLETE_IDL" | ${pkgs.jq}/bin/jq --argjson errors "$ERRORS" '.errors = $errors')
        fi
        
        # Save to workspace
        mkdir -p "$ORIGINAL_PWD/target/idl"
        echo "$COMPLETE_IDL" > "$ORIGINAL_PWD/target/idl/$CRATE_NAME.json"
        echo "IDL saved to: $ORIGINAL_PWD/target/idl/$CRATE_NAME.json"
        
        # Show preview
        echo ""
        echo "=== IDL Generated Successfully ==="
        echo "Instructions:"
        echo "$COMPLETE_IDL" | ${pkgs.jq}/bin/jq -r '.instructions[].name'
      else
        echo "Failed to extract IDL from test output"
        echo "Test output:"
        echo "$TEST_OUTPUT"
        exit 1
      fi
      
      # Cleanup
      rm -rf "$TEMP_DIR"
    ''}/bin/idl-build";
  };
}