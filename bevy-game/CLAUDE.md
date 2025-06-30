# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **GalaChain Desktop Wallet** reference implementation built with the Bevy game engine. It serves as a comprehensive example for developers who want to integrate GalaChain functionality into desktop applications or games. This is the desktop equivalent of the web-based dapp-template but uses secure local storage via OS keychain instead of MetaMask.

## Architecture

The project is structured as a single Rust binary with a modern plugin-based architecture:

### Core Plugins
- **MenuPlugin**: Navigation system with state management (AppState and WalletState)
- **WalletPlugin**: Legacy systems (being phased out)

### Key Systems
- **Menu System**: Professional sidebar layout with persistent navigation
- **Secure Storage**: OS-native keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **GalaChain Integration**: API client for registration, balance queries, and transactions
- **Wallet Operations**: Generate, import, export, transfer, and burn functionality

### State Management
- **AppState**: Main navigation (MainMenu, WalletMenu, Settings, Info)
- **WalletState**: Wallet operations (Overview, Generate, Import, Export, Balance, Transfer, Burn)

## Key Dependencies

### Core Framework
- **bevy**: Game engine framework (v0.15.3 with dynamic_linking feature)

### Cryptographic Operations
- **secp256k1**: Elliptic curve cryptography for key generation
- **bip39**: BIP39 mnemonic phrase generation and parsing
- **sha3**: Keccak256 hashing for Ethereum address generation
- **rand**: Random number generation
- **hex**: Hexadecimal encoding/decoding

### Network & Storage
- **reqwest**: HTTP client for GalaChain API integration  
- **serde**: JSON serialization for API communication
- **tokio**: Async runtime for network operations
- **File Storage**: Temporary file storage for wallet data (can be upgraded to OS keychain)

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

### Core Resources
- **KeychainManager**: Secure wallet storage using OS keychain
- **GalaChainClient**: HTTP client for blockchain API operations
- **WalletData**: Current wallet state (private key, address, mnemonic)
- **ImportState**: Manages 12-word seed phrase input
- **ExportState**: Controls seed phrase visibility
- **TransferState**: Transfer form state (recipient, amount)
- **BurnState**: Burn operation state (amount)

### UI Components
- **MenuTitle**: Main page headers
- **ContentArea**: Dynamic content area for wallet operations
- **BackButton**: Navigation back buttons
- **Various operation-specific components**: Generate, Import, Export, Transfer, Burn buttons and inputs

### Key Systems

#### Navigation Systems
- `main_menu_system`: Main menu interactions
- `wallet_menu_system`: Wallet menu navigation
- `back_button_system`: Universal back button handling

#### Wallet Operation Systems
- `wallet_generate_system`: Complete wallet generation with keychain storage
- `wallet_import_system`: 12-word seed phrase import with grid UI
- `wallet_export_system`: Secure seed phrase display with warnings
- `wallet_balance_system`: GalaChain balance queries with registration
- `wallet_transfer_system`: Transfer UI (reference implementation)
- `wallet_burn_system`: Token burning UI based on dapp-template patterns

### Security Features
- **File-based Storage**: Secure wallet storage in temporary files (upgradeable to OS keychain)
- **Crypto Security**: Proper BIP39 mnemonic and secp256k1 key generation
- **Secure Memory Handling**: Proper cleanup of sensitive data
- **Warning Systems**: Clear warnings for irreversible operations

### GalaChain Integration
- **API Structure**: Demonstrates GalaChain API integration patterns
- **Address Conversion**: Ethereum to GalaChain address mapping
- **Reference Implementation**: UI patterns for registration, balance queries, transfers, and burns
- **dapp-template Compatibility**: Follows same patterns as the Vue.js reference template

## Development Notes

### Project Structure
- **main.rs**: Complete application in single file (~3000+ lines)
- **ext/**: Reference repositories (bevy, bevy-website, dapp-template)
- **CLAUDE.md**: This documentation file

### Implementation Patterns
- **Reference Implementation**: All GalaChain operations (balance, transfer, burn) demonstrate UI patterns and would require full SDK integration for production use
- **Secure Storage**: Real keychain integration for wallet persistence
- **Professional UI**: Sidebar layout with persistent navigation and visual feedback
- **Cross-Platform**: Uses OS-native security features and works on Windows, macOS, and Linux
- **No Network Dependencies**: Runs completely offline as a reference implementation

### Security Considerations
- Never stores private keys in plaintext
- Uses secure JSON storage in temporary files (can be upgraded to OS keychain)
- Provides clear warnings for irreversible operations
- Implements proper cryptographic key generation and handling
- No network dependencies - runs completely offline