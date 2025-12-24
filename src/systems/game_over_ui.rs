use bevy::prelude::*;

use crate::components::{Creature, Enemy, Player, PlayerAnimation, PlayerStats, Velocity};
use crate::resources::{
    AffinityState, ArtifactBuffs, DamageNumberPool, GameOverState, GamePhase, GameState,
    PlayerSprites, ProjectilePool,
};
use crate::systems::combat::Pooled;
use crate::systems::death::RespawnQueue;

// =============================================================================
// COMPONENTS
// =============================================================================

/// Marker for game over overlay (dark background)
#[derive(Component)]
pub struct GameOverOverlay;

/// Marker for game over panel
#[derive(Component)]
pub struct GameOverPanel;

/// Marker for game over stats text
#[derive(Component)]
pub struct GameOverStatsText;

/// Marker for restart run button
#[derive(Component)]
pub struct GameOverRestartButton;

/// Marker for return to deck builder button
#[derive(Component)]
pub struct GameOverDeckBuilderButton;

// =============================================================================
// CONSTANTS
// =============================================================================

const BUTTON_BG: Color = Color::srgb(0.25, 0.25, 0.35);
const BUTTON_HOVER: Color = Color::srgb(0.35, 0.35, 0.45);
const BUTTON_PRESSED: Color = Color::srgb(0.2, 0.2, 0.3);

// =============================================================================
// SYSTEMS
// =============================================================================

/// Spawn the game over UI (initially hidden)
pub fn spawn_game_over_ui_system(mut commands: Commands) {
    // Overlay (dark background)
    commands.spawn((
        GameOverOverlay,
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        Visibility::Hidden,
        ZIndex(95),
    )).with_children(|parent| {
        // Panel
        parent.spawn((
            GameOverPanel,
            Node {
                width: Val::Px(400.0),
                padding: UiRect::all(Val::Px(30.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.95)),
            ZIndex(96),
        )).with_children(|panel| {
            // "GAME OVER" title
            panel.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.2, 0.2)),
            ));

            // Stats text
            panel.spawn((
                GameOverStatsText,
                Text::new("Kills: 0\nWave: 1\nLevel: 1"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Restart button
            panel.spawn((
                GameOverRestartButton,
                Button,
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new("Restart Run"),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // Return to Deck Builder button
            panel.spawn((
                GameOverDeckBuilderButton,
                Button,
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(BUTTON_BG),
            )).with_children(|btn| {
                btn.spawn((
                    Text::new("Return to Deck Builder"),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

/// Shows/hides game over UI based on GameOverState
pub fn game_over_visibility_system(
    game_over_state: Res<GameOverState>,
    game_state: Res<GameState>,
    mut overlay_query: Query<&mut Visibility, With<GameOverOverlay>>,
    mut stats_query: Query<&mut Text, With<GameOverStatsText>>,
) {
    let is_visible = game_over_state.show_menu;

    for mut vis in overlay_query.iter_mut() {
        *vis = if is_visible { Visibility::Visible } else { Visibility::Hidden };
    }

    // Update stats text
    if is_visible {
        for mut text in stats_query.iter_mut() {
            **text = format!(
                "Kills: {}\nWave: {}\nLevel: {}",
                game_state.total_kills,
                game_state.current_wave,
                game_state.current_level
            );
        }
    }
}

/// Handle restart button interaction
pub fn game_over_restart_button_system(
    mut commands: Commands,
    mut game_over_state: ResMut<GameOverState>,
    mut game_state: ResMut<GameState>,
    mut affinity_state: ResMut<AffinityState>,
    mut artifact_buffs: ResMut<ArtifactBuffs>,
    mut respawn_queue: ResMut<RespawnQueue>,
    mut projectile_pool: ResMut<ProjectilePool>,
    mut damage_number_pool: ResMut<DamageNumberPool>,
    player_sprites: Option<Res<PlayerSprites>>,
    mut button_query: Query<(&Interaction, &mut BackgroundColor), (With<GameOverRestartButton>, Changed<Interaction>)>,
    // Query entities to despawn
    creature_query: Query<Entity, With<Creature>>,
    enemy_query: Query<Entity, With<Enemy>>,
    pooled_query: Query<Entity, With<Pooled>>,
    player_query: Query<Entity, With<Player>>,
) {
    for (interaction, mut bg) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Despawn all creatures
                for entity in creature_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Despawn all enemies
                for entity in enemy_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Despawn all pooled entities
                for entity in pooled_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Despawn player
                for entity in player_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                // Respawn player with full HP
                if let Some(ref sprites) = player_sprites {
                    commands.spawn((
                        Player,
                        PlayerStats::default(),
                        PlayerAnimation::new(),
                        Velocity::default(),
                        Sprite::from_atlas_image(
                            sprites.wizard_spritesheet.clone(),
                            bevy::sprite::TextureAtlas {
                                layout: sprites.wizard_atlas.clone(),
                                index: 0,
                            },
                        ),
                        Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(0.5)),
                    ));
                } else {
                    // Fallback to placeholder sprite
                    commands.spawn((
                        Player,
                        PlayerStats::default(),
                        PlayerAnimation::new(),
                        Velocity::default(),
                        Sprite {
                            color: Color::WHITE,
                            custom_size: Some(Vec2::new(48.0, 48.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 1.0),
                    ));
                }

                // Reset game state
                *game_state = GameState::default();
                *game_over_state = GameOverState::default();

                // Reset affinity and artifact buffs
                *affinity_state = AffinityState::default();
                *artifact_buffs = ArtifactBuffs::default();

                // Clear respawn queue
                respawn_queue.entries.clear();

                // Reset pools
                *projectile_pool = ProjectilePool::default();
                *damage_number_pool = DamageNumberPool::default();

                *bg = BackgroundColor(BUTTON_PRESSED);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_BG);
            }
        }
    }
}

/// Handle deck builder button interaction
pub fn game_over_deck_builder_button_system(
    mut game_over_state: ResMut<GameOverState>,
    mut game_phase: ResMut<GamePhase>,
    mut game_state: ResMut<GameState>,
    mut button_query: Query<(&Interaction, &mut BackgroundColor), (With<GameOverDeckBuilderButton>, Changed<Interaction>)>,
) {
    for (interaction, mut bg) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Reset game over state
                *game_over_state = GameOverState::default();

                // Reset game state
                *game_state = GameState::default();

                // Switch to deck builder phase
                *game_phase = GamePhase::DeckBuilder;

                *bg = BackgroundColor(BUTTON_PRESSED);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_BG);
            }
        }
    }
}
