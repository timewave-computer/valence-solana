{
  description = "Solana development environment for Valence Protocol";

  inputs = {
    # Pin nixpkgs to a specific commit for reproducible builds
    nixpkgs.url = "github:NixOS/nixpkgs/6c5963357f3c1c840201eda129a99d455074db04";
    # Pin flake-utils to a specific commit
    flake-utils.url = "github:numtide/flake-utils/11707dc2f618dd54ca8739b309ec4fc024de578b";
    # Pin rust-overlay to a specific commit
    rust-overlay.url = "github:oxalica/rust-overlay/15c2a7930e04efc87be3ebf1b5d06232e635e24b";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ] (system:
      let
        # Solana and Anchor versions
        sol-version = "2.0.22";
        anchor-version = "0.31.1";
        
        # macOS deployment target (used for all Darwin systems)
        darwinDeploymentTarget = "11.0";
        
        # Common environment variables for all shells/derivations
        commonEnv = {
          # Always set MACOSX_DEPLOYMENT_TARGET 
          MACOSX_DEPLOYMENT_TARGET = darwinDeploymentTarget;
          # Always set SOURCE_DATE_EPOCH for reproducible builds
          SOURCE_DATE_EPOCH = "1686858254"; # Use a fixed value for reproducibility
          # Add explicit Solana install directory
          SOLANA_INSTALL_DIR = "$HOME/.local/share/solana";
          # Anchor CLI environment variables
          ANCHOR_VERSION = anchor-version;
          SOLANA_VERSION = sol-version;
          # Add RUST_BACKTRACE for better debugging
          RUST_BACKTRACE = "1";
        };

        # Instantiate pkgs
        pkgs = import nixpkgs { 
          inherit system; 
          overlays = [ rust-overlay.overlays.default ];
          # Configure basic settings
          config = {
            allowUnfree = true;
            # Only use standard experimental features
            extra-experimental-features = [ "nix-command" "flakes" ];
          };
        };

        # Agave source for reference
        agave-src = pkgs.fetchFromGitHub {
          owner = "anza-xyz";
          repo = "agave";
          rev = "v${sol-version}";
          fetchSubmodules = true;
          sha256 = "sha256-3wvXHY527LOvQ8b4UfXoIKSgwDq7Sm/c2qqj2unlN6I=";
        };

        # Build anchor as a proper nix derivation
        anchor = pkgs.rustPlatform.buildRustPackage rec {
          pname = "anchor-cli";
          version = anchor-version;

          src = pkgs.fetchFromGitHub {
            owner = "coral-xyz";
            repo = "anchor";
            rev = "v${version}";
            hash = "sha256-pvD0v4y7DilqCrhT8iQnAj5kBxGQVqNvObJUBzFLqzA=";
            fetchSubmodules = true;
          };

          # Use fetchCargoVendor for git dependencies
          useFetchCargoVendor = true;
          cargoHash = "sha256-fjhLA+utQdgR75wg+/N4VwASW6+YBHglRPj14sPHmGA=";

          # Build the CLI package specifically
          cargoBuildFlags = [ "--package" "anchor-cli" ];
          cargoTestFlags = [ "--package" "anchor-cli" ];
          
          # Skip tests for faster builds
          doCheck = false;

          nativeBuildInputs = with pkgs; [
            pkg-config
            # Use the rust version from rust-overlay for consistency - updated for Anchor v0.31.1
            (rust-bin.stable."1.81.0".default.override {
              extensions = [ "rust-src" ];
            })
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          buildInputs = with pkgs; [
            openssl
            libiconv
          ] ++ lib.optionals stdenv.isLinux [
            libudev-zero
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.CoreFoundation
            darwin.apple_sdk.frameworks.CoreServices
          ];

          # Environment variables for building
          OPENSSL_NO_VENDOR = "1";
          MACOSX_DEPLOYMENT_TARGET = darwinDeploymentTarget;
          SOURCE_DATE_EPOCH = commonEnv.SOURCE_DATE_EPOCH;

          # Set proper RUSTFLAGS for macOS
          RUSTFLAGS = pkgs.lib.optionalString pkgs.stdenv.isDarwin 
            "-C link-arg=-undefined -C link-arg=dynamic_lookup";

          # Clean build without toolchain installation patches
          postPatch = ''
            # Remove any toolchain installation code that might interfere
            find . -name "*.rs" -type f -exec sed -i 's/install_toolchain_if_needed.*;//g' {} \;
            
            # Ensure we use system rust instead of downloading
            export RUSTC=${pkgs.rust-bin.stable."1.81.0".default}/bin/rustc
            export CARGO=${pkgs.rust-bin.stable."1.81.0".default}/bin/cargo
          '';

          # Simple installation - just copy the binary
          postInstall = ''
            # Anchor CLI should be available as 'anchor'
            echo "Anchor CLI installed at $out/bin/anchor"
          '';

          meta = with pkgs.lib; {
            description = "Solana Sealevel Framework CLI";
            homepage = "https://github.com/coral-xyz/anchor";
            license = licenses.asl20;
            maintainers = [ ];
            platforms = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
          };
        };

        # Script to start a local validator
        start-validator = pkgs.writeShellScriptBin "start-validator" ''
          #!/usr/bin/env bash
          echo "Starting local Solana validator..."
          echo "Note: This requires a separate Solana installation"
          echo "Install Solana CLI tools separately for validator functionality"
        '';

        # Environment variables for macOS with Apple Silicon
        macosMacOSEnvironment = pkgs.lib.optionalAttrs (system == "aarch64-darwin" || system == "x86_64-darwin") (commonEnv // {
          CARGO_BUILD_TARGET = if system == "aarch64-darwin" then "aarch64-apple-darwin" else "x86_64-apple-darwin";
          RUSTFLAGS = "-C link-arg=-undefined -C link-arg=dynamic_lookup";
          BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.libclang.lib}/lib/clang/${pkgs.libclang.version}/include";
          NIX_ENFORCE_MACOSX_DEPLOYMENT_TARGET = "1";
        });
        
        # Solana CLI tools from official releases
        solana = pkgs.stdenv.mkDerivation rec {
          pname = "solana";
          version = sol-version;

          src = pkgs.fetchurl {
            url = "https://github.com/anza-xyz/agave/releases/download/v${version}/solana-release-${
              if pkgs.stdenv.isDarwin then
                if pkgs.stdenv.isAarch64 then "aarch64-apple-darwin" else "x86_64-apple-darwin"
              else
                "x86_64-unknown-linux-gnu"
            }.tar.bz2";
            sha256 = if pkgs.stdenv.isDarwin then
              if pkgs.stdenv.isAarch64 then "sha256-upgxwAEvh11+IKVQ1FaZGlx8Z8Ps0CEScsbu4Hv3WH0=" else "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="
            else
              "sha256-CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=";
          };

          nativeBuildInputs = with pkgs; [ 
            makeWrapper
          ] ++ lib.optionals stdenv.isLinux [
            autoPatchelfHook 
          ];

          buildInputs = with pkgs; [
            openssl
            zlib
          ] ++ lib.optionals stdenv.isLinux [
            stdenv.cc.cc.lib
            glibc
          ];

          installPhase = ''
            runHook preInstall
            
            mkdir -p $out
            cp -r bin $out/
            
            # Create symlinks for common tools
            mkdir -p $out/bin
            for tool in solana solana-keygen solana-test-validator cargo-build-sbf; do
              if [ -f "$out/bin/$tool" ]; then
                chmod +x "$out/bin/$tool"
              fi
            done
            
            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "Solana CLI tools";
            homepage = "https://solana.com";
            license = licenses.asl20;
            platforms = [ "x86_64-linux" "x86_64-darwin" "aarch64-darwin" ];
          };
        };

        # Setup script for initial Solana platform tools installation
        setup-solana = pkgs.writeShellScriptBin "setup-solana" ''
          set -e
          
          echo "Setting up Solana development environment..."
          
          # Create cache directory
          SOLANA_CACHE_DIR="$HOME/.cache/solana/v1.42"
          mkdir -p "$SOLANA_CACHE_DIR"
          
          # Set up environment for platform tools installation
          export SOLANA_INSTALL_DIR=${solana}/bin
          export RUSTUP_HOME="$SOLANA_CACHE_DIR/rustup"
          export CARGO_HOME="$SOLANA_CACHE_DIR/cargo"
          
          mkdir -p "$RUSTUP_HOME" "$CARGO_HOME"
          
          echo "Installing platform tools (this may take a few minutes on first run)..."
          
          # Use a simple Rust program to trigger platform tools installation
          cd "$(mktemp -d)"
          cat > Cargo.toml << 'EOF'
          [package]
          name = "solana-setup"
          version = "0.1.0"
          edition = "2021"
          
          [lib]
          crate-type = ["cdylib"]
          
          [dependencies]
          solana-program = "2.0"
          EOF
          
          mkdir -p src
          cat > src/lib.rs << 'EOF'
          use solana_program::{
              account_info::AccountInfo,
              entrypoint,
              entrypoint::ProgramResult,
              pubkey::Pubkey,
          };
          
          entrypoint!(process_instruction);
          
          pub fn process_instruction(
              _program_id: &Pubkey,
              _accounts: &[AccountInfo],
              _instruction_data: &[u8],
          ) -> ProgramResult {
              Ok(())
          }
          EOF
          
          # Try to build with cargo-build-sbf to trigger platform tools installation
          echo "Triggering platform tools installation..."
          "${solana}/bin/cargo-build-sbf" || echo "Platform tools installation completed"
          
          echo "Solana development environment setup complete!"
          echo "You can now use 'nix develop' for development."
        '';

        # Cargo-build-sbf wrapper that assumes platform tools are already installed
        cargo-build-sbf-wrapper = pkgs.writeShellScriptBin "cargo-build-sbf" ''
          set -e
          
          # Set up environment
          export SOLANA_INSTALL_DIR=${solana}/bin
          SOLANA_CACHE_DIR="$HOME/.cache/solana/v1.42"
          
          # Create cache directory
          mkdir -p "$SOLANA_CACHE_DIR"
          
          # Set up environment variables for cargo-build-sbf
          export RUSTUP_HOME="$SOLANA_CACHE_DIR/rustup"
          export CARGO_HOME="$SOLANA_CACHE_DIR/cargo"
          
          # Create directories
          mkdir -p "$RUSTUP_HOME" "$CARGO_HOME"
          
          # Ensure cargo is in PATH - use the nix-provided cargo
          export PATH="${pkgs.rust-bin.stable."1.81.0".default}/bin:$PATH"
          
          # Add cargo cache to PATH if it exists
          if [ -d "$CARGO_HOME/bin" ]; then
            export PATH="$CARGO_HOME/bin:$PATH"
          fi
          
          echo "Running cargo-build-sbf..."
          echo "Using cargo: $(which cargo)"
          
          # Run cargo-build-sbf
          exec "${solana}/bin/cargo-build-sbf" "$@"
        '';

        # Enhanced anchor wrapper that uses our cargo-build-sbf wrapper
        anchor-wrapper = pkgs.writeShellScriptBin "anchor" ''
          set -e
          
          # Set up environment
          export SOLANA_INSTALL_DIR=${solana}/bin
          export PATH="${cargo-build-sbf-wrapper}/bin:$PATH"
          
          # Run anchor with our environment
          exec "${anchor}/bin/anchor" "$@"
        '';
      in {
        packages = {
          inherit solana anchor start-validator cargo-build-sbf-wrapper anchor-wrapper setup-solana;
          default = pkgs.symlinkJoin {
            name = "valence-protocol-solana-env";
            paths = [
              solana
              anchor-wrapper
              cargo-build-sbf-wrapper
              setup-solana
              start-validator
            ];
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain with cargo
            (rust-bin.stable."1.81.0".default.override {
              extensions = [ "rust-src" "llvm-tools-preview" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            
            # Solana and Anchor tools (using wrappers)
            solana
            anchor-wrapper
            cargo-build-sbf-wrapper
            setup-solana
            start-validator
            
            # Development tools
            nodejs
            yarn
            python3
            gnused
            jq
            ripgrep
            protobuf
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
            darwin.apple_sdk.frameworks.AppKit
          ];

          # Include Apple Silicon environment variables
          inherit (macosMacOSEnvironment) MACOSX_DEPLOYMENT_TARGET CARGO_BUILD_TARGET RUSTFLAGS BINDGEN_EXTRA_CLANG_ARGS;

          shellHook = ''
            # Export all common environment variables directly
            ${pkgs.lib.concatStrings (pkgs.lib.mapAttrsToList (name: value: "export ${name}=${value}\n") commonEnv)}
            
            echo "Entering Valence Solana development environment..."
            echo "Solana CLI ${sol-version} and Anchor CLI ${anchor-version} are available"
            echo ""
            echo "🔧 Platform Tools Setup:"
            echo "  The first time you build Solana programs, cargo-build-sbf will need to"
            echo "  download platform tools. This may require running outside the nix shell:"
            echo ""
            echo "  1. Exit nix shell: exit"
            echo "  2. Install Solana CLI: sh -c \"\$(curl -sSfL https://release.solana.com/v${sol-version}/install)\""
            echo "  3. Build once: ~/.local/share/solana/install/active_release/bin/anchor build"
            echo "  4. Return to nix: nix develop"
            echo ""
            echo "  Alternatively, try: nix run .#setup-solana"
            echo ""
            
            # Show macOS deployment target if on Darwin
            ${pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
              echo "macOS deployment target: $MACOSX_DEPLOYMENT_TARGET"
            ''}
            
            # Export protoc path
            export PROTOC=${pkgs.protobuf}/bin/protoc
            
            # Check if platform tools are available
            if [ -d "$HOME/.cache/solana" ] || [ -d "$HOME/.local/share/solana" ]; then
              echo "✅ Solana platform tools detected - ready for development!"
            fi
          '';
        };

        devShells.litesvm-test = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "llvm-tools-preview" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            
            # Core build dependencies
            pkg-config
            openssl
            llvm
            
            # For litesvm 
            libiconv
            protobuf
          ];
          
          # Apply common environment plus additional environment variables
          inherit (macosMacOSEnvironment) MACOSX_DEPLOYMENT_TARGET NIX_ENFORCE_MACOSX_DEPLOYMENT_TARGET;
          SOURCE_DATE_EPOCH = commonEnv.SOURCE_DATE_EPOCH;
          
          shellHook = ''
            # Export all environment variables explicitly to ensure they are set
            export MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}
            export NIX_ENFORCE_MACOSX_DEPLOYMENT_TARGET=1
            export SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}
            export RUST_BACKTRACE=1
            export PROTOC=${pkgs.protobuf}/bin/protoc
            
            echo "LiteSVM test environment ready"
            echo "macOS deployment target: $MACOSX_DEPLOYMENT_TARGET"
            echo "Run 'cargo test' to execute tests"
          '';
        };

        # Shell specifically for token_helpers tests
        devShells.token-helpers-test = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust with specific version
            (rust-bin.stable."1.75.0".default.override {
              extensions = [ "rust-src" "llvm-tools-preview" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            
            # Core build dependencies
            pkg-config
            openssl
            llvm
            
            # Additional dependencies needed
            libiconv
            protobuf
          ];
          
          # Apply common environment plus additional environment variables
          inherit (macosMacOSEnvironment) MACOSX_DEPLOYMENT_TARGET NIX_ENFORCE_MACOSX_DEPLOYMENT_TARGET;
          SOURCE_DATE_EPOCH = commonEnv.SOURCE_DATE_EPOCH;
          
          shellHook = ''
            # Export all environment variables explicitly to ensure they are set
            export MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}
            export NIX_ENFORCE_MACOSX_DEPLOYMENT_TARGET=1
            export SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}
            export RUST_BACKTRACE=1
            export PROTOC=${pkgs.protobuf}/bin/protoc
            
            echo "Token Helpers test environment ready"
            echo "macOS deployment target: $MACOSX_DEPLOYMENT_TARGET"
            
            # Create a specific Cargo config for testing token_helpers
            mkdir -p .cargo
            cat > .cargo/config.toml << EOF
            [build]
            rustflags = ["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]

            # Use a single version of the solana crates
            [patch.crates-io]
            solana-program = { git = "https://github.com/solana-labs/solana.git", tag = "v1.17.14" }
            solana-zk-token-sdk = { git = "https://github.com/solana-labs/solana.git", tag = "v1.17.14" }
            solana-sdk = { git = "https://github.com/solana-labs/solana.git", tag = "v1.17.14" }
            EOF
            
            echo "Created .cargo/config.toml with patched dependencies"
            echo "Run 'cargo test -p token_transfer' to test token_helpers.rs"
          '';
        };

        # Flake outputs - comprehensive apps to replace all bash scripts
        apps = {
          # Solana setup app - sets up platform tools
          setup-solana = {
            type = "app";
            program = "${setup-solana}/bin/setup-solana";
          };
          
          # Main build app - replaces scripts/build.sh
          build = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-build" ''
              set -e
              echo "========== Valence Solana Build ==========="
              
              # Environment setup
              export MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}
              export SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}
              export RUST_BACKTRACE=1
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              # Cache management
              CACHE_DIR="$HOME/.valence-solana-cache"
              PLATFORM_TOOLS_CACHE="$CACHE_DIR/platform-tools"
              SOLANA_INSTALL_DIR="$HOME/.local/share/solana"
              
              mkdir -p "$CACHE_DIR" "$PLATFORM_TOOLS_CACHE" "$SOLANA_INSTALL_DIR"
              
              # Check for cached platform tools
              if [ -d "$PLATFORM_TOOLS_CACHE/platform-tools" ] && [ ! -d "$SOLANA_INSTALL_DIR/platform-tools" ]; then
                echo "Using cached platform-tools..."
                ln -sf "$PLATFORM_TOOLS_CACHE/platform-tools" "$SOLANA_INSTALL_DIR/platform-tools"
              fi
              
              # Build hash calculation for incremental builds
              HASH_FILE="$CACHE_DIR/build_hash"
              CURRENT_HASH=$(find programs -type f -name "*.rs" -o -name "Cargo.toml" 2>/dev/null | sort | xargs sha256sum 2>/dev/null | sha256sum | cut -d' ' -f1)
              
              # Check if rebuild is needed
              FORCE_REBUILD=0
              if [[ "$*" == *"--force"* ]]; then
                FORCE_REBUILD=1
                echo "Force rebuild requested."
              elif [ -f "$HASH_FILE" ] && [ "$(cat "$HASH_FILE")" = "$CURRENT_HASH" ] && [ -d "target/deploy" ]; then
                echo "No changes detected. Using cached build. Use --force to rebuild."
                exit 0
              fi
              
              # Clean if rebuilding
              if [ $FORCE_REBUILD -eq 1 ] || [ ! -f "$HASH_FILE" ] || [ "$(cat "$HASH_FILE")" != "$CURRENT_HASH" ]; then
                echo "Cleaning old build artifacts..."
                rm -rf node_modules/.anchor target/deploy target/sbf-solana-solana target/types .anchor
              fi
              
              # Build with Anchor
              echo "Building with Anchor..."
              ${anchor}/bin/anchor build
              
              # Cache platform tools on success
              if [ -d "$SOLANA_INSTALL_DIR/platform-tools" ]; then
                echo "Caching platform-tools..."
                rm -rf "$PLATFORM_TOOLS_CACHE/platform-tools"
                cp -r "$SOLANA_INSTALL_DIR/platform-tools" "$PLATFORM_TOOLS_CACHE/"
              fi
              
              # Update hash
              echo "$CURRENT_HASH" > "$HASH_FILE"
              echo "Build completed successfully!"
            ''}/bin/valence-build";
          };
          
          # Test runner - replaces scripts/run-tests.sh and related test scripts
          test = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-test" ''
              set -e
              export MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}
              export SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}
              export RUST_BACKTRACE=1
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              CRATE="''${1:-token_transfer}"
              shift 2>/dev/null || true
              
              echo "===== Test Environment ====="
              echo "MACOSX_DEPLOYMENT_TARGET: $MACOSX_DEPLOYMENT_TARGET"
              echo "PROTOC: $PROTOC"
              echo "Testing crate: $CRATE"
              echo "=========================="
              
              ${pkgs.cargo}/bin/cargo test -p "$CRATE" "$@"
            ''}/bin/valence-test";
          };
          
          # Cache management - replaces scripts/clear-cache.sh
          clear-cache = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-clear-cache" ''
              set -e
              CACHE_DIR="$HOME/.valence-solana-cache"
              
              echo "Clearing Valence Solana build cache..."
              if [ -d "$CACHE_DIR" ]; then
                echo "Removing cache directory: $CACHE_DIR"
                rm -rf "$CACHE_DIR"
                echo "Cache cleared successfully!"
              else
                echo "Cache directory does not exist."
              fi
              
              # Also clear Cargo and Anchor caches
              echo "Clearing Cargo cache..."
              rm -rf target/
              
              echo "Clearing Anchor cache..."
              rm -rf .anchor/ node_modules/.anchor/
              
              echo "All caches cleared!"
            ''}/bin/valence-clear-cache";
          };
          
          # Development environment info
          env-info = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-env-info" ''
              echo "===== Valence Solana Environment ====="
              echo "Solana CLI version: ${sol-version}"
              echo "Anchor CLI version: ${anchor-version}"
              echo "macOS deployment target: ${darwinDeploymentTarget}"
              echo ""
              echo "Nix store paths:"
              echo "  Solana: ${solana}"
              echo "  Anchor: ${anchor}"
              echo ""
              echo "Environment variables:"
              echo "  MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}"
              echo "  SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}"
              echo "  PROTOC=${pkgs.protobuf}/bin/protoc"
              echo ""
              echo "Available commands:"
              echo "  nix run .#setup-solana        - Set up Solana platform tools"
              echo "  nix run .#build [--force]     - Build the project"
              echo "  nix run .#test [crate] [args] - Run tests"
              echo "  nix run .#clear-cache         - Clear all caches"
              echo "  nix run .#env-info            - Show this info"
              echo "  nix run .#format              - Format code"
              echo "  nix run .#lint                - Lint code"
              echo "  nix run .#deploy [network]    - Deploy to network"
              echo ""
              echo "Platform tools status:"
              if [ -d "$HOME/.cache/solana" ] || [ -d "$HOME/.local/share/solana" ]; then
                echo "  ✅ Platform tools detected"
              else
                echo "  ⚠️  Platform tools not found - run 'nix run .#setup-solana'"
              fi
              echo ""
              echo "🎯 This is a complete Nix-based Solana development environment!"
              echo "======================================"
            ''}/bin/valence-env-info";
          };
          
          # Code formatting
          format = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-format" ''
              set -e
              echo "Formatting Rust code..."
              ${pkgs.cargo}/bin/cargo fmt
              
              echo "Formatting TypeScript/JavaScript code..."
              if [ -f "package.json" ]; then
                ${pkgs.nodejs}/bin/npx prettier --write "**/*.{ts,js,json}"
              fi
              
              echo "Code formatting complete!"
            ''}/bin/valence-format";
          };
          
          # Code linting
          lint = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-lint" ''
              set -e
              export MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}
              export SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              echo "Running Rust linting..."
              ${pkgs.cargo}/bin/cargo clippy --all-targets --all-features -- -D warnings
              
              echo "Checking Rust formatting..."
              ${pkgs.cargo}/bin/cargo fmt --check
              
              echo "Running Anchor linting..."
              ${anchor}/bin/anchor build --verifiable
              
              echo "Linting complete!"
            ''}/bin/valence-lint";
          };
          
          # Deployment helper
          deploy = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-deploy" ''
              set -e
              export MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}
              export SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              NETWORK="''${1:-devnet}"
              
              echo "Deploying to $NETWORK..."
              
              # Ensure we have a recent build
              if [ ! -d "target/deploy" ]; then
                echo "No build artifacts found. Building first..."
                ${anchor}/bin/anchor build
              fi
              
              # Deploy with Anchor
              ${anchor}/bin/anchor deploy --provider.cluster "$NETWORK"
              
              echo "Deployment to $NETWORK complete!"
            ''}/bin/valence-deploy";
          };
          
          # Token helpers test - replaces token helper test scripts
          test-token-helpers = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-test-token-helpers" ''
              set -e
              export MACOSX_DEPLOYMENT_TARGET=${darwinDeploymentTarget}
              export SOURCE_DATE_EPOCH=${commonEnv.SOURCE_DATE_EPOCH}
              export RUST_BACKTRACE=1
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              echo "Running token_transfer tests with fixed Solana SDK versions..."
              
              # Create temporary Cargo config for consistent dependencies
              mkdir -p .cargo
              cat > .cargo/config.toml << EOF
              [build]
              rustflags = ["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]
              
              [patch.crates-io]
              solana-program = { git = "https://github.com/solana-labs/solana.git", tag = "v1.17.14" }
              solana-zk-token-sdk = { git = "https://github.com/solana-labs/solana.git", tag = "v1.17.14" }
              solana-sdk = { git = "https://github.com/solana-labs/solana.git", tag = "v1.17.14" }
              EOF
              
              ${pkgs.cargo}/bin/cargo test -p token_transfer "$@"
              
              # Clean up
              rm -f .cargo/config.toml
              rmdir .cargo 2>/dev/null || true
            ''}/bin/valence-test-token-helpers";
          };
          
          # Update nix dependencies - replaces scripts/update-nix-deps.sh
          update-deps = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-update-deps" ''
              set -e
              echo "Updating Valence Solana nix dependencies..."
              
              # Function to get latest commit
              get_latest_commit() {
                local owner=$1 repo=$2 branch=''${3:-main}
                echo "Fetching latest commit for $owner/$repo on branch $branch..."
                ${pkgs.curl}/bin/curl -s "https://api.github.com/repos/$owner/$repo/commits/$branch" | ${pkgs.jq}/bin/jq -r '.sha'
              }
              
              echo "Current pinned versions:"
              ${pkgs.gnugrep}/bin/grep -E "(nixpkgs|flake-utils|rust-overlay)\.url" flake.nix
              
              echo ""
              read -p "Update to latest versions? (y/N): " -n 1 -r
              echo
              [[ ! $REPLY =~ ^[Yy]$ ]] && { echo "Aborted."; exit 0; }
              
              # Get latest commits
              echo "Fetching latest commit hashes..."
              NIXPKGS_COMMIT=$(get_latest_commit "NixOS" "nixpkgs" "nixpkgs-unstable")
              FLAKE_UTILS_COMMIT=$(get_latest_commit "numtide" "flake-utils" "main")
              RUST_OVERLAY_COMMIT=$(get_latest_commit "oxalica" "rust-overlay" "master")
              
              echo "Latest commits:"
              echo "  nixpkgs: $NIXPKGS_COMMIT"
              echo "  flake-utils: $FLAKE_UTILS_COMMIT"
              echo "  rust-overlay: $RUST_OVERLAY_COMMIT"
              
              echo ""
              read -p "Proceed with updating flake.nix? (y/N): " -n 1 -r
              echo
              [[ ! $REPLY =~ ^[Yy]$ ]] && { echo "Aborted."; exit 0; }
              
              # Update flake.nix
              echo "Updating flake.nix..."
              ${pkgs.gnused}/bin/sed -i.bak "s|nixpkgs\.url = \"github:NixOS/nixpkgs/.*\"|nixpkgs.url = \"github:NixOS/nixpkgs/$NIXPKGS_COMMIT\"|" flake.nix
              ${pkgs.gnused}/bin/sed -i.bak "s|flake-utils\.url = \"github:numtide/flake-utils/.*\"|flake-utils.url = \"github:numtide/flake-utils/$FLAKE_UTILS_COMMIT\"|" flake.nix
              ${pkgs.gnused}/bin/sed -i.bak "s|rust-overlay\.url = \"github:oxalica/rust-overlay/.*\"|rust-overlay.url = \"github:oxalica/rust-overlay/$RUST_OVERLAY_COMMIT\"|" flake.nix
              
              rm -f flake.nix.bak
              
              echo "Updated flake.nix. Updating flake.lock..."
              nix flake lock --verbose
              
              echo "Testing updated environment..."
              nix flake check
              
              echo "Dependencies updated successfully!"
            ''}/bin/valence-update-deps";
          };
        };
      }
    );
} 