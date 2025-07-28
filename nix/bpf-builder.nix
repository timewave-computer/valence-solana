# BPF Builder for Solana programs
{
  pkgs,
  inputs',
  ...
}: rec {
  # BPF Builder function for Solana programs using zero.nix tools
  buildBPFProgram = { name, src, cargoToml ? null, extraBuildInputs ? [] }:
    pkgs.stdenv.mkDerivation {
      pname = "${name}-bpf";
      version = "0.1.0";
      
      inherit src;
      
      nativeBuildInputs = with pkgs; [
        inputs'.zero-nix.packages.solana-tools  # This includes solana-node, anchor, and nightly-rust
        pkg-config
        openssl.dev
      ] ++ extraBuildInputs;
      
      buildInputs = with pkgs; [
        openssl
      ] ++ lib.optionals pkgs.stdenv.isDarwin [
        darwin.apple_sdk.frameworks.Security
        darwin.apple_sdk.frameworks.SystemConfiguration
      ];
      
      # Set up Solana BPF build environment using zero.nix patterns
      RUST_BACKTRACE = "1";
      SOLANA_BPF_OUT_DIR = "$out/deploy";
      MACOSX_DEPLOYMENT_TARGET = "11.0";
      SOURCE_DATE_EPOCH = "1686858254";
      PROTOC = "${pkgs.protobuf}/bin/protoc";
      
      # Use platform tools from zero.nix
      PLATFORM_TOOLS_DIR = "${inputs'.zero-nix.packages.solana-node}/platform-tools";
      SBF_SDK_PATH = "${inputs'.zero-nix.packages.solana-node}/platform-tools";
      
      buildPhase = ''
        export PATH="${inputs'.zero-nix.packages.solana-tools}/bin:$PATH"
        export HOME=$TMPDIR
        
        # Set up platform tools environment (from zero.nix pattern)
        export PLATFORM_TOOLS_DIR="${inputs'.zero-nix.packages.solana-node}/platform-tools"
        export SBF_SDK_PATH="${inputs'.zero-nix.packages.solana-node}/platform-tools"
        export PATH="${inputs'.zero-nix.packages.solana-node}/platform-tools/rust/bin:${inputs'.zero-nix.packages.solana-node}/platform-tools/llvm/bin:$PATH"
        
        # Set up cargo and rustup cache directories (from zero.nix pattern)
        export CARGO_HOME="$TMPDIR/.cargo"
        export RUSTUP_HOME="$TMPDIR/.rustup"
        mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
        
        # Create output directory
        mkdir -p $out/deploy
        
        # Debug: Check what's available
        echo "Available cargo commands:"
        ls -la ${inputs'.zero-nix.packages.solana-tools}/bin/ | grep cargo || true
        
        # Build from workspace root to ensure all dependencies are available
        echo "Building BPF program..."
        ${if cargoToml != null then ''
          # Build specific program using workspace
          cargo build-sbf --manifest-path ${cargoToml}
          # Copy built artifacts from workspace target/deploy
          if [ -d "target/deploy" ]; then
            cp target/deploy/*.so $out/deploy/ || true
          fi
        '' else ''
          cargo build-sbf
          # Copy built artifacts
          if [ -d "target/deploy" ]; then
            cp target/deploy/*.so $out/deploy/ || true
          fi
        ''}
        
        # Verify we built something
        if [ -z "$(ls -A $out/deploy)" ]; then
          echo "No .so files found, checking other possible locations..."
          find . -name "*.so" -type f
        fi
      '';
      
      installPhase = ''
        # Programs are already in $out/deploy from buildPhase
        # Just ensure they exist
        if [ ! -d "$out/deploy" ] || [ -z "$(ls -A $out/deploy)" ]; then
          echo "Error: No BPF programs were built"
          exit 1
        fi
        
        # List built programs
        echo "Built BPF programs:"
        ls -la $out/deploy/
      '';
      
      meta = with pkgs.lib; {
        description = "Solana BPF program: ${name}";
        platforms = platforms.all;
        maintainers = [ ];
      };
    };
  
  # Helper function to build all Valence programs
  buildValencePrograms = src: {
    core = buildBPFProgram {
      name = "valence-kernel";
      inherit src;
      cargoToml = "programs/valence-kernel/Cargo.toml";
    };
  };
  
  # App to build all Valence programs using Nix
  build-bpf-programs = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-build-bpf-programs" ''
      set -e
      
      echo "=== Building Valence BPF Programs with Nix ==="
      echo "This builds all Valence programs using the Nix BPF builder"
      echo ""
      
      # Colors for output
      GREEN='\033[0;32m'
      YELLOW='\033[1;33m'
      RED='\033[0;31m'
      NC='\033[0m' # No Color
      
      # Build programs using nix build
      echo -e "''${YELLOW}Building valence-kernel...''${NC}"
      nix build .#valence-kernel --out-link ./target/nix-core
      
      echo ""
      echo -e "''${GREEN}=== BPF Programs Built Successfully ===''${NC}"
      echo ""
      echo "Built programs available in:"
      echo "  - ./target/nix-core/deploy/"
      echo ""
      echo "To deploy programs:"
      echo "  solana program deploy ./target/nix-core/deploy/<program>.so"
    ''}/bin/valence-build-bpf-programs";
  };
  
  # Test BPF builder app
  test-bpf-builder = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-test-bpf-builder" ''
      set -e
      
      echo "=== Testing BPF Builder ==="
      echo ""
      
      # Test building the Valence programs using the BPF builder
      echo "Testing BPF builder with Valence programs..."
      nix build .#valence-kernel --out-link ./target/test-core-bpf
      
      echo ""
      echo "BPF builder test completed successfully"
      echo "Built programs available in:"
      echo "  - ./target/test-core-bpf/deploy/"
      
      if [ -d "./target/test-core-bpf/deploy" ]; then
        echo ""
        echo "Core programs:"
        ls -la ./target/test-core-bpf/deploy/ || echo "No files found"
      fi
    ''}/bin/valence-test-bpf-builder";
  };
} 