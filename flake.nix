{
  description = "Solana development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "aarch64-darwin";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      rustWithComponents = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-analyzer" "clippy" ];
      };

      # Define a simple setup script for Anchor
      setupAnchorScript = pkgs.writeShellScriptBin "setup-anchor" ''
        if ! command -v anchor &> /dev/null; then
          echo "Installing Anchor CLI..."
          cargo install --git https://github.com/coral-xyz/anchor --tag v0.29.0 anchor-cli
        else
          echo "Anchor CLI is already installed"
        fi
      '';
      
      # Script to start a Solana validator
      solanaNodeScript = pkgs.writeShellScriptBin "start-solana-node" ''
        echo "Starting Solana validator..."
        ${pkgs.solana-cli}/bin/solana-test-validator "$@"
      '';
      
      # Script to setup a local Solana wallet and configuration
      setupLocalScript = pkgs.writeShellScriptBin "setup-solana-local" ''
        # Configure Solana to use localhost
        ${pkgs.solana-cli}/bin/solana config set --url http://127.0.0.1:8899
        
        # Create a wallet if it doesn't exist
        if [ ! -f "$HOME/.config/solana/id.json" ]; then
          echo "Creating a new Solana wallet..."
          ${pkgs.solana-cli}/bin/solana-keygen new --no-bip39-passphrase --force
        fi
        
        # Airdrop some SOL
        echo "Airdropping SOL to wallet..."
        ${pkgs.solana-cli}/bin/solana airdrop 100
        ${pkgs.solana-cli}/bin/solana balance
        
        echo "Local Solana configuration completed!"
      '';
      
    in {
      packages.${system} = {
        inherit setupAnchorScript;
        inherit solanaNodeScript;
        inherit setupLocalScript;
        default = pkgs.buildEnv {
          name = "solana-environment";
          paths = [
            setupAnchorScript
            solanaNodeScript
            setupLocalScript
            pkgs.solana-cli
          ];
        };
      };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          # Solana tools
          solana-cli
          setupAnchorScript
          solanaNodeScript
          setupLocalScript
          
          # Rust development
          rustWithComponents
          pkg-config
          
          # Build dependencies
          openssl
          libiconv
        ] ++ lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        shellHook = ''
          export RUST_SRC_PATH="${rustWithComponents}/lib/rustlib/src/rust/library"
          export RUST_BACKTRACE=1
          export PATH=$PATH:$HOME/.cargo/bin
          
          # Set Solana home directories
          export SOLANA_HOME="$HOME/.local/share/solana"
          mkdir -p "$SOLANA_HOME"
          
          # Clean up any test-ledger directories
          find . -type d -name "test-ledger" -exec rm -rf {} + 2>/dev/null || true
          
          # Make sure Solana config directory exists
          mkdir -p "$HOME/.config/solana"
          
          # Set Anchor environment variables 
          export ANCHOR_WALLET="$HOME/.config/solana/id.json"
          
          # Create a wallet if it doesn't exist
          if [ ! -f "$HOME/.config/solana/id.json" ]; then
            echo "Creating a new Solana wallet..."
            ${pkgs.solana-cli}/bin/solana-keygen new --no-bip39-passphrase -o "$HOME/.config/solana/id.json" --silent
          fi
          
          # Configure Solana to use localhost
          ${pkgs.solana-cli}/bin/solana config set --url http://127.0.0.1:8899 > /dev/null
          
          # Setup Anchor CLI if needed
          if ! command -v anchor &> /dev/null; then
            echo "Note: Anchor CLI not found in path. Installing with setup-anchor..."
            setup-anchor
          fi
          
          echo "Solana development environment activated!"
          echo "Commands available:"
          echo "  start-solana-node  - Start a local Solana validator"
          echo "  setup-solana-local - Configure and fund a local wallet"
          echo "  setup-anchor       - Install Anchor CLI if not already installed"
        '';
      };
    };
} 