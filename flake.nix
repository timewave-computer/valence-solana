{
  description = "Valence Solana Protocol using zero.nix";

  nixConfig.extra-experimental-features = "nix-command flakes";
  nixConfig.extra-substituters = "https://timewave.cachix.org";
  nixConfig.extra-trusted-public-keys = ''
    timewave.cachix.org-1:nu3Uqsm3sikI9xFK3Mt4AD4Q6z+j6eS9+kND1vtznq4=
  '';

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    zero-nix.url = "path:./zero.nix";
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
      }: {
        devshells.default = {pkgs, ...}: {
          packages = with pkgs; [
            openssl
            pkg-config
          ];
          
          commands = [
            {package = inputs'.zero-nix.packages.solana-tools;}
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
          ];
          
          devshell.startup.setup-solana = {
            deps = [];
            text = ''
              echo "Valence Solana Development Environment (using zero.nix)"
              echo "Available tools:"
              echo "  - solana CLI (v2.0.22)"
              echo "  - anchor CLI (v0.31.1)"
              echo "  - platform tools (v1.48)"
              echo "  - cargo-build-sbf"
              echo ""
              echo "Run 'setup-solana' to initialize the development environment"
              echo ""
              echo "Quick commands:"
              echo "  anchor build        - Build Anchor programs"
              echo "  anchor test         - Run tests"
              echo "  solana --version    - Check Solana CLI version"
              echo ""
            '';
          };
        };

        # Build app using zero.nix tools
        apps = {
          build = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-build" ''
              set -e
              echo "Building Valence Solana programs..."
              
              # Set up environment
              export RUST_BACKTRACE=1
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              # Run anchor build
              ${inputs'.zero-nix.packages.solana-tools}/bin/anchor build
              
              echo "Build completed!"
            ''}/bin/valence-build";
          };
          
          test = {
            type = "app";
            program = "${pkgs.writeShellScriptBin "valence-test" ''
              set -e
              echo "Running Valence Solana tests..."
              
              # Set up environment
              export RUST_BACKTRACE=1
              export PROTOC=${pkgs.protobuf}/bin/protoc
              
              # Run anchor test
              ${inputs'.zero-nix.packages.solana-tools}/bin/anchor test
              
              echo "Tests completed!"
            ''}/bin/valence-test";
          };
        };
      };
    };
}