{
  description = "Valence E2E Test";

  nixConfig.extra-experimental-features = "nix-command flakes";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
    valence-root.url = "../..";
  };

  outputs = {
    self,
    flake-parts,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        pkgs,
        system,
        ...
      }: let
        # Get tools from parent flake
        parentRoot = inputs.valence-root;
        zeroNix = parentRoot.inputs.zero-nix;
        nightly-rust = zeroNix.packages.${system}.nightly-rust;
        solana-tools = zeroNix.packages.${system}.solana-tools;
        solana-node = zeroNix.packages.${system}.solana-node;
        anchor = zeroNix.packages.${system}.anchor or null;
        
        # Import BPF builder from parent flake
        buildBPFProgram = parentRoot.packages.${system}.buildBPFProgram;
        
        # Pre-built test program using the BPF builder
        testProgram = buildBPFProgram {
          name = "capability_enforcement_test";
          src = ./capability_enforcement_test;
          cargoToml = "Cargo.toml";
        };
        
        # Use pre-built valence programs from parent flake
        valencePrograms = parentRoot.packages.${system}.valencePrograms;
        
        # Build off-chain services
        sessionBuilder = pkgs.rustPlatform.buildRustPackage {
          pname = "session-builder";
          version = "0.1.0";
          src = parentRoot;
          cargoToml = parentRoot + "/services/session_builder/Cargo.toml";
          
          buildAndTestSubdir = "services/session_builder";
          
          cargoLock = {
            lockFile = parentRoot + "/Cargo.lock";
            allowBuiltinFetchGit = true;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl.dev
          ];
          
          buildInputs = with pkgs; [
            openssl
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          
          # Use the same rust toolchain as the parent
          RUSTC = "${nightly-rust}/bin/rustc";
          CARGO = "${nightly-rust}/bin/cargo";
          
          meta = {
            description = "Valence Session Builder Service";
          };
        };
        
        lifecycleManager = pkgs.rustPlatform.buildRustPackage {
          pname = "lifecycle-manager";
          version = "0.1.0";
          src = parentRoot;
          cargoToml = parentRoot + "/services/lifecycle_manager/Cargo.toml";
          
          buildAndTestSubdir = "services/lifecycle_manager";
          
          cargoLock = {
            lockFile = parentRoot + "/Cargo.lock";
            allowBuiltinFetchGit = true;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl.dev
            postgresql
          ];
          
          buildInputs = with pkgs; [
            openssl
            postgresql
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          
          # Use the same rust toolchain as the parent
          RUSTC = "${nightly-rust}/bin/rustc";
          CARGO = "${nightly-rust}/bin/cargo";
          
          meta = {
            description = "Valence Lifecycle Manager Service";
          };
        };
        
        # E2E test runner
        testRunner = pkgs.writeShellScriptBin "valence-e2e-test" ''
          #!${pkgs.bash}/bin/bash
          set -e
          
          # Create test directory
          TEST_DIR=$(mktemp -d)
          trap "rm -rf $TEST_DIR" EXIT
          
          # Copy test files
          cp ${./run_e2e_test.sh} "$TEST_DIR/run_e2e_test.sh"
          chmod +x "$TEST_DIR/run_e2e_test.sh"
          
          # Copy template project
          cp -r ${./capability_enforcement_test} "$TEST_DIR/capability_enforcement_test"
          chmod -R u+w "$TEST_DIR/capability_enforcement_test"
          
          # Add test database for lifecycle manager
          export PGDATA="$TEST_DIR/postgres"
          export PGHOST="$TEST_DIR/postgres"
          export PGUSER="valence"
          export PGDATABASE="valence_test"
          export DATABASE_URL="postgresql://valence@localhost/valence_test"
          
          # Make postgresql available for lifecycle manager testing
          export PATH="${pkgs.postgresql}/bin:$PATH"
          
          # Make testing tools available
          export PATH="${pkgs.curl}/bin:${pkgs.jq}/bin:$PATH"
          
          # Set up environment
          export PATH="${solana-node}/bin:${solana-tools}/bin:${nightly-rust}/bin:$PATH"
          ${if anchor != null then ''export PATH="${anchor}/bin:$PATH"'' else ""}
          export PATH="${sessionBuilder}/bin:${lifecycleManager}/bin:$PATH"
          export E2E_TEST_ISOLATED=1
          
          # Make services available to test
          export SESSION_BUILDER_BIN="${sessionBuilder}/bin/session-builder"
          export LIFECYCLE_MANAGER_BIN="${lifecycleManager}/bin/lifecycle-manager"
          
          # Make pre-built BPF programs available
          export TEST_PROGRAM_PATH="${testProgram}/deploy"
          export VALENCE_SHARD_PATH="${valencePrograms.shard}/deploy"
          export VALENCE_GATEWAY_PATH="${valencePrograms.gateway}/deploy"
          export VALENCE_REGISTRY_PATH="${valencePrograms.registry}/deploy"
          export VALENCE_VERIFIER_PATH="${valencePrograms.verifier}/deploy"
          
          # Run the test
          cd "$TEST_DIR"
          ./run_e2e_test.sh "$@"
        '';
        
      in {
        # Default app runs the e2e test
        apps.default = {
          type = "app";
          program = "${testRunner}/bin/valence-e2e-test";
        };
      };
    };
}