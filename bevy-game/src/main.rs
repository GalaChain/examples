use bevy::prelude::*;
use bip39::{Mnemonic, Language};
use secp256k1::{SecretKey, PublicKey};
use rand::rngs::OsRng;
use sha3::{Digest, Keccak256};

// Components
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
struct WalletState {
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
    mut wallet_state: ResMut<WalletState>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<AddressText>>,
        Query<&mut Text, With<MnemonicText>>,
    )>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let (secret_key, address, mnemonic) = generate_wallet();
                wallet_state.private_key = Some(secret_key);
                wallet_state.address = Some(address.clone());
                wallet_state.mnemonic = Some(mnemonic.clone());
                wallet_state.show_mnemonic = false;

                // Update UI text
                if let Ok(mut text) = text_queries.p0().get_single_mut() {
                    *text = Text::new(format!("Address: {}", address));
                }
                if let Ok(mut text) = text_queries.p1().get_single_mut() {
                    *text = Text::new("");
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
    mut wallet_state: ResMut<WalletState>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                wallet_state.show_mnemonic = !wallet_state.show_mnemonic;

                // Update button text
                if let Some(child) = children.first() {
                    if let Ok(mut text) = text_queries.p0().get_mut(*child) {
                        *text = Text::new(
                            if wallet_state.show_mnemonic {
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
                        if wallet_state.show_mnemonic {
                            if let Some(mnemonic) = wallet_state.mnemonic.as_ref() {
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
    mut wallet_state: ResMut<WalletState>,
    mut commands: Commands,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                wallet_state.show_import = !wallet_state.show_import;

                if wallet_state.show_import {
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
    mut wallet_state: ResMut<WalletState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (interaction, word_input, children) in &mut interaction_query {
        if let Interaction::Pressed = interaction {
            let word_index = word_input.0;
            let mut current_word = wallet_state.import_words[word_index].clone();

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

            wallet_state.import_words[word_index] = current_word.clone();

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
    mut wallet_state: ResMut<WalletState>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<AddressText>>,
        Query<&mut Text, With<MnemonicText>>,
    )>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mnemonic_string = wallet_state.import_words.join(" ");
                if let Ok(mnemonic) = Mnemonic::parse_in_normalized(Language::English, &mnemonic_string) {
                    // Generate private key from mnemonic
                    let seed = mnemonic.to_seed("");
                    let secp = secp256k1::Secp256k1::new();

                    // Use first 32 bytes of seed as private key
                    let secret_key = SecretKey::from_slice(&seed[..32])
                        .expect("32 bytes of seed should be a valid private key");

                    // Generate public key and address
                    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
                    let public_key_bytes = public_key.serialize_uncompressed();

                    // Generate Ethereum address
                    let mut hasher = Keccak256::new();
                    hasher.update(&public_key_bytes[1..]); // Skip recovery id byte
                    let result = hasher.finalize();
                    let address = format!("0x{}", hex::encode(&result[12..])); // Take last 20 bytes

                    // Update wallet state
                    wallet_state.private_key = Some(secret_key);
                    wallet_state.address = Some(address.clone());
                    wallet_state.mnemonic = Some(mnemonic.to_string());
                    wallet_state.show_import = false; // Hide import form

                    // Update UI
                    if let Ok(mut text) = text_queries.p0().get_single_mut() {
                        *text = Text::new(format!("Address: {}", address));
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

pub struct WalletPlugin;

impl Plugin for WalletPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WalletState {
            private_key: None,
            address: None,
            mnemonic: None,
            show_mnemonic: false,
            show_import: false,
            import_words: vec![String::new(); 12],
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                generate_wallet_button_system,
                export_seed_button_system,
                import_button_system,
                import_word_system,
                import_confirm_system,
            ),
        );
    }
}
