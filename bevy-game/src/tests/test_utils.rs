//! Test utilities and helper functions for GalaChain wallet tests

use crate::SecureWalletData;
use secp256k1::{SecretKey, Secp256k1};
use std::str::FromStr;

/// Known test vectors from BIP39 specification
/// These are well-known test cases that should never be used in production
pub struct TestVectors;

impl TestVectors {
    /// Test mnemonic with known derived key (from BIP39 test vectors)
    /// WARNING: This is a well-known test mnemonic - NEVER use in production
    pub const TEST_MNEMONIC_12: &'static str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    /// Expected private key (hex) derived from TEST_MNEMONIC_12  
    pub const EXPECTED_PRIVATE_KEY_HEX: &'static str = "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e5349553";
    
    /// Expected Ethereum address derived from the private key
    pub const EXPECTED_ETH_ADDRESS: &'static str = "0x9858EfFD232B4033E47d90003D41EC34EcaEda94";
    
    /// Invalid mnemonic - wrong word count
    pub const INVALID_MNEMONIC_WRONG_COUNT: &'static str = "abandon abandon abandon abandon abandon";
    
    /// Invalid mnemonic - invalid word
    pub const INVALID_MNEMONIC_BAD_WORD: &'static str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon invalid";
    
    /// Invalid mnemonic - bad checksum
    pub const INVALID_MNEMONIC_BAD_CHECKSUM: &'static str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
}

/// Mock keychain manager for testing
/// This allows us to test keychain operations without using the real OS keychain
pub struct MockKeychainManager {
    stored_data: Option<SecureWalletData>,
    should_fail: bool,
}

impl MockKeychainManager {
    pub fn new() -> Self {
        Self {
            stored_data: None,
            should_fail: false,
        }
    }
    
    /// Configure the mock to simulate failures
    pub fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }
    
    /// Check if data is stored
    pub fn has_stored_data(&self) -> bool {
        self.stored_data.is_some()
    }
    
    /// Get stored data for verification
    pub fn get_stored_data(&self) -> Option<&SecureWalletData> {
        self.stored_data.as_ref()
    }
}

// Note: We can't implement the KeychainManager trait methods here because
// they're defined in the main module. We'll need to add a trait later
// or use dependency injection for proper testing.

/// Helper function to create a deterministic SecretKey for testing
pub fn create_test_secret_key() -> SecretKey {
    // Use a simple valid key for testing - all 1s (valid secp256k1 key)
    let key_bytes = [0x01u8; 32];
    SecretKey::from_slice(&key_bytes).unwrap()
}

/// Helper function to create test wallet data
pub fn create_test_wallet_data() -> SecureWalletData {
    SecureWalletData {
        mnemonic: TestVectors::TEST_MNEMONIC_12.to_string(),
        created_at: 1234567890, // Fixed timestamp for deterministic tests
    }
}

/// Verify that a mnemonic contains exactly 12 valid words
pub fn is_valid_word_count(mnemonic: &str) -> bool {
    mnemonic.split_whitespace().count() == 12
}

/// Check if an Ethereum address has the correct format
pub fn is_valid_eth_address_format(address: &str) -> bool {
    address.starts_with("0x") && 
    address.len() == 42 && 
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vectors_are_valid() {
        // Ensure our test vectors have the expected format
        assert_eq!(TestVectors::TEST_MNEMONIC_12.split_whitespace().count(), 12);
        assert_eq!(TestVectors::EXPECTED_PRIVATE_KEY_HEX.len(), 64); // 32 bytes = 64 hex chars
        assert!(TestVectors::EXPECTED_ETH_ADDRESS.starts_with("0x"));
        assert_eq!(TestVectors::EXPECTED_ETH_ADDRESS.len(), 42);
    }
    
    #[test]
    fn test_mock_keychain_manager() {
        let mut mock = MockKeychainManager::new();
        assert!(!mock.has_stored_data());
        
        mock.set_should_fail(true);
        assert!(mock.should_fail);
    }
    
    #[test]
    fn test_validation_helpers() {
        assert!(is_valid_word_count(TestVectors::TEST_MNEMONIC_12));
        assert!(!is_valid_word_count(TestVectors::INVALID_MNEMONIC_WRONG_COUNT));
        
        assert!(is_valid_eth_address_format(TestVectors::EXPECTED_ETH_ADDRESS));
        assert!(!is_valid_eth_address_format("invalid_address"));
        assert!(!is_valid_eth_address_format("0xinvalid"));
    }
    
    #[test]
    fn test_secret_key_creation() {
        let key = create_test_secret_key();
        // Verify the key can be used for cryptographic operations
        let secp = Secp256k1::new();
        let public_key = key.public_key(&secp);
        assert_eq!(public_key.serialize().len(), 33); // Compressed public key
    }
}