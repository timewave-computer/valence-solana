# Valence Solana Project
# Run 'just' to see available commands

# Default command - show help
default:
    @just --list

# Project paths
export PROJECT_ROOT := justfile_directory()

# Build all programs using Nix BPF builder (default)
build:
    @echo "Building programs with Nix BPF builder..."
    nix build .#valence-kernel --out-link target/nix-kernel
    nix build .#valence-functions --out-link target/nix-functions
    @mkdir -p target/deploy
    @cp target/nix-kernel/deploy/*.so target/deploy/ 2>/dev/null || true
    @cp target/nix-functions/deploy/*.so target/deploy/ 2>/dev/null || true
    @echo "Programs built with Nix and copied to target/deploy/"

# Build all programs using cargo (fallback)
build-cargo:
    @echo "Building programs with cargo build-sbf..."
    @mkdir -p target/deploy
    cd programs/valence-kernel && cargo build-sbf
    cd programs/valence-functions && cargo build-sbf
    @echo "All programs built with cargo"

# Run all tests
test:
    cargo test

# Clean all build artifacts
clean:
    cargo clean
    rm -rf target/

# Generate Cargo.nix for fast nix builds
generate-cargo-nix:
    nix run .#generate-cargo-nix

# Regenerate Cargo.nix (clean generation)
regenerate-cargo-nix:
    nix run .#regenerate-cargo-nix

# Launch complete local development environment
local:
    nix run .#valence-local

# Run clippy on all code
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# E2E test commands (delegates to e2e/justfile)
e2e *ARGS:
    cd e2e && just {{ARGS}}

# Quick shortcuts for common e2e commands
e2e-test:
    cd e2e && just test

e2e-test-debug:
    cd e2e && just test-debug

e2e-clean:
    cd e2e && just clean

# Build and view program sizes
sizes: build
    @echo "Program sizes:"
    @ls -lah target/deploy/*.so | awk '{print $9 ": " $5}'

# Run the full CI pipeline locally
ci: fmt-check clippy test e2e-test
    @echo "All CI checks passed!"

# Update all dependencies
update:
    cargo update

# Build documentation
doc:
    cargo doc --no-deps --open

# Install git hooks for development
install-hooks:
    @echo "Installing git hooks..."
    @mkdir -p .git/hooks
    @echo '#!/bin/bash' > .git/hooks/pre-commit
    @echo 'just fmt-check' >> .git/hooks/pre-commit
    @chmod +x .git/hooks/pre-commit
    @echo "Git hooks installed"

# Help with detailed command descriptions
help:
    @echo "Valence Solana Development Commands:"
    @echo ""
    @echo "Building:"
    @echo "  just build              - Build all programs with Nix BPF builder"
    @echo "  just build-cargo        - Build all programs with cargo (fallback)"
    @echo "  just sizes              - Show program sizes after building"
    @echo ""
    @echo "Testing:"
    @echo "  just test               - Run unit tests"
    @echo "  just e2e-test           - Run e2e tests"
    @echo "  just e2e-test-debug     - Run e2e tests with debug output"
    @echo "  just ci                 - Run full CI pipeline"
    @echo ""
    @echo "Code Quality:"
    @echo "  just fmt                - Format code"
    @echo "  just fmt-check          - Check formatting"
    @echo "  just clippy             - Run clippy lints"
    @echo "  just doc                - Build and view documentation"
    @echo ""
    @echo "Development:"
    @echo "  just local              - Launch local devnet with programs"
    @echo "  just e2e <cmd>          - Run any e2e just command"
    @echo "  just clean              - Clean all build artifacts"
    @echo ""
    @echo "Nix:"
    @echo "  just generate-cargo-nix - Generate Cargo.nix"
    @echo "  just regenerate-cargo-nix - Regenerate Cargo.nix from scratch"