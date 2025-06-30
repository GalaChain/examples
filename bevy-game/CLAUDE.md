# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bevy game framework example project that demonstrates integration with GalaChain. The project creates a basic UI application for cryptocurrency wallet operations including key generation, seed phrase management, and address derivation using the Bevy game engine.

## Architecture

The project is structured as a single Rust binary with the following key components:

- **WalletPlugin**: Main Bevy plugin that handles wallet functionality
- **WalletState**: Resource containing wallet data (private key, address, mnemonic)
- **UI Components**: Various UI marker components for buttons and text displays
- **Systems**: Event-driven systems for wallet operations and UI interactions

The application uses Bevy's ECS (Entity Component System) architecture with:
- Components for tagging UI elements
- Systems for handling user interactions
- Resources for maintaining wallet state
- Plugin structure for modular organization

## Key Dependencies

- **bevy**: Game engine framework (v0.15.3 with dynamic_linking feature)
- **secp256k1**: Elliptic curve cryptography for key generation
- **bip39**: BIP39 mnemonic phrase generation and parsing
- **sha3**: Keccak256 hashing for Ethereum address generation
- **rand**: Random number generation
- **hex**: Hexadecimal encoding/decoding

## Common Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Run the application
cargo run

# Build for release
cargo build --release
```

### Development
```bash
# Check code without building
cargo check

# Run tests (if any exist)
cargo test

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

## External Resources Directory

The `ext/` directory contains external repositories for reference:
- **bevy/**: Full Bevy framework source code
- **bevy-website/**: Bevy documentation and website
- **dapp-template/**: Vue.js template for decentralized applications

These are working copies useful for context during development but are not part of the main project build.

## Code Structure

### Main Components
- `WalletButton`: Generate new wallet button
- `ImportButton`: Import wallet from seed phrase button  
- `ExportButton`: Export/show seed phrase button
- `AddressText`: Display wallet address
- `MnemonicText`: Display seed phrase when visible
- `WordInput`: Input fields for seed phrase import

### Key Systems
- `generate_wallet_button_system`: Handles wallet generation
- `export_seed_button_system`: Toggles seed phrase visibility
- `import_button_system`: Shows/hides import interface
- `import_word_system`: Handles text input for seed words
- `import_confirm_system`: Processes imported seed phrase

### Cryptographic Operations
- Uses secp256k1 for private/public key generation
- Generates BIP39 mnemonic phrases for seed backup
- Derives Ethereum-compatible addresses using Keccak256
- Supports wallet import from 12-word seed phrases

## Development Notes

The project uses Bevy 2024 edition with optimized development builds. The `ext/` directory should not be modified as it contains reference materials only. The main application logic is contained entirely in `src/main.rs`.