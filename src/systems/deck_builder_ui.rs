use bevy::prelude::*;

use crate::resources::{
    AffinityState, CardTab, CardType, DeckBuilderState, GameData, GamePhase, PlayerDeck,
};
use crate::systems::spawn_weapon;

// =============================================================================
// CONSTANTS
// =============================================================================

const PANEL_WIDTH: f32 = 1200.0;
const PANEL_HEIGHT: f32 = 800.0;
const PANEL_PADDING: f32 = 24.0;

// Colors from spec
const DECK_BUILDER_BG: Color = Color::srgba(0.05, 0.05, 0.10, 0.95);
const PANEL_BG: Color = Color::srgb(0.10, 0.10, 0.18);
const PANEL_BORDER: Color = Color::srgb(0.16, 0.16, 0.30);
const DIVIDER: Color = Color::srgb(0.23, 0.23, 0.37);
const ACCENT_GREEN: Color = Color::srgb(0.13, 0.77, 0.37);
const ACCENT_GREEN_HOVER: Color = Color::srgb(0.20, 0.84, 0.42);
const ACCENT_RED: Color = Color::srgb(0.91, 0.27, 0.38);
const ACCENT_RED_HOVER: Color = Color::srgb(0.95, 0.35, 0.45);
const BAR_CREATURE: Color = Color::srgb(0.94, 0.27, 0.27);
const BAR_WEAPON: Color = Color::srgb(0.23, 0.51, 0.96);
const BAR_ARTIFACT: Color = Color::srgb(0.66, 0.33, 0.97);
const BAR_EMPTY: Color = Color::srgb(0.16, 0.16, 0.30);
const TEXT_PRIMARY: Color = Color::WHITE;
const TEXT_MUTED: Color = Color::srgb(0.63, 0.63, 0.63);
const BUTTON_BG: Color = Color::srgb(0.16, 0.16, 0.30);
const BUTTON_HOVER: Color = Color::srgb(0.23, 0.23, 0.37);
const MINI_CARD_BG: Color = Color::srgb(0.07, 0.07, 0.12);
const TAB_SELECTED: Color = Color::srgb(0.13, 0.77, 0.37);

// Affinity colors for card color boxes
const COLOR_RED: Color = Color::srgb(0.94, 0.27, 0.27);
const COLOR_BLUE: Color = Color::srgb(0.23, 0.51, 0.96);
const COLOR_GREEN: Color = Color::srgb(0.27, 0.78, 0.38);
const COLOR_WHITE: Color = Color::srgb(0.95, 0.95, 0.95);
const COLOR_BLACK: Color = Color::srgb(0.4, 0.2, 0.5);
const COLOR_GRAY: Color = Color::srgb(0.5, 0.5, 0.5);

// =============================================================================
// MARKER COMPONENTS
// =============================================================================

/// Marker for the deck builder overlay (full screen)
#[derive(Component)]
pub struct DeckBuilderOverlay;

/// Marker for the deck builder panel
#[derive(Component)]
pub struct DeckBuilderPanel;

/// Marker for the card list section (scrollable)
#[derive(Component)]
pub struct CardListSection;

/// Marker for the available cards section
#[derive(Component)]
pub struct AvailableCardsSection;

/// Row displaying a card in the deck
#[derive(Component)]
pub struct DeckCardRow {
    pub card_id: String,
}

/// Button to adjust copy count
#[derive(Component)]
pub struct CardCopyButton {
    pub card_id: String,
    pub delta: i32, // +1 or -1
}

/// Available card mini-card (clickable to add)
#[derive(Component)]
pub struct AvailableMiniCard {
    pub card_id: String,
    pub card_type: CardType,
}

/// Tab selector for card types
#[derive(Component)]
pub struct CardTypeTab {
    pub tab: CardTab,
}

/// Start run button
#[derive(Component)]
pub struct StartRunButton;

/// Clear deck button
#[derive(Component)]
pub struct ClearDeckButton;

/// Probability bar fill element
#[derive(Component)]
pub struct ProbabilityBarFill {
    pub card_id: String,
}

/// Percentage text display
#[derive(Component)]
pub struct PercentageText {
    pub card_id: String,
}

/// Total cards count text
#[derive(Component)]
pub struct TotalCardsText;

/// Type breakdown display
#[derive(Component)]
pub struct TypeBreakdownText;

/// Tab underline indicator
#[derive(Component)]
pub struct TabUnderline {
    pub tab: CardTab,
}

/// Starting weapon selection section
#[derive(Component)]
pub struct StartingWeaponSection;

/// Individual starting weapon card (clickable to select)
#[derive(Component)]
pub struct StartingWeaponCard {
    pub weapon_id: String,
}

/// Text showing selected starting weapon
#[derive(Component)]
pub struct SelectedWeaponText;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn get_color_for_affinity(color: &str) -> Color {
    match color.to_lowercase().as_str() {
        "red" => COLOR_RED,
        "blue" => COLOR_BLUE,
        "green" => COLOR_GREEN,
        "white" => COLOR_WHITE,
        "black" => COLOR_BLACK,
        _ => COLOR_GRAY,
    }
}

fn get_bar_color_for_type(card_type: &CardType) -> Color {
    match card_type {
        CardType::Creature => BAR_CREATURE,
        CardType::Weapon => BAR_WEAPON,
        CardType::Artifact => BAR_ARTIFACT,
    }
}

// =============================================================================
// SPAWN SYSTEM
// =============================================================================

/// Spawns the deck builder UI (initially visible since game starts in DeckBuilder phase)
pub fn spawn_deck_builder_system(mut commands: Commands, game_data: Res<GameData>) {
    // Full screen overlay
    commands
        .spawn((
            DeckBuilderOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(DECK_BUILDER_BG),
            ZIndex(50),
        ))
        .with_children(|parent| {
            // Main panel
            parent
                .spawn((
                    DeckBuilderPanel,
                    Node {
                        width: Val::Px(PANEL_WIDTH),
                        height: Val::Px(PANEL_HEIGHT),
                        padding: UiRect::all(Val::Px(PANEL_PADDING)),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                    BorderColor(PANEL_BORDER),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|panel| {
                    // Header row
                    spawn_header_row(panel);

                    // Starting weapon selection section
                    spawn_starting_weapon_section(panel, &game_data);

                    // Divider
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(DIVIDER),
                    ));

                    // Card list section (scrollable)
                    spawn_card_list_section(panel);

                    // Divider
                    panel.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(DIVIDER),
                    ));

                    // Add card section with tabs
                    spawn_add_card_section(panel, &game_data);

                    // Footer row
                    spawn_footer_row(panel);
                });
        });
}

fn spawn_header_row(parent: &mut ChildBuilder) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        })
        .with_children(|row| {
            // Title
            row.spawn((
                Text::new("DECK BUILDER"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            // Start Run button
            row.spawn((
                StartRunButton,
                Button,
                Node {
                    padding: UiRect::new(Val::Px(24.0), Val::Px(24.0), Val::Px(12.0), Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(ACCENT_GREEN),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("START RUN"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                ));
            });
        });
}

fn spawn_starting_weapon_section(parent: &mut ChildBuilder, game_data: &GameData) {
    parent
        .spawn((
            StartingWeaponSection,
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|section| {
            // Section header
            section
                .spawn(Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("STARTING WEAPON"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(TEXT_MUTED),
                    ));

                    // Selected weapon display
                    header.spawn((
                        SelectedWeaponText,
                        Text::new("Ember Staff"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(ACCENT_GREEN),
                    ));
                });

            // Weapon cards row (only tier 1 weapons)
            section
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    overflow: Overflow::scroll_x(),
                    ..default()
                })
                .with_children(|row| {
                    // Get only tier 1 weapons as starting options
                    for weapon in game_data.weapons.iter().filter(|w| w.tier == 1) {
                        spawn_starting_weapon_card(
                            row,
                            &weapon.id,
                            &weapon.name,
                            get_color_for_affinity(&weapon.color),
                            weapon.id == "ember_staff", // Default selected
                        );
                    }
                });
        });
}

fn spawn_starting_weapon_card(
    parent: &mut ChildBuilder,
    weapon_id: &str,
    weapon_name: &str,
    weapon_color: Color,
    selected: bool,
) {
    let border_color = if selected { ACCENT_GREEN } else { PANEL_BORDER };
    let bg_color = if selected {
        Color::srgba(0.13, 0.77, 0.37, 0.15)
    } else {
        MINI_CARD_BG
    };

    parent
        .spawn((
            StartingWeaponCard {
                weapon_id: weapon_id.to_string(),
            },
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|card| {
            // Weapon name
            card.spawn((
                Text::new(weapon_name),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            // Color indicator
            card.spawn((
                Node {
                    width: Val::Px(12.0),
                    height: Val::Px(12.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(weapon_color),
                BorderRadius::all(Val::Px(6.0)),
            ));
        });
}

fn spawn_card_list_section(parent: &mut ChildBuilder) {
    parent.spawn((
        CardListSection,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(250.0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
}

fn spawn_add_card_section(parent: &mut ChildBuilder, _game_data: &GameData) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|section| {
            // Tab row
            section
                .spawn(Node {
                    width: Val::Percent(100.0),
                    margin: UiRect::bottom(Val::Px(12.0)),
                    column_gap: Val::Px(24.0),
                    ..default()
                })
                .with_children(|tabs| {
                    spawn_tab_button(tabs, "Creatures", CardTab::Creatures, true);
                    spawn_tab_button(tabs, "Weapons", CardTab::Weapons, false);
                    spawn_tab_button(tabs, "Artifacts", CardTab::Artifacts, false);
                });

            // Available cards scroll area
            section.spawn((
                AvailableCardsSection,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    flex_direction: FlexDirection::Row,
                    overflow: Overflow::scroll_x(),
                    column_gap: Val::Px(8.0),
                    ..default()
                },
            ));
        });
}

fn spawn_tab_button(parent: &mut ChildBuilder, label: &str, tab: CardTab, selected: bool) {
    parent
        .spawn((
            CardTypeTab { tab },
            Button,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if selected { TEXT_PRIMARY } else { TEXT_MUTED }),
            ));
            // Underline
            btn.spawn((
                TabUnderline { tab },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(if selected { TAB_SELECTED } else { Color::NONE }),
            ));
        });
}

fn spawn_footer_row(parent: &mut ChildBuilder) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        })
        .with_children(|row| {
            // Total count
            row.spawn((
                TotalCardsText,
                Text::new("Total: 0 cards"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(TEXT_MUTED),
            ));

            // Type breakdown
            row.spawn((
                TypeBreakdownText,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(TEXT_MUTED),
            ));

            // Clear deck button
            row.spawn((
                ClearDeckButton,
                Button,
                Node {
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(6.0), Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor(ACCENT_RED),
                BorderRadius::all(Val::Px(4.0)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("CLEAR DECK"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(ACCENT_RED),
                ));
            });
        });
}

// =============================================================================
// VISIBILITY SYSTEM
// =============================================================================

/// Shows/hides deck builder based on GamePhase
pub fn deck_builder_visibility_system(
    game_phase: Res<GamePhase>,
    mut query: Query<&mut Visibility, With<DeckBuilderOverlay>>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = if *game_phase == GamePhase::DeckBuilder {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// =============================================================================
// CARD LIST UPDATE SYSTEM
// =============================================================================

/// Updates the card list section to reflect current deck state
pub fn deck_builder_update_cards_system(
    mut commands: Commands,
    deck_state: Res<DeckBuilderState>,
    game_data: Res<GameData>,
    game_phase: Res<GamePhase>,
    card_list_query: Query<Entity, With<CardListSection>>,
    existing_rows: Query<Entity, With<DeckCardRow>>,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    // Only rebuild if deck changed
    if !deck_state.is_changed() {
        return;
    }

    // Get the card list section entity
    let Ok(card_list_entity) = card_list_query.get_single() else {
        return;
    };

    // Despawn existing rows
    for entity in existing_rows.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Sort cards by type then name
    let mut cards: Vec<_> = deck_state.cards.iter().collect();
    cards.sort_by(|a, b| {
        let type_order = |t: &CardType| match t {
            CardType::Creature => 0,
            CardType::Weapon => 1,
            CardType::Artifact => 2,
        };
        type_order(&a.card_type)
            .cmp(&type_order(&b.card_type))
            .then(a.id.cmp(&b.id))
    });

    // Spawn new rows
    commands.entity(card_list_entity).with_children(|parent| {
        for card in cards {
            let probability = deck_state.get_probability(&card.id);
            let bar_color = get_bar_color_for_type(&card.card_type);

            // Get card color from game data
            let card_color = get_card_affinity_color(&card.id, &card.card_type, &game_data);

            // Get card name from game data
            let card_name = get_card_name(&card.id, &card.card_type, &game_data);

            spawn_card_row(
                parent,
                &card.id,
                &card_name,
                card_color,
                bar_color,
                probability,
                card.copies,
            );
        }
    });
}

fn get_card_affinity_color(id: &str, card_type: &CardType, game_data: &GameData) -> Color {
    match card_type {
        CardType::Creature => game_data
            .creatures
            .iter()
            .find(|c| c.id == id)
            .map(|c| get_color_for_affinity(&c.color))
            .unwrap_or(COLOR_GRAY),
        CardType::Weapon => game_data
            .weapons
            .iter()
            .find(|w| w.id == id)
            .map(|w| get_color_for_affinity(&w.color))
            .unwrap_or(COLOR_GRAY),
        CardType::Artifact => COLOR_GRAY, // Artifacts don't have color
    }
}

fn get_card_name(id: &str, card_type: &CardType, game_data: &GameData) -> String {
    match card_type {
        CardType::Creature => game_data
            .creatures
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| id.to_string()),
        CardType::Weapon => game_data
            .weapons
            .iter()
            .find(|w| w.id == id)
            .map(|w| w.name.clone())
            .unwrap_or_else(|| id.to_string()),
        CardType::Artifact => game_data
            .artifacts
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| id.to_string()),
    }
}

fn spawn_card_row(
    parent: &mut ChildBuilder,
    card_id: &str,
    card_name: &str,
    card_color: Color,
    bar_color: Color,
    probability: f32,
    copies: u32,
) {
    parent
        .spawn((
            DeckCardRow {
                card_id: card_id.to_string(),
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(8.0)),
                column_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|row| {
            // Color box
            row.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(card_color),
                BorderRadius::all(Val::Px(2.0)),
            ));

            // Card name
            row.spawn((
                Text::new(card_name),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                Node {
                    width: Val::Px(150.0),
                    ..default()
                },
            ));

            // Probability bar container
            row.spawn(Node {
                width: Val::Px(200.0),
                height: Val::Px(12.0),
                ..default()
            })
            .with_children(|bar_container| {
                // Background
                bar_container.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(BAR_EMPTY),
                    BorderRadius::all(Val::Px(6.0)),
                ));
                // Fill
                bar_container.spawn((
                    ProbabilityBarFill {
                        card_id: card_id.to_string(),
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(probability),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(bar_color),
                    BorderRadius::all(Val::Px(6.0)),
                ));
            });

            // Percentage text
            row.spawn((
                PercentageText {
                    card_id: card_id.to_string(),
                },
                Text::new(format!("{:.0}%", probability)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                Node {
                    width: Val::Px(45.0),
                    ..default()
                },
            ));

            // Minus button
            row.spawn((
                CardCopyButton {
                    card_id: card_id.to_string(),
                    delta: -1,
                },
                Button,
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
                BorderRadius::all(Val::Px(12.0)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("-"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                ));
            });

            // Copy count
            row.spawn((
                Text::new(format!("{}", copies)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                Node {
                    width: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));

            // Plus button
            row.spawn((
                CardCopyButton {
                    card_id: card_id.to_string(),
                    delta: 1,
                },
                Button,
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
                BorderRadius::all(Val::Px(12.0)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("+"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                ));
            });
        });
}

// =============================================================================
// AVAILABLE CARDS UPDATE SYSTEM
// =============================================================================

/// Updates the available cards section based on selected tab
pub fn deck_builder_available_cards_system(
    mut commands: Commands,
    deck_state: Res<DeckBuilderState>,
    game_data: Res<GameData>,
    game_phase: Res<GamePhase>,
    available_section: Query<Entity, With<AvailableCardsSection>>,
    existing_cards: Query<Entity, With<AvailableMiniCard>>,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    // Only rebuild if tab or deck changed
    if !deck_state.is_changed() {
        return;
    }

    let Ok(section_entity) = available_section.get_single() else {
        return;
    };

    // Despawn existing mini cards
    for entity in existing_cards.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Get available cards based on selected tab
    commands.entity(section_entity).with_children(|parent| {
        match deck_state.selected_tab {
            CardTab::Creatures => {
                for creature in &game_data.creatures {
                    // Show all creatures (even if in deck, just for visibility)
                    spawn_mini_card(
                        parent,
                        &creature.id,
                        &creature.name,
                        CardType::Creature,
                        creature.tier,
                        get_color_for_affinity(&creature.color),
                        deck_state.has_card(&creature.id),
                    );
                }
            }
            CardTab::Weapons => {
                for weapon in &game_data.weapons {
                    spawn_mini_card(
                        parent,
                        &weapon.id,
                        &weapon.name,
                        CardType::Weapon,
                        weapon.tier,
                        get_color_for_affinity(&weapon.color),
                        deck_state.has_card(&weapon.id),
                    );
                }
            }
            CardTab::Artifacts => {
                for artifact in &game_data.artifacts {
                    spawn_mini_card(
                        parent,
                        &artifact.id,
                        &artifact.name,
                        CardType::Artifact,
                        artifact.tier,
                        COLOR_GRAY,
                        deck_state.has_card(&artifact.id),
                    );
                }
            }
        }
    });
}

fn spawn_mini_card(
    parent: &mut ChildBuilder,
    card_id: &str,
    card_name: &str,
    card_type: CardType,
    tier: u8,
    card_color: Color,
    in_deck: bool,
) {
    let bg_color = if in_deck {
        Color::srgb(0.12, 0.12, 0.18)
    } else {
        MINI_CARD_BG
    };

    parent
        .spawn((
            AvailableMiniCard {
                card_id: card_id.to_string(),
                card_type,
            },
            Button,
            Node {
                width: Val::Px(80.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(if in_deck { card_color } else { PANEL_BORDER }),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|card| {
            // Tier indicator
            card.spawn((
                Text::new(format!("T{}", tier)),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(TEXT_MUTED),
            ));

            // Card name (truncated)
            let display_name = if card_name.len() > 10 {
                format!("{}...", &card_name[..8])
            } else {
                card_name.to_string()
            };
            card.spawn((
                Text::new(display_name),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));

            // Color indicator
            card.spawn((
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(card_color),
                BorderRadius::all(Val::Px(4.0)),
            ));
        });
}

// =============================================================================
// INTERACTION SYSTEMS
// =============================================================================

/// Handles tab button clicks
pub fn deck_builder_tab_system(
    mut deck_state: ResMut<DeckBuilderState>,
    game_phase: Res<GamePhase>,
    interaction_query: Query<(&Interaction, &CardTypeTab), Changed<Interaction>>,
    mut underline_query: Query<(&TabUnderline, &mut BackgroundColor)>,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    for (interaction, tab_btn) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            deck_state.selected_tab = tab_btn.tab;

            // Update tab visuals
            for (underline, mut bg) in underline_query.iter_mut() {
                *bg = if underline.tab == tab_btn.tab {
                    BackgroundColor(TAB_SELECTED)
                } else {
                    BackgroundColor(Color::NONE)
                };
            }
        }
    }
}

/// Handles starting weapon selection
pub fn deck_builder_weapon_select_system(
    mut deck_state: ResMut<DeckBuilderState>,
    game_data: Res<GameData>,
    game_phase: Res<GamePhase>,
    mut interaction_query: Query<
        (&Interaction, &StartingWeaponCard, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
    mut all_weapon_cards: Query<
        (&StartingWeaponCard, &mut BackgroundColor, &mut BorderColor),
        Without<Interaction>,
    >,
    mut selected_text: Query<&mut Text, With<SelectedWeaponText>>,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    for (interaction, card, mut bg, mut border) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Update selected weapon
                deck_state.starting_weapon = Some(card.weapon_id.clone());

                // Update all weapon cards to reflect selection
                // First reset all cards
                for (other_card, mut other_bg, mut other_border) in all_weapon_cards.iter_mut() {
                    if other_card.weapon_id == card.weapon_id {
                        *other_bg = BackgroundColor(Color::srgba(0.13, 0.77, 0.37, 0.15));
                        *other_border = BorderColor(ACCENT_GREEN);
                    } else {
                        *other_bg = BackgroundColor(MINI_CARD_BG);
                        *other_border = BorderColor(PANEL_BORDER);
                    }
                }

                // Update the pressed card
                *bg = BackgroundColor(Color::srgba(0.13, 0.77, 0.37, 0.15));
                *border = BorderColor(ACCENT_GREEN);

                // Update selected text
                if let Some(weapon) = game_data.weapons.iter().find(|w| w.id == card.weapon_id) {
                    for mut text in selected_text.iter_mut() {
                        **text = weapon.name.clone();
                    }
                }
            }
            Interaction::Hovered => {
                if deck_state.starting_weapon.as_ref() != Some(&card.weapon_id) {
                    *border = BorderColor(ACCENT_GREEN_HOVER);
                }
            }
            Interaction::None => {
                if deck_state.starting_weapon.as_ref() == Some(&card.weapon_id) {
                    *bg = BackgroundColor(Color::srgba(0.13, 0.77, 0.37, 0.15));
                    *border = BorderColor(ACCENT_GREEN);
                } else {
                    *bg = BackgroundColor(MINI_CARD_BG);
                    *border = BorderColor(PANEL_BORDER);
                }
            }
        }
    }
}

/// Handles +/- button clicks
pub fn deck_builder_button_system(
    mut deck_state: ResMut<DeckBuilderState>,
    game_phase: Res<GamePhase>,
    mut interaction_query: Query<
        (&Interaction, &CardCopyButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    for (interaction, btn, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if btn.delta > 0 {
                    deck_state.increment_copies(&btn.card_id);
                } else {
                    deck_state.decrement_copies(&btn.card_id);
                }
            }
            Interaction::Hovered => {
                *bg = if btn.delta > 0 {
                    BackgroundColor(ACCENT_GREEN_HOVER)
                } else {
                    BackgroundColor(ACCENT_RED_HOVER)
                };
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_BG);
            }
        }
    }
}

/// Handles available card clicks (add to deck)
pub fn deck_builder_add_card_system(
    mut deck_state: ResMut<DeckBuilderState>,
    game_phase: Res<GamePhase>,
    mut interaction_query: Query<
        (&Interaction, &AvailableMiniCard, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    for (interaction, card, mut border) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                deck_state.add_card(card.card_type.clone(), &card.card_id);
            }
            Interaction::Hovered => {
                *border = BorderColor(ACCENT_GREEN);
            }
            Interaction::None => {
                if deck_state.has_card(&card.card_id) {
                    // Keep colored border if in deck
                } else {
                    *border = BorderColor(PANEL_BORDER);
                }
            }
        }
    }
}

/// Handles Start Run button
pub fn deck_builder_start_run_system(
    mut commands: Commands,
    deck_state: Res<DeckBuilderState>,
    game_data: Res<GameData>,
    mut game_phase: ResMut<GamePhase>,
    mut player_deck: ResMut<PlayerDeck>,
    mut affinity_state: ResMut<AffinityState>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartRunButton>),
    >,
) {
    for (interaction, mut bg) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if !deck_state.is_empty() {
                    // Convert deck builder state to player deck
                    *player_deck = deck_state.to_player_deck();

                    // Spawn starting weapon if one is selected
                    if let Some(ref weapon_id) = deck_state.starting_weapon {
                        spawn_weapon(&mut commands, &game_data, &mut affinity_state, weapon_id);
                    }

                    // Transition to playing
                    *game_phase = GamePhase::Playing;
                }
            }
            Interaction::Hovered => {
                *bg = if deck_state.is_empty() {
                    BackgroundColor(TEXT_MUTED)
                } else {
                    BackgroundColor(ACCENT_GREEN_HOVER)
                };
            }
            Interaction::None => {
                *bg = if deck_state.is_empty() {
                    BackgroundColor(TEXT_MUTED)
                } else {
                    BackgroundColor(ACCENT_GREEN)
                };
            }
        }
    }
}

/// Handles Clear Deck button
pub fn deck_builder_clear_deck_system(
    mut deck_state: ResMut<DeckBuilderState>,
    game_phase: Res<GamePhase>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<ClearDeckButton>),
    >,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    for (interaction, mut bg, mut border) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                deck_state.clear();
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(ACCENT_RED);
                *border = BorderColor(ACCENT_RED);
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::NONE);
                *border = BorderColor(ACCENT_RED);
            }
        }
    }
}

/// Updates footer text (total cards and breakdown)
pub fn deck_builder_footer_system(
    deck_state: Res<DeckBuilderState>,
    game_phase: Res<GamePhase>,
    mut total_text: Query<&mut Text, (With<TotalCardsText>, Without<TypeBreakdownText>)>,
    mut breakdown_text: Query<&mut Text, (With<TypeBreakdownText>, Without<TotalCardsText>)>,
) {
    if *game_phase != GamePhase::DeckBuilder {
        return;
    }

    if !deck_state.is_changed() {
        return;
    }

    // Update total count
    for mut text in total_text.iter_mut() {
        **text = format!("Total: {} cards", deck_state.total_copies());
    }

    // Update type breakdown
    let (creatures, weapons, artifacts) = deck_state.type_breakdown();
    for mut text in breakdown_text.iter_mut() {
        **text = format!(
            "Creatures {:.0}% | Weapons {:.0}% | Artifacts {:.0}%",
            creatures, weapons, artifacts
        );
    }
}
