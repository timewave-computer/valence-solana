# Purpose: Solana development tools including CLI, Anchor, and platform tools for SBF compilation
{ pkgs, lib, inputs, ... }:

let
  # Apply rust-overlay to pkgs for rust-bin support
  rustPkgs = pkgs.appendOverlays [
    inputs.rust-overlay.overlays.rust-overlay
  ];
  
  # Solana and Anchor versions - pinned for stability
  sol-version = "2.0.22";
  anchor-version = "0.31.1";
  platform-tools-version = "1.48";
  
  # macOS deployment target (used for all Darwin systems)
  darwinDeploymentTarget = "11.0";
  
  # Common environment variables for Solana development
  commonEnv = {
    SOURCE_DATE_EPOCH = "1686858254"; # Fixed value for reproducible builds
    SOLANA_INSTALL_DIR = "$HOME/.cache/solana";
    ANCHOR_VERSION = anchor-version;
    SOLANA_VERSION = sol-version;
    RUST_BACKTRACE = "1";
  } // lib.optionalAttrs pkgs.stdenv.isDarwin {
    MACOSX_DEPLOYMENT_TARGET = darwinDeploymentTarget;
  };

  # Nightly Rust environment specifically for IDL generation
  nightly-rust = rustPkgs.rust-bin.nightly."2024-12-01".default.override {
    extensions = [ "rust-src" "llvm-tools-preview" ];
  };

  # Import the original solana-tools.nix
  original = import ./solana-tools.nix { inherit pkgs lib inputs; };

  # Enhanced anchor wrapper that properly handles nightly rust
  anchor-wrapper-fixed = pkgs.writeShellScriptBin "anchor" ''
    set -e
    
    # Set up platform tools environment for SBF compilation  
    export PLATFORM_TOOLS_DIR=${original.solana-node}/platform-tools
    export SBF_SDK_PATH=${original.solana-node}/platform-tools
    
    # Set required environment variables
    export SOURCE_DATE_EPOCH="${commonEnv.SOURCE_DATE_EPOCH}" 
    export RUST_BACKTRACE=1
    export PROTOC=${pkgs.protobuf}/bin/protoc
    
    # Platform-specific environment variables
    ${lib.optionalString pkgs.stdenv.isDarwin ''
      export MACOSX_DEPLOYMENT_TARGET="${darwinDeploymentTarget}"
      export CARGO_BUILD_TARGET="${if pkgs.stdenv.isAarch64 then "aarch64" else "x86_64"}-apple-darwin"
      export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"
    ''}
    ${lib.optionalString pkgs.stdenv.isLinux ''
      export CARGO_BUILD_TARGET="${if pkgs.stdenv.isAarch64 then "aarch64" else "x86_64"}-unknown-linux-gnu"
    ''}
    
    # Critical: Set cargo/rustup cache directories to prevent redownloading
    export CARGO_HOME="$HOME/.cache/solana/v${platform-tools-version}/cargo"
    export RUSTUP_HOME="$HOME/.cache/solana/v${platform-tools-version}/rustup"
    
    # Ensure cache directories exist
    mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
    
    # Check if this is for IDL generation
    if [[ "$*" == *"idl"* ]]; then
      # Use nightly rust for IDL generation
      export PATH="${nightly-rust}/bin:${original.solana-node}/bin:$PATH"
      export RUSTC="${nightly-rust}/bin/rustc"
      export CARGO="${nightly-rust}/bin/cargo"
      
      # Create a wrapper script that intercepts cargo +nightly calls
      mkdir -p /tmp/anchor-cargo-wrapper
      cat > /tmp/anchor-cargo-wrapper/cargo <<'EOF'
#!/usr/bin/env bash
# Intercept +nightly directives and use our nightly cargo directly
if [[ "$1" == "+nightly" ]]; then
  shift
  exec "${nightly-rust}/bin/cargo" "$@"
else
  exec "${original.solana-node}/platform-tools/rust/bin/cargo" "$@"
fi
EOF
      chmod +x /tmp/anchor-cargo-wrapper/cargo
      export PATH="/tmp/anchor-cargo-wrapper:$PATH"
    else
      # Use platform tools rust for normal builds
      export PATH="${original.solana-node}/platform-tools/rust/bin:${original.solana-node}/bin:$PATH"
    fi
    
    # Run anchor with platform tools environment
    exec "${original.anchor}/bin/anchor" "$@"
  '';

  # Create a patched anchor that doesn't use +nightly syntax
  anchor-patched = pkgs.stdenv.mkDerivation {
    pname = "anchor-patched";
    version = anchor-version;
    
    src = original.anchor;
    
    installPhase = ''
      mkdir -p $out
      cp -r $src/* $out/
      
      # Patch the anchor binary to remove +nightly usage
      # This is a bit hacky but works for our use case
      if [ -f $out/bin/anchor ]; then
        wrapProgram $out/bin/anchor \
          --set ANCHOR_NO_TOOLCHAIN_OVERRIDE "1"
      fi
    '';
    
    nativeBuildInputs = [ pkgs.makeWrapper ];
  };

in {
  # Export all original packages
  inherit (original) solana-node setup-solana nightly-rust;
  
  # Export the fixed anchor wrapper
  anchor = anchor-wrapper-fixed;
  
  # Combined solana development environment with fixed anchor
  solana-tools = pkgs.symlinkJoin {
    name = "solana-tools";
    paths = [
      original.solana-node
      anchor-wrapper-fixed
      original.setup-solana
      nightly-rust
    ];
    
    postBuild = ''
      # Ensure anchor wrapper takes precedence
      rm -f $out/bin/anchor
      ln -sf ${anchor-wrapper-fixed}/bin/anchor $out/bin/anchor
    '';
  };
}