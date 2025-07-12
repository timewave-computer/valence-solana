# Test-related commands
{
  pkgs,
  inputs',
  ...
}: {
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
      export PATH="${inputs'.zero-nix.packages.nightly-rust}/bin:${inputs'.zero-nix.packages.solana-tools}/bin:$PATH"
      export CARGO="${inputs'.zero-nix.packages.nightly-rust}/bin/cargo"
      export RUSTC="${inputs'.zero-nix.packages.nightly-rust}/bin/rustc"
      
      # Run anchor test
      ${inputs'.zero-nix.packages.solana-tools}/bin/anchor test
      
      echo "Tests completed!"
    ''}/bin/valence-test";
  };

  # Run comprehensive e2e tests
  e2e-test = {
    type = "app";
    program = "${pkgs.writeShellScriptBin "valence-e2e-test" ''
      set -e
      
      echo "=== Running Valence Solana E2E Tests ==="
      echo ""
      
      # Set up environment
      export PATH="${inputs'.zero-nix.packages.solana-tools}/bin:${inputs'.zero-nix.packages.solana-node}/bin:${inputs'.zero-nix.packages.nightly-rust}/bin:$PATH"
      export RUST_BACKTRACE=1
      export RUST_LOG=info
      export MACOSX_DEPLOYMENT_TARGET=11.0
      export PROTOC=${pkgs.protobuf}/bin/protoc
      
      # Run the e2e test script
      ./tests/e2e/run_e2e_test.sh
      
      echo ""
      echo "=== E2E Tests Completed ==="
    ''}/bin/valence-e2e-test";
  };
}