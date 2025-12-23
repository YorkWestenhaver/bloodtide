use bevy::prelude::*;

use crate::components::Creature;
use crate::resources::{ArtifactBuffs, DebugSettings, Director, GameState};

// =============================================================================
// COMPONENTS
// =============================================================================

/// Marker component for the main HUD container
#[derive(Component)]
pub struct HudContainer;

/// Marker component for level progress bar
#[derive(Component)]
pub struct LevelProgressBar;

/// Marker component for level progress fill
#[derive(Component)]
pub struct LevelProgressFill;

/// Marker component for HUD line 1 (Level with progress)
#[derive(Component)]
pub struct HudLine1;

/// Marker component for HUD line 2 (Kills, Wave)
#[derive(Component)]
pub struct HudLine2;

/// Marker component for HUD line 3 (Creatures, DPS, status)
#[derive(Component)]
pub struct HudLine3;

// Backwards compat
#[derive(Component)]
pub struct HudText;

// =============================================================================
// CONSTANTS
// =============================================================================

const PROGRESS_BAR_WIDTH: f32 = 100.0;
const PROGRESS_BAR_HEIGHT: f32 = 8.0;
const PROGRESS_BAR_BG: Color = Color::srgb(0.2, 0.2, 0.25);
const PROGRESS_BAR_FILL: Color = Color::srgb(0.4, 0.8, 0.3);

// =============================================================================
// SYSTEMS
// =============================================================================

/// System that spawns the UI on startup
pub fn spawn_ui_system(mut commands: Commands) {
    // Spawn HUD container with multiple lines
    commands
        .spawn((
            HudContainer,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Line 1: Level with progress bar
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            }).with_children(|row| {
                // Level text
                row.spawn((
                    HudLine1,
                    HudText,
                    Text::new("Level 1"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Progress bar container
                row.spawn(Node {
                    width: Val::Px(PROGRESS_BAR_WIDTH),
                    height: Val::Px(PROGRESS_BAR_HEIGHT),
                    ..default()
                }).with_children(|bar| {
                    // Background
                    bar.spawn((
                        LevelProgressBar,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(PROGRESS_BAR_BG),
                    ));

                    // Fill
                    bar.spawn((
                        LevelProgressFill,
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(PROGRESS_BAR_FILL),
                    ));
                });
            });

            // Line 2: Kills with rate, Wave
            parent.spawn((
                HudLine2,
                Text::new("Kills: 0 | Wave: 1"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Line 3: Creatures, Enemies, FPS, Status
            parent.spawn((
                HudLine3,
                Text::new("Creatures: 0 | Enemies: 0"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });

    println!("UI spawned");
}

/// System that updates kill rate tracking
pub fn kill_rate_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut game_state: ResMut<GameState>,
) {
    if debug_settings.is_paused() {
        return;
    }

    game_state.kill_rate_timer += time.delta_secs();

    // Every second, update kill rate
    if game_state.kill_rate_timer >= 1.0 {
        game_state.kill_rate_timer -= 1.0;
        game_state.kills_last_second = game_state.kills_this_second;
        game_state.kills_this_second = 0;
    }
}

/// System that updates the HUD with current game state
pub fn update_ui_system(
    game_state: Res<GameState>,
    artifact_buffs: Res<ArtifactBuffs>,
    director: Res<Director>,
    debug_settings: Res<DebugSettings>,
    creature_query: Query<&Creature>,
    mut line1_query: Query<&mut Text, With<HudLine1>>,
    mut line2_query: Query<&mut Text, (With<HudLine2>, Without<HudLine1>)>,
    mut line3_query: Query<&mut Text, (With<HudLine3>, Without<HudLine1>, Without<HudLine2>)>,
    mut progress_fill_query: Query<&mut Node, With<LevelProgressFill>>,
) {
    let creature_count = creature_query.iter().count();
    let _artifact_count = artifact_buffs.acquired_artifacts.len();

    // Calculate level progress percentage
    let progress_percent = if game_state.kills_for_next_level > 0 {
        (game_state.kill_count as f32 / game_state.kills_for_next_level as f32 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Update Line 1: Level with percentage
    for mut text in line1_query.iter_mut() {
        **text = format!("Level {} ({:.0}%)", game_state.current_level, progress_percent);
    }

    // Update progress bar fill
    for mut node in progress_fill_query.iter_mut() {
        node.width = Val::Percent(progress_percent);
    }

    // Update Line 2: Kills with rate, Wave
    for mut text in line2_query.iter_mut() {
        let kill_rate = if game_state.kills_last_second > 0 {
            format!(" (+{}/s)", game_state.kills_last_second)
        } else {
            String::new()
        };
        **text = format!(
            "Kills: {}{} | Wave: {}",
            game_state.total_kills, kill_rate, game_state.current_wave
        );
    }

    // Update Line 3: Creatures, Enemies, FPS, Status
    for mut text in line3_query.iter_mut() {
        let mut parts = vec![format!("C:{}", creature_count)];

        if debug_settings.show_enemy_count {
            parts.push(format!("E:{}", director.enemies_alive));
        }

        if debug_settings.show_fps {
            let fps_text = if director.current_fps < 30.0 {
                format!("FPS:{:.0}!", director.current_fps)
            } else {
                format!("FPS:{:.0}", director.current_fps)
            };
            parts.push(fps_text);
        }

        // Estimate DPS if we have creatures
        if creature_count > 0 && director.player_dps > 0.0 {
            if director.player_dps >= 1000.0 {
                parts.push(format!("DPS:{:.1}k", director.player_dps / 1000.0));
            } else {
                parts.push(format!("DPS:{:.0}", director.player_dps));
            }
        }

        if debug_settings.god_mode {
            parts.push("GOD".to_string());
        }

        if debug_settings.is_paused() {
            parts.push("PAUSED".to_string());
        }

        **text = parts.join(" | ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_colors_are_valid() {
        // Just ensure the colors are defined
        let _ = PROGRESS_BAR_BG;
        let _ = PROGRESS_BAR_FILL;
    }
}
