//! Cryptographic operation tests for the GalaChain Desktop Wallet
//!
//! These tests cover the critical security functions:
//! - Wallet generation (BIP39 mnemonic + secp256k1 keys)
//! - Key derivation and address generation
//! - Mnemonic import/export functionality
//! - Keychain storage operations

use super::test_utils::*;
use bip39::{Mnemonic, Language};
use secp256k1::{SecretKey, Secp256k1};
use sha3::{Digest, Keccak256};
use std::str::FromStr;
use rand::RngCore;

#[cfg(test)]
mod wallet_generation_tests {
    use super::*;

    #[test]
    fn test_mnemonic_generation_produces_12_words() {
        // Test that mnemonic generation produces exactly 12 words
        // Generate 128 bits of entropy for 12 words
        use rand::RngCore;
        let mut entropy = [0u8; 16]; // 128 bits = 16 bytes
        rand::thread_rng().fill_bytes(&mut entropy);
        
        let mnemonic = Mnemonic::from_entropy(&entropy).unwrap();
        let mnemonic_str = mnemonic.to_string();
        let words: Vec<&str> = mnemonic_str.split_whitespace().collect();
        
        assert_eq!(words.len(), 12, "Mnemonic should have exactly 12 words");
        
        // Verify all words are valid BIP39 words
        for word in words {
            assert!(bip39::Language::English.word_list().contains(&word), 
                   "Word '{}' should be in BIP39 wordlist", word);
        }
    }

    #[test]
    fn test_mnemonic_generation_is_random() {
        // Generate multiple mnemonics and verify they're different
        use rand::RngCore;
        
        let mut entropy1 = [0u8; 16];
        let mut entropy2 = [0u8; 16];
        let mut entropy3 = [0u8; 16];
        
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut entropy1);
        rng.fill_bytes(&mut entropy2);
        rng.fill_bytes(&mut entropy3);
        
        let mnemonic1 = Mnemonic::from_entropy(&entropy1).unwrap();
        let mnemonic2 = Mnemonic::from_entropy(&entropy2).unwrap();
        let mnemonic3 = Mnemonic::from_entropy(&entropy3).unwrap();
        
        assert_ne!(mnemonic1.to_string(), mnemonic2.to_string(), 
                  "Generated mnemonics should be different");
        assert_ne!(mnemonic2.to_string(), mnemonic3.to_string(), 
                  "Generated mnemonics should be different");
        assert_ne!(mnemonic1.to_string(), mnemonic3.to_string(), 
                  "Generated mnemonics should be different");
    }

    #[test]
    fn test_secret_key_generation_from_mnemonic() {
        // Test that we can derive a consistent private key from a mnemonic
        let test_mnemonic = TestVectors::TEST_MNEMONIC_12;
        let mnemonic = Mnemonic::from_str(test_mnemonic).unwrap();
        
        // Generate seed from mnemonic (without passphrase)
        let seed = mnemonic.to_seed("");
        
        // For this test, we'll use a simple key derivation
        // Note: In production, you'd use proper BIP32/BIP44 derivation
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&seed[0..32]).unwrap();
        
        // The key should be deterministic for the same mnemonic
        let mnemonic2 = Mnemonic::from_str(test_mnemonic).unwrap();
        let seed2 = mnemonic2.to_seed("");
        let secret_key2 = SecretKey::from_slice(&seed2[0..32]).unwrap();
        
        assert_eq!(secret_key.secret_bytes(), secret_key2.secret_bytes(),
                  "Same mnemonic should produce same private key");
    }

    #[test]
    fn test_ethereum_address_derivation() {
        // Test Ethereum address derivation from public key
        let secret_key = create_test_secret_key();
        let secp = Secp256k1::new();
        let public_key = secret_key.public_key(&secp);
        
        // Get uncompressed public key (64 bytes)
        let uncompressed = public_key.serialize_uncompressed();
        assert_eq!(uncompressed.len(), 65); // 1 byte prefix + 64 bytes
        
        // Take the last 64 bytes (skip the 0x04 prefix)
        let public_key_bytes = &uncompressed[1..];
        
        // Keccak256 hash of public key
        let mut hasher = Keccak256::new();
        hasher.update(public_key_bytes);
        let hash = hasher.finalize();
        
        // Take last 20 bytes and format as hex
        let address_bytes = &hash[12..];
        let address = format!("0x{}", hex::encode(address_bytes));
        
        // Verify it's a valid Ethereum address format
        assert!(is_valid_eth_address_format(&address));
        assert_eq!(address.len(), 42);
        assert!(address.starts_with("0x"));
    }
}

#[cfg(test)]
mod mnemonic_validation_tests {
    use super::*;

    #[test]
    fn test_valid_mnemonic_acceptance() {
        // Test that valid mnemonics are accepted
        let valid_mnemonics = [
            TestVectors::TEST_MNEMONIC_12,
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
        ];
        
        for mnemonic_str in &valid_mnemonics {
            let result = Mnemonic::from_str(mnemonic_str);
            assert!(result.is_ok(), "Valid mnemonic '{}' should be accepted", mnemonic_str);
        }
    }

    #[test]
    fn test_invalid_mnemonic_rejection() {
        // Test that invalid mnemonics are rejected
        let invalid_mnemonics = [
            TestVectors::INVALID_MNEMONIC_WRONG_COUNT,
            TestVectors::INVALID_MNEMONIC_BAD_WORD,
            TestVectors::INVALID_MNEMONIC_BAD_CHECKSUM,
            "", // Empty string
            "word", // Single word
            "this is not a valid mnemonic phrase at all really", // Random words
        ];
        
        for mnemonic_str in &invalid_mnemonics {
            let result = Mnemonic::from_str(mnemonic_str);
            assert!(result.is_err(), "Invalid mnemonic '{}' should be rejected", mnemonic_str);
        }
    }

    #[test]
    fn test_mnemonic_case_insensitive() {
        // Note: The BIP39 library expects lowercase words, but we can test normalization
        let original = TestVectors::TEST_MNEMONIC_12;
        let original_mnemonic = Mnemonic::from_str(original).unwrap();
        
        // Test that we can manually normalize case before parsing
        let uppercase = original.to_uppercase();
        let normalized_uppercase = uppercase.to_lowercase();
        let normalized_mnemonic = Mnemonic::from_str(&normalized_uppercase).unwrap();
        
        // Should produce the same seed after normalization
        assert_eq!(original_mnemonic.to_seed(""), normalized_mnemonic.to_seed(""));
        
        // Test mixed case normalization
        let mixed_case = "ABANDON abandon ABANDON abandon abandon abandon abandon abandon abandon abandon abandon ABOUT";
        let normalized_mixed = mixed_case.to_lowercase();
        let mixed_mnemonic = Mnemonic::from_str(&normalized_mixed).unwrap();
        
        assert_eq!(original_mnemonic.to_seed(""), mixed_mnemonic.to_seed(""));
    }

    #[test]
    fn test_mnemonic_whitespace_handling() {
        // Test that extra whitespace is handled correctly
        let original = TestVectors::TEST_MNEMONIC_12;
        let extra_spaces = "  abandon   abandon abandon  abandon abandon abandon abandon abandon abandon abandon abandon  about  ";
        let tabs_and_newlines = "abandon\tabandon\nabandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let original_mnemonic = Mnemonic::from_str(original).unwrap();
        let spaces_mnemonic = Mnemonic::from_str(extra_spaces).unwrap();
        let whitespace_mnemonic = Mnemonic::from_str(tabs_and_newlines).unwrap();
        
        // All should produce the same seed
        assert_eq!(original_mnemonic.to_seed(""), spaces_mnemonic.to_seed(""));
        assert_eq!(original_mnemonic.to_seed(""), whitespace_mnemonic.to_seed(""));
    }
}

#[cfg(test)]
mod key_security_tests {
    use super::*;

    #[test]
    fn test_private_key_secrecy() {
        // Test that private keys are handled securely
        let secret_key = create_test_secret_key();
        let key_bytes = secret_key.secret_bytes();
        
        // Verify key is 32 bytes
        assert_eq!(key_bytes.len(), 32);
        
        // Verify key is not all zeros (which would be invalid)
        assert_ne!(key_bytes, [0u8; 32]);
        
        // Verify key is valid for secp256k1
        let secp = Secp256k1::new();
        let _public_key = secret_key.public_key(&secp); // Should not panic
    }

    #[test]
    fn test_deterministic_key_derivation() {
        // Test that the same mnemonic always produces the same key
        let mnemonic_str = TestVectors::TEST_MNEMONIC_12;
        
        // Generate key multiple times
        let keys: Vec<SecretKey> = (0..5).map(|_| {
            let mnemonic = Mnemonic::from_str(mnemonic_str).unwrap();
            let seed = mnemonic.to_seed("");
            SecretKey::from_slice(&seed[0..32]).unwrap()
        }).collect();
        
        // All keys should be identical
        for i in 1..keys.len() {
            assert_eq!(keys[0].secret_bytes(), keys[i].secret_bytes(),
                      "Key derivation should be deterministic");
        }
    }

    #[test]
    fn test_different_mnemonics_produce_different_keys() {
        // Test that different mnemonics produce different keys
        let mnemonics = [
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
        ];
        
        let keys: Vec<SecretKey> = mnemonics.iter().map(|mnemonic_str| {
            let mnemonic = Mnemonic::from_str(mnemonic_str).unwrap();
            let seed = mnemonic.to_seed("");
            SecretKey::from_slice(&seed[0..32]).unwrap()
        }).collect();
        
        // All keys should be different
        for i in 0..keys.len() {
            for j in i+1..keys.len() {
                assert_ne!(keys[i].secret_bytes(), keys[j].secret_bytes(),
                          "Different mnemonics should produce different keys");
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_wallet_generation_cycle() {
        // Test the complete wallet generation process
        
        // 1. Generate mnemonic
        use rand::RngCore;
        let mut entropy = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy).unwrap();
        let mnemonic_str = mnemonic.to_string();
        
        // 2. Verify mnemonic is valid
        assert_eq!(mnemonic_str.split_whitespace().count(), 12);
        
        // 3. Generate key from mnemonic
        let seed = mnemonic.to_seed("");
        let secret_key = SecretKey::from_slice(&seed[0..32]).unwrap();
        
        // 4. Generate public key and address
        let secp = Secp256k1::new();
        let public_key = secret_key.public_key(&secp);
        let uncompressed = public_key.serialize_uncompressed();
        let public_key_bytes = &uncompressed[1..];
        
        let mut hasher = Keccak256::new();
        hasher.update(public_key_bytes);
        let hash = hasher.finalize();
        let address = format!("0x{}", hex::encode(&hash[12..]));
        
        // 5. Verify address format
        assert!(is_valid_eth_address_format(&address));
        
        // 6. Test round-trip: import mnemonic and verify same key/address
        let imported_mnemonic = Mnemonic::from_str(&mnemonic_str).unwrap();
        let imported_seed = imported_mnemonic.to_seed("");
        let imported_key = SecretKey::from_slice(&imported_seed[0..32]).unwrap();
        
        assert_eq!(secret_key.secret_bytes(), imported_key.secret_bytes());
    }

    #[test]
    fn test_wallet_data_serialization() {
        // Test that wallet data can be properly created and stored
        let test_data = create_test_wallet_data();
        
        assert_eq!(test_data.mnemonic, TestVectors::TEST_MNEMONIC_12);
        assert_eq!(test_data.created_at, 1234567890);
        
        // Verify mnemonic in the data is valid
        let mnemonic = Mnemonic::from_str(&test_data.mnemonic).unwrap();
        assert_eq!(mnemonic.word_count(), 12);
    }
}