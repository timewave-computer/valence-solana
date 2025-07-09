{
  description = "Valence Protocol for Solana";

  nixConfig.extra-experimental-features = "nix-command flakes";
  nixConfig.extra-substituters = "https://timewave.cachix.org";
  nixConfig.extra-trusted-public-keys = ''
    timewave.cachix.org-1:nu3Uqsm3sikI9xFK3Mt4AD4Q6z+j6eS9+kND1vtznq4=
  '';

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    zero-nix.url = "git+file:///Users/hxrts/projects/timewave/valence-solana/submodules/zero.nix";
  };

  outputs = {
    self,
    flake-parts,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devshell.flakeModule
      ];

      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        pkgs,
        inputs',
        ...
      }: let
        # Use crate2nix from nixpkgs for faster Rust builds
        inherit (pkgs) crate2nix;
        
        # Generate Cargo.nix for off-chain workspace (excluding on-chain programs)
        offchainCargoNix = if builtins.pathExists ./Cargo.nix 
          then crate2nix.appliedCargoNix {
            name = "valence-offchain";
            src = ./.;
            cargoNix = ./Cargo.nix;
          }
          else null;
        
        # Create derivations for off-chain components (only if Cargo.nix exists)
        offchainCrates = if offchainCargoNix != null then offchainCargoNix.workspaceMembers else {};
        
        # Fast build environment with nightly rust and proper caching
        rustEnv = {
          RUST_BACKTRACE = "1";
          MACOSX_DEPLOYMENT_TARGET = "11.0";
          SOURCE_DATE_EPOCH = "1686858254";
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        };
      in {
        devshells.default = {pkgs, ...}: {
          packages = with pkgs; [
            openssl
            pkg-config
            protobuf  # For off-chain builds
            crate2nix  # For generating Cargo.nix
          ];
          
          commands = [
            # Separate Solana node and dev tools
            {
              name = "solana";
              package = inputs'.zero-nix.packages.solana-node;
              help = "Solana CLI and node tools";
            }
            {
              name = "anchor";
              package = inputs'.zero-nix.packages.solana-dev-tools;
              help = "Anchor and SBF development tools";
            }
            # Remove nightly-rust from commands to avoid collision
            {package = inputs'.zero-nix.packages.setup-solana;}
          ];
          
          env = [
            {
              name = "PKG_CONFIG_PATH";
              value = "${pkgs.openssl.dev}/lib/pkgconfig";
            }
            {
              name = "OPENSSL_DIR";
              value = "${pkgs.openssl.dev}";
            }
            {
              name = "OPENSSL_LIB_DIR";
              value = "${pkgs.openssl.out}/lib";
            }
            {
              name = "OPENSSL_INCLUDE_DIR";
              value = "${pkgs.openssl.dev}/include";
            }
            {
              name = "MACOSX_DEPLOYMENT_TARGET";
              value = "11.0";
            }
            {
              name = "SOURCE_DATE_EPOCH";
              value = "1686858254";
            }
          ];
          
          devshell.startup.setup-solana = {
            deps = [];
            text = ''
              echo "Valence Solana Development Environment"
              echo "====================================="
              echo ""
              echo "Available packages from zero.nix:"
              echo "  - solana-node: Solana CLI and validator"
              echo "  - solana-dev-tools: Anchor, cargo-build-sbf"
              echo "  - nightly-rust: Nightly Rust with Edition 2024 support"
              echo ""
              echo "Build commands:"
              echo "  - nix run .#build-onchain   - Build on-chain programs"
              echo "  - nix run .#build-offchain  - Build client libraries"
              echo "  - nix run .#build           - Build everything"
              echo ""
              echo "The client libraries from valence-domain-clients are"
              echo "available for off-chain builds with full coprocessor support."
              echo ""
              echo "Nightly Rust (Edition 2024) available at: ${inputs'.zero-nix.packages.nightly-rust}/bin"
              echo ""
            '';
          };
        };

        # Packages built with crate2nix for faster incremental builds
        packages = {
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
            ${pkgs.crate2nix}/bin/crate2nix generate \
              --output ./Cargo.nix
            
            echo ""
            echo "✓ Cargo.nix generated successfully!"
            echo ""
            echo "This file contains Nix expressions for all Rust dependencies."
            echo "Commit Cargo.nix to version control for reproducible builds."
            echo ""
            echo "Usage:"
            echo "  - Run this command whenever you modify Cargo.toml or add/remove dependencies"
            echo "  - The generated Cargo.nix enables fast incremental builds with Nix caching"
            echo "  - Each dependency becomes a separate Nix derivation for maximum caching efficiency"
          '';
          
          # Off-chain packages built with crate2nix (only if Cargo.nix exists)
        } // (if offchainCargoNix != null then {
          valence-sdk = offchainCrates.valence-sdk.build;
          session-builder = offchainCrates.session-builder.build;
          valence-tests = offchainCrates.valence-tests.build;
          
          # All off-chain components in one derivation
          offchain-all = pkgs.symlinkJoin {
            name = "valence-offchain-all";
            paths = [
              offchainCrates.valence-sdk.build
              offchainCrates.session-builder.build
            ];
            meta = {
              description = "All Valence off-chain components built with crate2nix";
            };
          };
        } else {
          # Fallback message when Cargo.nix doesn't exist
          missing-cargo-nix = pkgs.writeTextFile {
            name = "missing-cargo-nix";
            text = ''
              Cargo.nix not found. Run 'nix run .#generate-cargo-nix' to create it.
            '';
          };
        });

        # Build apps with proper separation
        apps = {
          # Build only on-chain programs (still uses cargo build-sbf)
          build-onchain = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-build-onchain" ''
              set -e
              
              # Set up environment for SBF builds
              export RUST_BACKTRACE=1
              export MACOSX_DEPLOYMENT_TARGET=11.0
              export PATH="${inputs'.zero-nix.packages.solana-dev-tools}/bin:$PATH"
              
              # Run the build script
              ./scripts/build-onchain.sh
            ''}/bin/valence-build-onchain";
          };
          
          # Build only off-chain components using crate2nix for speed
          build-offchain = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-build-offchain" ''
              set -e
              
              echo "=== Building Off-Chain Components with crate2nix ==="
              echo "This builds client libraries, SDKs, and services using Nix for fast incremental builds"
              echo ""
              
              # Check if Cargo.nix exists
              if [ ! -f Cargo.nix ]; then
                echo "Cargo.nix not found. Generating it first..."
                nix run .#generate-cargo-nix
                echo ""
              fi
              
              # Build using crate2nix derivations for maximum speed
              echo "Building valence-sdk..."
              nix build .#valence-sdk
              
              echo "Building session-builder..."
              nix build .#session-builder
              
              echo "✓ Off-chain components built successfully with crate2nix!"
              echo ""
              echo "Built artifacts available in:"
              echo "  - result: symlink to all built components"
              echo "  - ./result/bin: CLI tools and executables"
              echo "  - ./result/lib: Libraries"
              echo ""
              echo "Note: crate2nix provides incremental builds - unchanged dependencies won't rebuild!"
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
              ./scripts/build-onchain.sh
              echo "✓ On-chain programs built successfully"
              echo ""
              
              echo "Step 2: Building off-chain components with crate2nix..."
              
              # Check if Cargo.nix exists
              if [ ! -f Cargo.nix ]; then
                echo "Cargo.nix not found. Generating it first..."
                nix run .#generate-cargo-nix
                echo ""
              fi
              
              nix build .#offchain-all
              echo "✓ Off-chain components built successfully with crate2nix!"
              echo ""
              
              echo "=== Build Complete ==="
              echo "On-chain artifacts: ./target/deploy/"
              echo "Off-chain artifacts: ./result/"
              echo ""
              echo "Next steps:"
              echo "1. Deploy programs using: solana program deploy target/deploy/<program>.so"
              echo "2. Use CLI tools from: ./result/bin/"
            ''}/bin/valence-build";
          };
          
          # Regenerate Cargo.nix when dependencies change
          regenerate-cargo-nix = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "regenerate-cargo-nix" ''
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
              echo "Generating Cargo.nix from workspace..."
              ${pkgs.crate2nix}/bin/crate2nix generate \
                --output ./Cargo.nix
              
              echo ""
              echo "✓ Cargo.nix generated successfully!"
              echo ""
              echo "This file contains Nix expressions for all Rust dependencies."
              echo "Commit Cargo.nix to version control for reproducible builds."
              echo ""
              echo "Usage:"
              echo "  - Run this command whenever you modify Cargo.toml or add/remove dependencies"
              echo "  - The generated Cargo.nix enables fast incremental builds with Nix caching"
              echo "  - Each dependency becomes a separate Nix derivation for maximum caching efficiency"
            ''}/bin/regenerate-cargo-nix";
          };
          
          # Run local Solana node (without dev tools)
          node = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-node" ''
              set -e
              
              echo "Starting local Solana test validator..."
              echo "This uses only the solana-node package (no dev tools)"
              
              # Use only the node package
              ${inputs'.zero-nix.packages.solana-node}/bin/solana-test-validator
            ''}/bin/valence-node";
          };
          
          test = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-test" ''
              set -e
              echo "Running Valence Solana tests..."
              
              # Set up environment with nightly rust for Edition 2024 support
              export RUST_BACKTRACE=1
              export MACOSX_DEPLOYMENT_TARGET=11.0
              export SOURCE_DATE_EPOCH=$(date +%s)
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              # Use nightly rust for Edition 2024 support
              export PATH="${inputs'.zero-nix.packages.nightly-rust}/bin:${inputs'.zero-nix.packages.solana-dev-tools}/bin:$PATH"
              export CARGO="${inputs'.zero-nix.packages.nightly-rust}/bin/cargo"
              export RUSTC="${inputs'.zero-nix.packages.nightly-rust}/bin/rustc"
              
              # Run anchor test
              ${inputs'.zero-nix.packages.solana-dev-tools}/bin/anchor test
              
              echo "Tests completed!"
            ''}/bin/valence-test";
          };
        };
      };
    };
}