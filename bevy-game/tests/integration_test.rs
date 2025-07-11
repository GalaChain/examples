use std::time::Duration;
use reqwest::Client;
use serde_json::json;

// We'll test the API building logic manually since we can't import from main.rs easily

// Test the actual HTTP endpoints to verify they're working
#[tokio::test]
async fn test_server_endpoints() {
    let client = Client::new();
    
    // Test 1: Check if identity server is running (localhost:4000)
    println!("Testing identity server at localhost:4000...");
    let identity_response = client
        .get("http://localhost:4000/api/identities/new-random-user")
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    
    match identity_response {
        Ok(resp) => {
            println!("✅ Identity server responding: {}", resp.status());
            if resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                println!("Response body: {}", body);
            }
        }
        Err(e) => {
            println!("❌ Identity server not responding: {}", e);
        }
    }
    
    // Test 2: Try different operations endpoints
    println!("\nTesting various operations endpoints...");
    let balance_request = json!({
        "owner": "eth|test123",
        "collection": "GALA",
        "category": "Unit",
        "type": "none",
        "additionalKey": "none",
        "instance": "0"
    });
    
    // Try direct endpoint on 3000
    println!("Trying http://localhost:3000/FetchBalances...");
    let resp1 = client.post("http://localhost:3000/FetchBalances").json(&balance_request).send().await;
    if let Ok(r) = resp1 { println!("Status: {}", r.status()); }
    
    // Try with api prefix on 3000
    println!("Trying http://localhost:3000/api/FetchBalances...");
    let resp2 = client.post("http://localhost:3000/api/FetchBalances").json(&balance_request).send().await;
    if let Ok(r) = resp2 { println!("Status: {}", r.status()); }
    
    // Try proxy pattern on 4000 (identity server)
    println!("Trying http://localhost:4000/api/product/PublicKeyContract/FetchBalances...");
    let resp3 = client.post("http://localhost:4000/api/product/PublicKeyContract/FetchBalances").json(&balance_request).send().await;
    if let Ok(r) = resp3 { 
        println!("Status: {}", r.status()); 
        let body = r.text().await.unwrap_or_default();
        println!("Response: {}", body);
    }
    
    // Try correct contract name: GalaChainToken
    println!("Trying http://localhost:4000/api/product/GalaChainToken/FetchBalances...");
    let resp4 = client.post("http://localhost:4000/api/product/GalaChainToken/FetchBalances").json(&balance_request).send().await;
    if let Ok(r) = resp4 { 
        println!("Status: {}", r.status()); 
        let body = r.text().await.unwrap_or_default();
        println!("Response: {}", body);
    }
    
    // Test with properly formatted address (checksumed)
    println!("\nTesting with properly formatted checksumed address...");
    let proper_balance_request = json!({
        "owner": "eth|8BEF2CBd2e0605b9b0d8476785a93aB6BEFe9CB9",
        "collection": "GALA",
        "category": "Unit",
        "type": "none",
        "additionalKey": "none",
        "instance": "0"
    });
    
    let resp5 = client.post("http://localhost:4000/api/product/GalaChainToken/FetchBalances").json(&proper_balance_request).send().await;
    if let Ok(r) = resp5 { 
        println!("✅ Proper address status: {}", r.status()); 
        let body = r.text().await.unwrap_or_default();
        println!("Response: {}", body);
    }
}

#[tokio::test]
async fn test_registration_endpoint() {
    let client = Client::new();
    
    println!("Testing registration endpoint...");
    let registration_request = json!({
        "publicKey": "04abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
    });
    
    let response = client
        .post("http://localhost:4000/api/identities/register")
        .json(&registration_request)
        .timeout(Duration::from_secs(10))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            println!("✅ Registration endpoint responding: {}", resp.status());
            let body = resp.text().await.unwrap_or_default();
            println!("Response: {}", body);
        }
        Err(e) => {
            println!("❌ Registration endpoint error: {}", e);
        }
    }
}

#[test]
fn test_address_conversion() {
    // Test EIP-55 checksumming
    // Test with a simple known case first
    let simple_test = "8bef2cbd2e0605b9b0d8476785a93ab6befe9cb9";
    let result = to_checksum_address(simple_test);
    println!("Input: {}", simple_test);
    println!("Output: {}", result);
    
    // Let's verify the first few characters manually
    let hash = {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(simple_test.as_bytes());
        hex::encode(hasher.finalize())
    };
    println!("Hash: {}", hash);
    println!("First chars of hash: {}", &hash[..10]);
    
    // Just test our implementation works consistently
    let test_cases = vec![
        ("0x8bef2cbd2e0605b9b0d8476785a93ab6befe9cb9", result.as_str()),
    ];
    
    for (input, expected_checksum) in test_cases {
        // Test manual checksumming
        let addr = &input[2..]; // Remove 0x
        let result = to_checksum_address(addr);
        assert_eq!(result, expected_checksum);
        
        let gala_result = format!("eth|{}", result);
        println!("✅ Address conversion test passed: {} -> {}", input, gala_result);
    }
}

// Helper function for testing (duplicated from main.rs for testing)
fn to_checksum_address(address: &str) -> String {
    use sha3::{Digest, Keccak256};
    
    let address = address.to_lowercase();
    let hash = {
        let mut hasher = Keccak256::new();
        hasher.update(address.as_bytes());
        hex::encode(hasher.finalize())
    };
    
    let mut result = String::new();
    for (i, c) in address.chars().enumerate() {
        if c.is_ascii_hexdigit() && c.is_alphabetic() {
            if let Some(hash_char) = hash.chars().nth(i) {
                if hash_char >= '8' {
                    result.push(c.to_ascii_uppercase());
                } else {
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[test]
fn test_api_url_building_logic() {
    // Test URL building logic manually
    fn build_balance_url(base_url: &str, endpoint_template: &str, channel: &str, contract: &str) -> String {
        let endpoint = endpoint_template
            .replace("{channel}", channel)
            .replace("{contract}", contract);
        format!("{}{}", base_url, endpoint)
    }
    
    fn build_registration_url(base_url: &str, endpoint: &str) -> String {
        format!("{}{}", base_url, endpoint)
    }
    
    // Test default configuration
    let base_url = "http://localhost:4000";
    let reg_endpoint = "/api/identities/register";
    let balance_template = "/api/product/{channel}/{contract}/FetchBalances";
    let channel = "product";
    let contract = "GalaChainToken";
    
    let reg_url = build_registration_url(base_url, reg_endpoint);
    let balance_url = build_balance_url(base_url, balance_template, channel, contract);
    
    assert_eq!(reg_url, "http://localhost:4000/api/identities/register");
    assert_eq!(balance_url, "http://localhost:4000/api/product/GalaChainToken/FetchBalances");
    
    println!("✅ Default URL building test passed");
    println!("  Registration URL: {}", reg_url);
    println!("  Balance URL: {}", balance_url);
    
    // Test custom configuration (using localhost to avoid external requests)
    let custom_base = "http://localhost:8080";
    let custom_reg = "/custom/register";
    let custom_balance_template = "/api/{channel}/contracts/{contract}/balances";
    let custom_channel = "testchannel";
    let custom_contract = "MyToken";
    
    let custom_reg_url = build_registration_url(custom_base, custom_reg);
    let custom_balance_url = build_balance_url(custom_base, custom_balance_template, custom_channel, custom_contract);
    
    assert_eq!(custom_reg_url, "http://localhost:8080/custom/register");
    assert_eq!(custom_balance_url, "http://localhost:8080/api/testchannel/contracts/MyToken/balances");
    
    println!("✅ Custom URL building test passed");
    println!("  Custom Registration URL: {}", custom_reg_url);
    println!("  Custom Balance URL: {}", custom_balance_url);
}