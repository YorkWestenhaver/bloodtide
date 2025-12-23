use bevy::prelude::*;
use rand::Rng;

use std::collections::HashMap;

use crate::components::{Creature, CreatureColor, CreatureStats};
use crate::components::weapon::{Weapon, WeaponData, WeaponStats};
use crate::resources::{AffinityState, ArtifactBuffs, DebugSettings, GameData, GameState};
use crate::systems::creature_xp::EvolutionReadyState;
use crate::systems::death::RespawnQueue;
use crate::systems::tooltips::{TooltipContent, TooltipTarget};

// =============================================================================
// UI PANEL CONSTANTS
// =============================================================================

const PANEL_BACKGROUND: Color = Color::srgba(0.0, 0.0, 0.0, 0.75);
const PANEL_PADDING: f32 = 10.0;
const PANEL_MARGIN: f32 = 10.0;

// Creature panel
const CREATURE_PANEL_WIDTH: f32 = 220.0;
const CREATURE_ROW_HEIGHT: f32 = 50.0;

// Artifact panel
const ARTIFACT_PANEL_WIDTH: f32 = 250.0;
const ARTIFACT_PANEL_MAX_HEIGHT: f32 = 200.0;

// Affinity display
const AFFINITY_BAR_WIDTH: f32 = 150.0;
const AFFINITY_BAR_HEIGHT: f32 = 16.0;

// Card popup
const POPUP_WIDTH: f32 = 300.0;
const POPUP_HEIGHT: f32 = 150.0;
const POPUP_DURATION: f32 = 2.5;

// Wave announcement
const WAVE_ANNOUNCEMENT_DURATION: f32 = 1.5;

// =============================================================================
// MARKER COMPONENTS
// =============================================================================

/// Marker for the creature panel container
#[derive(Component)]
pub struct CreaturePanel;

/// Marker for creature panel content (the list of creatures)
#[derive(Component)]
pub struct CreaturePanelContent;

/// Marker for the artifact panel container
#[derive(Component)]
pub struct ArtifactPanel;

/// Marker for artifact panel content
#[derive(Component)]
pub struct ArtifactPanelContent;

/// Marker for the affinity display container (now "Weapons & Affinity")
#[derive(Component)]
pub struct AffinityDisplay;

/// Marker for affinity display content
#[derive(Component)]
pub struct AffinityDisplayContent;

/// Marker for the weapon stats section within affinity display
#[derive(Component)]
pub struct WeaponStatsDisplay;

/// Marker for individual weapon row in the list
#[derive(Component)]
pub struct WeaponListItem {
    pub weapon_entity: Entity,
}

/// Marker for weapon stats summary text
#[derive(Component)]
pub struct WeaponStatsSummary;

/// Card roll popup component
#[derive(Component)]
pub struct CardRollPopup {
    pub timer: Timer,
    pub card_name: String,
    pub card_type: String,
    pub tier: u8,
}

/// Wave announcement component
#[derive(Component)]
pub struct WaveAnnouncement {
    pub timer: Timer,
    pub wave_number: u32,
}

/// Resource to track last announced wave
#[derive(Resource, Default)]
pub struct WaveAnnouncementState {
    pub last_announced_wave: u32,
}

/// Resource to track last rolled card for popup
#[derive(Resource, Default)]
pub struct CardRollState {
    pub pending_popup: Option<(String, String, u8)>, // (name, type, tier)
}

// =============================================================================
// CREATURE PANEL
// =============================================================================

/// Spawns the creature panel on the right side of the screen
pub fn spawn_creature_panel_system(mut commands: Commands) {
    commands
        .spawn((
            CreaturePanel,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(PANEL_MARGIN),
                top: Val::Px(PANEL_MARGIN),
                width: Val::Px(CREATURE_PANEL_WIDTH),
                max_height: Val::Percent(70.0),
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Creatures"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Content container
            parent.spawn((
                CreaturePanelContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ));
        });
}

/// Info about evolution readiness for a creature type
struct EvolutionInfo {
    is_ready: bool,
    evolves_into_name: String,
    count: usize,
    evolution_count: u32,
}

/// Updates the creature panel to show current creatures and respawning creatures
pub fn update_creature_panel_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &CreatureStats), With<Creature>>,
    respawn_queue: Res<RespawnQueue>,
    game_data: Res<GameData>,
    debug_settings: Res<DebugSettings>,
    evolution_state: Res<EvolutionReadyState>,
    panel_content_query: Query<Entity, With<CreaturePanelContent>>,
) {
    let Ok(panel_entity) = panel_content_query.get_single() else {
        return;
    };

    // Count creatures by ID to determine evolution readiness
    let mut creature_counts: HashMap<String, (usize, u32, String)> = HashMap::new(); // (count, evolution_count, evolves_into)
    for (_, stats) in creature_query.iter() {
        creature_counts
            .entry(stats.id.clone())
            .or_insert((0, stats.evolution_count, stats.evolves_into.clone()))
            .0 += 1;
    }

    // Build evolution info map
    let mut evolution_info: HashMap<String, EvolutionInfo> = HashMap::new();
    for (id, (count, evolution_count, evolves_into)) in &creature_counts {
        let is_ready = !evolves_into.is_empty()
            && *evolution_count > 0
            && *count >= *evolution_count as usize;

        let evolves_into_name = if is_ready {
            game_data
                .creatures
                .iter()
                .find(|c| c.id == *evolves_into)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| evolves_into.clone())
        } else {
            String::new()
        };

        evolution_info.insert(id.clone(), EvolutionInfo {
            is_ready,
            evolves_into_name,
            count: *count,
            evolution_count: *evolution_count,
        });
    }

    // Clear existing content
    commands.entity(panel_entity).despawn_descendants();

    // Add active creatures
    commands.entity(panel_entity).with_children(|parent| {
        // Group creatures by ID for display
        let mut creatures_by_id: HashMap<String, Vec<(Entity, CreatureStats)>> = HashMap::new();
        for (entity, stats) in creature_query.iter() {
            creatures_by_id
                .entry(stats.id.clone())
                .or_default()
                .push((entity, stats.clone()));
        }

        // Sort creature groups by name for consistent display
        let mut sorted_groups: Vec<_> = creatures_by_id.into_iter().collect();
        sorted_groups.sort_by(|a, b| {
            a.1.first().map(|(_, s)| &s.name).cmp(&b.1.first().map(|(_, s)| &s.name))
        });

        for (creature_id, creatures) in sorted_groups {
            let info = evolution_info.get(&creature_id);
            let is_evolution_ready = info.map(|i| i.is_ready).unwrap_or(false);
            let evolution_count = info.map(|i| i.evolution_count).unwrap_or(3);

            // Sort by level to show which ones will be consumed (lowest first)
            let mut sorted_creatures = creatures;
            sorted_creatures.sort_by(|a, b| a.1.level.cmp(&b.1.level));

            for (idx, (creature_entity, stats)) in sorted_creatures.iter().enumerate() {
                // Show green arrow for creatures that will be consumed (first N where N = evolution_count)
                let will_be_consumed = is_evolution_ready && idx < evolution_count as usize;
                spawn_creature_row(
                    parent,
                    *creature_entity,
                    stats,
                    debug_settings.show_expanded_creature_stats,
                    will_be_consumed,
                );
            }

            // Show evolution target preview after the group
            if is_evolution_ready {
                if let Some(info) = evolution_info.get(&creature_id) {
                    spawn_evolution_preview(
                        parent,
                        &info.evolves_into_name,
                        debug_settings.auto_evolve,
                        debug_settings.evolution_hotkey,
                    );
                }
            }
        }

        // Add respawning creatures
        for entry in &respawn_queue.entries {
            // Look up creature name from game data
            let name = game_data
                .creatures
                .iter()
                .find(|c| c.id == entry.creature_id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| entry.creature_id.clone());

            let remaining = entry.timer.remaining_secs();

            // Create a minimal stats struct for display
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    height: Val::Px(36.0),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
            )).with_children(|row| {
                // Name (grayed out)
                row.spawn((
                    Text::new(name),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
                // Respawn timer
                row.spawn((
                    Text::new(format!("Respawn: {:.0}s", remaining)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.3)),
                ));
            });
        }
    });
}

fn spawn_creature_row(
    parent: &mut ChildBuilder,
    creature_entity: Entity,
    stats: &CreatureStats,
    show_expanded: bool,
    will_be_consumed: bool,
) {
    let hp_percent = (stats.current_hp / stats.max_hp).clamp(0.0, 1.0) as f32;
    let hp_color = if hp_percent > 0.6 {
        Color::srgb(0.3, 0.8, 0.3)
    } else if hp_percent > 0.3 {
        Color::srgb(0.8, 0.8, 0.3)
    } else {
        Color::srgb(0.8, 0.3, 0.3)
    };

    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(4.0)),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.5)),
        // Add interaction for tooltip hover detection
        Interaction::default(),
        // Add tooltip target
        TooltipTarget {
            content: TooltipContent::Creature(creature_entity),
        },
    )).with_children(|row| {
        // Top row: Name (with evolution indicator), Level, Kills
        row.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|top| {
            // Name with optional evolution indicator
            top.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            }).with_children(|name_row| {
                // Green up arrow if this creature will be consumed in evolution
                if will_be_consumed {
                    name_row.spawn((
                        Text::new("▲ "),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.3, 0.9, 0.3)), // Green
                    ));
                }
                // Name
                name_row.spawn((
                    Text::new(&stats.name),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(stats.color.to_bevy_color()),
                ));
            });
            // Level and kills
            top.spawn((
                Text::new(format!("Lv.{} K:{}", stats.level, stats.kills)),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });

        // HP bar
        row.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(8.0),
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        }).with_children(|bar_container| {
            // Background
            bar_container.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ));
            // Fill
            bar_container.spawn((
                Node {
                    width: Val::Percent(hp_percent * 100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(hp_color),
            ));
        });

        // Expanded stats (if enabled)
        if show_expanded {
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(4.0)),
                ..default()
            }).with_children(|expanded| {
                expanded.spawn((
                    Text::new(format!("DMG: {:.0} | SPD: {:.0}", stats.base_damage, stats.movement_speed)),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
                if stats.crit_t1 > 0.0 || stats.crit_t2 > 0.0 || stats.crit_t3 > 0.0 {
                    expanded.spawn((
                        Text::new(format!("Crit: {:.0}%/{:.0}%/{:.0}%", stats.crit_t1, stats.crit_t2, stats.crit_t3)),
                        TextFont { font_size: 10.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                }
            });
        }
    });
}

/// Spawn the evolution preview row showing what creatures will evolve into
fn spawn_evolution_preview(
    parent: &mut ChildBuilder,
    evolves_into_name: &str,
    auto_evolve: bool,
    evolution_hotkey: KeyCode,
) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        margin: UiRect::bottom(Val::Px(8.0)),
        padding: UiRect::new(Val::Px(12.0), Val::Px(4.0), Val::Px(2.0), Val::Px(2.0)),
        ..default()
    }).with_children(|col| {
        // Evolution target: "→ Flame Fiend"
        col.spawn((
            Text::new(format!("→ {}", evolves_into_name)),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.5, 0.7, 0.5)), // Grayish green
        ));

        // Keybind hint (only in manual mode)
        if !auto_evolve {
            col.spawn((
                Text::new(format!("[{:?}] to evolve", evolution_hotkey)),
                TextFont { font_size: 10.0, ..default() },
                TextColor(Color::srgb(0.4, 0.6, 0.4)),
            ));
        }
    });
}

// =============================================================================
// ARTIFACT PANEL
// =============================================================================

/// Spawns the artifact panel on the bottom-left of the screen
pub fn spawn_artifact_panel_system(mut commands: Commands) {
    commands
        .spawn((
            ArtifactPanel,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PANEL_MARGIN),
                bottom: Val::Px(PANEL_MARGIN),
                width: Val::Px(ARTIFACT_PANEL_WIDTH),
                max_height: Val::Px(ARTIFACT_PANEL_MAX_HEIGHT),
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Artifacts"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Content container
            parent.spawn((
                ArtifactPanelContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ));
        });
}

/// Updates the artifact panel to show acquired artifacts
pub fn update_artifact_panel_system(
    mut commands: Commands,
    artifact_buffs: Res<ArtifactBuffs>,
    game_data: Res<GameData>,
    panel_content_query: Query<Entity, With<ArtifactPanelContent>>,
) {
    let Ok(panel_entity) = panel_content_query.get_single() else {
        return;
    };

    // Clear existing content
    commands.entity(panel_entity).despawn_descendants();

    // Add artifacts
    commands.entity(panel_entity).with_children(|parent| {
        for artifact_id in &artifact_buffs.acquired_artifacts {
            if let Some(artifact) = game_data.artifacts.iter().find(|a| a.id == *artifact_id) {
                let tier_color = get_tier_color(artifact.tier);

                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                )).with_children(|row| {
                    // Name
                    row.spawn((
                        Text::new(&artifact.name),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(tier_color),
                    ));
                    // Effect
                    row.spawn((
                        Text::new(format_artifact_effect(artifact)),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
            }
        }

        // Show "None" if no artifacts
        if artifact_buffs.acquired_artifacts.is_empty() {
            parent.spawn((
                Text::new("None yet"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        }
    });
}

fn format_artifact_effect(artifact: &crate::data::Artifact) -> String {
    let mut effects = Vec::new();

    if artifact.damage_bonus != 0.0 {
        effects.push(format!("+{:.0}% Damage", artifact.damage_bonus));
    }
    if artifact.attack_speed_bonus != 0.0 {
        effects.push(format!("+{:.0}% Attack Speed", artifact.attack_speed_bonus));
    }
    if artifact.hp_bonus != 0.0 {
        effects.push(format!("+{:.0}% HP", artifact.hp_bonus));
    }
    if artifact.crit_t1_bonus != 0.0 {
        effects.push(format!("+{:.0}% Crit", artifact.crit_t1_bonus));
    }

    if effects.is_empty() {
        artifact.description.chars().take(40).collect::<String>() + "..."
    } else {
        effects.join(", ")
    }
}

// =============================================================================
// WEAPONS & AFFINITY DISPLAY
// =============================================================================

/// Spawns the weapons & affinity display on the left side
pub fn spawn_affinity_display_system(mut commands: Commands) {
    commands
        .spawn((
            AffinityDisplay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PANEL_MARGIN),
                top: Val::Px(PANEL_MARGIN), // Top left corner
                width: Val::Px(250.0),
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(PANEL_BACKGROUND),
        ))
        .with_children(|parent| {
            // Main header: WEAPONS & AFFINITY
            parent.spawn((
                Text::new("WEAPONS & AFFINITY"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // WEAPONS section header
            parent.spawn((
                Text::new("WEAPONS"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Weapon stats content container
            parent.spawn((
                WeaponStatsDisplay,
                Node {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Separator line
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(4.0), Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            ));

            // AFFINITY section header
            parent.spawn((
                Text::new("AFFINITY"),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Affinity bars content container
            parent.spawn((
                AffinityDisplayContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ));
        });
}

/// Updates the affinity display to show current affinity levels
pub fn update_affinity_display_system(
    mut commands: Commands,
    affinity_state: Res<AffinityState>,
    display_content_query: Query<Entity, With<AffinityDisplayContent>>,
) {
    let Ok(content_entity) = display_content_query.get_single() else {
        return;
    };

    commands.entity(content_entity).despawn_descendants();

    commands.entity(content_entity).with_children(|parent| {
        let colors = [
            (CreatureColor::Red, "Red", affinity_state.red),
            (CreatureColor::Blue, "Blue", affinity_state.blue),
            (CreatureColor::Green, "Green", affinity_state.green),
            (CreatureColor::White, "White", affinity_state.white),
            (CreatureColor::Black, "Black", affinity_state.black),
        ];

        let mut has_any = false;
        for (color, name, value) in colors {
            if value > 0.0 {
                has_any = true;
                spawn_affinity_bar(parent, color, name, value);
            }
        }

        if !has_any {
            parent.spawn((
                Text::new("None"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        }
    });
}

/// Updates the weapon stats display section
pub fn update_weapon_stats_display_system(
    mut commands: Commands,
    weapon_query: Query<(Entity, &WeaponData, &WeaponStats), With<Weapon>>,
    debug_settings: Res<DebugSettings>,
    game_data: Res<GameData>,
    weapon_display_query: Query<Entity, With<WeaponStatsDisplay>>,
) {
    let Ok(display_entity) = weapon_display_query.get_single() else {
        return;
    };

    // Clear existing content
    commands.entity(display_entity).despawn_descendants();

    // Collect all weapons
    let weapons: Vec<_> = weapon_query.iter().collect();

    commands.entity(display_entity).with_children(|parent| {
        if weapons.is_empty() {
            // No weapons equipped message
            parent.spawn((
                Text::new("No weapons equipped"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Show zero stats
            parent.spawn((
                WeaponStatsSummary,
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.0)),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
            )).with_children(|summary| {
                summary.spawn((
                    Text::new("Wpn Damage: 0"),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
                summary.spawn((
                    Text::new("Wpn Speed: 0.0/sec"),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
                summary.spawn((
                    Text::new("Wpn Count: 0"),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            });
        } else {
            // Calculate totals
            let mut total_damage = 0.0;
            let mut fastest_speed = 0.0;

            for (_, _, stats) in &weapons {
                total_damage += stats.auto_damage;
                if stats.auto_speed > fastest_speed {
                    fastest_speed = stats.auto_speed;
                }
            }

            // Weapon list
            for (weapon_entity, data, stats) in &weapons {
                spawn_weapon_row(
                    parent,
                    *weapon_entity,
                    data,
                    stats,
                    debug_settings.show_advanced_tooltips,
                    &game_data,
                );
            }

            // Weapon stats summary box
            parent.spawn((
                WeaponStatsSummary,
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.0)),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
            )).with_children(|summary| {
                summary.spawn((
                    Text::new(format!("Wpn Damage: {:.0}", total_damage)),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
                summary.spawn((
                    Text::new(format!("Wpn Speed: {:.1}/sec", fastest_speed)),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
                summary.spawn((
                    Text::new(format!("Wpn Count: {}", weapons.len())),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            });
        }
    });
}

/// Spawns a single weapon row in the weapon list
fn spawn_weapon_row(
    parent: &mut ChildBuilder,
    weapon_entity: Entity,
    data: &WeaponData,
    stats: &WeaponStats,
    show_tooltips: bool,
    game_data: &GameData,
) {
    let tier_color = get_tier_color(data.tier);

    let mut row = parent.spawn((
        WeaponListItem { weapon_entity },
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(2.0), Val::Px(2.0)),
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)),
    ));

    // Add tooltip support if enabled
    if show_tooltips {
        row.insert((
            Interaction::default(),
            TooltipTarget {
                content: TooltipContent::TitleAndDescription {
                    title: format!("{} (T{})", data.name, data.tier),
                    description: build_weapon_tooltip_description(data, stats, game_data),
                },
            },
        ));
    }

    row.with_children(|row_inner| {
        // Weapon name with tier
        row_inner.spawn((
            Text::new(format!("{} (T{})", data.name, data.tier)),
            TextFont { font_size: 11.0, ..default() },
            TextColor(tier_color),
        ));

        // Color indicator (small colored box)
        row_inner.spawn((
            Node {
                width: Val::Px(8.0),
                height: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(data.color.to_bevy_color()),
        ));
    });
}

/// Builds the tooltip description for a weapon
fn build_weapon_tooltip_description(data: &WeaponData, stats: &WeaponStats, game_data: &GameData) -> String {
    let mut lines = Vec::new();

    lines.push(format!("Damage: {:.0}", stats.auto_damage));
    lines.push(format!("Attack Speed: {:.2}/sec", stats.auto_speed));
    lines.push(format!("Range: {:.0}", stats.auto_range));
    lines.push(format!("Affinity: +{:.0} {}", data.affinity_amount, format_color_name(&data.color)));

    if stats.projectile_count > 1 {
        lines.push(format!("Projectiles: {}", stats.projectile_count));
    }

    if stats.projectile_pattern != "single" {
        lines.push(format!("Pattern: {}", stats.projectile_pattern));
    }

    // Check for evolution info
    if let Some(weapon_data) = game_data.weapons.iter().find(|w| w.id == data.id) {
        if !weapon_data.evolution_recipe.is_empty() {
            lines.push(format!("Evolves from: {}", weapon_data.evolution_recipe.join(" + ")));
        }
    }

    lines.join("\n")
}

/// Format color name for display
fn format_color_name(color: &CreatureColor) -> &'static str {
    match color {
        CreatureColor::Red => "Red",
        CreatureColor::Blue => "Blue",
        CreatureColor::Green => "Green",
        CreatureColor::White => "White",
        CreatureColor::Black => "Black",
        CreatureColor::Colorless => "Colorless",
    }
}

fn spawn_affinity_bar(parent: &mut ChildBuilder, color: CreatureColor, name: &str, value: f64) {
    // Thresholds: 11, 26, 51, 76, 100
    let thresholds = [11.0, 26.0, 51.0, 76.0, 100.0];
    let max_value = 100.0;
    let fill_percent = ((value / max_value).min(1.0) * 100.0) as f32;

    // Determine current threshold level
    let threshold_level = thresholds.iter().filter(|&&t| value >= t).count();
    let is_active = threshold_level > 0;

    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        margin: UiRect::bottom(Val::Px(4.0)),
        ..default()
    }).with_children(|row| {
        // Color name
        row.spawn((
            Text::new(format!("{}: ", name)),
            TextFont { font_size: 11.0, ..default() },
            TextColor(color.to_bevy_color()),
            Node {
                width: Val::Px(45.0),
                ..default()
            },
        ));

        // Value
        row.spawn((
            Text::new(format!("{:.0}", value)),
            TextFont { font_size: 11.0, ..default() },
            TextColor(if is_active { Color::srgb(0.3, 1.0, 0.3) } else { Color::WHITE }),
            Node {
                width: Val::Px(30.0),
                ..default()
            },
        ));

        // Bar background
        row.spawn(Node {
            width: Val::Px(AFFINITY_BAR_WIDTH),
            height: Val::Px(AFFINITY_BAR_HEIGHT),
            ..default()
        }).with_children(|bar| {
            // Background
            bar.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            ));

            // Fill
            bar.spawn((
                Node {
                    width: Val::Percent(fill_percent),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(color.to_bevy_color().with_alpha(0.7)),
            ));

            // Threshold markers
            for (i, &threshold) in thresholds.iter().enumerate() {
                let marker_pos = (threshold / max_value * 100.0) as f32;
                let is_reached = value >= threshold;
                bar.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(marker_pos),
                        width: Val::Px(2.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(if is_reached {
                        Color::srgb(1.0, 1.0, 0.3)
                    } else {
                        Color::srgb(0.4, 0.4, 0.4)
                    }),
                ));
            }
        });
    });
}

// =============================================================================
// CARD ROLL POPUP
// =============================================================================

/// Shows card roll popup when a card is rolled on level up
pub fn show_card_roll_popup_system(
    mut commands: Commands,
    mut card_roll_state: ResMut<CardRollState>,
    existing_popup: Query<Entity, With<CardRollPopup>>,
) {
    // Only show if there's a pending popup and no existing popup
    if let Some((name, card_type, tier)) = card_roll_state.pending_popup.take() {
        // Don't spawn if one already exists
        if !existing_popup.is_empty() {
            return;
        }

        let tier_color = get_tier_color(tier);

        commands
            .spawn((
                CardRollPopup {
                    timer: Timer::from_seconds(POPUP_DURATION, TimerMode::Once),
                    card_name: name.clone(),
                    card_type: card_type.clone(),
                    tier,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(30.0),
                    width: Val::Px(POPUP_WIDTH),
                    margin: UiRect::left(Val::Px(-POPUP_WIDTH / 2.0)),
                    padding: UiRect::all(Val::Px(20.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.95)),
                BorderColor(tier_color),
                Outline {
                    width: Val::Px(3.0),
                    color: tier_color,
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Card type
                parent.spawn((
                    Text::new(format!("New {}!", card_type)),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                // Card name
                parent.spawn((
                    Text::new(name),
                    TextFont { font_size: 28.0, ..default() },
                    TextColor(tier_color),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                // Tier
                let tier_name = match tier {
                    1 => "Common",
                    2 => "Uncommon",
                    3 => "Rare",
                    4 => "Epic",
                    _ => "Legendary",
                };
                parent.spawn((
                    Text::new(format!("Tier {} - {}", tier, tier_name)),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(tier_color.with_alpha(0.8)),
                ));
            });
    }
}

/// Updates and dismisses the card roll popup
pub fn card_roll_popup_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut popup_query: Query<(Entity, &mut CardRollPopup, &mut BackgroundColor)>,
) {
    for (entity, mut popup, mut bg) in popup_query.iter_mut() {
        popup.timer.tick(time.delta());

        // Fade out in last 0.5 seconds
        let remaining = popup.timer.remaining_secs();
        if remaining < 0.5 {
            let alpha = remaining / 0.5;
            bg.0 = Color::srgba(0.1, 0.1, 0.15, 0.95 * alpha);
        }

        // Dismiss on click or timer
        if popup.timer.finished() || mouse_input.just_pressed(MouseButton::Left) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// =============================================================================
// WAVE ANNOUNCEMENT
// =============================================================================

/// Shows wave announcement when wave changes
pub fn show_wave_announcement_system(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut wave_state: ResMut<WaveAnnouncementState>,
    existing_announcement: Query<Entity, With<WaveAnnouncement>>,
) {
    // Check if wave changed
    if game_state.current_wave != wave_state.last_announced_wave && game_state.current_wave > 1 {
        wave_state.last_announced_wave = game_state.current_wave;

        // Don't spawn if one already exists
        if !existing_announcement.is_empty() {
            return;
        }

        let is_milestone = game_state.current_wave % 10 == 0;
        let text_color = if is_milestone {
            Color::srgb(1.0, 0.85, 0.2) // Gold for milestones
        } else {
            Color::WHITE
        };

        commands.spawn((
            WaveAnnouncement {
                timer: Timer::from_seconds(WAVE_ANNOUNCEMENT_DURATION, TimerMode::Once),
                wave_number: game_state.current_wave,
            },
            Text2d::new(format!("WAVE {}", game_state.current_wave)),
            TextFont { font_size: 72.0, ..default() },
            TextColor(text_color),
            Transform::from_xyz(0.0, 100.0, 100.0).with_scale(Vec3::splat(0.5)),
        ));
    }
}

/// Updates wave announcement animation
pub fn wave_announcement_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut announcement_query: Query<(Entity, &mut WaveAnnouncement, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut announcement, mut transform, mut text_color) in announcement_query.iter_mut() {
        announcement.timer.tick(time.delta());

        let progress = announcement.timer.fraction();

        // Scale up quickly, then fade out
        if progress < 0.3 {
            // Scale up phase
            let scale_progress = progress / 0.3;
            transform.scale = Vec3::splat(0.5 + scale_progress * 0.5);
        } else {
            // Hold and fade phase
            transform.scale = Vec3::splat(1.0);
            let fade_progress = (progress - 0.3) / 0.7;
            let alpha = 1.0 - fade_progress;
            text_color.0 = text_color.0.with_alpha(alpha);
        }

        if announcement.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// DAMAGE NUMBER IMPROVEMENTS
// =============================================================================

/// Resource to track recent damage number positions for offset calculation
#[derive(Resource, Default)]
pub struct DamageNumberOffsets {
    pub recent_positions: Vec<(Vec2, f32)>, // (position, spawn_time)
}

/// Calculates offset for a new damage number to avoid overlap
pub fn calculate_damage_number_offset(
    offsets: &mut DamageNumberOffsets,
    base_pos: Vec2,
    current_time: f32,
) -> Vec2 {
    // Clean up old entries (older than 0.5 seconds)
    offsets.recent_positions.retain(|(_, time)| current_time - time < 0.5);

    // Random horizontal offset
    let mut rng = rand::thread_rng();
    let x_offset = rng.gen_range(-20.0..20.0);

    // Calculate vertical offset based on nearby recent numbers
    let nearby_count = offsets
        .recent_positions
        .iter()
        .filter(|(pos, _)| pos.distance(base_pos) < 30.0)
        .count();

    let y_offset = nearby_count as f32 * 15.0;

    // Record this position
    offsets.recent_positions.push((base_pos + Vec2::new(x_offset, y_offset), current_time));

    Vec2::new(x_offset, y_offset)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn get_tier_color(tier: u8) -> Color {
    match tier {
        1 => Color::srgb(0.8, 0.8, 0.8),     // Common - gray/white
        2 => Color::srgb(0.3, 0.8, 0.3),     // Uncommon - green
        3 => Color::srgb(0.3, 0.5, 1.0),     // Rare - blue
        4 => Color::srgb(0.7, 0.3, 0.9),     // Epic - purple
        _ => Color::srgb(1.0, 0.75, 0.2),    // Legendary - gold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_colors_are_distinct() {
        let t1 = get_tier_color(1);
        let t2 = get_tier_color(2);
        let t3 = get_tier_color(3);
        let t4 = get_tier_color(4);
        let t5 = get_tier_color(5);

        // Just verify they don't panic and return colors
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
        assert_ne!(t3, t4);
        assert_ne!(t4, t5);
    }

    #[test]
    fn wave_announcement_state_default() {
        let state = WaveAnnouncementState::default();
        assert_eq!(state.last_announced_wave, 0);
    }

    #[test]
    fn card_roll_state_default() {
        let state = CardRollState::default();
        assert!(state.pending_popup.is_none());
    }
}
