# Solana + Anchor Development Environment

This repository provides a minimal setup for Solana blockchain development with the Anchor framework using Nix.

## Requirements

- [Nix package manager](https://nixos.org/download.html) with flakes enabled
- macOS with Apple Silicon (aarch64-darwin) - for other platforms, modify the `system` in flake.nix

## Structure

- `flake.nix` - Defines the Nix development environment with Solana and Anchor tools
- `solana-demo.sh` - Script that demonstrates the full workflow (validator, wallet, build, deploy)
- `hello_world/` - Example Anchor program

## Getting Started

1. Enter the Nix development environment:

```bash
nix develop --impure
```

2. Run the demo script:

```bash
./solana-demo.sh
```

This will:
- Start a local Solana validator
- Create and fund a wallet
- Initialize the Hello World program
- Build and deploy the program (requires Solana SDK with BPF support)

## Available Commands

Inside the Nix shell environment:

- `start-solana-node` - Start a local Solana validator
- `setup-solana-local` - Configure and fund a local wallet
- `setup-anchor` - Install Anchor CLI if not already installed

## Additional Setup

To build and deploy Anchor programs, you'll need the full Solana SDK with BPF support.
If the `anchor build` command fails with "no such command: `build-bpf`", install the Solana CLI tools from:
https://docs.solana.com/cli/install-solana-cli-tools

## License

MIT 