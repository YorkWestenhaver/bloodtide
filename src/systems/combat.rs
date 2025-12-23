use bevy::prelude::*;

use crate::components::{
    AttackRange, AttackTimer, Creature, CreatureStats, Enemy, EnemyAttackTimer, EnemyStats,
    Velocity,
};

/// Projectile speed in pixels per second
pub const PROJECTILE_SPEED: f32 = 500.0;

/// Projectile size in pixels
pub const PROJECTILE_SIZE: f32 = 8.0;

/// Maximum projectile lifetime in seconds
pub const PROJECTILE_LIFETIME: f32 = 1.0;

/// Marker component for projectiles
#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f64,
    pub lifetime: Timer,
}

/// System that handles creature attacks
pub fn creature_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    mut creature_query: Query<(
        &CreatureStats,
        &mut AttackTimer,
        &AttackRange,
        &Transform,
    )>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (stats, mut attack_timer, attack_range, creature_transform) in creature_query.iter_mut() {
        // Tick the attack timer
        attack_timer.timer.tick(time.delta());

        // Check if attack is ready
        if attack_timer.timer.just_finished() {
            let creature_pos = creature_transform.translation.truncate();

            // Find nearest enemy within range
            let mut nearest_enemy: Option<(Entity, f32, Vec2)> = None;

            for (enemy_entity, enemy_transform) in enemy_query.iter() {
                let enemy_pos = enemy_transform.translation.truncate();
                let distance = creature_pos.distance(enemy_pos);

                if distance <= attack_range.0 {
                    if nearest_enemy.is_none() || distance < nearest_enemy.unwrap().1 {
                        nearest_enemy = Some((enemy_entity, distance, enemy_pos));
                    }
                }
            }

            // Attack nearest enemy if one is in range
            if let Some((target_entity, _distance, target_pos)) = nearest_enemy {
                // Spawn projectile
                let direction = (target_pos - creature_pos).normalize_or_zero();

                commands.spawn((
                    Projectile {
                        target: target_entity,
                        damage: stats.base_damage,
                        lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                    },
                    Velocity {
                        x: direction.x * PROJECTILE_SPEED,
                        y: direction.y * PROJECTILE_SPEED,
                    },
                    Sprite {
                        color: stats.color.to_bevy_color(),
                        custom_size: Some(Vec2::new(PROJECTILE_SIZE, PROJECTILE_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(
                        creature_pos.x,
                        creature_pos.y,
                        0.6, // Above creatures
                    )),
                ));
            }
        }
    }
}

/// System that handles projectile movement and collision
pub fn projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Projectile, &Transform)>,
    mut enemy_query: Query<(&Transform, &mut EnemyStats), With<Enemy>>,
) {
    for (projectile_entity, mut projectile, projectile_transform) in projectile_query.iter_mut() {
        // Tick lifetime
        projectile.lifetime.tick(time.delta());

        // Despawn if lifetime expired
        if projectile.lifetime.finished() {
            commands.entity(projectile_entity).despawn();
            continue;
        }

        // Check if target still exists and get its position
        if let Ok((enemy_transform, mut enemy_stats)) = enemy_query.get_mut(projectile.target) {
            let projectile_pos = projectile_transform.translation.truncate();
            let enemy_pos = enemy_transform.translation.truncate();
            let distance = projectile_pos.distance(enemy_pos);

            // Hit detection - if projectile is close enough to target
            if distance < 20.0 {
                // Deal damage
                enemy_stats.current_hp -= projectile.damage;

                // Despawn projectile
                commands.entity(projectile_entity).despawn();
            }
        } else {
            // Target no longer exists, despawn projectile
            commands.entity(projectile_entity).despawn();
        }
    }
}

/// Enemy attack range for melee enemies
pub const ENEMY_ATTACK_RANGE: f32 = 40.0;

/// System that handles enemies attacking creatures
pub fn enemy_attack_system(
    time: Res<Time>,
    mut enemy_query: Query<(&EnemyStats, &mut EnemyAttackTimer, &Transform), With<Enemy>>,
    mut creature_query: Query<(Entity, &Transform, &mut CreatureStats), With<Creature>>,
) {
    for (enemy_stats, mut attack_timer, enemy_transform) in enemy_query.iter_mut() {
        // Tick the attack timer
        attack_timer.timer.tick(time.delta());

        // Check if attack is ready
        if attack_timer.timer.just_finished() {
            let enemy_pos = enemy_transform.translation.truncate();

            // Find nearest creature within range
            let mut nearest_creature: Option<(Entity, f32)> = None;

            for (creature_entity, creature_transform, _) in creature_query.iter() {
                let creature_pos = creature_transform.translation.truncate();
                let distance = enemy_pos.distance(creature_pos);

                if distance <= ENEMY_ATTACK_RANGE {
                    if nearest_creature.is_none() || distance < nearest_creature.unwrap().1 {
                        nearest_creature = Some((creature_entity, distance));
                    }
                }
            }

            // Attack nearest creature if one is in range
            if let Some((target_entity, _distance)) = nearest_creature {
                if let Ok((_, _, mut creature_stats)) = creature_query.get_mut(target_entity) {
                    creature_stats.current_hp -= enemy_stats.base_damage;
                }
            }
        }
    }
}
