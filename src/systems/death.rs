use bevy::prelude::*;

use crate::components::{Creature, CreatureStats, Enemy, EnemyStats, Player};
use crate::resources::GameState;

/// System that checks for and handles enemy deaths
pub fn enemy_death_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    enemy_query: Query<(Entity, &EnemyStats, &Transform), With<Enemy>>,
) {
    for (entity, stats, transform) in enemy_query.iter() {
        if stats.current_hp <= 0.0 {
            // Spawn death effect (small white flash)
            let death_pos = transform.translation;
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

            // Despawn the enemy
            commands.entity(entity).despawn();

            // Increment kill counts
            game_state.kill_count += 1;
            game_state.total_kills += 1;

            println!(
                "{} killed! Total kills: {}",
                stats.name, game_state.total_kills
            );
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
fn get_respawn_time(tier: u8) -> f32 {
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
    creature_query: Query<(Entity, &CreatureStats, &Transform), With<Creature>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    for (entity, stats, transform) in creature_query.iter() {
        if stats.current_hp <= 0.0 {
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

            println!(
                "{} died! Respawning in {:.0} seconds...",
                stats.name, respawn_time
            );

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
