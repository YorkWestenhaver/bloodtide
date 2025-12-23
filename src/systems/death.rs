use bevy::prelude::*;
use bevy::sprite::TextureAtlas;

use crate::components::{Creature, CreatureStats, DeathAnimation, Enemy, EnemyStats, Player};
use crate::resources::{DeathSprites, DebugSettings, GameState};

/// System that checks for and handles enemy deaths
pub fn enemy_death_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    debug_settings: Res<DebugSettings>,
    death_sprites: Option<Res<DeathSprites>>,
    enemy_query: Query<(Entity, &EnemyStats, &Transform), With<Enemy>>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        return;
    }

    for (entity, stats, transform) in enemy_query.iter() {
        if stats.current_hp <= 0.0 {
            let death_pos = transform.translation;
            // Preserve scale from enemy (elites are larger)
            let scale = transform.scale;

            // Spawn death animation if sprites are loaded, otherwise fall back to simple flash
            if let Some(ref sprites) = death_sprites {
                // Spawn animated death using unified spritesheet starting at frame 3 (death1)
                commands.spawn((
                    DeathAnimation::new(stats.id.clone(), death_pos),
                    Sprite::from_atlas_image(
                        sprites.goblin_spritesheet.clone(),
                        TextureAtlas {
                            layout: sprites.goblin_atlas.clone(),
                            index: 3, // Frame 3 = death1 (hit recoil)
                        },
                    ),
                    Transform::from_translation(Vec3::new(death_pos.x, death_pos.y, 0.7))
                        .with_scale(scale),
                ));
            } else {
                // Fallback: simple white flash (no sprites loaded)
                commands.spawn((
                    DeathEffect {
                        timer: Timer::from_seconds(0.2, TimerMode::Once),
                    },
                    Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 0.8),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(death_pos.x, death_pos.y, 0.7)),
                ));
            }

            // Despawn the enemy
            commands.entity(entity).despawn();

            // Increment kill counts
            game_state.kill_count += 1;
            game_state.total_kills += 1;
        }
    }
}

/// Marker component for death effects
#[derive(Component)]
pub struct DeathEffect {
    pub timer: Timer,
}

/// System that updates and removes death effects
pub fn death_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_query: Query<(Entity, &mut DeathEffect, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut effect, mut sprite, mut transform) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());

        // Shrink and fade the effect
        let remaining = effect.timer.fraction_remaining();
        let scale = remaining * 1.5;
        transform.scale = Vec3::splat(scale);

        // Fade out
        sprite.color = Color::srgba(1.0, 1.0, 1.0, remaining * 0.8);

        // Remove when timer finishes
        if effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Entry in the respawn queue
#[derive(Clone)]
pub struct RespawnEntry {
    pub creature_id: String,
    pub tier: u8,
    pub timer: Timer,
    pub position: Vec3,
}

/// Resource for tracking creature respawns
#[derive(Resource, Default)]
pub struct RespawnQueue {
    pub entries: Vec<RespawnEntry>,
}

/// Get respawn time based on creature tier
pub fn get_respawn_time(tier: u8) -> f32 {
    match tier {
        1 => 20.0,
        2 => 30.0,
        3 => 45.0,
        _ => 60.0,
    }
}

/// System that checks for and handles creature deaths
pub fn creature_death_system(
    mut commands: Commands,
    mut respawn_queue: ResMut<RespawnQueue>,
    debug_settings: Res<DebugSettings>,
    mut creature_query: Query<(Entity, &mut CreatureStats, &Transform), With<Creature>>,
    player_query: Query<&Transform, With<Player>>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        return;
    }

    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    for (entity, mut stats, transform) in creature_query.iter_mut() {
        if stats.current_hp <= 0.0 {
            // If god mode is enabled, heal the creature instead of killing it
            if debug_settings.god_mode {
                stats.current_hp = stats.max_hp;
                continue;
            }

            // Spawn death effect (colored flash based on creature)
            let death_pos = transform.translation;
            commands.spawn((
                DeathEffect {
                    timer: Timer::from_seconds(0.3, TimerMode::Once),
                },
                Sprite {
                    color: stats.color.to_bevy_color().with_alpha(0.8),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(death_pos.x, death_pos.y, 0.7)),
            ));

            // Get respawn time based on tier
            let respawn_time = get_respawn_time(stats.tier);

            // Add to respawn queue
            respawn_queue.entries.push(RespawnEntry {
                creature_id: stats.id.clone(),
                tier: stats.tier,
                timer: Timer::from_seconds(respawn_time, TimerMode::Once),
                position: player_pos,
            });

            // Despawn the creature
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Respawn Time Tests
    // =========================================================================

    #[test]
    fn tier_1_creatures_respawn_in_20_seconds() {
        assert_eq!(get_respawn_time(1), 20.0);
    }

    #[test]
    fn tier_2_creatures_respawn_in_30_seconds() {
        assert_eq!(get_respawn_time(2), 30.0);
    }

    #[test]
    fn tier_3_creatures_respawn_in_45_seconds() {
        assert_eq!(get_respawn_time(3), 45.0);
    }

    #[test]
    fn tier_4_creatures_respawn_in_60_seconds() {
        assert_eq!(get_respawn_time(4), 60.0);
    }

    #[test]
    fn tier_5_creatures_respawn_in_60_seconds() {
        assert_eq!(get_respawn_time(5), 60.0);
    }

    #[test]
    fn tier_0_creatures_respawn_in_60_seconds() {
        // Edge case: tier 0 should fall through to default
        assert_eq!(get_respawn_time(0), 60.0);
    }

    #[test]
    fn very_high_tier_creatures_respawn_in_60_seconds() {
        assert_eq!(get_respawn_time(100), 60.0);
        assert_eq!(get_respawn_time(255), 60.0);
    }

    // =========================================================================
    // RespawnQueue Tests
    // =========================================================================

    #[test]
    fn respawn_queue_default_is_empty() {
        let queue = RespawnQueue::default();
        assert!(queue.entries.is_empty());
    }

    #[test]
    fn respawn_entry_stores_creature_data() {
        let entry = RespawnEntry {
            creature_id: "fire_imp".to_string(),
            tier: 1,
            timer: Timer::from_seconds(20.0, TimerMode::Once),
            position: Vec3::new(100.0, 200.0, 0.5),
        };
        assert_eq!(entry.creature_id, "fire_imp");
        assert_eq!(entry.tier, 1);
        assert_eq!(entry.position, Vec3::new(100.0, 200.0, 0.5));
    }
}
