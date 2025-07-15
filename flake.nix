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
    zero-nix.url = "github:timewave-computer/zero.nix/main";
    crate2nix.url = "github:nix-community/crate2nix";
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
      
      # Flake-level outputs
      flake = {
        # Expose builder functions at the flake level
        lib = {
          # Users can access these with: 
          # let valence = inputs.valence-solana; in
          # valence.lib.buildBPFProgram { ... }
          buildBPFProgram = system: let
            pkgs = inputs.nixpkgs.legacyPackages.${system};
            inputs' = {
              zero-nix.packages = inputs.zero-nix.packages.${system};
              crate2nix.packages = inputs.crate2nix.packages.${system};
            };
            bpfBuilderConfig = import ./nix/bpf-builder.nix {inherit pkgs inputs';};
          in bpfBuilderConfig.buildBPFProgram;
        };
      };

      perSystem = {
        pkgs,
        inputs',
        system,
        ...
      }: let
        # Import subflakes
        devshellConfig = import ./nix/devshell.nix {inherit pkgs inputs';};
        buildApps = import ./nix/build.nix {inherit pkgs inputs';};
        crate2nixApps = import ./nix/crate2nix.nix {inherit pkgs inputs';};
        fastBuildApps = import ./nix/fast-build.nix {inherit pkgs inputs';};
        localApps = import ./nix/local.nix {inherit pkgs inputs';};
        testApps = import ./nix/test.nix {inherit pkgs inputs';};
        templateApps = import ./nix/template.nix {inherit pkgs inputs';};
        simpleTemplateApps = import ./nix/template-simple.nix {inherit pkgs inputs';};
        flakeTemplateApps = import ./nix/template-with-flake.nix {inherit pkgs inputs';};
        packagesConfig = import ./nix/packages.nix {inherit pkgs inputs';};
        bpfBuilderConfig = import ./nix/bpf-builder.nix {inherit pkgs inputs';};
        
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
        # Development shell
        devshells.default = devshellConfig;

        # Packages
        packages = packagesConfig // {
          # Re-export packages that were in apps as packages too
          inherit (packagesConfig) default generate-cargo-nix regenerate-cargo-nix;
        } // (let
          # Build all Valence programs and expose them individually
          valencePrograms = bpfBuilderConfig.buildValencePrograms ./.;
        in {
          # Individual programs
          valence-shard = valencePrograms.shard;
          valence-registry = valencePrograms.registry;
          
          # Also expose the full set
          valencePrograms = pkgs.symlinkJoin {
            name = "valence-programs";
            paths = builtins.attrValues valencePrograms;
          };
        });

        # Apps - combine all app definitions
        apps = buildApps // crate2nixApps // fastBuildApps // localApps // testApps // templateApps // simpleTemplateApps // flakeTemplateApps // {
          # Only include actual apps from bpfBuilderConfig, not functions
          inherit (bpfBuilderConfig) build-bpf-programs test-bpf-builder;
        };
      };
    };
}