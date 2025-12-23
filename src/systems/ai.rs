use bevy::prelude::*;

use crate::components::{Creature, CreatureStats, Enemy, EnemyStats, Player, Velocity};

/// Distance creatures try to maintain from player
pub const CREATURE_FOLLOW_DISTANCE: f32 = 100.0;

/// Distance threshold - stop moving when within this range of target
pub const CREATURE_STOP_DISTANCE: f32 = 10.0;

/// Distance at which creatures move at boosted speed to catch up
pub const CREATURE_CATCHUP_DISTANCE: f32 = 200.0;

/// Speed multiplier when catching up
pub const CREATURE_CATCHUP_MULTIPLIER: f32 = 2.5;

/// Base speed multiplier for formation movement (creatures move faster than their base speed)
pub const CREATURE_FORMATION_SPEED_MULTIPLIER: f32 = 1.8;

/// System that makes creatures follow the player
pub fn creature_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Creature>)>,
    mut creature_query: Query<(&Transform, &mut Velocity, &CreatureStats), With<Creature>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let creature_count = creature_query.iter().count();

    for (index, (creature_transform, mut velocity, stats)) in
        creature_query.iter_mut().enumerate()
    {
        let creature_pos = creature_transform.translation.truncate();

        // Calculate target position in a circle around player
        // Each creature gets a different angle based on their index
        let angle = if creature_count > 0 {
            (index as f32 / creature_count as f32) * std::f32::consts::TAU
        } else {
            0.0
        };

        let target_pos = player_pos
            + Vec2::new(
                angle.cos() * CREATURE_FOLLOW_DISTANCE,
                angle.sin() * CREATURE_FOLLOW_DISTANCE,
            );

        // Calculate direction and distance to target
        let to_target = target_pos - creature_pos;
        let distance = to_target.length();

        // Only move if we're far enough from target position
        if distance > CREATURE_STOP_DISTANCE {
            let direction = to_target.normalize();
            // Use movement speed from creature stats with formation multiplier
            let base_speed = stats.movement_speed as f32 * CREATURE_FORMATION_SPEED_MULTIPLIER;

            // Apply catch-up boost if far from target
            let speed = if distance > CREATURE_CATCHUP_DISTANCE {
                base_speed * CREATURE_CATCHUP_MULTIPLIER
            } else {
                // Smooth interpolation: faster when further, slower when closer
                let t = (distance / CREATURE_CATCHUP_DISTANCE).min(1.0);
                base_speed * (1.0 + t * (CREATURE_CATCHUP_MULTIPLIER - 1.0))
            };

            velocity.x = direction.x * speed;
            velocity.y = direction.y * speed;
        } else {
            // Stop when close to target
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}

/// System that makes enemies chase the player
pub fn enemy_chase_system(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&Transform, &mut Velocity, &EnemyStats), With<Enemy>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (enemy_transform, mut velocity, stats) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();

        // Calculate direction to player
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        // Move toward player if not already on top of them
        if distance > 5.0 {
            let direction = to_player.normalize();
            // Use movement speed from enemy stats
            let speed = stats.movement_speed as f32;
            velocity.x = direction.x * speed;
            velocity.y = direction.y * speed;
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}
