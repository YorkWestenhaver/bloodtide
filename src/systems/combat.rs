use bevy::prelude::*;

use crate::components::{
    AttackRange, AttackTimer, Creature, CreatureStats, Enemy, EnemyAttackTimer, EnemyStats,
    Player, ProjectileConfig, Velocity, Weapon, WeaponAttackTimer, WeaponData, WeaponStats,
};
use crate::math::{calculate_damage_with_crits, CritTier};
use crate::resources::{get_affinity_bonuses, AffinityState, ArtifactBuffs, DebugSettings, GameData};
use crate::systems::creature_xp::PendingKillCredit;

/// Projectile speed in pixels per second
pub const PROJECTILE_SPEED: f32 = 500.0;

/// Projectile size in pixels
pub const PROJECTILE_SIZE: f32 = 8.0;

/// Weapon projectile size in pixels (smaller than creature projectiles)
pub const WEAPON_PROJECTILE_SIZE: f32 = 6.0;

/// Maximum projectile lifetime in seconds
pub const PROJECTILE_LIFETIME: f32 = 1.0;

/// Floating damage number lifetime in seconds
pub const DAMAGE_NUMBER_LIFETIME: f32 = 0.8;

/// Floating damage number rise speed in pixels per second
pub const DAMAGE_NUMBER_RISE_SPEED: f32 = 60.0;

/// Marker component for projectiles
#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub damage: f64,
    pub crit_tier: CritTier,
    pub lifetime: Timer,
    /// The creature entity that fired this projectile (for XP tracking)
    pub source_creature: Option<Entity>,
    /// Size of this projectile in pixels
    pub size: f32,
    /// Speed of this projectile in pixels per second
    pub speed: f32,
}

/// Screen shake resource
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: Timer,
}

impl ScreenShake {
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = Timer::from_seconds(duration, TimerMode::Once);
    }
}

/// Floating damage number component
#[derive(Component)]
pub struct DamageNumber {
    pub lifetime: Timer,
    pub start_alpha: f32,
}

impl DamageNumber {
    pub fn new() -> Self {
        Self {
            lifetime: Timer::from_seconds(DAMAGE_NUMBER_LIFETIME, TimerMode::Once),
            start_alpha: 1.0,
        }
    }
}

/// Get projectile color based on crit tier
fn get_projectile_color(base_color: Color, crit_tier: CritTier) -> Color {
    match crit_tier {
        CritTier::None => base_color,
        CritTier::Normal => Color::srgb(1.0, 1.0, 0.2),   // Yellow
        CritTier::Mega => Color::srgb(1.0, 0.5, 0.0),     // Orange
        CritTier::Super => Color::srgb(0.8, 0.2, 0.8),    // Red/Purple
    }
}

/// Get damage number color based on crit tier
fn get_damage_number_color(crit_tier: CritTier) -> Color {
    match crit_tier {
        CritTier::None => Color::WHITE,
        CritTier::Normal => Color::srgb(1.0, 1.0, 0.2),   // Yellow
        CritTier::Mega => Color::srgb(1.0, 0.5, 0.0),     // Orange
        CritTier::Super => Color::srgb(1.0, 0.2, 0.2),    // Red
    }
}

/// Format damage for display (uses scientific notation for large numbers)
fn format_damage(damage: f64) -> String {
    if damage >= 1_000_000.0 {
        format!("{:.2e}", damage)
    } else if damage >= 1000.0 {
        format!("{:.1}k", damage / 1000.0)
    } else {
        format!("{:.0}", damage)
    }
}

/// System that handles creature attacks
pub fn creature_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    artifact_buffs: Res<ArtifactBuffs>,
    affinity_state: Res<AffinityState>,
    game_data: Res<GameData>,
    debug_settings: Res<DebugSettings>,
    mut creature_query: Query<(
        Entity,
        &CreatureStats,
        &mut AttackTimer,
        &AttackRange,
        &ProjectileConfig,
        &Transform,
    )>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        return;
    }

    for (creature_entity, stats, mut attack_timer, attack_range, projectile_config, creature_transform) in creature_query.iter_mut() {
        // Tick the attack timer (apply attack speed multiplier by scaling delta time)
        let scaled_delta = time.delta().mul_f32(debug_settings.attack_speed_multiplier);
        attack_timer.timer.tick(scaled_delta);

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
                // Get artifact bonuses for this creature
                let artifact_bonus = artifact_buffs.get_total_bonuses(
                    &stats.id,
                    stats.color,
                    stats.creature_type,
                );

                // Get affinity bonuses for this creature's color
                let affinity_bonus = get_affinity_bonuses(&game_data, stats.color, &affinity_state);

                // Combine damage bonuses from artifacts and affinity, then apply debug multiplier
                let total_damage_bonus = artifact_bonus.damage_bonus + affinity_bonus.damage_bonus;
                let modified_damage = stats.base_damage
                    * (1.0 + total_damage_bonus / 100.0)
                    * debug_settings.creature_damage_multiplier as f64;

                // Apply crit bonuses from artifacts, affinity, and debug settings
                let modified_crit_t1 = stats.crit_t1
                    + artifact_bonus.crit_t1_bonus
                    + affinity_bonus.crit_t1_bonus
                    + debug_settings.crit_t1_bonus as f64;

                // Crit T2 and T3 require affinity unlocks (but debug bonus bypasses this)
                let modified_crit_t2 = if affinity_bonus.crit_t2_unlock || debug_settings.crit_t2_bonus > 0.0 {
                    stats.crit_t2 + artifact_bonus.crit_t2_bonus + debug_settings.crit_t2_bonus as f64
                } else {
                    0.0 // Can't mega crit without affinity unlock
                };

                let modified_crit_t3 = if affinity_bonus.crit_t3_unlock || debug_settings.crit_t3_bonus > 0.0 {
                    stats.crit_t3 + artifact_bonus.crit_t3_bonus + debug_settings.crit_t3_bonus as f64
                } else {
                    0.0 // Can't super crit without affinity unlock
                };

                // Calculate damage with crits
                let crit_result = calculate_damage_with_crits(
                    modified_damage,
                    modified_crit_t1,
                    modified_crit_t2,
                    modified_crit_t3,
                );

                // Get projectile color based on crit tier
                let projectile_color = get_projectile_color(stats.color.to_bevy_color(), crit_result.tier);

                // Calculate direction toward target
                let base_direction = (target_pos - creature_pos).normalize_or_zero();

                // Apply debug settings modifiers to projectile config
                let projectile_count = (projectile_config.count as i32 + debug_settings.projectile_count_bonus) as u32;
                let projectile_count = projectile_count.max(1); // Ensure at least 1 projectile
                let projectile_size = projectile_config.size * debug_settings.projectile_size_multiplier;
                let projectile_speed = projectile_config.speed * debug_settings.projectile_speed_multiplier;

                // Spawn multiple projectiles with spread
                for i in 0..projectile_count {
                    // Calculate spread angle for this projectile
                    let spread_angle = if projectile_count > 1 {
                        let half_spread = projectile_config.spread / 2.0;
                        let t = i as f32 / (projectile_count - 1) as f32;
                        -half_spread + t * projectile_config.spread
                    } else {
                        0.0
                    };

                    // Rotate the base direction by the spread angle
                    let cos_angle = spread_angle.cos();
                    let sin_angle = spread_angle.sin();
                    let direction = Vec2::new(
                        base_direction.x * cos_angle - base_direction.y * sin_angle,
                        base_direction.x * sin_angle + base_direction.y * cos_angle,
                    );

                    commands.spawn((
                        Projectile {
                            target: target_entity,
                            damage: crit_result.final_damage,
                            crit_tier: crit_result.tier,
                            lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                            source_creature: Some(creature_entity),
                            size: projectile_size,
                            speed: projectile_speed,
                        },
                        Velocity {
                            x: direction.x * projectile_speed,
                            y: direction.y * projectile_speed,
                        },
                        Sprite {
                            color: projectile_color,
                            custom_size: Some(Vec2::new(projectile_size, projectile_size)),
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
}

/// System that handles projectile movement and collision
pub fn projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut projectile_query: Query<(Entity, &mut Projectile, &Transform)>,
    mut enemy_query: Query<(&Transform, &mut EnemyStats), With<Enemy>>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        return;
    }

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
                // Check if this hit will kill the enemy
                let will_kill = enemy_stats.current_hp - projectile.damage <= 0.0;

                // Deal damage
                enemy_stats.current_hp -= projectile.damage;

                // If this projectile killed the enemy and came from a creature, spawn kill credit
                if will_kill {
                    if let Some(source_creature) = projectile.source_creature {
                        commands.spawn(PendingKillCredit {
                            creature_entity: source_creature,
                        });
                    }
                }

                // Spawn floating damage number
                let damage_color = get_damage_number_color(projectile.crit_tier);
                let damage_text = format_damage(projectile.damage);

                // Scale font size based on crit tier
                let font_size = match projectile.crit_tier {
                    CritTier::None => 16.0,
                    CritTier::Normal => 20.0,
                    CritTier::Mega => 26.0,
                    CritTier::Super => 34.0,
                };

                commands.spawn((
                    DamageNumber::new(),
                    Text2d::new(damage_text),
                    TextFont {
                        font_size,
                        ..default()
                    },
                    TextColor(damage_color),
                    Transform::from_translation(Vec3::new(
                        enemy_pos.x,
                        enemy_pos.y + 20.0, // Start slightly above enemy
                        10.0, // Above everything
                    )),
                ));

                // Trigger screen shake for Mega and Super crits
                match projectile.crit_tier {
                    CritTier::Mega => {
                        screen_shake.trigger(4.0, 0.15);
                    }
                    CritTier::Super => {
                        screen_shake.trigger(10.0, 0.25);
                    }
                    _ => {}
                }

                // Despawn projectile
                commands.entity(projectile_entity).despawn();
            }
        } else {
            // Target no longer exists, despawn projectile
            commands.entity(projectile_entity).despawn();
        }
    }
}

/// System that updates floating damage numbers (rise and fade)
pub fn damage_number_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DamageNumber, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut damage_number, mut transform, mut text_color) in query.iter_mut() {
        // Tick lifetime
        damage_number.lifetime.tick(time.delta());

        // Despawn if lifetime expired
        if damage_number.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Rise upward
        transform.translation.y += DAMAGE_NUMBER_RISE_SPEED * time.delta_secs();

        // Fade out based on remaining lifetime
        let progress = damage_number.lifetime.fraction();
        let alpha = 1.0 - progress; // Fade from 1.0 to 0.0

        // Update text color alpha
        let current_color = text_color.0;
        text_color.0 = current_color.with_alpha(alpha);
    }
}

/// System that applies screen shake to the camera
pub fn screen_shake_system(
    time: Res<Time>,
    mut screen_shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if screen_shake.intensity <= 0.0 {
        return;
    }

    // Tick the shake timer
    screen_shake.duration.tick(time.delta());

    if screen_shake.duration.finished() {
        screen_shake.intensity = 0.0;
        return;
    }

    // Calculate remaining shake intensity based on time left
    let remaining = 1.0 - screen_shake.duration.fraction();
    let current_intensity = screen_shake.intensity * remaining;

    // Apply random offset to camera
    for mut transform in camera_query.iter_mut() {
        let offset_x = (rand::random::<f32>() - 0.5) * 2.0 * current_intensity;
        let offset_y = (rand::random::<f32>() - 0.5) * 2.0 * current_intensity;

        // Note: This is additive shake. The camera_follow_system will reset the position
        // We need to apply the shake on top of the follow position
        transform.translation.x += offset_x;
        transform.translation.y += offset_y;
    }
}

/// Enemy attack range for melee enemies
pub const ENEMY_ATTACK_RANGE: f32 = 40.0;

/// System that handles enemies attacking creatures
pub fn enemy_attack_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut enemy_query: Query<(&EnemyStats, &mut EnemyAttackTimer, &Transform), With<Enemy>>,
    mut creature_query: Query<(Entity, &Transform, &mut CreatureStats), With<Creature>>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        return;
    }

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
                    // Apply enemy damage multiplier from debug settings
                    let damage = enemy_stats.base_damage * debug_settings.enemy_damage_multiplier as f64;
                    creature_stats.current_hp -= damage;
                }
            }
        }
    }
}

/// Weapon projectile color (silver/white)
const WEAPON_PROJECTILE_COLOR: Color = Color::srgb(0.9, 0.9, 0.95);

/// System that handles weapon auto-attacks
pub fn weapon_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut weapon_query: Query<(&WeaponData, &WeaponStats, &mut WeaponAttackTimer), With<Weapon>>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (weapon_data, weapon_stats, mut attack_timer) in weapon_query.iter_mut() {
        // Tick the attack timer
        attack_timer.timer.tick(time.delta());

        // Check if attack is ready
        if attack_timer.timer.just_finished() {
            // Find nearest enemy within weapon's range
            let mut nearest_enemy: Option<(Entity, f32, Vec2)> = None;

            for (enemy_entity, enemy_transform) in enemy_query.iter() {
                let enemy_pos = enemy_transform.translation.truncate();
                let distance = player_pos.distance(enemy_pos);

                if distance <= weapon_stats.auto_range as f32 {
                    if nearest_enemy.is_none() || distance < nearest_enemy.unwrap().1 {
                        nearest_enemy = Some((enemy_entity, distance, enemy_pos));
                    }
                }
            }

            // Attack nearest enemy if one is in range
            if let Some((target_entity, _distance, target_pos)) = nearest_enemy {
                // Spawn projectiles based on projectile_count
                for i in 0..weapon_stats.projectile_count {
                    let direction = (target_pos - player_pos).normalize_or_zero();

                    // Calculate projectile spread for multiple projectiles
                    let spread_angle = if weapon_stats.projectile_count > 1 {
                        let spread_range = 0.3; // ~17 degrees total spread
                        let offset = (i as f32 / (weapon_stats.projectile_count - 1) as f32) - 0.5;
                        offset * spread_range * 2.0
                    } else {
                        0.0
                    };

                    // Rotate direction by spread angle
                    let rotated_dir = Vec2::new(
                        direction.x * spread_angle.cos() - direction.y * spread_angle.sin(),
                        direction.x * spread_angle.sin() + direction.y * spread_angle.cos(),
                    );

                    let projectile_speed = if weapon_stats.projectile_speed > 0.0 {
                        weapon_stats.projectile_speed as f32
                    } else {
                        PROJECTILE_SPEED
                    };

                    commands.spawn((
                        Projectile {
                            target: target_entity,
                            damage: weapon_stats.auto_damage,
                            crit_tier: CritTier::None, // Weapons don't crit (for now)
                            lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                            source_creature: None, // Weapon projectiles don't give creature XP
                            size: WEAPON_PROJECTILE_SIZE,
                            speed: projectile_speed,
                        },
                        Velocity {
                            x: rotated_dir.x * projectile_speed,
                            y: rotated_dir.y * projectile_speed,
                        },
                        Sprite {
                            color: weapon_data.color.to_bevy_color().lighter(0.3),
                            custom_size: Some(Vec2::new(WEAPON_PROJECTILE_SIZE, WEAPON_PROJECTILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            player_pos.x,
                            player_pos.y,
                            0.6, // Above creatures
                        )),
                    ));
                }
            }
        }
    }
}
