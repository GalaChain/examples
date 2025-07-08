//! Test modules for the GalaChain Desktop Wallet
//!
//! This module contains comprehensive test coverage for:
//! - Cryptographic operations (wallet generation, key derivation)
//! - Input validation (mnemonics, addresses, amounts)
//! - UI focus system functionality
//! - Security and error handling

#[cfg(test)]
pub mod crypto;

#[cfg(test)]
pub mod validation;

#[cfg(test)]
pub mod focus;

#[cfg(test)]
pub mod test_utils;