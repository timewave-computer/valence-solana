# Scripts Directory - MIGRATION COMPLETE ✅

🎉 **MIGRATION COMPLETE**: All bash scripts have been successfully migrated to self-contained nix apps with **crate2nix** for lightning-fast incremental builds.

## Available Nix Commands

All functionality is now available through nix apps in `flake.nix`. Use these commands:

### Core Development Commands

| Command | Description |
|---------|-------------|
| `nix run .#build` | Build with crate2nix + Anchor (recommended) |
| `nix run .#build-fast` | Fast incremental build (crate2nix only) |
| `nix run .#build-crate [name]` | Build individual crate with crate2nix |
| `nix run .#test [crate]` | Run tests (optionally specify crate) |
| `nix develop` | Enter development environment |

### Environment & Deployment Commands

| Command | Description |
|---------|-------------|
| `nix run .#setup-solana` | Set up Solana platform tools |
| `nix run .#generate-idls` | Generate IDLs with nightly Rust |
| `nix run .#deploy [network]` | Deploy to devnet/mainnet |
| `nix run .#env-info` | Show environment information |
| `nix run .#clear-cache` | Clear all build caches |

### Benefits of crate2nix + Nix Approach

- ✅ **Lightning-fast incremental builds**: crate2nix provides Rust-native incremental compilation with Nix caching
- ✅ **Reproducible**: Pinned dependency versions ensure consistent builds
- ✅ **Self-contained**: No external bash scripts or system dependencies
- ✅ **Cross-platform**: Works consistently across Linux, macOS Intel/Apple Silicon
- ✅ **Cacheable**: Nix provides intelligent caching and binary substitution
- ✅ **Declarative**: All configuration is explicit in `flake.nix`

## Development Workflow

1. **Enter development environment**: `nix develop`
2. **Fast development builds**: `nix run .#build-fast`
3. **Full build with deployment artifacts**: `nix run .#build`
4. **Run tests**: `nix run .#test [crate]`
5. **Clear caches if needed**: `nix run .#clear-cache`

## Build System Architecture

- **crate2nix**: Handles Rust compilation with incremental builds and Nix caching
- **Anchor**: Used for IDL generation and deployment artifact creation
- **Platform Tools**: Integrated Solana BPF/SBF compilation toolchain
- **Unified Environment**: All tools work together seamlessly

## Legacy Scripts Removed

All bash scripts have been successfully removed and replaced with the modern crate2nix-based build system. This provides significantly faster builds and better developer experience.

This directory can be safely deleted once this README is no longer needed for reference. 