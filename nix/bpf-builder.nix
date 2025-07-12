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
        
        # Use the cargo build-sbf approach from zero.nix (bypassing platform tools installation)
        echo "Building BPF program..."
        ${if cargoToml != null then ''
          cargo build --release --target sbf-solana-solana --manifest-path ${cargoToml}
          # Copy built artifacts
          find . -name "*.so" -path "*/target/sbf-solana-solana/release/*" -exec cp {} $out/deploy/ \;
        '' else ''
          cargo build --release --target sbf-solana-solana
          # Copy built artifacts
          find . -name "*.so" -path "*/target/sbf-solana-solana/release/*" -exec cp {} $out/deploy/ \;
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
    shard = buildBPFProgram {
      name = "valence-shard";
      inherit src;
      cargoToml = "programs/shard/Cargo.toml";
    };
    
    gateway = buildBPFProgram {
      name = "valence-gateway";
      inherit src;
      cargoToml = "programs/gateway/Cargo.toml";
    };
    
    registry = buildBPFProgram {
      name = "valence-registry";
      inherit src;
      cargoToml = "programs/registry/Cargo.toml";
    };
    
    verifier = buildBPFProgram {
      name = "valence-verifier";
      inherit src;
      cargoToml = "programs/verifier/Cargo.toml";
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
      echo -e "''${YELLOW}Building valence-shard...''${NC}"
      nix build .#valence-shard --out-link ./target/nix-shard
      
      echo -e "''${YELLOW}Building valence-gateway...''${NC}"
      nix build .#valence-gateway --out-link ./target/nix-gateway
      
      echo -e "''${YELLOW}Building valence-registry...''${NC}"
      nix build .#valence-registry --out-link ./target/nix-registry
      
      echo -e "''${YELLOW}Building valence-verifier...''${NC}"
      nix build .#valence-verifier --out-link ./target/nix-verifier
      
      echo ""
      echo -e "''${GREEN}=== BPF Programs Built Successfully ===''${NC}"
      echo ""
      echo "Built programs available in:"
      echo "  - ./target/nix-shard/deploy/"
      echo "  - ./target/nix-gateway/deploy/"
      echo "  - ./target/nix-registry/deploy/"
      echo "  - ./target/nix-verifier/deploy/"
      echo ""
      echo "To deploy programs:"
      echo "  solana program deploy ./target/nix-shard/deploy/<program>.so"
    ''}/bin/valence-build-bpf-programs";
  };
  
  # Test BPF builder app
  test-bpf-builder = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-test-bpf-builder" ''
      set -e
      
      echo "=== Testing BPF Builder ==="
      echo ""
      
      # Test building the valence programs using the BPF builder
      echo "Testing BPF builder with Valence programs..."
      nix build .#valence-shard --out-link ./target/test-shard-bpf
      nix build .#valence-gateway --out-link ./target/test-gateway-bpf
      
      echo ""
      echo "✓ BPF builder test completed successfully"
      echo "Built programs available in:"
      echo "  - ./target/test-shard-bpf/deploy/"
      echo "  - ./target/test-gateway-bpf/deploy/"
      
      if [ -d "./target/test-shard-bpf/deploy" ]; then
        echo ""
        echo "Shard programs:"
        ls -la ./target/test-shard-bpf/deploy/ || echo "No files found"
      fi
      
      if [ -d "./target/test-gateway-bpf/deploy" ]; then
        echo ""
        echo "Gateway programs:"
        ls -la ./target/test-gateway-bpf/deploy/ || echo "No files found"
      fi
    ''}/bin/valence-test-bpf-builder";
  };
} 