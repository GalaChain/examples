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
        let entry = Entry::new(&self.service_name, &self.username)
            .map_err(|e| KeychainError::Access(e.to_string()))?;

        let json_data = wallet_data.to_json()?;
        
        entry.set_password(&json_data)
            .map_err(|e| KeychainError::Access(e.to_string()))?;

        info!("Wallet stored securely in OS keychain");
        Ok(())
    }

    pub fn load_wallet(&self) -> Result<SecureWalletData, KeychainError> {
        let entry = Entry::new(&self.service_name, &self.username)
            .map_err(|e| KeychainError::Access(e.to_string()))?;

        let json_data = entry.get_password()
            .map_err(|e| match e {
                keyring::Error::NoEntry => KeychainError::NotFound,
                _ => KeychainError::Access(e.to_string()),
            })?;

        SecureWalletData::from_json(&json_data)
    }

    pub fn delete_wallet(&self) -> Result<(), KeychainError> {
        let entry = Entry::new(&self.service_name, &self.username)
            .map_err(|e| KeychainError::Access(e.to_string()))?;

        entry.delete_credential()
            .map_err(|e| KeychainError::Access(e.to_string()))?;

        info!("Wallet deleted from OS keychain");
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
    NotRegistered,
}

impl fmt::Display for GalaChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GalaChainError::Network(msg) => write!(f, "Network error: {}", msg),
            GalaChainError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            GalaChainError::Parse(msg) => write!(f, "Parsing error: {}", msg),
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
    pub burn_gateway_api: String,
    pub burn_gateway_public_key_api: String,
    pub galaswap_api: String,
}

impl GalaChainClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            burn_gateway_api: "https://gateway-mainnet.galachain.com/api/asset/token-contract".to_string(),
            burn_gateway_public_key_api: "https://gateway-mainnet.galachain.com/api/asset/public-key-contract".to_string(),
            galaswap_api: "https://api-galaswap.gala.com/galachain".to_string(),
        }
    }

    // Check if user is registered with GalaChain
    pub async fn check_registration(&self, gala_address: &str) -> Result<bool, GalaChainError> {
        let request = PublicKeyRequest {
            user: gala_address.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/GetPublicKey", self.burn_gateway_public_key_api))
            .json(&request)
            .send()
            .await
            .map_err(|e| GalaChainError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }

    // Register user with GalaChain
    pub async fn register_user(&self, public_key: &str) -> Result<(), GalaChainError> {
        let request = RegistrationRequest {
            public_key: public_key.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/CreateHeadlessWallet", self.galaswap_api))
            .json(&request)
            .send()
            .await
            .map_err(|e| GalaChainError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(GalaChainError::Auth(format!(
                "Registration failed with status: {}",
                response.status()
            )))
        }
    }

    // Get GALA token balance
    pub async fn get_gala_balance(&self, gala_address: &str) -> Result<(f64, f64), GalaChainError> {
        let request = BalanceRequest {
            owner: gala_address.to_string(),
            collection: "GALA".to_string(),
            category: "Unit".to_string(),
            r#type: "none".to_string(),
            additional_key: "none".to_string(),
            instance: "0".to_string(),
        };

        let response = self
            .client
            .post(format!("{}/FetchBalances", self.burn_gateway_api))
            .json(&request)
            .send()
            .await
            .map_err(|e| GalaChainError::Network(e.to_string()))?;

        let balance_response: BalanceResponse = response
            .json()
            .await
            .map_err(|e| GalaChainError::Parse(e.to_string()))?;

        if let Some(balance) = balance_response.data.first() {
            let total = balance.quantity.parse::<f64>()
                .map_err(|e| GalaChainError::Parse(e.to_string()))?;
            
            let locked: f64 = balance.locked_holds
                .iter()
                .map(|hold| hold.quantity.parse::<f64>().unwrap_or(0.0))
                .sum();

            Ok((total - locked, locked))
        } else {
            Ok((0.0, 0.0))
        }
    }

    // Convert Ethereum address to GalaChain format
    pub fn ethereum_to_galachain_address(eth_address: &str) -> String {
        if eth_address.starts_with("0x") {
            format!("eth|{}", &eth_address[2..])
        } else {
            format!("eth|{}", eth_address)
        }
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
        Self::new()
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
        app.insert_resource(KeychainManager::new())
            .insert_resource(GalaChainClient::new())
            .add_systems(Startup, setup_main_menu)
            .add_systems(
                Update,
                (
                    main_menu_system.run_if(in_state(AppState::MainMenu)),
                    wallet_menu_system.run_if(in_state(AppState::WalletMenu)),
                    back_button_system, // Run back button system in all states
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

fn show_settings(mut commands: Commands) {
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
            parent.spawn((Text::new("Settings"), MenuTitle));
            parent.spawn((Text::new("Settings page coming soon..."), MenuTitle));
            
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

fn wallet_balance_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    wallet_data: Res<WalletData>,
    galachain_client: Res<GalaChainClient>,
    query: Query<Entity, With<ContentArea>>,
    mut balance_task: Local<Option<bevy::tasks::Task<Result<(f64, f64), GalaChainError>>>>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Balance {
        // Update the content area, not replace the whole UI
        for entity in query.iter() {
            commands.entity(entity).despawn_descendants();
            
            if let Some(address) = &wallet_data.address {
                let gala_address = GalaChainClient::ethereum_to_galachain_address(address);
                
                // Show loading state
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Text::new("GALA Token Balance"),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        Text::new("Loading balance..."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                });

                // Start async balance fetch
                let client = (*galachain_client).clone();
                let address_clone = gala_address.clone();
                
                *balance_task = Some(bevy::tasks::IoTaskPool::get().spawn(async move {
                    client.get_gala_balance(&address_clone).await
                }));
            } else {
                // Show no wallet message
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Text::new("No wallet available. Please generate or import a wallet first."),
                        Node {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                });
            }
        }
    }

    // Check if balance task is complete
    if let Some(task) = balance_task.as_mut() {
        if let Some(result) = bevy::tasks::block_on(bevy::tasks::poll_once(task)) {
            // Clean up task
            *balance_task = None;
            
            // Clone result for use in closure
            let balance_result = result.clone();
            
            // Update content area with balance result
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

                    match &balance_result {
                        Ok((available, locked)) => {
                            parent.spawn((
                                Text::new(format!("Available: {:.2} GALA", available)),
                                Node {
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                            ));

                            if *locked > 0.0 {
                                parent.spawn((
                                    Text::new(format!("Locked: {:.2} GALA", locked)),
                                    Node {
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                ));
                            }

                            parent.spawn((
                                Text::new(format!("Total: {:.2} GALA", available + locked)),
                                Node {
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                            ));
                        }
                        Err(e) => {
                            parent.spawn((
                                Text::new(format!("Error fetching balance: {}", e)),
                                Node {
                                    margin: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                            ));
                        }
                    }
                });
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
                client.check_registration(&address_clone).await
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
                            client.register_user(&public_key).await
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

fn wallet_generate_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
) {
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

                parent.spawn((
                    Text::new("This feature will generate a new wallet with a secure seed phrase.\nImplementation coming soon..."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }
}

fn wallet_import_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Import {
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

                parent.spawn((
                    Text::new("This feature will allow you to import a wallet from your seed phrase.\nImplementation coming soon..."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }
}

fn wallet_export_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Export {
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
                    Text::new("This feature will securely display your seed phrase for backup.\nImplementation coming soon..."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }
}

fn wallet_transfer_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Transfer {
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

                parent.spawn((
                    Text::new("This feature will allow you to transfer GALA tokens to other addresses.\nImplementation coming soon..."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
        }
    }
}

fn wallet_burn_system(
    wallet_state: Res<State<WalletState>>,
    mut commands: Commands,
    query: Query<Entity, With<ContentArea>>,
) {
    if wallet_state.is_changed() && *wallet_state.get() == WalletState::Burn {
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

                parent.spawn((
                    Text::new("This feature will allow you to burn GALA tokens permanently.\nImplementation coming soon..."),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                ));
            });
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
                export_seed_button_system,
                import_button_system,
                import_word_system,
                import_confirm_system,
                wallet_overview_system,
                wallet_balance_system,
                wallet_registration_system,
                wallet_generate_system,
                wallet_import_system,
                wallet_export_system,
                wallet_transfer_system,
                wallet_burn_system,
            ).run_if(in_state(AppState::WalletMenu)),
        );
    }
}
