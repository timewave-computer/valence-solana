# Valence Protocol for Solana

A trust-minimized cross-chain DeFi development environment for the Solana ecosystem. Valence Protocol enables the creation of secure, configurable cross-chain applications with minimal smart contract development and reduced reliance on centralized components.

## Overview

Valence Protocol is a unified framework for building trustless cross-chain DeFi applications, called **Valence Programs**. The protocol allows developers to configure and deploy secure cross-chain solutions through a combination of specialized accounts, libraries, and authorization mechanisms.

Key benefits:
- **Configuration-driven**: Many programs can be built with minimal code
- **Extensible**: Modular design for easy integration of new DeFi components
- **Trust-minimized**: Reduced reliance on trusted third parties
- **Solana-optimized**: Built for Solana's high-performance environment

## Architecture

The Valence Protocol for Solana consists of the following core components:

```
Core Components:
├── Authorization Program - Entry point and permissions manager
├── Processor Program - Execution engine for messages
├── Valence Registry - Library and component registry
├── Account Factory - Efficient account creation service
│
Account Programs:
├── Base Account Program - Token custody and operations
├── Storage Account Program - Key-value data storage
├── Single-Use Account Program - One-time use accounts
│
Library Programs:
├── Token Transfer Library - Token movement operations
├── Vault Deposit Library - Vault interaction operations
└── Other Libraries - Extensible library ecosystem
```

See [Architecture Overview](./docs/valence_solana.md) for details.

## Components

### Core Infrastructure

- **Authorization Program**: Manages permissions and routes messages
- **Processor Program**: Executes operations via priority queues
- **Valence Registry**: Maintains approved libraries and configurations
- **Account Factory**: Efficient account creation and initialization

### Account Types

- **Base Accounts**: Hold tokens and authorize library operations
- **Storage Accounts**: Store data in efficient key-value format
- **Single-Use Accounts**: Enforce one-time use with complete fund withdrawal

### Libraries

- **Token Transfer Library**: Facilitates token transfers with validation
- **Vault Deposit Library**: Manages deposits into various vault protocols
- **Additional Libraries**: Extensible system for custom DeFi operations

## Getting Started

### Nix Development Environment

This project uses Nix to provide a consistent and reproducible development environment. All development work must be performed within this environment to ensure consistent tooling, dependencies, and build processes.

#### Setting up Nix

1. Install Nix following the [official guide](https://nixos.org/download.html)

2. Enable Flakes (if not already enabled):
   
   Add the following to `~/.config/nix/nix.conf` or `/etc/nix/nix.conf`:
   ```
   experimental-features = nix-command flakes
   ```

#### Entering the Development Environment

To enter the development environment with all required tools and dependencies:

```bash
# From the project root
nix develop
```

This will provide a shell with:
- Rust toolchain with Solana dependencies
- Anchor framework
- Solana CLI tools
- All other required dependencies at their pinned versions

#### Working with Nix

Common operations within the Nix environment:

```bash
# Start a local Solana validator
start-validator

# Build all programs
anchor build

# Run tests
anchor test

# ... other operations defined in flake.nix
```

### Installation

1. Clone the repository

```bash
git clone https://github.com/your-org/valence-solana.git
cd valence-solana
```

2. Enter the Nix development environment

```bash
nix develop
```

3. Install dependencies

```bash
yarn install
```

4. Build the programs

```bash
anchor build
```

### Testing

Run the test suite from within the Nix environment:

```bash
anchor test
```

## Development

### Program Development

To create a new library program:

1. Create a new directory under `programs/libraries/`
2. Implement the standard library interface
3. Register your library with the Valence Registry

### Client Integration

To integrate with Valence Protocol in a client application:

1. Import the Valence client SDK
2. Connect to the Authorization Program
3. Create and configure the necessary accounts
4. Execute operations through the Authorization Program

## Security Considerations

- All state transitions are atomic where possible
- Authorization Program enforces strict permissions
- PDAs are used for secure token handling
- Single-Use Accounts ensure complete withdrawals
