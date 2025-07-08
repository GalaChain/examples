use bevy::prelude::*;
use bip39::{Mnemonic, Language};
use secp256k1::{SecretKey, PublicKey};
use rand::rngs::OsRng;
use sha3::{Digest, Keccak256};
use keyring::Entry;
use std::error::Error as StdError;
use std::fmt;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

#[cfg(test)]
mod tests;

// Menu System
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    MainMenu,
    WalletMenu,
    Settings,
    Info,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum WalletState {
    #[default]
    Overview,
    Generate,
    Import,
    Export,
    Registration,
    Balance,
    Transfer,
    Burn,
}

// Keychain Management
#[derive(Debug)]
pub enum KeychainError {
    NotFound,
    Access(String),
    Serialize(String),
    Deserialize(String),
}

impl fmt::Display for KeychainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeychainError::NotFound => write!(f, "Wallet not found in keychain"),
            KeychainError::Access(msg) => write!(f, "Keychain access error: {}", msg),
            KeychainError::Serialize(msg) => write!(f, "Serialization error: {}", msg),
            KeychainError::Deserialize(msg) => write!(f, "Deserialization error: {}", msg),
        }
    }
}

impl StdError for KeychainError {}

#[derive(Debug, Clone)]
pub struct SecureWalletData {
    pub mnemonic: String,
    pub created_at: u64, // Unix timestamp
}

impl SecureWalletData {
    fn to_json(&self) -> Result<String, KeychainError> {
        // Simple JSON serialization without serde for now
        let json = format!(
            r#"{{"mnemonic":"{}","created_at":{}}}"#,
            self.mnemonic.replace('"', "\\\""),
            self.created_at
        );
        Ok(json)
    }

    fn from_json(json: &str) -> Result<Self, KeychainError> {
        // Simple JSON parsing without serde for now
        // This is a basic implementation - in production, use proper JSON parsing
        let json = json.trim();
        if !json.starts_with('{') || !json.ends_with('}') {
            return Err(KeychainError::Deserialize("Invalid JSON format".to_string()));
        }

        let content = &json[1..json.len()-1]; // Remove braces
        let mut mnemonic = String::new();
        let mut created_at = 0u64;

        for part in content.split(',') {
            let part = part.trim();
            if let Some(colon_pos) = part.find(':') {
                let key = part[..colon_pos].trim().trim_matches('"');
                let value = part[colon_pos+1..].trim();

                match key {
                    "mnemonic" => {
                        mnemonic = value.trim_matches('"').replace("\\\"", "\"").to_string();
                    }
                    "created_at" => {
                        created_at = value.parse().map_err(|_|
                            KeychainError::Deserialize("Invalid timestamp".to_string())
                        )?;
                    }
                    _ => {} // Ignore unknown fields
                }
            }
        }

        if mnemonic.is_empty() {
            return Err(KeychainError::Deserialize("Missing mnemonic".to_string()));
        }

        Ok(SecureWalletData {
            mnemonic,
            created_at,
        })
    }
}

#[derive(Resource)]
pub struct KeychainManager {
    service_name: String,
    username: String,
}

impl KeychainManager {
    pub fn new() -> Self {
        Self {
            service_name: "GalaChain-Desktop-Wallet".to_string(),
            username: "default-wallet".to_string(),
        }
    }


    pub fn store_wallet(&self, wallet_data: &SecureWalletData) -> Result<(), KeychainError> {
        let json_data = wallet_data.to_json()?;

        let entry = Entry::new(&self.service_name, &self.username)
            .map_err(|e| KeychainError::Access(format!("Failed to create keychain entry: {}", e)))?;

        entry.set_password(&json_data)
            .map_err(|e| KeychainError::Access(format!("Failed to store wallet in keychain: {}", e)))?;

        info!("Wallet stored securely in OS keychain service: {}", self.service_name);
        Ok(())
    }

    pub fn load_wallet(&self) -> Result<SecureWalletData, KeychainError> {
        let entry = Entry::new(&self.service_name, &self.username)
            .map_err(|e| KeychainError::Access(format!("Failed to create keychain entry: {}", e)))?;

        let json_data = entry.get_password()
            .map_err(|e| match e {
                keyring::Error::NoEntry => KeychainError::NotFound,
                _ => KeychainError::Access(format!("Failed to load wallet from keychain: {}", e)),
            })?;

        SecureWalletData::from_json(&json_data)
    }

    pub fn delete_wallet(&self) -> Result<(), KeychainError> {
        let entry = Entry::new(&self.service_name, &self.username)
            .map_err(|e| KeychainError::Access(format!("Failed to create keychain entry: {}", e)))?;

        entry.delete_credential()
            .map_err(|e| match e {
                keyring::Error::NoEntry => KeychainError::NotFound,
                _ => KeychainError::Access(format!("Failed to delete wallet from keychain: {}", e)),
            })?;

        info!("Wallet deleted from OS keychain service: {}", self.service_name);
        Ok(())
    }

    pub fn wallet_exists(&self) -> bool {
        self.load_wallet().is_ok()
    }

    // Generate wallet data from mnemonic
    pub fn generate_wallet_from_mnemonic(&self, mnemonic: &str) -> Result<(SecretKey, String), String> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic)
            .map_err(|e| format!("Invalid mnemonic: {}", e))?;

        let seed = mnemonic.to_seed("");
        let secp = secp256k1::Secp256k1::new();

        // Use first 32 bytes of seed as private key
        let secret_key = SecretKey::from_slice(&seed[..32])
            .map_err(|e| format!("Failed to create private key: {}", e))?;

        // Generate public key and address
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize_uncompressed();

        // Generate Ethereum address
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]); // Skip recovery id byte
        let result = hasher.finalize();
        let address = format!("0x{}", hex::encode(&result[12..])); // Take last 20 bytes

        Ok((secret_key, address))
    }
}

impl Default for KeychainManager {
    fn default() -> Self {
        Self::new()
    }
}

// GalaChain Client
#[derive(Debug, Clone)]
pub enum GalaChainError {
    Network(String),
    Auth(String),
    Parse(String),
    Api(String),
    NotRegistered,
}

impl fmt::Display for GalaChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GalaChainError::Network(msg) => write!(f, "Network error: {}", msg),
            GalaChainError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            GalaChainError::Parse(msg) => write!(f, "Parsing error: {}", msg),
            GalaChainError::Api(msg) => write!(f, "API error: {}", msg),
            GalaChainError::NotRegistered => write!(f, "User not registered with GalaChain"),
        }
    }
}

impl StdError for GalaChainError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub collection: String,
    pub category: String,
    pub r#type: String,
    #[serde(rename = "additionalKey")]
    pub additional_key: String,
    pub instance: String,
    pub quantity: String,
    #[serde(rename = "lockedHolds")]
    pub locked_holds: Vec<TokenHold>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenHold {
    pub quantity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    #[serde(rename = "Data")]
    pub data: Vec<TokenBalance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceRequest {
    pub owner: String,
    pub collection: String,
    pub category: String,
    pub r#type: String,
    #[serde(rename = "additionalKey")]
    pub additional_key: String,
    pub instance: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyRequest {
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationRequest {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPublicKeyResponse {
    #[serde(rename = "Status")]
    pub status: i32,
    #[serde(rename = "Data")]
    pub data: Option<PublicKeyData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyData {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signing: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInstance {
    pub quantity: String,
    #[serde(rename = "tokenInstanceKey")]
    pub token_instance_key: TokenInstanceKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInstanceKey {
    pub collection: String,
    pub category: String,
    pub r#type: String,
    #[serde(rename = "additionalKey")]
    pub additional_key: String,
    pub instance: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BurnRequest {
    pub owner: String,
    #[serde(rename = "tokenInstances")]
    pub token_instances: Vec<TokenInstance>,
    #[serde(rename = "uniqueKey")]
    pub unique_key: String,
}

#[derive(Resource, Clone)]
pub struct GalaChainClient {
    client: Client,
    pub operations_api: String,
    pub identity_api: String,
    pub settings: ApiSettings,
}

#[derive(Resource, Clone)]
pub struct ApiSettings {
    // Base URLs for the servers
    pub operations_base_url: String,
    pub identity_base_url: String,

    // Configurable API endpoints
    /// Registration endpoint path (e.g., "/api/identities/register")
    pub registration_endpoint: String,
    /// Balance endpoint template with placeholders (e.g., "/api/product/{channel}/{contract}/FetchBalances")
    pub balance_endpoint: String,

    // Blockchain configuration
    /// Smart contract name for token operations (e.g., "GalaChainToken")
    pub contract_name: String,
    /// Smart contract name for identity operations (e.g., "PublicKeyContract")
    pub identity_contract_name: String,
    /// Channel name (e.g., "product")
    pub channel_name: String,
    /// Token collection name (e.g., "GALA")
    pub token_collection: String,
    /// Registration check endpoint (e.g., "/api/product/{channel}/{contract}/GetPublicKey")
    pub registration_check_endpoint: String,
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            operations_base_url: "http://localhost:3000".to_string(),
            identity_base_url: "http://localhost:4000".to_string(),
            // Default endpoints based on API documentation
            registration_endpoint: "/api/identities/register".to_string(),  // Special endpoint on identity server
            registration_check_endpoint: "/api/{channel}/{contract}/GetPublicKey".to_string(),
            balance_endpoint: "/api/{channel}/{contract}/FetchBalances".to_string(),
            contract_name: "GalaChainToken".to_string(),  // For balance operations
            identity_contract_name: "PublicKeyContract".to_string(),  // For identity operations
            channel_name: "product".to_string(),
            token_collection: "GALA".to_string(),
        }
    }
}

impl GalaChainClient {
    pub fn new(settings: &ApiSettings) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            operations_api: settings.operations_base_url.clone(),
            identity_api: settings.identity_base_url.clone(),
            settings: settings.clone(),
        }
    }

    // Helper method to build the registration URL (uses identity server)
    pub fn get_registration_url(&self) -> String {
        let url = format!("{}{}", self.identity_api, self.settings.registration_endpoint);
        info!("🔗 Built registration URL: {}", url);
        url
    }

    // Helper method to build the registration check URL (GetPublicKey)
    pub fn get_registration_check_url(&self) -> String {
        let endpoint = self.settings.registration_check_endpoint
            .replace("{channel}", &self.settings.channel_name)
            .replace("{contract}", &self.settings.identity_contract_name);
        format!("{}{}", self.operations_api, endpoint)  // Use operations API for GetPublicKey
    }

    // Helper method to build the balance URL
    pub fn get_balance_url(&self) -> String {
        let endpoint = self.settings.balance_endpoint
            .replace("{channel}", &self.settings.channel_name)
            .replace("{contract}", &self.settings.contract_name);
        format!("{}{}", self.identity_api, endpoint)
    }

    // Helper method for retry logic
    async fn retry_request<F, Fut, T>(&self, operation: F, max_retries: u32) -> Result<T, GalaChainError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, GalaChainError>>,
    {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        // Simple delay without Tokio dependency
                        let delay_ms = (1000 << attempt) as u64; // 1s, 2s, 4s in milliseconds
                        std::thread::sleep(Duration::from_millis(delay_ms));
                        info!("Request failed, retrying in {}ms (attempt {}/{})", delay_ms, attempt + 1, max_retries + 1);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    // Blocking wrapper for HTTP requests that creates its own Tokio runtime
    fn run_with_tokio<F, R>(&self, future: F) -> R
    where
        F: std::future::Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        // Create a new Tokio runtime for this operation
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(future)
    }

    // Check if user is registered with GalaChain by attempting a test operation
    // Note: The server doesn't have a direct check endpoint, so we use balance fetch as a proxy
    pub fn check_registration_blocking(&self, gala_address: &str) -> Result<bool, GalaChainError> {
        let client = self.clone();
        let address = gala_address.to_string();
        self.run_with_tokio(async move {
            client.check_registration_async(address).await
        })
    }

    async fn check_registration_async(&self, gala_address: String) -> Result<bool, GalaChainError> {
        let request = PublicKeyRequest {
            user: gala_address.clone(),
        };

        let url = self.get_registration_check_url();
        let request_body = serde_json::to_string_pretty(&request).unwrap_or_default();

        info!("🔍 Checking registration with GetPublicKey for: {}", gala_address);
        info!("📍 Request URL: {}", url);
        info!("📤 Request Body: {}", request_body);

        self.retry_request(|| async {
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        GalaChainError::Network("Request timeout".to_string())
                    } else if e.is_connect() {
                        GalaChainError::Network(format!("Connection failed: {}", e))
                    } else {
                        GalaChainError::Network(e.to_string())
                    }
                })?;

            let status_code = response.status();
            let response_body = response.text().await.unwrap_or_default();

            info!("📡 GetPublicKey Response Status: {}", status_code);
            info!("📥 Response Body: {}", response_body);

            if status_code.is_success() {
                // Parse the GetPublicKey response
                let get_pk_response: GetPublicKeyResponse = serde_json::from_str(&response_body)
                    .map_err(|e| GalaChainError::Parse(format!("Failed to parse GetPublicKey response: {}", e)))?;

                info!("📋 Parsed GetPublicKey response - Status: {}, Has Data: {}",
                      get_pk_response.status, get_pk_response.data.is_some());

                // Status 1 means success, and if we have data, user is registered
                if get_pk_response.status == 1 && get_pk_response.data.is_some() {
                    info!("✅ User is registered!");
                    Ok(true)
                } else {
                    info!("❌ User is not registered (Status: {}, Data present: {})",
                          get_pk_response.status, get_pk_response.data.is_some());
                    Ok(false)
                }
            } else if status_code == 404 {
                // 404 means user not found, so not registered
                Ok(false)
            } else {
                info!("⚠️ GetPublicKey failed - Status: {}, Body: {}", status_code, response_body);

                // Check if the error indicates user doesn't exist
                if response_body.contains("not found") || response_body.contains("does not exist") || status_code == 400 {
                    info!("👤 User not found - treating as not registered");
                    Ok(false)
                } else {
                    Err(GalaChainError::Api(format!(
                        "Registration check failed with status {}: {}",
                        status_code,
                        response_body
                    )))
                }
            }
        }, 2).await // Use fewer retries for registration checks
    }

    // Register user with GalaChain (blocking version)
    pub fn register_user_blocking(&self, public_key: &str) -> Result<(), GalaChainError> {
        let client = self.clone();
        let key = public_key.to_string();
        self.run_with_tokio(async move {
            client.register_user_async(key).await
        })
    }

    async fn register_user_async(&self, public_key: String) -> Result<(), GalaChainError> {
        let url = self.get_registration_url();
        let request_body = serde_json::json!({
            "publicKey": public_key
        });
        let request_body_str = serde_json::to_string_pretty(&request_body).unwrap_or_default();

        info!("🔐 Registering user with RegisterEthUser");
        info!("📍 Request URL: {}", url);
        info!("📤 Request Body: {}", request_body_str);

        self.retry_request(|| async {
            let response = self
                .client
                .post(&url)
                .json(&request_body)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        GalaChainError::Network("Request timeout".to_string())
                    } else if e.is_connect() {
                        GalaChainError::Network(format!("Connection failed: {}", e))
                    } else {
                        GalaChainError::Network(e.to_string())
                    }
                })?;

            let status_code = response.status();
            let response_body = response.text().await.unwrap_or_default();

            info!("📡 RegisterEthUser Response Status: {}", status_code);
            info!("📥 Response Body: {}", response_body);

            if status_code.is_success() {
                info!("✅ User registration successful!");
                Ok(())
            } else {
                error!("❌ Registration failed with status {}: {}", status_code, response_body);
                Err(GalaChainError::Api(format!(
                    "Registration failed with status {}: {}",
                    status_code,
                    response_body
                )))
            }
        }, 3).await
    }

    // Get token balance (blocking version)
    pub fn get_gala_balance_blocking(&self, gala_address: &str) -> Result<(f64, f64), GalaChainError> {
        let client = self.clone();
        let address = gala_address.to_string();
        self.run_with_tokio(async move {
            client.get_gala_balance_async(address).await
        })
    }

    async fn get_gala_balance_async(&self, gala_address: String) -> Result<(f64, f64), GalaChainError> {
        let request = BalanceRequest {
            owner: gala_address.clone(),
            collection: self.settings.token_collection.clone(),
            category: "Unit".to_string(),
            r#type: "none".to_string(),
            additional_key: "none".to_string(),
            instance: "0".to_string(),
        };

        let url = self.get_balance_url();
        let request_body_str = serde_json::to_string_pretty(&request).unwrap_or_default();

        info!("💰 Fetching balance with FetchBalances for: {}", gala_address);
        info!("📍 Request URL: {}", url);
        info!("📤 Request Body: {}", request_body_str);

        self.retry_request(|| async {
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        GalaChainError::Network("Balance request timeout".to_string())
                    } else if e.is_connect() {
                        GalaChainError::Network(format!("Failed to connect to identity API: {}", e))
                    } else {
                        GalaChainError::Network(e.to_string())
                    }
                })?;

            let status_code = response.status();
            let response_body = response.text().await.unwrap_or_default();

            info!("📡 FetchBalances Response Status: {}", status_code);
            info!("📥 Response Body: {}", response_body);

            if !status_code.is_success() {
                error!("❌ Balance request failed with status {}: {}", status_code, response_body);
                return Err(GalaChainError::Api(format!(
                    "Balance request failed with status {}: {}",
                    status_code,
                    response_body
                )));
            }

            let balance_response: BalanceResponse = serde_json::from_str(&response_body)
                .map_err(|e| GalaChainError::Parse(format!("Failed to parse balance response: {}", e)))?;

            if let Some(balance) = balance_response.data.first() {
                let total = balance.quantity.parse::<f64>()
                    .map_err(|e| GalaChainError::Parse(format!("Invalid balance quantity: {}", e)))?;

                let locked: f64 = balance.locked_holds
                    .iter()
                    .map(|hold| hold.quantity.parse::<f64>().unwrap_or(0.0))
                    .sum();

                let available = total - locked;
                info!("💰 Balance parsed successfully - Available: {}, Locked: {}, Total: {}", available, locked, total);
                Ok((available, locked))
            } else {
                info!("💰 No balance data found - returning 0.0");
                Ok((0.0, 0.0))
            }
        }, 3).await
    }

    // Convert Ethereum address to GalaChain format with proper checksumming
    pub fn ethereum_to_galachain_address(eth_address: &str) -> String {
        let addr = if eth_address.starts_with("0x") {
            &eth_address[2..]
        } else {
            eth_address
        };

        // Apply EIP-55 checksumming
        let checksummed = Self::to_checksum_address(addr);
        format!("eth|{}", checksummed)
    }

    // EIP-55 Ethereum address checksumming
    fn to_checksum_address(address: &str) -> String {
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

    // Get public key from private key
    pub fn get_public_key_from_private(private_key: &SecretKey) -> String {
        let secp = secp256k1::Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, private_key);
        hex::encode(public_key.serialize_uncompressed())
    }
}

impl Default for GalaChainClient {
    fn default() -> Self {
        Self::new(&ApiSettings::default())
    }
}

// UI Components
#[derive(Component)]
struct MainMenuButton(MainMenuAction);

#[derive(Component)]
struct WalletMenuButton(WalletMenuAction);

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct MenuTitle;

#[derive(Component)]
struct ContentArea;

// Menu Actions
#[derive(Debug, Clone)]
enum MainMenuAction {
    Wallet,
    Settings,
    Info,
    Exit,
}

#[derive(Debug, Clone)]
enum WalletMenuAction {
    Overview,
    Generate,
    Import,
    Export,
    Registration,
    Balance,
    Transfer,
    Burn,
}

// Legacy Components (to be refactored)
#[derive(Component)]
struct WalletButton;

#[derive(Component)]
struct AddressText;

#[derive(Component)]
struct MnemonicText;

#[derive(Component)]
struct ImportText;

#[derive(Component)]
struct ImportButton;

#[derive(Component)]
struct ExportButton;

#[derive(Component)]
struct WordInput(usize);

#[derive(Component)]
struct WordText;

#[derive(Component)]
struct ImportConfirmButton;

#[derive(Resource)]
struct WalletData {
    private_key: Option<SecretKey>,
    address: Option<String>,
    mnemonic: Option<String>,
    show_mnemonic: bool,
    show_import: bool,
    import_words: Vec<String>,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.35, 0.35);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .init_state::<WalletState>()
        .add_plugins(MenuPlugin)
        .add_plugins(WalletPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    // UI Camera
    commands.spawn(Camera2d);

    // Root node
    commands
        .spawn((Node {
            display: Display::Flex,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        }, BackgroundColor(Color::srgb(0.1, 0.1, 0.1))))
        .with_children(|parent| {
            // Address Text
            parent.spawn((Text::new("Address: None"), AddressText));

            // Mnemonic Text
            parent.spawn((Text::new(""), MnemonicText));

            // Buttons container
            parent
                .spawn((Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                }, BackgroundColor(Color::NONE)))
                .with_children(|parent| {
                    // Generate Wallet Button
                    parent
                        .spawn((Button, WalletButton, Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::horizontal(Val::Px(5.0)),
                            ..default()
                          },
                          BorderColor(Color::BLACK),
                          BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                          BackgroundColor(NORMAL_BUTTON)
                        ))
                        .with_child(Text::new("Generate Wallet"),);

                    // Import Wallet Button
                    parent
                        .spawn((
                            Button,
                            ImportButton,
                            Node {
                              width: Val::Px(200.0),
                              height: Val::Px(50.0),
                              border: UiRect::all(Val::Px(2.0)),
                              justify_content: JustifyContent::Center,
                              align_items: AlignItems::Center,
                              margin: UiRect::horizontal(Val::Px(5.0)),
                              ..default()
                            },
                            BorderColor(Color::BLACK),
                            BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Import"), ImportText));
                        });

                    // Export Seed Button
                    parent
                        .spawn((
                            Button,
                            ExportButton,
                            Node {
                              width: Val::Px(200.0),
                              height: Val::Px(50.0),
                              border: UiRect::all(Val::Px(2.0)),
                              justify_content: JustifyContent::Center,
                              align_items: AlignItems::Center,
                              margin: UiRect::horizontal(Val::Px(5.0)),
                              ..default()
                            },
                            BorderColor(Color::BLACK),
                            BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                            BackgroundColor(NORMAL_BUTTON)
                        ))
                        .with_child(Text::new("Export Seed Phrase"),);
                });
        });
}

fn generate_wallet_secure(keychain: &KeychainManager) -> Result<(SecretKey, String, String), String> {
    // Generate mnemonic
    let entropy = rand::random::<[u8; 16]>();
    let mnemonic = Mnemonic::from_entropy(&entropy)
        .map_err(|e| format!("Failed to generate mnemonic: {}", e))?;

    let mnemonic_str = mnemonic.to_string();

    // Generate wallet data from mnemonic
    let (private_key, address) = keychain.generate_wallet_from_mnemonic(&mnemonic_str)?;

    // Store in keychain
    let secure_data = SecureWalletData {
        mnemonic: mnemonic_str.clone(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    keychain.store_wallet(&secure_data)
        .map_err(|e| format!("Failed to store wallet: {}", e))?;

    Ok((private_key, address, mnemonic_str))
}

// Legacy function for compatibility
fn generate_wallet() -> (SecretKey, String, String) {
    let mut rng = OsRng;

    // Generate mnemonic
    let entropy = rand::random::<[u8; 16]>();
    let mnemonic = Mnemonic::from_entropy(&entropy).expect("Failed to generate mnemonic");

    // Generate private key
    let private_key = SecretKey::new(&mut rng);

    // Generate public key and address
    let public_key = PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &private_key);
    let public_key_bytes = public_key.serialize_uncompressed();

    // Generate Ethereum address (last 20 bytes of keccak256 of public key)
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]); // Skip recovery id byte
    let result = hasher.finalize();
    let address = format!("0x{}", hex::encode(&result[12..])); // Take last 20 bytes

    (private_key, address, mnemonic.to_string())
}

fn generate_wallet_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<WalletButton>),
    >,
    mut wallet_data: ResMut<WalletData>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<AddressText>>,
        Query<&mut Text, With<MnemonicText>>,
    )>,
    keychain: Res<KeychainManager>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match generate_wallet_secure(&keychain) {
                    Ok((secret_key, address, mnemonic)) => {
                        wallet_data.private_key = Some(secret_key);
                        wallet_data.address = Some(address.clone());
                        wallet_data.mnemonic = Some(mnemonic.clone());
                        wallet_data.show_mnemonic = false;

                        // Update UI text
                        if let Ok(mut text) = text_queries.p0().get_single_mut() {
                            *text = Text::new(format!("Address: {}", address));
                        }
                        if let Ok(mut text) = text_queries.p1().get_single_mut() {
                            *text = Text::new("Wallet generated and stored securely!");
                        }
                        info!("New wallet generated and stored in keychain");
                    }
                    Err(error) => {
                        error!("Failed to generate wallet: {}", error);
                        if let Ok(mut text) = text_queries.p1().get_single_mut() {
                            *text = Text::new(format!("Error: {}", error));
                        }
                    }
                }

                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn export_seed_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &Children),
        (Changed<Interaction>, With<ExportButton>),
    >,
    mut text_queries: ParamSet<(
        Query<&mut Text>,
        Query<&mut Text, With<MnemonicText>>,
    )>,
    mut wallet_data: ResMut<WalletData>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                wallet_data.show_mnemonic = !wallet_data.show_mnemonic;

                // Update button text
                if let Some(child) = children.first() {
                    if let Ok(mut text) = text_queries.p0().get_mut(*child) {
                        *text = Text::new(
                            if wallet_data.show_mnemonic {
                                "Hide Seed Phrase"
                            } else {
                                "Export Seed Phrase"
                            }
                        );
                    }
                }

                // Update mnemonic text
                if let Ok(mut text) = text_queries.p1().get_single_mut() {
                    *text = Text::new(
                        if wallet_data.show_mnemonic {
                            if let Some(mnemonic) = wallet_data.mnemonic.as_ref() {
                                format!("Seed Phrase: {}", mnemonic)
                            } else {
                                "No wallet generated yet".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    );
                }

                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn import_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ImportButton>),
    >,
    mut wallet_data: ResMut<WalletData>,
    mut commands: Commands,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                wallet_data.show_import = !wallet_data.show_import;

                if wallet_data.show_import {
                    // Spawn import form
                    commands
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Word input fields
                            for i in 0..12 {
                                parent
                                    .spawn((Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(30.0),
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },))
                                    .with_children(|parent| {
                                        parent.spawn((Text::new(format!("Word {}: ", i + 1)),));
                                        parent
                                            .spawn((
                                                Button,
                                                WordInput(i),
                                                Node {
                                                    width: Val::Px(150.0),
                                                    height: Val::Px(30.0),
                                                    border: UiRect::all(Val::Px(1.0)),
                                                    ..default()
                                                },
                                                BorderColor(Color::WHITE),
                                                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((Text::new(""), WordText));
                                            });
                                    });
                            }

                            // Confirm button
                            parent
                                .spawn((Button, ImportConfirmButton, Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(50.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                }, BorderColor(Color::BLACK), BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)), BackgroundColor(NORMAL_BUTTON)))
                                .with_child((Text::new("Import"),));
                        });
                } else {
                    // TODO: Clean up import form entities
                }

                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn import_word_system(
    mut interaction_query: Query<(&Interaction, &WordInput, &Children), Changed<Interaction>>,
    mut text_query: Query<&mut Text>,
    mut wallet_data: ResMut<WalletData>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (interaction, word_input, children) in &mut interaction_query {
        if let Interaction::Pressed = interaction {
            let word_index = word_input.0;
            let mut current_word = wallet_data.import_words[word_index].clone();

            // Handle backspace
            if keyboard_input.just_pressed(KeyCode::Backspace) || keyboard_input.just_pressed(KeyCode::Delete) {
                current_word.pop();
            }
            // Handle space
            else if keyboard_input.just_pressed(KeyCode::Space) {
                current_word.push(' ');
            }
            // Handle letters
            else {
                for key_code in keyboard_input.get_just_pressed() {
                    match key_code {
                        KeyCode::KeyA => current_word.push('a'),
                        KeyCode::KeyB => current_word.push('b'),
                        KeyCode::KeyC => current_word.push('c'),
                        KeyCode::KeyD => current_word.push('d'),
                        KeyCode::KeyE => current_word.push('e'),
                        KeyCode::KeyF => current_word.push('f'),
                        KeyCode::KeyG => current_word.push('g'),
                        KeyCode::KeyH => current_word.push('h'),
                        KeyCode::KeyI => current_word.push('i'),
                        KeyCode::KeyJ => current_word.push('j'),
                        KeyCode::KeyK => current_word.push('k'),
                        KeyCode::KeyL => current_word.push('l'),
                        KeyCode::KeyM => current_word.push('m'),
                        KeyCode::KeyN => current_word.push('n'),
                        KeyCode::KeyO => current_word.push('o'),
                        KeyCode::KeyP => current_word.push('p'),
                        KeyCode::KeyQ => current_word.push('q'),
                        KeyCode::KeyR => current_word.push('r'),
                        KeyCode::KeyS => current_word.push('s'),
                        KeyCode::KeyT => current_word.push('t'),
                        KeyCode::KeyU => current_word.push('u'),
                        KeyCode::KeyV => current_word.push('v'),
                        KeyCode::KeyW => current_word.push('w'),
                        KeyCode::KeyX => current_word.push('x'),
                        KeyCode::KeyY => current_word.push('y'),
                        KeyCode::KeyZ => current_word.push('z'),
                        _ => {}
                    }
                }
            }

            wallet_data.import_words[word_index] = current_word.clone();

            // Update text display
            if let Some(child) = children.first() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    *text = Text::new(current_word);
                }
            }
        }
    }
}

fn import_confirm_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ImportConfirmButton>),
    >,
    mut wallet_data: ResMut<WalletData>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<AddressText>>,
        Query<&mut Text, With<MnemonicText>>,
    )>,
    keychain: Res<KeychainManager>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mnemonic_string = wallet_data.import_words.join(" ");

                match keychain.generate_wallet_from_mnemonic(&mnemonic_string) {
                    Ok((secret_key, address)) => {
                        // Store in keychain
                        let secure_data = SecureWalletData {
                            mnemonic: mnemonic_string.clone(),
                            created_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };

                        match keychain.store_wallet(&secure_data) {
                            Ok(_) => {
                                // Update wallet state
                                wallet_data.private_key = Some(secret_key);
                                wallet_data.address = Some(address.clone());
                                wallet_data.mnemonic = Some(mnemonic_string);
                                wallet_data.show_import = false; // Hide import form

                                // Update UI
                                if let Ok(mut text) = text_queries.p0().get_single_mut() {
                                    *text = Text::new(format!("Address: {}", address));
                                }
                                if let Ok(mut text) = text_queries.p1().get_single_mut() {
                                    *text = Text::new("Wallet imported and stored securely!");
                                }
                                info!("Wallet imported and stored in keychain");
                            }
                            Err(e) => {
                                error!("Failed to store imported wallet: {}", e);
                                if let Ok(mut text) = text_queries.p1().get_single_mut() {
                                    *text = Text::new(format!("Storage error: {}", e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to import wallet: {}", e);
                        if let Ok(mut text) = text_queries.p1().get_single_mut() {
                            *text = Text::new(format!("Import error: {}", e));
                        }
                    }
                }

                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

// Menu Plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let api_settings = ApiSettings::default();
        app.insert_resource(api_settings.clone())
            .insert_resource(KeychainManager::new())
            .insert_resource(GalaChainClient::new(&api_settings))
            .insert_resource(BalanceState::default())
            .insert_resource(RegistrationState::default())
            .insert_resource(AsyncTasks::default())
            .insert_resource(ImportState::default())
            .insert_resource(ExportState::default())
            .insert_resource(TransferState::default())
            .insert_resource(BurnState::default())
            .insert_resource(FocusedInput::default())
            .add_systems(Startup, setup_main_menu)
            .add_systems(
                Update,
                (
                    main_menu_system.run_if(in_state(AppState::MainMenu)),
                    wallet_menu_system.run_if(in_state(AppState::WalletMenu)),
                    back_button_system, // Run back button system in all states
                    async_task_polling_system, // Run async polling in all states
                    wallet_generate_system.run_if(in_state(WalletState::Generate)),
                    wallet_import_system.run_if(in_state(WalletState::Import)),
                    wallet_export_system.run_if(in_state(WalletState::Export)),
                    wallet_registration_ui_system.run_if(in_state(WalletState::Registration)),
                    wallet_balance_system.run_if(in_state(WalletState::Balance)),
                    wallet_transfer_system.run_if(in_state(WalletState::Transfer)),
                    wallet_burn_system.run_if(in_state(WalletState::Burn)),
                ),
            )
            .add_systems(OnEnter(AppState::MainMenu), show_main_menu)
            .add_systems(OnExit(AppState::MainMenu), cleanup_menu)
            .add_systems(OnEnter(AppState::WalletMenu), show_wallet_menu)
            .add_systems(OnExit(AppState::WalletMenu), cleanup_menu)
            .add_systems(OnEnter(AppState::Settings), show_settings)
            .add_systems(OnExit(AppState::Settings), cleanup_menu)
            .add_systems(OnEnter(AppState::Info), show_info)
            .add_systems(OnExit(AppState::Info), cleanup_menu)
            .add_systems(Startup, load_wallet_from_keychain.after(setup_main_menu));
    }
}

fn setup_main_menu(mut commands: Commands) {
    // UI Camera
    commands.spawn(Camera2d);
}

fn load_wallet_from_keychain(
    mut wallet_data: ResMut<WalletData>,
    keychain: Res<KeychainManager>,
) {
    match keychain.load_wallet() {
        Ok(secure_data) => {
            match keychain.generate_wallet_from_mnemonic(&secure_data.mnemonic) {
                Ok((secret_key, address)) => {
                    wallet_data.private_key = Some(secret_key);
                    wallet_data.address = Some(address.clone());
                    wallet_data.mnemonic = Some(secure_data.mnemonic);

                    info!("Wallet loaded from keychain: {}", address);
                }
                Err(e) => {
                    error!("Failed to derive wallet from stored mnemonic: {}", e);
                }
            }
        }
        Err(KeychainError::NotFound) => {
            info!("No wallet found in keychain - user will need to generate or import one");
        }
        Err(e) => {
            error!("Error accessing keychain: {}", e);
        }
    }
}

fn show_main_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuTitle,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("GalaChain Desktop Wallet"), MenuTitle));

            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    create_menu_button(parent, "Wallet", MainMenuAction::Wallet);
                    create_menu_button(parent, "Settings", MainMenuAction::Settings);
                    create_menu_button(parent, "Info", MainMenuAction::Info);
                    create_menu_button(parent, "Exit", MainMenuAction::Exit);
                });
        });
}

fn show_wallet_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuTitle,
        ))
        .with_children(|parent| {
            // Left sidebar with menu buttons
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        width: Val::Percent(30.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::right(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Wallet Operations"),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    create_wallet_menu_button(parent, "Overview", WalletMenuAction::Overview);
                    create_wallet_menu_button(parent, "Generate Wallet", WalletMenuAction::Generate);
                    create_wallet_menu_button(parent, "Import Wallet", WalletMenuAction::Import);
                    create_wallet_menu_button(parent, "Export Seed", WalletMenuAction::Export);
                    create_wallet_menu_button(parent, "Registration", WalletMenuAction::Registration);
                    create_wallet_menu_button(parent, "Check Balance", WalletMenuAction::Balance);
                    create_wallet_menu_button(parent, "Transfer", WalletMenuAction::Transfer);
                    create_wallet_menu_button(parent, "Burn Tokens", WalletMenuAction::Burn);

                    // Back button
                    parent
                        .spawn((
                            Button,
                            BackButton,
                            Node {
                                width: Val::Px(180.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(20.0)),
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                            BackgroundColor(Color::srgb(0.6, 0.15, 0.15)),
                        ))
                        .with_child(Text::new("← Back to Main"));
                });

            // Right content area
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        width: Val::Percent(70.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    ContentArea,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Select an operation from the menu on the left"),
                        Node {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                    ));
                });
        });
}

fn show_settings(mut commands: Commands, api_settings: Res<ApiSettings>) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuTitle,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("API Settings"), MenuTitle));

            parent.spawn((
                Text::new("Configure GalaChain API endpoints for HTTP integration:"),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Operations API Setting
            parent.spawn((
                Text::new("GalaChain Operations API Base URL:"),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));

            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        max_width: Val::Px(400.0),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.7, 0.7, 0.7)),
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                ))
                .with_child(Text::new(&api_settings.operations_base_url));

            // Identity API Setting
            parent.spawn((
                Text::new("GalaChain Identity Registration Base URL:"),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));

            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        max_width: Val::Px(400.0),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.7, 0.7, 0.7)),
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                ))
                .with_child(Text::new(&api_settings.identity_base_url));

            parent.spawn((
                Text::new("💡 Note: These endpoints are currently using default localhost values.\nIn a production app, these would be configurable via UI inputs."),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    max_width: Val::Px(500.0),
                    ..default()
                },
            ));

            // Back button
            parent
                .spawn((
                    Button,
                    BackButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                    BackgroundColor(Color::srgb(0.6, 0.15, 0.15)),
                ))
                .with_child(Text::new("Back"));
        });
}

fn show_info(mut commands: Commands) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MenuTitle,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("About GalaChain Desktop Wallet"), MenuTitle));

            parent.spawn((
                Text::new("This application demonstrates how to integrate GalaChain\nfunctionality into a desktop application using the Bevy game engine.\n\nKey features:\n• Secure local wallet storage using OS keychain\n• GalaChain user registration and authentication\n• Token balance queries and transactions\n• Cross-platform compatibility\n\nThis serves as a reference implementation for developers\nwho want to build desktop applications or games that\nintegrate with the GalaChain ecosystem."),
                Node {
                    max_width: Val::Px(600.0),
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Back button
            parent
                .spawn((
                    Button,
                    BackButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                    BackgroundColor(Color::srgb(0.6, 0.15, 0.15)),
                ))
                .with_child(Text::new("Back"));
        });
}

fn create_menu_button(parent: &mut ChildBuilder, text: &str, action: MainMenuAction) {
    parent
        .spawn((
            Button,
            MainMenuButton(action),
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
            BackgroundColor(NORMAL_BUTTON),
        ))
        .with_child(Text::new(text));
}

fn create_wallet_menu_button(parent: &mut ChildBuilder, text: &str, action: WalletMenuAction) {
    parent
        .spawn((
            Button,
            WalletMenuButton(action),
            Node {
                width: Val::Px(180.0),
                height: Val::Px(40.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::new(Val::Px(3.0), Val::Px(3.0), Val::Px(3.0), Val::Px(3.0)),
            BackgroundColor(NORMAL_BUTTON),
        ))
        .with_child(Text::new(text));
}

fn main_menu_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &MainMenuButton),
        (Changed<Interaction>, With<MainMenuButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    for (interaction, mut color, mut border_color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match &button.0 {
                    MainMenuAction::Wallet => next_state.set(AppState::WalletMenu),
                    MainMenuAction::Settings => next_state.set(AppState::Settings),
                    MainMenuAction::Info => next_state.set(AppState::Info),
                    MainMenuAction::Exit => {
                        exit.send(bevy::app::AppExit::Success);
                    }
                }
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn wallet_menu_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &WalletMenuButton),
        (Changed<Interaction>, With<WalletMenuButton>),
    >,
    mut next_wallet_state: ResMut<NextState<WalletState>>,
) {
    for (interaction, mut color, mut border_color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                match &button.0 {
                    WalletMenuAction::Overview => next_wallet_state.set(WalletState::Overview),
                    WalletMenuAction::Generate => next_wallet_state.set(WalletState::Generate),
                    WalletMenuAction::Import => next_wallet_state.set(WalletState::Import),
                    WalletMenuAction::Export => next_wallet_state.set(WalletState::Export),
                    WalletMenuAction::Registration => next_wallet_state.set(WalletState::Registration),
                    WalletMenuAction::Balance => next_wallet_state.set(WalletState::Balance),
                    WalletMenuAction::Transfer => next_wallet_state.set(WalletState::Transfer),
                    WalletMenuAction::Burn => next_wallet_state.set(WalletState::Burn),
                }
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn back_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<BackButton>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(AppState::MainMenu);
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.6, 0.15, 0.15).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuTitle>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn wallet_overview_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    wallet_data: Res<WalletData>,
    keychain: Res<KeychainManager>,
    query: Query<Entity, With<ContentArea>>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Overview {
        // Update the content area, not replace the whole UI
        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                // Show wallet overview
                let has_wallet = wallet_data.address.is_some();
                let keychain_status = if keychain.wallet_exists() {
                    "✓ Stored in OS keychain"
                } else {
                    "✗ Not stored in keychain"
                };

                parent.spawn((
                    Text::new("Wallet Overview"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if has_wallet {
                    if let Some(address) = &wallet_data.address {
                        parent.spawn((
                            Text::new(format!("Address: {}", address)),
                            Node {
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                        ));
                    }

                    parent.spawn((
                        Text::new(format!("Security: {}", keychain_status)),
                        Node {
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Text::new("Status: ✓ Wallet Active"),
                        Node {
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                } else {
                    parent.spawn((
                        Text::new("No wallet found. Please generate or import a wallet."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }

                parent.spawn((
                    Text::new("\nUse the menu buttons on the left to:\n• Generate a new wallet\n• Import an existing wallet\n• Export your seed phrase\n• Check token balances\n• Transfer or burn tokens"),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }
}

#[derive(Component)]
struct RefreshBalanceButton;

#[derive(Component)]
struct CheckRegistrationButton;

#[derive(Component)]
struct RegisterIdentityButton;

#[derive(Resource)]
struct BalanceState {
    loading: bool,
    available: f64,
    locked: f64,
    error: Option<String>,
    last_updated: Option<std::time::SystemTime>,
}

impl Default for BalanceState {
    fn default() -> Self {
        Self {
            loading: false,
            available: 0.0,
            locked: 0.0,
            error: None,
            last_updated: None,
        }
    }
}

#[derive(Resource)]
struct RegistrationState {
    checking: bool,
    registering: bool,
    is_registered: Option<bool>,
    error: Option<String>,
    last_checked: Option<std::time::SystemTime>,
}

impl Default for RegistrationState {
    fn default() -> Self {
        Self {
            checking: false,
            registering: false,
            is_registered: None,
            error: None,
            last_checked: None,
        }
    }
}

#[derive(Resource)]
struct AsyncTasks {
    balance_task: Option<bevy::tasks::Task<Result<(f64, f64), GalaChainError>>>,
    registration_check_task: Option<bevy::tasks::Task<Result<bool, GalaChainError>>>,
    registration_task: Option<bevy::tasks::Task<Result<(), GalaChainError>>>,
}

impl Default for AsyncTasks {
    fn default() -> Self {
        Self {
            balance_task: None,
            registration_check_task: None,
            registration_task: None,
        }
    }
}

fn wallet_balance_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    wallet_data: Res<WalletData>,
    mut balance_state: ResMut<BalanceState>,
    mut async_tasks: ResMut<AsyncTasks>,
    query: Query<Entity, With<ContentArea>>,
    mut refresh_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<RefreshBalanceButton>),
    >,
    galachain_client: Res<GalaChainClient>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Balance {
        // Reset balance state when entering balance view
        balance_state.loading = false;
        balance_state.error = None;

        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("GALA Token Balance"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if let Some(address) = &wallet_data.address {
                    parent.spawn((
                        Text::new(format!("Wallet Address: {}", address)),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    // Balance display
                    if balance_state.loading {
                        parent.spawn((
                            Text::new("🔄 Loading balance..."),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    } else if let Some(error) = &balance_state.error {
                        parent.spawn((
                            Text::new(format!("❌ Error: {}", error)),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    } else if balance_state.last_updated.is_some() {
                        parent.spawn((
                            Text::new(format!("Available: {:.2} GALA", balance_state.available)),
                            Node {
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                        ));

                        if balance_state.locked > 0.0 {
                            parent.spawn((
                                Text::new(format!("Locked: {:.2} GALA", balance_state.locked)),
                                Node {
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                            ));
                        }

                        parent.spawn((
                            Text::new(format!("Total: {:.2} GALA", balance_state.available + balance_state.locked)),
                            Node {
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                        ));

                        if let Some(last_updated) = balance_state.last_updated {
                            if let Ok(elapsed) = last_updated.elapsed() {
                                parent.spawn((
                                    Text::new(format!("Last updated: {:.0} seconds ago", elapsed.as_secs())),
                                    Node {
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    } else {
                        parent.spawn((
                            Text::new("Click 'Refresh Balance' to fetch your GALA balance"),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    }

                    // Refresh button
                    parent
                        .spawn((
                            Button,
                            RefreshBalanceButton,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(20.0)),
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                            BackgroundColor(if balance_state.loading {
                                Color::srgb(0.3, 0.3, 0.3)
                            } else {
                                Color::srgb(0.2, 0.7, 0.2)
                            }),
                        ))
                        .with_child(Text::new(if balance_state.loading {
                            "Loading..."
                        } else {
                            "Refresh Balance"
                        }));

                    parent.spawn((
                        Text::new("💡 This will make an HTTP call to your configured GalaChain Operations API endpoint"),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            max_width: Val::Px(500.0),
                            ..default()
                        },
                    ));
                } else {
                    parent.spawn((
                        Text::new("❌ No wallet available.\nPlease generate or import a wallet first."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }
            });
        }
    }

    // Handle refresh button clicks
    for (interaction, mut color, mut border_color) in &mut refresh_button_query {
        match *interaction {
            Interaction::Pressed => {
                if !balance_state.loading {
                    if let Some(address) = &wallet_data.address {
                        balance_state.loading = true;
                        balance_state.error = None;

                        // Spawn async task to fetch balance
                        let client = galachain_client.clone();
                        let gala_address = GalaChainClient::ethereum_to_galachain_address(address);

                        info!("Balance refresh requested for address: {}", gala_address);
                        info!("Calling: {}/api/product/FetchBalances", client.operations_api);

                        // Spawn task using blocking method
                        info!("Creating balance task for address: {}", gala_address);
                        async_tasks.balance_task = Some(bevy::tasks::IoTaskPool::get().spawn(async move {
                            info!("Balance task executing HTTP request to: {}", client.get_balance_url());
                            let result = client.get_gala_balance_blocking(&gala_address);
                            info!("Balance task completed with result: {:?}", result);
                            result
                        }));
                    }
                }

                *color = Color::srgb(0.1, 0.5, 0.1).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.8, 0.3).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.7, 0.2).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn wallet_registration_system(
    wallet_data: Res<WalletData>,
    galachain_client: Res<GalaChainClient>,
    mut registration_task: Local<Option<bevy::tasks::Task<Result<(), GalaChainError>>>>,
    mut registration_check_task: Local<Option<bevy::tasks::Task<Result<bool, GalaChainError>>>>,
    mut last_address: Local<Option<String>>,
) {
    // Check if we have a new wallet address
    if let Some(address) = &wallet_data.address {
        if last_address.as_ref() != Some(address) {
            *last_address = Some(address.clone());

            // Check registration status
            let gala_address = GalaChainClient::ethereum_to_galachain_address(address);
            let client = (*galachain_client).clone();
            let address_clone = gala_address.clone();

            *registration_check_task = Some(bevy::tasks::IoTaskPool::get().spawn(async move {
                client.check_registration_blocking(&address_clone)
            }));
        }
    }

    // Check registration status result
    if let Some(task) = registration_check_task.as_mut() {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            *registration_check_task = None;

            match result {
                Ok(is_registered) => {
                    if !is_registered && wallet_data.private_key.is_some() {
                        // Auto-register the user
                        info!("User not registered, attempting auto-registration...");

                        let private_key = wallet_data.private_key.as_ref().unwrap();
                        let public_key = GalaChainClient::get_public_key_from_private(private_key);
                        let client = (*galachain_client).clone();

                        *registration_task = Some(bevy::tasks::IoTaskPool::get().spawn(async move {
                            client.register_user_blocking(&public_key)
                        }));
                    } else if is_registered {
                        info!("User is already registered with GalaChain");
                    }
                }
                Err(e) => {
                    error!("Failed to check registration status: {}", e);
                }
            }
        }
    }

    // Check registration result
    if let Some(task) = registration_task.as_mut() {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            *registration_task = None;

            match result {
                Ok(_) => {
                    info!("User successfully registered with GalaChain");
                }
                Err(e) => {
                    error!("Failed to register user with GalaChain: {}", e);
                }
            }
        }
    }
}

fn wallet_registration_ui_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
    wallet_data: Res<WalletData>,
    mut registration_state: ResMut<RegistrationState>,
    mut async_tasks: ResMut<AsyncTasks>,
    mut check_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<CheckRegistrationButton>),
    >,
    mut register_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<RegisterIdentityButton>, Without<CheckRegistrationButton>),
    >,
    galachain_client: Res<GalaChainClient>,
) {
    // Show registration UI when state changes or registration state updates
    let entering_registration = wallet_state.is_changed() && *wallet_state.get() == WalletState::Registration;
    let registration_state_changed = registration_state.is_changed() && *wallet_state.get() == WalletState::Registration;

    if entering_registration {
        // Reset registration state when entering registration view
        registration_state.checking = false;
        registration_state.registering = false;
        registration_state.error = None;
    }

    if entering_registration || registration_state_changed {

        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("Identity Registration"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if let Some(address) = &wallet_data.address {
                    parent.spawn((
                        Text::new(format!("Wallet Address: {}", address)),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    let gala_address = GalaChainClient::ethereum_to_galachain_address(address);
                    parent.spawn((
                        Text::new(format!("GalaChain Address: {}", gala_address)),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    // Registration status display
                    if registration_state.checking {
                        parent.spawn((
                            Text::new("🔄 Checking registration status..."),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    } else if registration_state.registering {
                        parent.spawn((
                            Text::new("🔄 Registering identity..."),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    } else if let Some(error) = &registration_state.error {
                        parent.spawn((
                            Text::new(format!("❌ Error: {}", error)),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    } else if let Some(is_registered) = registration_state.is_registered {
                        if is_registered {
                            parent.spawn((
                                Text::new("✅ Identity is registered with GalaChain"),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                            ));
                        } else {
                            parent.spawn((
                                Text::new("❌ Identity is NOT registered with GalaChain"),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                            ));
                        }

                        if let Some(last_checked) = registration_state.last_checked {
                            if let Ok(elapsed) = last_checked.elapsed() {
                                parent.spawn((
                                    Text::new(format!("Last checked: {:.0} seconds ago", elapsed.as_secs())),
                                    Node {
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    } else {
                        parent.spawn((
                            Text::new("Click 'Check Registration' to verify your identity status"),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));
                    }

                    // Check Registration button
                    parent
                        .spawn((
                            Button,
                            CheckRegistrationButton,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                            BackgroundColor(if registration_state.checking {
                                Color::srgb(0.3, 0.3, 0.3)
                            } else {
                                Color::srgb(0.2, 0.2, 0.7)
                            }),
                        ))
                        .with_child(Text::new(if registration_state.checking {
                            "Checking..."
                        } else {
                            "Check Registration"
                        }));

                    // Register Identity button (only show if not registered or if registration failed)
                    if let Some(is_registered) = registration_state.is_registered {
                        if !is_registered {
                            parent
                                .spawn((
                                    Button,
                                    RegisterIdentityButton,
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(50.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                    BorderColor(Color::BLACK),
                                    BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                                    BackgroundColor(if registration_state.registering {
                                        Color::srgb(0.3, 0.3, 0.3)
                                    } else {
                                        Color::srgb(0.2, 0.7, 0.2)
                                    }),
                                ))
                                .with_child(Text::new(if registration_state.registering {
                                    "Registering..."
                                } else {
                                    "Register Identity"
                                }));
                        }
                    }

                    parent.spawn((
                        Text::new("💡 Registration allows your wallet to interact with the GalaChain network.\nCheck your status first, then register if needed."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            max_width: Val::Px(500.0),
                            ..default()
                        },
                    ));
                } else {
                    parent.spawn((
                        Text::new("❌ No wallet available.\nPlease generate or import a wallet first."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }
            });
        }
    }

    // Handle check registration button clicks
    for (interaction, mut color, mut border_color) in &mut check_button_query {
        match *interaction {
            Interaction::Pressed => {
                if !registration_state.checking && !registration_state.registering {
                    if let Some(address) = &wallet_data.address {
                        registration_state.checking = true;
                        registration_state.error = None;
                        registration_state.is_registered = None;

                        let client = galachain_client.clone();
                        let gala_address = GalaChainClient::ethereum_to_galachain_address(address);

                        info!("Checking registration status for address: {}", gala_address);

                        // Spawn task to check registration using blocking method
                        async_tasks.registration_check_task = Some(bevy::tasks::IoTaskPool::get().spawn(async move {
                            client.check_registration_blocking(&gala_address)
                        }));
                    }
                }

                *color = Color::srgb(0.1, 0.1, 0.5).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.3, 0.8).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.2, 0.7).into();
                border_color.0 = Color::BLACK;
            }
        }
    }

    // Handle register identity button clicks
    for (interaction, mut color, mut border_color) in &mut register_button_query {
        match *interaction {
            Interaction::Pressed => {
                if !registration_state.checking && !registration_state.registering {
                    if let Some(private_key) = &wallet_data.private_key {
                        registration_state.registering = true;
                        registration_state.error = None;

                        let public_key = GalaChainClient::get_public_key_from_private(private_key);
                        let client = galachain_client.clone();

                        info!("Registering identity with public key: {}", public_key);

                        // Spawn task to register using blocking method
                        async_tasks.registration_task = Some(bevy::tasks::IoTaskPool::get().spawn(async move {
                            client.register_user_blocking(&public_key)
                        }));
                    }
                }

                *color = Color::srgb(0.1, 0.5, 0.1).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.8, 0.3).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.7, 0.2).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn async_task_polling_system(
    mut async_tasks: ResMut<AsyncTasks>,
    mut balance_state: ResMut<BalanceState>,
    mut registration_state: ResMut<RegistrationState>,
) {
    // Debug: Check if we have any active tasks
    let has_balance_task = async_tasks.balance_task.is_some();
    let has_reg_check_task = async_tasks.registration_check_task.is_some();
    let has_reg_task = async_tasks.registration_task.is_some();

    if has_balance_task || has_reg_check_task || has_reg_task {
        info!("Polling tasks - Balance: {}, RegCheck: {}, Reg: {}", has_balance_task, has_reg_check_task, has_reg_task);
    }

    // Poll balance task
    if let Some(task) = async_tasks.balance_task.as_mut() {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            async_tasks.balance_task = None;
            balance_state.loading = false;

            match result {
                Ok((available, locked)) => {
                    balance_state.available = available;
                    balance_state.locked = locked;
                    balance_state.last_updated = Some(std::time::SystemTime::now());
                    balance_state.error = None;
                    info!("Balance fetched successfully: {:.2} available, {:.2} locked", available, locked);
                }
                Err(e) => {
                    balance_state.error = Some(e.to_string());
                    error!("Failed to fetch balance: {}", e);
                }
            }
        }
    }

    // Poll registration check task
    if let Some(task) = async_tasks.registration_check_task.as_mut() {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            async_tasks.registration_check_task = None;
            registration_state.checking = false;

            info!("Registration check task completed, processing result...");

            match result {
                Ok(is_registered) => {
                    registration_state.is_registered = Some(is_registered);
                    registration_state.last_checked = Some(std::time::SystemTime::now());
                    registration_state.error = None;
                    info!("✅ Registration check completed: {}", if is_registered { "registered" } else { "not registered" });
                }
                Err(e) => {
                    registration_state.error = Some(e.to_string());
                    error!("❌ Registration check failed: {}", e);
                }
            }
        }
    }

    // Poll registration task
    if let Some(task) = async_tasks.registration_task.as_mut() {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            async_tasks.registration_task = None;
            registration_state.registering = false;

            match result {
                Ok(_) => {
                    registration_state.is_registered = Some(true);
                    registration_state.last_checked = Some(std::time::SystemTime::now());
                    registration_state.error = None;
                    info!("Identity registration completed successfully");
                }
                Err(e) => {
                    registration_state.error = Some(e.to_string());
                    error!("Failed to register identity: {}", e);
                }
            }
        }
    }
}

// New component for generate button
#[derive(Component)]
struct GenerateWalletButton;

fn wallet_generate_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
    mut wallet_data: ResMut<WalletData>,
    keychain: Res<KeychainManager>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<GenerateWalletButton>),
    >,
) {
    // Show generate wallet UI when state changes
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Generate {
        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("Generate New Wallet"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if wallet_data.address.is_some() {
                    parent.spawn((
                        Text::new("⚠️ WARNING: You already have a wallet!\nGenerating a new wallet will replace your current one.\nMake sure you have backed up your current seed phrase."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                } else {
                    parent.spawn((
                        Text::new("This will create a new wallet with a secure 12-word seed phrase.\nThe wallet will be stored securely in your OS keychain."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }

                // Generate button
                parent
                    .spawn((
                        Button,
                        GenerateWalletButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                        BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
                    ))
                    .with_child(Text::new("Generate New Wallet"));

                if wallet_data.address.is_some() {
                    parent.spawn((
                        Text::new("\nNote: Your new wallet will be automatically registered with GalaChain."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }
            });
        }
    }

    // Handle generate button interactions
    for (interaction, mut color, mut border_color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                match generate_wallet_secure(&keychain) {
                    Ok((secret_key, address, mnemonic)) => {
                        wallet_data.private_key = Some(secret_key);
                        wallet_data.address = Some(address.clone());
                        wallet_data.mnemonic = Some(mnemonic.clone());
                        wallet_data.show_mnemonic = false;

                        // Update UI to show success
                        for entity in query.iter() {
                            commands.entity(entity).despawn_descendants();
                            commands.entity(entity).with_children(|parent| {
                                parent.spawn((
                                    Text::new("✅ Wallet Generated Successfully!"),
                                    Node {
                                        margin: UiRect::bottom(Val::Px(20.0)),
                                        ..default()
                                    },
                                ));

                                parent.spawn((
                                    Text::new(format!("Address: {}", address)),
                                    Node {
                                        margin: UiRect::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                ));

                                parent.spawn((
                                    Text::new("Your wallet has been securely stored in your OS keychain.\nYou can now use the other wallet operations."),
                                    Node {
                                        margin: UiRect::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                ));

                                parent.spawn((
                                    Text::new("⚠️ IMPORTANT: Use 'Export Seed' to backup your recovery phrase!"),
                                    Node {
                                        margin: UiRect::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                ));
                            });
                        }
                        info!("New wallet generated: {}", address);
                    }
                    Err(error) => {
                        error!("Failed to generate wallet: {}", error);
                        // Update UI to show error
                        for entity in query.iter() {
                            commands.entity(entity).despawn_descendants();
                            commands.entity(entity).with_children(|parent| {
                                parent.spawn((
                                    Text::new("❌ Failed to Generate Wallet"),
                                    Node {
                                        margin: UiRect::bottom(Val::Px(20.0)),
                                        ..default()
                                    },
                                ));

                                parent.spawn((
                                    Text::new(format!("Error: {}", error)),
                                    Node {
                                        margin: UiRect::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                ));
                            });
                        }
                    }
                }

                *color = Color::srgb(0.1, 0.5, 0.1).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.8, 0.3).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.7, 0.2).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

// Components for import functionality
#[derive(Component)]
struct ImportWalletButton;

#[derive(Component)]
struct SeedWordInput(usize);

#[derive(Resource)]
struct ImportState {
    seed_words: Vec<String>,
    focused_input: Option<usize>,  // Track which input field is currently focused
}

impl Default for ImportState {
    fn default() -> Self {
        Self {
            seed_words: vec![String::new(); 12],
            focused_input: None,
        }
    }
}

#[derive(Resource, Default)]
struct FocusedInput {
    entity: Option<Entity>,
    input_type: FocusedInputType,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum FocusedInputType {
    #[default]
    None,
    SeedWord(usize),
    SettingsUrl,
    TransferRecipient,
    TransferAmount,
    BurnAmount,
}

fn wallet_import_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
    mut wallet_data: ResMut<WalletData>,
    keychain: Res<KeychainManager>,
    mut import_state: ResMut<ImportState>,
    mut focused_input: ResMut<FocusedInput>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ImportWalletButton>, Without<SeedWordInput>),
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut word_input_query: Query<(Entity, &Interaction, &SeedWordInput, &Children, &mut BackgroundColor, &mut BorderColor), Without<ImportWalletButton>>,
    mut text_query: Query<&mut Text>,
) {
    // Show import wallet UI when state changes
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Import {
        // Reset import state
        import_state.seed_words = vec![String::new(); 12];
        focused_input.entity = None;
        focused_input.input_type = FocusedInputType::None;

        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("Import Existing Wallet"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if wallet_data.address.is_some() {
                    parent.spawn((
                        Text::new("⚠️ WARNING: You already have a wallet!\nImporting will replace your current wallet.\nMake sure you have backed up your current seed phrase."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }

                parent.spawn((
                    Text::new("Enter your 12-word seed phrase below:"),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Create a grid for seed word inputs
                parent
                    .spawn((
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![
                                RepeatedGridTrack::fr(1, 1.0),
                                RepeatedGridTrack::fr(1, 1.0),
                                RepeatedGridTrack::fr(1, 1.0),
                            ],
                            column_gap: Val::Px(10.0),
                            row_gap: Val::Px(10.0),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|parent| {
                        for i in 0..12 {
                            parent
                                .spawn((
                                    Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(format!("Word {}:", i + 1)),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(5.0)),
                                            ..default()
                                        },
                                    ));

                                    parent
                                        .spawn((
                                            Button,
                                            SeedWordInput(i),
                                            Node {
                                                width: Val::Px(120.0),
                                                height: Val::Px(30.0),
                                                border: UiRect::all(Val::Px(1.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BorderColor(Color::WHITE),
                                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                        ))
                                        .with_child(Text::new(""));
                                });
                        }
                    });

                // Import button
                parent
                    .spawn((
                        Button,
                        ImportWalletButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.7)),
                    ))
                    .with_child(Text::new("Import Wallet"));

                parent.spawn((
                    Text::new("Click on word fields above and type to enter your seed phrase."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }

    // Handle clicking on word input fields to focus them
    for (entity, interaction, word_input, _children, mut bg_color, mut border_color) in &mut word_input_query {
        // First, apply focused state styling if this is the focused field
        if focused_input.entity == Some(entity) {
            *border_color = BorderColor(Color::srgb(0.5, 0.5, 1.0));
            *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
        } else {
            // Apply non-focused styling based on interaction state
            match *interaction {
                Interaction::Hovered => {
                    *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                Interaction::None => {
                    *border_color = BorderColor(Color::WHITE);
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                _ => {}
            }
        }

        // Handle click to focus
        if *interaction == Interaction::Pressed {
            // Set this input as focused
            focused_input.entity = Some(entity);
            focused_input.input_type = FocusedInputType::SeedWord(word_input.0);
        }
    }

    // Handle keyboard input for the focused field
    if let Some(focused_entity) = focused_input.entity {
        if let FocusedInputType::SeedWord(word_index) = focused_input.input_type {
            let mut current_word = import_state.seed_words[word_index].clone();
            let mut word_changed = false;

            // Handle backspace
            if keyboard_input.just_pressed(KeyCode::Backspace) || keyboard_input.just_pressed(KeyCode::Delete) {
                if !current_word.is_empty() {
                    current_word.pop();
                    word_changed = true;
                }
            }

            // Handle Tab to move to next field
            if keyboard_input.just_pressed(KeyCode::Tab) {
                let next_index = (word_index + 1) % 12;
                // Find the entity with the next index
                for (entity, _, word_input, _, _, _) in &word_input_query {
                    if word_input.0 == next_index {
                        focused_input.entity = Some(entity);
                        focused_input.input_type = FocusedInputType::SeedWord(next_index);
                        break;
                    }
                }
            }

            // Handle letters
            for key_code in keyboard_input.get_just_pressed() {
                match key_code {
                    KeyCode::KeyA => { current_word.push('a'); word_changed = true; }
                    KeyCode::KeyB => { current_word.push('b'); word_changed = true; }
                    KeyCode::KeyC => { current_word.push('c'); word_changed = true; }
                    KeyCode::KeyD => { current_word.push('d'); word_changed = true; }
                    KeyCode::KeyE => { current_word.push('e'); word_changed = true; }
                    KeyCode::KeyF => { current_word.push('f'); word_changed = true; }
                    KeyCode::KeyG => { current_word.push('g'); word_changed = true; }
                    KeyCode::KeyH => { current_word.push('h'); word_changed = true; }
                    KeyCode::KeyI => { current_word.push('i'); word_changed = true; }
                    KeyCode::KeyJ => { current_word.push('j'); word_changed = true; }
                    KeyCode::KeyK => { current_word.push('k'); word_changed = true; }
                    KeyCode::KeyL => { current_word.push('l'); word_changed = true; }
                    KeyCode::KeyM => { current_word.push('m'); word_changed = true; }
                    KeyCode::KeyN => { current_word.push('n'); word_changed = true; }
                    KeyCode::KeyO => { current_word.push('o'); word_changed = true; }
                    KeyCode::KeyP => { current_word.push('p'); word_changed = true; }
                    KeyCode::KeyQ => { current_word.push('q'); word_changed = true; }
                    KeyCode::KeyR => { current_word.push('r'); word_changed = true; }
                    KeyCode::KeyS => { current_word.push('s'); word_changed = true; }
                    KeyCode::KeyT => { current_word.push('t'); word_changed = true; }
                    KeyCode::KeyU => { current_word.push('u'); word_changed = true; }
                    KeyCode::KeyV => { current_word.push('v'); word_changed = true; }
                    KeyCode::KeyW => { current_word.push('w'); word_changed = true; }
                    KeyCode::KeyX => { current_word.push('x'); word_changed = true; }
                    KeyCode::KeyY => { current_word.push('y'); word_changed = true; }
                    KeyCode::KeyZ => { current_word.push('z'); word_changed = true; }
                    _ => {}
                }
            }

            if word_changed {
                import_state.seed_words[word_index] = current_word.clone();

                // Update text display for the focused field
                if let Ok((_, _, _, children, _, _)) = word_input_query.get(focused_entity) {
                    if let Some(child) = children.first() {
                        if let Ok(mut text) = text_query.get_mut(*child) {
                            *text = Text::new(current_word);
                        }
                    }
                }
            }
        }
    }

    // Handle import button interactions
    for (interaction, mut color, mut border_color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                let mnemonic_string = import_state.seed_words.join(" ");

                match keychain.generate_wallet_from_mnemonic(&mnemonic_string) {
                    Ok((secret_key, address)) => {
                        // Store in keychain
                        let secure_data = SecureWalletData {
                            mnemonic: mnemonic_string.clone(),
                            created_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };

                        match keychain.store_wallet(&secure_data) {
                            Ok(_) => {
                                // Update wallet state
                                wallet_data.private_key = Some(secret_key);
                                wallet_data.address = Some(address.clone());
                                wallet_data.mnemonic = Some(mnemonic_string);

                                // Update UI to show success
                                for entity in query.iter() {
                                    commands.entity(entity).despawn_descendants();
                                    commands.entity(entity).with_children(|parent| {
                                        parent.spawn((
                                            Text::new("✅ Wallet Imported Successfully!"),
                                            Node {
                                                margin: UiRect::bottom(Val::Px(20.0)),
                                                ..default()
                                            },
                                        ));

                                        parent.spawn((
                                            Text::new(format!("Address: {}", address)),
                                            Node {
                                                margin: UiRect::all(Val::Px(10.0)),
                                                ..default()
                                            },
                                        ));

                                        parent.spawn((
                                            Text::new("Your wallet has been securely stored in your OS keychain.\nIt will be automatically registered with GalaChain."),
                                            Node {
                                                margin: UiRect::all(Val::Px(10.0)),
                                                ..default()
                                            },
                                        ));
                                    });
                                }
                                info!("Wallet imported successfully: {}", address);
                            }
                            Err(e) => {
                                error!("Failed to store imported wallet: {}", e);
                                // Update UI to show storage error
                                for entity in query.iter() {
                                    commands.entity(entity).despawn_descendants();
                                    commands.entity(entity).with_children(|parent| {
                                        parent.spawn((
                                            Text::new("❌ Failed to Store Wallet"),
                                            Node {
                                                margin: UiRect::bottom(Val::Px(20.0)),
                                                ..default()
                                            },
                                        ));

                                        parent.spawn((
                                            Text::new(format!("Storage error: {}", e)),
                                            Node {
                                                margin: UiRect::all(Val::Px(10.0)),
                                                ..default()
                                            },
                                        ));
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to import wallet: {}", e);
                        // Update UI to show import error
                        for entity in query.iter() {
                            commands.entity(entity).despawn_descendants();
                            commands.entity(entity).with_children(|parent| {
                                parent.spawn((
                                    Text::new("❌ Failed to Import Wallet"),
                                    Node {
                                        margin: UiRect::bottom(Val::Px(20.0)),
                                        ..default()
                                    },
                                ));

                                parent.spawn((
                                    Text::new(format!("Import error: {}\n\nPlease check that you entered all 12 words correctly.", e)),
                                    Node {
                                        margin: UiRect::all(Val::Px(10.0)),
                                        ..default()
                                    },
                                ));
                            });
                        }
                    }
                }

                *color = Color::srgb(0.1, 0.1, 0.5).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.3, 0.8).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.2, 0.7).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Component)]
struct ExportSeedButton;

#[derive(Resource)]
struct ExportState {
    show_seed: bool,
}

impl Default for ExportState {
    fn default() -> Self {
        Self { show_seed: false }
    }
}

fn wallet_export_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
    wallet_data: Res<WalletData>,
    keychain: Res<KeychainManager>,
    mut export_state: ResMut<ExportState>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ExportSeedButton>),
    >,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Export {
        export_state.show_seed = false;

        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("Export Seed Phrase"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if wallet_data.address.is_none() {
                    parent.spawn((
                        Text::new("❌ No wallet available to export.\nPlease generate or import a wallet first."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                    return;
                }

                parent.spawn((
                    Text::new("⚠️ WARNING: Never share your seed phrase with anyone!\nYour seed phrase gives complete access to your wallet.\nStore it securely offline."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Show/Hide seed button
                parent
                    .spawn((
                        Button,
                        ExportSeedButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.7)),
                    ))
                    .with_child(Text::new(if export_state.show_seed { "Hide Seed Phrase" } else { "Show Seed Phrase" }));

                // Display seed phrase if showing
                if export_state.show_seed {
                    match keychain.load_wallet() {
                        Ok(secure_data) => {
                            parent.spawn((
                                Text::new("📝 Your Recovery Seed Phrase:"),
                                Node {
                                    margin: UiRect::top(Val::Px(20.0)),
                                    ..default()
                                },
                            ));

                            parent
                                .spawn((
                                    Node {
                                        padding: UiRect::all(Val::Px(15.0)),
                                        margin: UiRect::all(Val::Px(10.0)),
                                        border: UiRect::all(Val::Px(2.0)),
                                        max_width: Val::Px(600.0),
                                        ..default()
                                    },
                                    BorderColor(Color::srgb(0.7, 0.7, 0.7)),
                                    BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                                ))
                                .with_child(Text::new(secure_data.mnemonic));

                            parent.spawn((
                                Text::new("💡 Write this down on paper and store it in a safe place.\nDo not save it digitally or take screenshots."),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                            ));
                        }
                        Err(e) => {
                            parent.spawn((
                                Text::new(format!("❌ Failed to load wallet from keychain: {}", e)),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                            ));
                        }
                    }
                }
            });
        }
    }

    // Handle button interactions
    for (interaction, mut color, mut border_color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                export_state.show_seed = !export_state.show_seed;

                // Refresh the UI
                for entity in query.iter() {
                    commands.entity(entity).despawn_descendants();
                    commands.entity(entity).with_children(|parent| {
                        parent.spawn((
                            Text::new("Export Seed Phrase"),
                            Node {
                                margin: UiRect::bottom(Val::Px(20.0)),
                                ..default()
                            },
                        ));

                        parent.spawn((
                            Text::new("⚠️ WARNING: Never share your seed phrase with anyone!\nYour seed phrase gives complete access to your wallet.\nStore it securely offline."),
                            Node {
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                        ));

                        // Show/Hide seed button
                        parent
                            .spawn((
                                Button,
                                ExportSeedButton,
                                Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(50.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::all(Val::Px(20.0)),
                                    ..default()
                                },
                                BorderColor(Color::BLACK),
                                BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.7)),
                            ))
                            .with_child(Text::new(if export_state.show_seed { "Hide Seed Phrase" } else { "Show Seed Phrase" }));

                        // Display seed phrase if showing
                        if export_state.show_seed {
                            match keychain.load_wallet() {
                                Ok(secure_data) => {
                                    parent.spawn((
                                        Text::new("📝 Your Recovery Seed Phrase:"),
                                        Node {
                                            margin: UiRect::top(Val::Px(20.0)),
                                            ..default()
                                        },
                                    ));

                                    parent
                                        .spawn((
                                            Node {
                                                padding: UiRect::all(Val::Px(15.0)),
                                                margin: UiRect::all(Val::Px(10.0)),
                                                border: UiRect::all(Val::Px(2.0)),
                                                max_width: Val::Px(600.0),
                                                ..default()
                                            },
                                            BorderColor(Color::srgb(0.7, 0.7, 0.7)),
                                            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                                        ))
                                        .with_child(Text::new(secure_data.mnemonic));

                                    parent.spawn((
                                        Text::new("💡 Write this down on paper and store it in a safe place.\nDo not save it digitally or take screenshots."),
                                        Node {
                                            margin: UiRect::all(Val::Px(10.0)),
                                            ..default()
                                        },
                                    ));
                                }
                                Err(e) => {
                                    parent.spawn((
                                        Text::new(format!("❌ Failed to load wallet from keychain: {}", e)),
                                        Node {
                                            margin: UiRect::all(Val::Px(10.0)),
                                            ..default()
                                        },
                                    ));
                                }
                            }
                        }
                    });
                }

                *color = Color::srgb(0.1, 0.1, 0.5).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.3, 0.8).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.2, 0.7).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Component)]
struct TransferAmountInput;

#[derive(Component)]
struct TransferAddressInput;

#[derive(Component)]
struct TransferButton;

#[derive(Resource)]
struct TransferState {
    recipient_address: String,
    amount: String,
    is_processing: bool,
}

impl Default for TransferState {
    fn default() -> Self {
        Self {
            recipient_address: String::new(),
            amount: String::new(),
            is_processing: false,
        }
    }
}

fn wallet_transfer_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
    wallet_data: Res<WalletData>,
    mut transfer_state: ResMut<TransferState>,
    mut focused_input: ResMut<FocusedInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut address_input_query: Query<
        (Entity, &Interaction, &Children, &mut BackgroundColor, &mut BorderColor),
        (With<TransferAddressInput>, Without<TransferAmountInput>, Without<TransferButton>),
    >,
    mut amount_input_query: Query<
        (Entity, &Interaction, &Children, &mut BackgroundColor, &mut BorderColor),
        (With<TransferAmountInput>, Without<TransferAddressInput>, Without<TransferButton>),
    >,
    mut transfer_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<TransferButton>, Without<TransferAddressInput>, Without<TransferAmountInput>),
    >,
    mut text_query: Query<&mut Text>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Transfer {
        transfer_state.recipient_address.clear();
        transfer_state.amount.clear();
        transfer_state.is_processing = false;
        focused_input.entity = None;
        focused_input.input_type = FocusedInputType::None;

        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("Transfer GALA Tokens"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if wallet_data.address.is_none() {
                    parent.spawn((
                        Text::new("❌ No wallet available.\nPlease generate or import a wallet first."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                    return;
                }

                parent.spawn((
                    Text::new("💡 NOTE: This is a reference implementation.\nTransfers would require additional GalaChain integration with proper signing."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Recipient address input
                parent.spawn((
                    Text::new("Recipient Address:"),
                    Node {
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                ));

                parent
                    .spawn((
                        Button,
                        TransferAddressInput,
                        Node {
                            width: Val::Px(400.0),
                            height: Val::Px(40.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BorderColor(Color::WHITE),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_child(Text::new(if transfer_state.recipient_address.is_empty() {
                        "Click to enter recipient address..."
                    } else {
                        &transfer_state.recipient_address
                    }));

                // Amount input
                parent.spawn((
                    Text::new("Amount (GALA):"),
                    Node {
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                ));

                parent
                    .spawn((
                        Button,
                        TransferAmountInput,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(40.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BorderColor(Color::WHITE),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_child(Text::new(if transfer_state.amount.is_empty() {
                        "0.0"
                    } else {
                        &transfer_state.amount
                    }));

                // Transfer button
                parent
                    .spawn((
                        Button,
                        TransferButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                        BackgroundColor(if transfer_state.is_processing {
                            Color::srgb(0.5, 0.5, 0.5)
                        } else {
                            Color::srgb(0.2, 0.7, 0.2)
                        }),
                    ))
                    .with_child(Text::new(if transfer_state.is_processing {
                        "Processing..."
                    } else {
                        "Transfer Tokens"
                    }));

                parent.spawn((
                    Text::new("⚠️ Network fee: 1 GALA\n📝 Click on input fields above to enter values"),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }

    // Handle clicking on address input field to focus it
    for (entity, interaction, _children, mut bg_color, mut border_color) in &mut address_input_query {
        // First, apply focused state styling if this is the focused field
        if focused_input.entity == Some(entity) {
            *border_color = BorderColor(Color::srgb(0.5, 0.5, 1.0));
            *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
        } else {
            // Apply non-focused styling based on interaction state
            match *interaction {
                Interaction::Hovered => {
                    *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                Interaction::None => {
                    *border_color = BorderColor(Color::WHITE);
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                _ => {}
            }
        }

        // Handle click to focus
        if *interaction == Interaction::Pressed {
            focused_input.entity = Some(entity);
            focused_input.input_type = FocusedInputType::TransferRecipient;
        }
    }

    // Handle clicking on amount input field to focus it
    for (entity, interaction, _children, mut bg_color, mut border_color) in &mut amount_input_query {
        // First, apply focused state styling if this is the focused field
        if focused_input.entity == Some(entity) {
            *border_color = BorderColor(Color::srgb(0.5, 0.5, 1.0));
            *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
        } else {
            // Apply non-focused styling based on interaction state
            match *interaction {
                Interaction::Hovered => {
                    *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                Interaction::None => {
                    *border_color = BorderColor(Color::WHITE);
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                _ => {}
            }
        }

        // Handle click to focus
        if *interaction == Interaction::Pressed {
            focused_input.entity = Some(entity);
            focused_input.input_type = FocusedInputType::TransferAmount;
        }
    }

    // Handle keyboard input for the focused field
    if let Some(focused_entity) = focused_input.entity {
        match focused_input.input_type {
            FocusedInputType::TransferRecipient => {
                let mut address_changed = false;

                // Handle backspace
                if keyboard_input.just_pressed(KeyCode::Backspace) || keyboard_input.just_pressed(KeyCode::Delete) {
                    if !transfer_state.recipient_address.is_empty() {
                        transfer_state.recipient_address.pop();
                        address_changed = true;
                    }
                }

                // Handle Tab to move to amount field
                if keyboard_input.just_pressed(KeyCode::Tab) {
                    // Find the amount input entity
                    for (entity, _, _, _, _) in &amount_input_query {
                        focused_input.entity = Some(entity);
                        focused_input.input_type = FocusedInputType::TransferAmount;
                        break;
                    }
                }

                // Handle letters and numbers
                for key_code in keyboard_input.get_just_pressed() {
                    if let Some(char) = key_to_char(*key_code) {
                        transfer_state.recipient_address.push(char);
                        address_changed = true;
                    }
                }

                if address_changed {
                    // Update text display for the focused field
                    if let Ok((_, _, children, _, _)) = address_input_query.get(focused_entity) {
                        if let Some(child) = children.first() {
                            if let Ok(mut text) = text_query.get_mut(*child) {
                                *text = Text::new(if transfer_state.recipient_address.is_empty() {
                                    "Click to enter recipient address..."
                                } else {
                                    &transfer_state.recipient_address
                                });
                            }
                        }
                    }
                }
            }
            FocusedInputType::TransferAmount => {
                let mut amount_changed = false;

                // Handle backspace
                if keyboard_input.just_pressed(KeyCode::Backspace) || keyboard_input.just_pressed(KeyCode::Delete) {
                    if !transfer_state.amount.is_empty() {
                        transfer_state.amount.pop();
                        amount_changed = true;
                    }
                }

                // Handle Tab to move back to address field
                if keyboard_input.just_pressed(KeyCode::Tab) {
                    // Find the address input entity
                    for (entity, _, _, _, _) in &address_input_query {
                        focused_input.entity = Some(entity);
                        focused_input.input_type = FocusedInputType::TransferRecipient;
                        break;
                    }
                }

                // Handle numbers and period
                for key_code in keyboard_input.get_just_pressed() {
                    match key_code {
                        KeyCode::Digit0 | KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4 |
                        KeyCode::Digit5 | KeyCode::Digit6 | KeyCode::Digit7 | KeyCode::Digit8 | KeyCode::Digit9 => {
                            if let Some(char) = key_to_char(*key_code) {
                                transfer_state.amount.push(char);
                                amount_changed = true;
                            }
                        }
                        KeyCode::Period => {
                            if !transfer_state.amount.contains('.') {
                                transfer_state.amount.push('.');
                                amount_changed = true;
                            }
                        }
                        _ => {}
                    }
                }

                if amount_changed {
                    // Update text display for the focused field
                    if let Ok((_, _, children, _, _)) = amount_input_query.get(focused_entity) {
                        if let Some(child) = children.first() {
                            if let Ok(mut text) = text_query.get_mut(*child) {
                                *text = Text::new(if transfer_state.amount.is_empty() {
                                    "0.0"
                                } else {
                                    &transfer_state.amount
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Handle transfer button
    for (interaction, mut color, mut border_color) in &mut transfer_button_query {
        match *interaction {
            Interaction::Pressed => {
                if !transfer_state.is_processing &&
                   !transfer_state.recipient_address.is_empty() &&
                   !transfer_state.amount.is_empty() {

                    transfer_state.is_processing = true;

                    // Simulate transfer process
                    info!("Transfer requested: {} GALA to {}", transfer_state.amount, transfer_state.recipient_address);

                    // Update UI to show result
                    for entity in query.iter() {
                        commands.entity(entity).despawn_descendants();
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                Text::new("Transfer Result"),
                                Node {
                                    margin: UiRect::bottom(Val::Px(20.0)),
                                    ..default()
                                },
                            ));

                            parent.spawn((
                                Text::new("🚧 Transfer Feature - Reference Implementation\n\nThis demonstrates the UI for token transfers.\nIn a full implementation, this would:\n\n• Validate the recipient address\n• Check your GALA balance\n• Create and sign a transfer transaction\n• Submit to GalaChain network\n• Show transaction confirmation"),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    max_width: Val::Px(600.0),
                                    ..default()
                                },
                            ));

                            parent.spawn((
                                Text::new(format!("Requested Transfer:\n• Amount: {} GALA\n• To: {}\n• From: {}",
                                    transfer_state.amount,
                                    transfer_state.recipient_address,
                                    wallet_data.address.as_ref().unwrap_or(&"Unknown".to_string())
                                )),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    max_width: Val::Px(600.0),
                                    ..default()
                                },
                            ));
                        });
                    }
                }

                *color = Color::srgb(0.1, 0.5, 0.1).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.3, 0.8, 0.3).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.7, 0.2).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn key_to_char(key_code: KeyCode) -> Option<char> {
    match key_code {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        _ => None,
    }
}

#[derive(Component)]
struct BurnAmountInput;

#[derive(Component)]
struct BurnButton;

#[derive(Resource)]
struct BurnState {
    amount: String,
    is_processing: bool,
}

impl Default for BurnState {
    fn default() -> Self {
        Self {
            amount: String::new(),
            is_processing: false,
        }
    }
}

fn wallet_burn_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
    wallet_data: Res<WalletData>,
    mut burn_state: ResMut<BurnState>,
    mut focused_input: ResMut<FocusedInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut amount_input_query: Query<
        (Entity, &Interaction, &Children, &mut BackgroundColor, &mut BorderColor),
        (With<BurnAmountInput>, Without<BurnButton>),
    >,
    mut burn_button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<BurnButton>, Without<BurnAmountInput>),
    >,
    mut text_query: Query<&mut Text>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Burn {
        burn_state.amount.clear();
        burn_state.is_processing = false;
        focused_input.entity = None;
        focused_input.input_type = FocusedInputType::None;

        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Text::new("Burn GALA Tokens"),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                if wallet_data.address.is_none() {
                    parent.spawn((
                        Text::new("❌ No wallet available.\nPlease generate or import a wallet first."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                    return;
                }

                parent.spawn((
                    Text::new("⚠️ WARNING: Burning tokens is PERMANENT and IRREVERSIBLE!\nTokens will be destroyed forever and cannot be recovered."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Text::new("💡 NOTE: This is a reference implementation based on the dapp-template.\nReal burning would require proper GalaChain integration."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Amount input
                parent.spawn((
                    Text::new("Amount to Burn (GALA):"),
                    Node {
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                ));

                parent
                    .spawn((
                        Button,
                        BurnAmountInput,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(40.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BorderColor(Color::WHITE),
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_child(Text::new(if burn_state.amount.is_empty() {
                        "0.0"
                    } else {
                        &burn_state.amount
                    }));

                // Burn button
                parent
                    .spawn((
                        Button,
                        BurnButton,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BorderColor(Color::BLACK),
                        BorderRadius::new(Val::Px(5.0), Val::Px(5.0), Val::Px(5.0), Val::Px(5.0)),
                        BackgroundColor(if burn_state.is_processing {
                            Color::srgb(0.5, 0.5, 0.5)
                        } else {
                            Color::srgb(0.8, 0.2, 0.2)
                        }),
                    ))
                    .with_child(Text::new(if burn_state.is_processing {
                        "Processing..."
                    } else {
                        "🔥 Burn Tokens"
                    }));

                parent.spawn((
                    Text::new("⚠️ Network fee: 1 GALA\n📝 Click on amount field above to enter value\n🔥 Tokens will be permanently destroyed"),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }

    // Handle clicking on amount input field to focus it
    for (entity, interaction, _children, mut bg_color, mut border_color) in &mut amount_input_query {
        // First, apply focused state styling if this is the focused field
        if focused_input.entity == Some(entity) {
            *border_color = BorderColor(Color::srgb(0.5, 0.5, 1.0));
            *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
        } else {
            // Apply non-focused styling based on interaction state
            match *interaction {
                Interaction::Hovered => {
                    *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                Interaction::None => {
                    *border_color = BorderColor(Color::WHITE);
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
                _ => {}
            }
        }

        // Handle click to focus
        if *interaction == Interaction::Pressed {
            focused_input.entity = Some(entity);
            focused_input.input_type = FocusedInputType::BurnAmount;
        }
    }

    // Handle keyboard input for the focused field
    if let Some(focused_entity) = focused_input.entity {
        if let FocusedInputType::BurnAmount = focused_input.input_type {
            let mut amount_changed = false;

            // Handle backspace
            if keyboard_input.just_pressed(KeyCode::Backspace) || keyboard_input.just_pressed(KeyCode::Delete) {
                if !burn_state.amount.is_empty() {
                    burn_state.amount.pop();
                    amount_changed = true;
                }
            }

            // Handle numbers and period
            for key_code in keyboard_input.get_just_pressed() {
                match key_code {
                    KeyCode::Digit0 | KeyCode::Digit1 | KeyCode::Digit2 | KeyCode::Digit3 | KeyCode::Digit4 |
                    KeyCode::Digit5 | KeyCode::Digit6 | KeyCode::Digit7 | KeyCode::Digit8 | KeyCode::Digit9 => {
                        if let Some(char) = key_to_char(*key_code) {
                            burn_state.amount.push(char);
                            amount_changed = true;
                        }
                    }
                    KeyCode::Period => {
                        if !burn_state.amount.contains('.') {
                            burn_state.amount.push('.');
                            amount_changed = true;
                        }
                    }
                    _ => {}
                }
            }

            if amount_changed {
                // Update text display for the focused field
                if let Ok((_, _, children, _, _)) = amount_input_query.get(focused_entity) {
                    if let Some(child) = children.first() {
                        if let Ok(mut text) = text_query.get_mut(*child) {
                            *text = Text::new(if burn_state.amount.is_empty() {
                                "0.0"
                            } else {
                                &burn_state.amount
                            });
                        }
                    }
                }
            }
        }
    }

    // Handle burn button
    for (interaction, mut color, mut border_color) in &mut burn_button_query {
        match *interaction {
            Interaction::Pressed => {
                if !burn_state.is_processing && !burn_state.amount.is_empty() {

                    burn_state.is_processing = true;

                    // Simulate burn process
                    info!("Burn requested: {} GALA from {}", burn_state.amount, wallet_data.address.as_ref().unwrap_or(&"Unknown".to_string()));

                    // Update UI to show result
                    for entity in query.iter() {
                        commands.entity(entity).despawn_descendants();
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                Text::new("Burn Result"),
                                Node {
                                    margin: UiRect::bottom(Val::Px(20.0)),
                                    ..default()
                                },
                            ));

                            parent.spawn((
                                Text::new("🚧 Burn Feature - Reference Implementation\n\nThis demonstrates the UI for token burning based on the dapp-template.\nIn a full implementation, this would:\n\n• Validate the burn amount against your balance\n• Create a signed burn transaction\n• Submit to GalaChain with BurnTokens API\n• Show transaction confirmation\n• Update your balance"),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    max_width: Val::Px(600.0),
                                    ..default()
                                },
                            ));

                            parent.spawn((
                                Text::new(format!("Requested Burn:\n• Amount: {} GALA\n• From: {}\n• Unique Key: january-2025-event-{}",
                                    burn_state.amount,
                                    wallet_data.address.as_ref().unwrap_or(&"Unknown".to_string()),
                                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
                                )),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    max_width: Val::Px(600.0),
                                    ..default()
                                },
                            ));

                            parent.spawn((
                                Text::new("📋 The dapp-template shows this burn pattern:\n\n• Collection: \"GALA\"\n• Category: \"Unit\"\n• Type: \"none\"\n• Instance: \"0\"\n• Signature via MetaMask integration"),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    max_width: Val::Px(600.0),
                                    ..default()
                                },
                            ));
                        });
                    }
                }

                *color = Color::srgb(0.5, 0.1, 0.1).into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.9, 0.3, 0.3).into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = Color::srgb(0.8, 0.2, 0.2).into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

pub struct WalletPlugin;

impl Plugin for WalletPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WalletData {
            private_key: None,
            address: None,
            mnemonic: None,
            show_mnemonic: false,
            show_import: false,
            import_words: vec![String::new(); 12],
        })
        .add_systems(
            Update,
            (
                generate_wallet_button_system,
                import_button_system,
                import_word_system,
                import_confirm_system,
                wallet_overview_system,
            ).run_if(in_state(AppState::WalletMenu)),
        );
    }
}
