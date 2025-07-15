# Development shell configuration
{
  pkgs,
  inputs',
  ...
}: let
  inherit (pkgs) lib stdenv;
in {
  packages = with pkgs; [
    openssl
    pkg-config
    protobuf  # For off-chain builds
    inputs'.crate2nix.packages.default  # For generating Cargo.nix
    jq  # For JSON parsing in scripts
  ] ++ lib.optionals stdenv.isDarwin [
    libiconv  # Required for macOS builds
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
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
      package = inputs'.zero-nix.packages.solana-tools;
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
  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    {
      name = "LIBRARY_PATH";
      value = "${pkgs.libiconv}/lib";
    }
    {
      name = "LDFLAGS";
      value = "-L${pkgs.libiconv}/lib";
    }
  ];
  
  devshell.startup.setup-solana = {
    deps = [];
    text = ''
      echo "Valence Solana Development Environment"
      echo "======================================"
      echo ""
      echo "Available tools:"
      echo "  - solana: Solana CLI and validator"
      echo "  - anchor: Anchor framework for Solana development"
      echo "  - crate2nix: Generate Cargo.nix for Nix builds"
      echo ""
      echo "Build commands:"
      echo "  - nix run .#build           - Build everything"
      echo "  - nix run .#build-onchain   - Build on-chain programs only"
      echo "  - nix run .#build-offchain  - Build client libraries only"
      echo ""
      echo "Test commands:"
      echo "  - nix run .#test            - Run all tests"
      echo "  - nix run .#e2e-test        - Run end-to-end tests"
      echo ""
      echo "Local development:"
      echo "  - nix run .#valence-local   - Launch complete local environment"
      echo "                                (validator + deploy + services)"
      echo ""
      echo "Template commands:"
      echo "  - nix run .#valence-new <name>    - Create new project from template"
      echo "  - nix run .#valence-template-*    - Template project commands"
      echo ""
      echo "crate2nix commands:"
      echo "  - nix run .#generate-cargo-nix    - Generate Cargo.nix for fast builds"
      echo "  - nix run .#regenerate-cargo-nix  - Regenerate Cargo.nix"
      echo ""
      echo "Note: For Rust development requiring Edition 2024, use the build"
      echo "      commands above which include nightly Rust automatically."
      echo ""
    '';
  };
}