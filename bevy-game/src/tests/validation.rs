//! Input validation tests for the GalaChain Desktop Wallet
//!
//! These tests cover validation of user inputs:
//! - Mnemonic word validation against BIP39 wordlist
//! - Ethereum address format validation
//! - Amount/numeric input validation
//! - Form completeness validation

use super::test_utils::*;
use bip39::{Mnemonic, Language};
use std::str::FromStr;

#[cfg(test)]
mod mnemonic_validation_tests {
    use super::*;

    #[test]
    fn test_individual_word_validation() {
        // Test validation of individual BIP39 words
        let wordlist = Language::English.word_list();
        
        // Valid words should pass (using words we know are in BIP39)
        let valid_words = ["abandon", "about", "zoo", "zebra", "zone"];
        for word in &valid_words {
            assert!(wordlist.contains(word), "Word '{}' should be valid", word);
        }
        
        // Invalid words should fail
        let invalid_words = ["invalid", "notaword", "blockchain", "crypto", ""];
        for word in &invalid_words {
            assert!(!wordlist.contains(word), "Word '{}' should be invalid", word);
        }
    }

    #[test]
    fn test_partial_word_matching() {
        // Test partial word matching for autocomplete functionality
        let wordlist = Language::English.word_list();
        
        // Find words that start with "aba"
        let aba_words: Vec<&str> = wordlist.iter()
            .filter(|word| word.starts_with("aba"))
            .cloned()
            .collect();
        
        assert!(aba_words.contains(&"abandon"));
        assert!(aba_words.len() > 0, "Should find at least one word starting with 'aba'");
        
        // Test case insensitive matching
        let uppercase_matches: Vec<&str> = wordlist.iter()
            .filter(|word| word.to_lowercase().starts_with("aba"))
            .cloned()
            .collect();
        
        assert_eq!(aba_words, uppercase_matches);
    }

    #[test]
    fn test_word_count_validation() {
        // Test that we enforce exactly 12 words
        assert!(is_valid_word_count("one two three four five six seven eight nine ten eleven twelve"));
        assert!(!is_valid_word_count("one two three"));
        assert!(!is_valid_word_count("one two three four five six seven eight nine ten eleven twelve thirteen"));
        assert!(!is_valid_word_count(""));
    }

    #[test]
    fn test_mnemonic_completeness_validation() {
        // Test validation of complete mnemonic phrases
        let test_cases = [
            // Valid complete mnemonics
            (TestVectors::TEST_MNEMONIC_12, true),
            ("legal winner thank year wave sausage worth useful legal winner thank yellow", true),
            
            // Incomplete mnemonics (should be invalid)
            ("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon", false),
            ("legal winner thank year wave sausage worth useful legal winner thank", false),
            
            // Wrong word count
            ("abandon abandon abandon", false),
            ("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon", false),
        ];
        
        for (mnemonic_str, should_be_valid) in &test_cases {
            let result = Mnemonic::from_str(mnemonic_str);
            if *should_be_valid {
                assert!(result.is_ok(), "Mnemonic '{}' should be valid", mnemonic_str);
            } else {
                assert!(result.is_err(), "Mnemonic '{}' should be invalid", mnemonic_str);
            }
        }
    }

    #[test]
    fn test_mnemonic_normalization() {
        // Test that mnemonics are properly normalized (whitespace, case)
        let variations = [
            TestVectors::TEST_MNEMONIC_12,
            "  abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about  ",
            "\tabandon\tabandon\tabandon\tabandon\tabandon\tabandon\tabandon\tabandon\tabandon\tabandon\tabandon\tabout\t",
        ];
        
        let normalized_seeds: Vec<Vec<u8>> = variations.iter()
            .map(|variant| {
                // Normalize the input before parsing
                let normalized = variant.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
                let mnemonic = Mnemonic::from_str(&normalized).unwrap();
                mnemonic.to_seed("").to_vec()
            })
            .collect();
        
        // All variations should produce the same seed after normalization
        for i in 1..normalized_seeds.len() {
            assert_eq!(normalized_seeds[0], normalized_seeds[i],
                      "All normalized variations should produce the same seed");
        }
    }
}

#[cfg(test)]
mod address_validation_tests {
    use super::*;

    #[test]
    fn test_ethereum_address_format_validation() {
        // Test valid Ethereum addresses
        let valid_addresses = [
            "0x0000000000000000000000000000000000000000",
            "0xFFfFfFffFFfffFFfFFfFFFFFffFFFffffFfFFFfF",
            "0x742d35Cc6574C0532E82e4b52b86B7d5dA99F64E",
            TestVectors::EXPECTED_ETH_ADDRESS,
        ];
        
        for address in &valid_addresses {
            assert!(is_valid_eth_address_format(address), 
                   "Address '{}' should be valid", address);
        }
        
        // Test invalid addresses
        let invalid_addresses = [
            "",
            "0x",
            "742d35Cc6574C0532E82e4b52b86B7d5dA99F64E", // Missing 0x prefix
            "0x742d35Cc6574C0532E82e4b52b86B7d5dA99F64", // Too short
            "0x742d35Cc6574C0532E82e4b52b86B7d5dA99F64EE", // Too long
            "0x742d35Cc6574C0532E82e4b52b86B7d5dA99G64E", // Invalid character 'G'
            "0X742d35Cc6574C0532E82e4b52b86B7d5dA99F64E", // Wrong case prefix
        ];
        
        for address in &invalid_addresses {
            assert!(!is_valid_eth_address_format(address), 
                   "Address '{}' should be invalid", address);
        }
    }

    #[test]
    fn test_address_checksum_validation() {
        // Test EIP-55 checksum validation (if we implement it)
        // For now, we'll test that mixed case addresses are handled
        let mixed_case_addresses = [
            "0x742d35Cc6574C0532E82e4b52b86B7d5dA99F64E",
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
        ];
        
        for address in &mixed_case_addresses {
            assert!(is_valid_eth_address_format(address), 
                   "Mixed case address '{}' should be valid", address);
        }
    }
}

#[cfg(test)]
mod amount_validation_tests {
    use super::*;

    fn is_valid_amount(amount_str: &str) -> bool {
        // Helper function to validate amount strings
        if amount_str.is_empty() {
            return false;
        }
        
        // Reject strings ending with "." or starting with "."
        if amount_str.ends_with('.') || amount_str.starts_with('.') {
            return false;
        }
        
        match amount_str.parse::<f64>() {
            Ok(amount) => amount >= 0.0 && amount.is_finite(),
            Err(_) => false,
        }
    }

    fn is_valid_decimal_precision(amount_str: &str, max_decimals: usize) -> bool {
        if let Some(decimal_pos) = amount_str.find('.') {
            let decimal_part = &amount_str[decimal_pos + 1..];
            decimal_part.len() <= max_decimals
        } else {
            true // No decimal point means 0 decimal places
        }
    }

    #[test]
    fn test_numeric_amount_validation() {
        // Test valid amounts
        let valid_amounts = [
            "0",
            "0.0",
            "1",
            "1.0",
            "100.50",
            "999999999.123456789",
            "0.000000001",
        ];
        
        for amount in &valid_amounts {
            assert!(is_valid_amount(amount), 
                   "Amount '{}' should be valid", amount);
        }
        
        // Test invalid amounts
        let invalid_amounts = [
            "",
            "-1",
            "-0.1",
            "abc",
            "1.2.3",
            "∞",
            "NaN",
            "1.",
            ".1",
        ];
        
        for amount in &invalid_amounts {
            assert!(!is_valid_amount(amount), 
                   "Amount '{}' should be invalid", amount);
        }
    }

    #[test]
    fn test_decimal_precision_validation() {
        // Test GALA token precision (typically 8 decimal places)
        let max_decimals = 8;
        
        let valid_precision = [
            "1",
            "1.0",
            "1.12345678",
            "0.00000001",
        ];
        
        for amount in &valid_precision {
            assert!(is_valid_decimal_precision(amount, max_decimals),
                   "Amount '{}' should have valid precision", amount);
        }
        
        let invalid_precision = [
            "1.123456789", // Too many decimals
            "0.000000001", // Too many decimals
        ];
        
        for amount in &invalid_precision {
            assert!(!is_valid_decimal_precision(amount, max_decimals),
                   "Amount '{}' should have invalid precision", amount);
        }
    }

    #[test]
    fn test_amount_range_validation() {
        // Test reasonable amount ranges for token transfers
        fn is_valid_amount_range(amount_str: &str) -> bool {
            match amount_str.parse::<f64>() {
                Ok(amount) => amount > 0.0 && amount <= 1_000_000_000.0, // 1 billion max
                Err(_) => false,
            }
        }
        
        let valid_ranges = [
            "0.00000001",
            "1",
            "1000",
            "1000000",
            "1000000000",
        ];
        
        for amount in &valid_ranges {
            assert!(is_valid_amount_range(amount),
                   "Amount '{}' should be in valid range", amount);
        }
        
        let invalid_ranges = [
            "0",
            "1000000001", // Too large
            "10000000000", // Too large
        ];
        
        for amount in &invalid_ranges {
            assert!(!is_valid_amount_range(amount),
                   "Amount '{}' should be out of valid range", amount);
        }
    }
}

#[cfg(test)]
mod form_validation_tests {
    use super::*;

    #[test]
    fn test_import_form_completeness() {
        // Test validation of complete import forms
        fn is_import_form_valid(words: &[String]) -> bool {
            if words.len() != 12 {
                return false;
            }
            
            // Check all words are non-empty and valid
            let wordlist = Language::English.word_list();
            words.iter().all(|word| !word.is_empty() && wordlist.contains(&word.as_str()))
        }
        
        // Valid complete form
        let complete_words: Vec<String> = TestVectors::TEST_MNEMONIC_12
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        assert!(is_import_form_valid(&complete_words));
        
        // Incomplete form (empty word)
        let mut incomplete_words = complete_words.clone();
        incomplete_words[5] = String::new();
        assert!(!is_import_form_valid(&incomplete_words));
        
        // Invalid word
        let mut invalid_words = complete_words.clone();
        invalid_words[5] = "invalid".to_string();
        assert!(!is_import_form_valid(&invalid_words));
        
        // Wrong count
        let short_words = vec!["abandon".to_string(); 6];
        assert!(!is_import_form_valid(&short_words));
    }

    #[test]
    fn test_transfer_form_completeness() {
        // Test validation of transfer forms
        fn is_transfer_form_valid(recipient: &str, amount: &str) -> bool {
            is_valid_eth_address_format(recipient) && is_valid_amount(amount)
        }
        
        // Valid transfer form
        assert!(is_transfer_form_valid(
            TestVectors::EXPECTED_ETH_ADDRESS,
            "100.5"
        ));
        
        // Invalid recipient
        assert!(!is_transfer_form_valid(
            "invalid_address",
            "100.5"
        ));
        
        // Invalid amount
        assert!(!is_transfer_form_valid(
            TestVectors::EXPECTED_ETH_ADDRESS,
            "-100"
        ));
        
        // Both invalid
        assert!(!is_transfer_form_valid(
            "invalid",
            "invalid"
        ));
        
        // Empty fields
        assert!(!is_transfer_form_valid("", ""));
    }

    fn is_valid_amount(amount_str: &str) -> bool {
        if amount_str.is_empty() {
            return false;
        }
        
        match amount_str.parse::<f64>() {
            Ok(amount) => amount >= 0.0 && amount.is_finite(),
            Err(_) => false,
        }
    }
}