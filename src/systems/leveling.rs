use bevy::prelude::*;

use crate::components::{Creature, Player, WeaponData};
use crate::resources::{AffinityState, ArtifactBuffs, CardType, GameData, GameState, PlayerDeck};
use crate::systems::{spawn_creature, spawn_weapon, try_weapon_evolution, CardRollState};

/// System that checks if player should level up based on kill count
pub fn level_check_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut artifact_buffs: ResMut<ArtifactBuffs>,
    mut affinity_state: ResMut<AffinityState>,
    mut card_roll_state: ResMut<CardRollState>,
    player_deck: Res<PlayerDeck>,
    game_data: Res<GameData>,
    player_query: Query<&Transform, With<Player>>,
    creature_query: Query<&Creature>,
    weapon_query: Query<(Entity, &WeaponData)>,
) {
    if game_state.kill_count >= game_state.kills_for_next_level {
        // Subtract kills used for this level (keep overflow)
        game_state.kill_count -= game_state.kills_for_next_level;

        // Increment level
        game_state.current_level += 1;

        // Increase kills needed for next level (multiply by 1.2, round up)
        game_state.kills_for_next_level = (game_state.kills_for_next_level as f32 * 1.2).ceil() as u32;

        // Flag that level up occurred (for effects and card rolling)
        game_state.level_up_pending = true;

        println!(
            "LEVEL UP! Now level {}. Next level at {} kills.",
            game_state.current_level, game_state.kills_for_next_level
        );

        // Roll a card from the deck
        if let Some(card) = player_deck.roll_card() {
            println!("Rolled card: {}!", card.id);

            // Get card name and tier for popup
            let (card_name, card_tier) = match card.card_type {
                CardType::Creature => {
                    let data = game_data.creatures.iter().find(|c| c.id == card.id);
                    (
                        data.map(|c| c.name.clone()).unwrap_or_else(|| card.id.clone()),
                        data.map(|c| c.tier).unwrap_or(1),
                    )
                }
                CardType::Weapon => {
                    let data = game_data.weapons.iter().find(|w| w.id == card.id);
                    (
                        data.map(|w| w.name.clone()).unwrap_or_else(|| card.id.clone()),
                        data.map(|w| w.tier).unwrap_or(1),
                    )
                }
                CardType::Artifact => {
                    let data = game_data.artifacts.iter().find(|a| a.id == card.id);
                    (
                        data.map(|a| a.name.clone()).unwrap_or_else(|| card.id.clone()),
                        data.map(|a| a.tier).unwrap_or(1),
                    )
                }
            };

            // Set pending popup for UI
            let card_type_str = match card.card_type {
                CardType::Creature => "Creature",
                CardType::Weapon => "Weapon",
                CardType::Artifact => "Artifact",
            };
            card_roll_state.pending_popup = Some((card_name, card_type_str.to_string(), card_tier));

            match card.card_type {
                CardType::Creature => {
                    // Spawn the creature near the player
                    if let Ok(player_transform) = player_query.get_single() {
                        // Count existing creatures for offset
                        let creature_count = creature_query.iter().count();
                        let angle = creature_count as f32 * 0.8;
                        let offset_distance = 80.0;

                        let spawn_pos = Vec3::new(
                            player_transform.translation.x + angle.cos() * offset_distance,
                            player_transform.translation.y + angle.sin() * offset_distance,
                            0.5,
                        );

                        spawn_creature(&mut commands, &game_data, &artifact_buffs, &card.id, spawn_pos);
                    }
                }
                CardType::Weapon => {
                    // Spawn the weapon and add its affinity
                    spawn_weapon(&mut commands, &game_data, &mut affinity_state, &card.id);

                    // Check for weapon evolution
                    try_weapon_evolution(&mut commands, &game_data, &mut affinity_state, &weapon_query);
                }
                CardType::Artifact => {
                    // Apply artifact bonuses
                    artifact_buffs.apply_artifact(&game_data, &card.id);
                }
            }
        }
    }
}

/// Marker component for level up visual effect
#[derive(Component)]
pub struct LevelUpEffect {
    pub timer: Timer,
}

/// System that spawns and manages level up visual effects
pub fn level_up_effect_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut effect_query: Query<(Entity, &mut LevelUpEffect, &mut Sprite, &mut Transform), Without<Player>>,
) {
    // Spawn effect when level up is pending
    if game_state.level_up_pending {
        if let Ok(player_transform) = player_query.get_single() {
            let player_pos = player_transform.translation;

            // Spawn expanding yellow ring effect
            commands.spawn((
                LevelUpEffect {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                },
                Sprite {
                    color: Color::srgba(1.0, 0.9, 0.2, 0.8),
                    custom_size: Some(Vec2::new(60.0, 60.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(player_pos.x, player_pos.y, 0.8)),
            ));
        }

        game_state.level_up_pending = false;
    }

    // Update existing effects
    for (entity, mut effect, mut sprite, mut transform) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());

        // Expand and fade the effect
        let progress = effect.timer.fraction();
        let scale = 1.0 + progress * 3.0; // Expand from 1x to 4x
        transform.scale = Vec3::splat(scale);

        // Fade out
        let alpha = (1.0 - progress) * 0.8;
        sprite.color = Color::srgba(1.0, 0.9, 0.2, alpha);

        // Remove when timer finishes
        if effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
