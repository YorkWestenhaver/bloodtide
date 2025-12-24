use bevy::prelude::*;

use crate::components::{
    AttackRange, AttackTimer, Creature, CreatureStats, Enemy, EnemyAttackTimer, EnemyStats,
    InvincibilityTimer, Player, PlayerStats, ProjectileConfig, ProjectileType, Velocity, Weapon, WeaponAttackTimer, WeaponData, WeaponStats,
};
use crate::math::{calculate_damage_with_crits, CritTier};
use crate::resources::{get_affinity_bonuses, AffinityState, ArtifactBuffs, CreatureSprites, DebugSettings, GameData, SpatialGrid, ProjectilePool, DamageNumberPool};
use crate::systems::creature_xp::PendingKillCredit;

/// Projectile speed in pixels per second
pub const PROJECTILE_SPEED: f32 = 500.0;

/// Projectile size in pixels
pub const PROJECTILE_SIZE: f32 = 8.0;

/// Weapon projectile size in pixels (smaller than creature projectiles)
pub const WEAPON_PROJECTILE_SIZE: f32 = 6.0;

/// Maximum projectile lifetime in seconds (short for non-penetrating)
pub const PROJECTILE_LIFETIME: f32 = 1.0;

/// Maximum projectile lifetime for penetrating projectiles (longer to allow passing through enemies)
pub const PROJECTILE_MAX_LIFETIME: f32 = 3.0;

/// Maximum distance from player before projectiles despawn
pub const PROJECTILE_DESPAWN_DISTANCE: f32 = 1200.0;

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
    /// How many more enemies this projectile can hit before despawning
    pub penetration_remaining: u32,
    /// Entities this projectile has already hit (to prevent double damage)
    pub enemies_hit: Vec<Entity>,
    /// Projectile behavior type
    pub projectile_type: ProjectileType,
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

    /// Reset for reuse from pool
    pub fn reset(&mut self) {
        self.lifetime = Timer::from_seconds(DAMAGE_NUMBER_LIFETIME, TimerMode::Once);
        self.start_alpha = 1.0;
    }
}

/// Marker for entities that came from a pool (projectiles, damage numbers)
#[derive(Component)]
pub struct Pooled;

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

/// Get visual properties (size, color) for projectile type
fn get_projectile_visual(projectile_type: ProjectileType, base_size: f32, base_color: Color) -> (Vec2, Color) {
    match projectile_type {
        ProjectileType::Basic => {
            // Standard square
            (Vec2::new(base_size, base_size), base_color)
        }
        ProjectileType::Piercing => {
            // Thin elongated rectangle
            (Vec2::new(base_size * 2.0, base_size * 0.5), base_color)
        }
        ProjectileType::Explosive => {
            // Slightly larger, tinted orange
            let Srgba { red, green, blue, alpha } = base_color.to_srgba();
            let tinted = Color::srgba(
                (red + 0.3).min(1.0),
                green * 0.7,
                blue * 0.5,
                alpha,
            );
            (Vec2::new(base_size * 1.2, base_size * 1.2), tinted)
        }
        ProjectileType::Homing => {
            // Diamond shape (rotated square), tinted cyan
            let Srgba { red, green, blue, alpha } = base_color.to_srgba();
            let tinted = Color::srgba(
                red * 0.7,
                (green + 0.2).min(1.0),
                (blue + 0.3).min(1.0),
                alpha,
            );
            (Vec2::new(base_size * 0.8, base_size * 0.8), tinted)
        }
        ProjectileType::Chain => {
            // Bright electric blue tint
            let Srgba { red, green, blue, alpha } = base_color.to_srgba();
            let tinted = Color::srgba(
                red * 0.5,
                (green * 0.8 + 0.2).min(1.0),
                (blue + 0.5).min(1.0),
                alpha,
            );
            (Vec2::new(base_size, base_size), tinted)
        }
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
    spatial_grid: Res<SpatialGrid>,
    creature_sprites: Option<Res<CreatureSprites>>,
    mut projectile_pool: ResMut<ProjectilePool>,
    mut creature_query: Query<(
        Entity,
        &CreatureStats,
        &mut AttackTimer,
        &AttackRange,
        &ProjectileConfig,
        &Transform,
    ), With<Creature>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut projectile_query: Query<(&mut Projectile, &mut Velocity, &mut Sprite, &mut Transform, &mut Visibility), (With<Projectile>, Without<Creature>, Without<Enemy>)>,
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

            // Find nearest enemy within range using spatial grid
            let mut nearest_enemy: Option<(Entity, f32, Vec2)> = None;

            // Only check enemies in nearby grid cells (huge performance win)
            let nearby_enemies = spatial_grid.get_entities_in_radius(creature_pos, attack_range.0);

            for enemy_entity in nearby_enemies {
                if let Ok(enemy_transform) = enemy_query.get(enemy_entity) {
                    let enemy_pos = enemy_transform.translation.truncate();
                    let distance = creature_pos.distance(enemy_pos);

                    if distance <= attack_range.0 {
                        if nearest_enemy.is_none() || distance < nearest_enemy.unwrap().1 {
                            nearest_enemy = Some((enemy_entity, distance, enemy_pos));
                        }
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
                let projectile_penetration = projectile_config.penetration + debug_settings.global_penetration_bonus;

                // Use longer lifetime for penetrating projectiles
                let lifetime_duration = if projectile_penetration > 1 {
                    PROJECTILE_MAX_LIFETIME
                } else {
                    PROJECTILE_LIFETIME
                };

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

                    // Get visual properties based on projectile type
                    let (sprite_size, sprite_color) = get_projectile_visual(
                        projectile_config.projectile_type,
                        projectile_size,
                        projectile_color,
                    );

                    // Check if this is a fire creature with flame sprite available
                    // Matches: fire_imp, flame_fiend, inferno_demon, ember_*, etc.
                    let is_fire_creature = stats.id.starts_with("fire")
                        || stats.id.contains("flame")
                        || stats.id.contains("ember")
                        || stats.id.contains("inferno");
                    let use_flame_sprite = is_fire_creature && creature_sprites.is_some();

                    if use_flame_sprite {
                        // Fire creature: spawn flame projectile with image sprite
                        let sprites = creature_sprites.as_ref().unwrap();

                        // Calculate rotation based on direction (flame points up by default)
                        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

                        commands.spawn((
                            Projectile {
                                target: target_entity,
                                damage: crit_result.final_damage,
                                crit_tier: crit_result.tier,
                                lifetime: Timer::from_seconds(lifetime_duration, TimerMode::Once),
                                source_creature: Some(creature_entity),
                                size: projectile_size,
                                speed: projectile_speed,
                                penetration_remaining: projectile_penetration,
                                enemies_hit: Vec::new(),
                                projectile_type: projectile_config.projectile_type,
                            },
                            Velocity {
                                x: direction.x * projectile_speed,
                                y: direction.y * projectile_speed,
                            },
                            Sprite::from_image(sprites.flame_projectile.clone()),
                            Transform::from_translation(Vec3::new(
                                creature_pos.x,
                                creature_pos.y,
                                0.6, // Above creatures
                            )).with_rotation(Quat::from_rotation_z(angle))
                              .with_scale(Vec3::splat(0.4)), // Scale down the flame
                        ));
                    } else if let Some(pooled_entity) = projectile_pool.get() {
                        // Try to get a projectile from the pool (non-fire creatures)
                        // Reuse pooled projectile
                        if let Ok((mut proj, mut vel, mut sprite, mut transform, mut vis)) = projectile_query.get_mut(pooled_entity) {
                            proj.target = target_entity;
                            proj.damage = crit_result.final_damage;
                            proj.crit_tier = crit_result.tier;
                            proj.lifetime = Timer::from_seconds(lifetime_duration, TimerMode::Once);
                            proj.source_creature = Some(creature_entity);
                            proj.size = projectile_size;
                            proj.speed = projectile_speed;
                            proj.penetration_remaining = projectile_penetration;
                            proj.enemies_hit.clear();
                            proj.projectile_type = projectile_config.projectile_type;

                            vel.x = direction.x * projectile_speed;
                            vel.y = direction.y * projectile_speed;

                            sprite.color = sprite_color;
                            sprite.custom_size = Some(sprite_size);

                            transform.translation = Vec3::new(creature_pos.x, creature_pos.y, 0.6);

                            *vis = Visibility::Visible;
                        }
                    } else {
                        // Pool exhausted, fall back to spawning (shouldn't happen often)
                        commands.spawn((
                            Projectile {
                                target: target_entity,
                                damage: crit_result.final_damage,
                                crit_tier: crit_result.tier,
                                lifetime: Timer::from_seconds(lifetime_duration, TimerMode::Once),
                                source_creature: Some(creature_entity),
                                size: projectile_size,
                                speed: projectile_speed,
                                penetration_remaining: projectile_penetration,
                                enemies_hit: Vec::new(),
                                projectile_type: projectile_config.projectile_type,
                            },
                            Velocity {
                                x: direction.x * projectile_speed,
                                y: direction.y * projectile_speed,
                            },
                            Sprite {
                                color: sprite_color,
                                custom_size: Some(sprite_size),
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
}

/// AoE explosion radius for explosive projectiles
pub const EXPLOSIVE_AOE_RADIUS: f32 = 100.0;

/// Chain lightning search radius
pub const CHAIN_SEARCH_RADIUS: f32 = 150.0;

/// Homing turn rate (radians per second)
pub const HOMING_TURN_RATE: f32 = 3.0;

/// Pending explosion effect to spawn after projectile system
#[derive(Component)]
pub struct PendingExplosion {
    pub position: Vec2,
    pub radius: f32,
    pub damage: f64,
    pub source_creature: Option<Entity>,
    pub enemies_to_skip: Vec<Entity>,
}

/// Pending chain target to redirect projectile
#[derive(Component)]
pub struct PendingChain {
    pub projectile_entity: Entity,
    pub new_target_pos: Vec2,
}

/// System that handles projectile movement and collision with penetration support
pub fn projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut projectile_pool: ResMut<ProjectilePool>,
    mut damage_number_pool: ResMut<DamageNumberPool>,
    player_query: Query<&Transform, (With<Player>, Without<Projectile>, Without<Enemy>, Without<DamageNumber>)>,
    mut projectile_query: Query<
        (Entity, &mut Projectile, &mut Transform, &mut Sprite, &mut Velocity, &mut Visibility, Option<&Pooled>),
        (With<Projectile>, Without<Player>, Without<Enemy>, Without<DamageNumber>)
    >,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyStats), (With<Enemy>, Without<Player>, Without<Projectile>, Without<DamageNumber>)>,
    mut damage_number_query: Query<
        (&mut DamageNumber, &mut Text2d, &mut TextFont, &mut TextColor, &mut Transform, &mut Visibility),
        (With<DamageNumber>, Without<Projectile>, Without<Enemy>, Without<Player>)
    >,
    mut screen_shake: ResMut<ScreenShake>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        return;
    }

    // Get player position for distance-based despawning
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    // Collect chain redirections to apply after the main loop
    let mut pending_chains: Vec<(Entity, Vec2)> = Vec::new();
    // Collect explosions to spawn after the main loop
    let mut pending_explosions: Vec<(Vec2, f32, f64, Option<Entity>, Vec<Entity>)> = Vec::new();

    // Collect entities to return to pool (can't modify pool while iterating)
    let mut to_release: Vec<Entity> = Vec::new();

    for (projectile_entity, mut projectile, projectile_transform, mut sprite, mut velocity, mut visibility, is_pooled) in projectile_query.iter_mut() {
        // Skip hidden pooled projectiles (they're inactive)
        if *visibility == Visibility::Hidden {
            continue;
        }

        // Tick lifetime
        projectile.lifetime.tick(time.delta());

        // Despawn/release if lifetime expired
        if projectile.lifetime.finished() {
            if is_pooled.is_some() {
                *visibility = Visibility::Hidden;
                to_release.push(projectile_entity);
            } else {
                commands.entity(projectile_entity).despawn();
            }
            continue;
        }

        let projectile_pos = projectile_transform.translation.truncate();

        // Despawn/release if too far from player
        if projectile_pos.distance(player_pos) > PROJECTILE_DESPAWN_DISTANCE {
            if is_pooled.is_some() {
                *visibility = Visibility::Hidden;
                to_release.push(projectile_entity);
            } else {
                commands.entity(projectile_entity).despawn();
            }
            continue;
        }

        // Check all enemies for collision (not just the original target)
        // This allows penetrating projectiles to hit any enemy they pass through
        for (enemy_entity, enemy_transform, mut enemy_stats) in enemy_query.iter_mut() {
            // Skip enemies we've already hit
            if projectile.enemies_hit.contains(&enemy_entity) {
                continue;
            }

            let enemy_pos = enemy_transform.translation.truncate();
            let distance = projectile_pos.distance(enemy_pos);

            // Hit detection - if projectile is close enough to enemy
            if distance < 20.0 {
                // Add this enemy to the hit list
                projectile.enemies_hit.push(enemy_entity);

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

                // Spawn floating damage number (if enabled)
                if debug_settings.show_damage_numbers {
                    let damage_color = get_damage_number_color(projectile.crit_tier);
                    let damage_text = format_damage(projectile.damage);

                    // Scale font size based on crit tier
                    let font_size = match projectile.crit_tier {
                        CritTier::None => 16.0,
                        CritTier::Normal => 20.0,
                        CritTier::Mega => 26.0,
                        CritTier::Super => 34.0,
                    };

                    // Try to get damage number from pool
                    if let Some(pooled_entity) = damage_number_pool.get() {
                        if let Ok((mut dmg_num, mut text, mut text_font, mut text_color, mut transform, mut vis)) = damage_number_query.get_mut(pooled_entity) {
                            dmg_num.reset();
                            *text = Text2d::new(damage_text.clone());
                            text_font.font_size = font_size;
                            *text_color = TextColor(damage_color);
                            transform.translation = Vec3::new(enemy_pos.x, enemy_pos.y + 20.0, 10.0);
                            *vis = Visibility::Visible;
                        }
                    } else {
                        // Pool exhausted, fall back to spawning
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
                    }
                }

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

                // Decrement penetration
                projectile.penetration_remaining = projectile.penetration_remaining.saturating_sub(1);

                // Check if projectile should despawn (final hit)
                if projectile.penetration_remaining == 0 {
                    // Handle explosive projectiles - AoE on final hit
                    if projectile.projectile_type == ProjectileType::Explosive {
                        pending_explosions.push((
                            projectile_pos,
                            EXPLOSIVE_AOE_RADIUS,
                            projectile.damage * 0.5, // AoE deals 50% damage
                            projectile.source_creature,
                            projectile.enemies_hit.clone(),
                        ));
                    }

                    // Return to pool or despawn
                    if is_pooled.is_some() {
                        *visibility = Visibility::Hidden;
                        to_release.push(projectile_entity);
                    } else {
                        commands.entity(projectile_entity).despawn();
                    }
                    break; // Exit the enemy loop since projectile is gone
                } else {
                    // Projectile continues flying - apply visual wear
                    // Reduce size slightly (10% per hit)
                    projectile.size *= 0.9;
                    sprite.custom_size = Some(Vec2::new(projectile.size, projectile.size));

                    // Reduce speed slightly (10% per hit)
                    projectile.speed *= 0.9;

                    // Handle chain projectiles - redirect toward nearby enemy
                    if projectile.projectile_type == ProjectileType::Chain {
                        // Find nearest enemy that hasn't been hit
                        let mut nearest_chain_target: Option<(Vec2, f32)> = None;
                        for (other_enemy, other_transform, _) in enemy_query.iter() {
                            if projectile.enemies_hit.contains(&other_enemy) {
                                continue;
                            }
                            let other_pos = other_transform.translation.truncate();
                            let chain_dist = projectile_pos.distance(other_pos);
                            if chain_dist < CHAIN_SEARCH_RADIUS {
                                if nearest_chain_target.is_none() || chain_dist < nearest_chain_target.unwrap().1 {
                                    nearest_chain_target = Some((other_pos, chain_dist));
                                }
                            }
                        }

                        if let Some((target_pos, _)) = nearest_chain_target {
                            pending_chains.push((projectile_entity, target_pos));
                        }
                    }

                    // Brief visual pulse effect - make it brighter momentarily
                    let current_color = sprite.color.to_srgba();
                    sprite.color = Color::srgba(
                        (current_color.red * 1.5).min(1.0),
                        (current_color.green * 1.5).min(1.0),
                        (current_color.blue * 1.5).min(1.0),
                        current_color.alpha,
                    );
                }

                // Only hit one enemy per frame to prevent multiple hits in same position
                break;
            }
        }
    }

    // Return projectiles to pool
    for entity in to_release {
        projectile_pool.release(entity);
    }

    // Apply chain redirections
    for (entity, target_pos) in pending_chains {
        if let Ok((_, projectile, transform, _, mut velocity, _, _)) = projectile_query.get_mut(entity) {
            let projectile_pos = transform.translation.truncate();
            let direction = (target_pos - projectile_pos).normalize_or_zero();
            velocity.x = direction.x * projectile.speed;
            velocity.y = direction.y * projectile.speed;

            // Spawn chain lightning visual effect
            spawn_chain_effect(&mut commands, projectile_pos, target_pos);
        }
    }

    // Spawn explosions
    for (pos, radius, damage, source, enemies_hit) in pending_explosions {
        spawn_explosion_effect(&mut commands, pos, radius);

        // Deal AoE damage to nearby enemies (excluding already hit ones)
        for (enemy_entity, enemy_transform, mut enemy_stats) in enemy_query.iter_mut() {
            if enemies_hit.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let dist = pos.distance(enemy_pos);
            if dist < radius {
                // Damage falloff based on distance
                let falloff = 1.0 - (dist / radius);
                let final_damage = damage * falloff as f64;

                let will_kill = enemy_stats.current_hp - final_damage <= 0.0;
                enemy_stats.current_hp -= final_damage;

                if will_kill {
                    if let Some(source_creature) = source {
                        commands.spawn(PendingKillCredit {
                            creature_entity: source_creature,
                        });
                    }
                }

                // Spawn damage number for AoE hit (if enabled)
                if debug_settings.show_damage_numbers {
                    commands.spawn((
                        DamageNumber::new(),
                        Text2d::new(format_damage(final_damage)),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.6, 0.2)), // Orange for AoE
                        Transform::from_translation(Vec3::new(
                            enemy_pos.x,
                            enemy_pos.y + 20.0,
                            10.0,
                        )),
                    ));
                }
            }
        }
    }
}

/// Spawn explosion visual effect
fn spawn_explosion_effect(commands: &mut Commands, position: Vec2, radius: f32) {
    // Spawn expanding circle effect
    commands.spawn((
        ExplosionEffect {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
            max_radius: radius,
        },
        Sprite {
            color: Color::srgba(1.0, 0.5, 0.1, 0.6), // Orange with transparency
            custom_size: Some(Vec2::new(20.0, 20.0)), // Start small
            ..default()
        },
        Transform::from_translation(Vec3::new(position.x, position.y, 0.7)),
    ));
}

/// Spawn chain lightning visual effect
fn spawn_chain_effect(commands: &mut Commands, from: Vec2, to: Vec2) {
    let midpoint = (from + to) / 2.0;
    let direction = to - from;
    let length = direction.length();
    let angle = direction.y.atan2(direction.x);

    commands.spawn((
        ChainEffect {
            timer: Timer::from_seconds(0.15, TimerMode::Once),
        },
        Sprite {
            color: Color::srgba(0.4, 0.8, 1.0, 0.8), // Electric blue
            custom_size: Some(Vec2::new(length, 3.0)), // Thin line
            ..default()
        },
        Transform::from_translation(Vec3::new(midpoint.x, midpoint.y, 0.7))
            .with_rotation(Quat::from_rotation_z(angle)),
    ));
}

/// Explosion visual effect component
#[derive(Component)]
pub struct ExplosionEffect {
    pub timer: Timer,
    pub max_radius: f32,
}

/// Chain lightning visual effect component
#[derive(Component)]
pub struct ChainEffect {
    pub timer: Timer,
}

/// System to update explosion visual effects
pub fn explosion_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionEffect, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut effect, mut sprite, mut _transform) in query.iter_mut() {
        effect.timer.tick(time.delta());

        if effect.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Expand the circle
        let progress = effect.timer.fraction();
        let current_size = effect.max_radius * 2.0 * progress;
        sprite.custom_size = Some(Vec2::new(current_size, current_size));

        // Fade out
        let alpha = 0.6 * (1.0 - progress);
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(current.red, current.green, current.blue, alpha);
    }
}

/// System to update chain lightning visual effects
pub fn chain_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ChainEffect, &mut Sprite)>,
) {
    for (entity, mut effect, mut sprite) in query.iter_mut() {
        effect.timer.tick(time.delta());

        if effect.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Fade out
        let progress = effect.timer.fraction();
        let alpha = 0.8 * (1.0 - progress);
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(current.red, current.green, current.blue, alpha);
    }
}

/// System that handles homing projectile behavior
pub fn homing_projectile_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut projectile_query: Query<(&Projectile, &Transform, &mut Velocity)>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    if debug_settings.is_paused() {
        return;
    }

    for (projectile, transform, mut velocity) in projectile_query.iter_mut() {
        if projectile.projectile_type != ProjectileType::Homing {
            continue;
        }

        let projectile_pos = transform.translation.truncate();

        // Find nearest enemy
        let mut nearest_enemy: Option<(Vec2, f32)> = None;
        for enemy_transform in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.truncate();

            // Skip enemies already hit
            // Note: We can't check this directly here since we don't have the entity
            // The homing will still work, it just might curve toward an already-hit enemy briefly

            let dist = projectile_pos.distance(enemy_pos);
            if nearest_enemy.is_none() || dist < nearest_enemy.unwrap().1 {
                nearest_enemy = Some((enemy_pos, dist));
            }
        }

        if let Some((target_pos, _)) = nearest_enemy {
            // Calculate desired direction
            let desired_direction = (target_pos - projectile_pos).normalize_or_zero();

            // Current direction
            let current_direction = Vec2::new(velocity.x, velocity.y).normalize_or_zero();

            // Blend toward desired direction based on turn rate
            let turn_amount = HOMING_TURN_RATE * time.delta_secs();
            let new_direction = (current_direction + desired_direction * turn_amount).normalize_or_zero();

            // Apply new direction while maintaining speed
            velocity.x = new_direction.x * projectile.speed;
            velocity.y = new_direction.y * projectile.speed;
        }
    }
}

/// System that rotates piercing projectiles to face their travel direction
pub fn piercing_rotation_system(
    debug_settings: Res<DebugSettings>,
    mut projectile_query: Query<(&Projectile, &Velocity, &mut Transform)>,
) {
    if debug_settings.is_paused() {
        return;
    }

    for (projectile, velocity, mut transform) in projectile_query.iter_mut() {
        if projectile.projectile_type != ProjectileType::Piercing {
            continue;
        }

        // Calculate rotation angle from velocity
        let angle = velocity.y.atan2(velocity.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

/// System that updates floating damage numbers (rise and fade)
pub fn damage_number_system(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_number_pool: ResMut<DamageNumberPool>,
    mut query: Query<(Entity, &mut DamageNumber, &mut Transform, &mut TextColor, &mut Visibility, Option<&Pooled>)>,
) {
    for (entity, mut damage_number, mut transform, mut text_color, mut visibility, is_pooled) in query.iter_mut() {
        // Skip hidden pooled damage numbers (they're inactive)
        if *visibility == Visibility::Hidden {
            continue;
        }

        // Tick lifetime
        damage_number.lifetime.tick(time.delta());

        // Despawn/release if lifetime expired
        if damage_number.lifetime.finished() {
            if is_pooled.is_some() {
                *visibility = Visibility::Hidden;
                damage_number_pool.release(entity);
            } else {
                commands.entity(entity).despawn();
            }
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

/// Contact damage range (player + enemy collision overlap)
pub const ENEMY_CONTACT_RANGE: f32 = 35.0;

/// Contact damage multiplier (contact = enemy base_damage * this)
pub const CONTACT_DAMAGE_MULTIPLIER: f64 = 0.5;

/// Invincibility duration after taking damage (seconds)
pub const INVINCIBILITY_DURATION: f32 = 0.5;

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

/// System that handles enemies attacking the player
pub fn enemy_attack_player_system(
    mut commands: Commands,
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    enemy_query: Query<(&EnemyStats, &EnemyAttackTimer, &Transform), With<Enemy>>,
    mut player_query: Query<(Entity, &Transform, &mut PlayerStats, Option<&InvincibilityTimer>), With<Player>>,
) {
    // Don't process if game is paused or god mode is enabled
    if debug_settings.is_paused() || debug_settings.god_mode {
        return;
    }

    let Ok((player_entity, player_transform, mut player_stats, invincibility_opt)) = player_query.get_single_mut() else {
        return;
    };

    // Check if player is invincible
    if let Some(invincibility) = invincibility_opt {
        if invincibility.is_active() {
            return;
        }
    }

    let player_pos = player_transform.translation.truncate();

    for (enemy_stats, attack_timer, enemy_transform) in enemy_query.iter() {
        // Only attack when timer just finished (enemies already ticked timer in enemy_attack_system)
        // We check the same condition to sync with creature attacks
        if !attack_timer.timer.just_finished() {
            continue;
        }

        let enemy_pos = enemy_transform.translation.truncate();
        let distance = enemy_pos.distance(player_pos);

        if distance <= ENEMY_ATTACK_RANGE {
            // Apply damage to player
            let damage = enemy_stats.base_damage * debug_settings.enemy_damage_multiplier as f64;
            player_stats.current_hp -= damage;

            // Add invincibility frames
            commands.entity(player_entity).insert(InvincibilityTimer::new(INVINCIBILITY_DURATION));

            // Only take damage from one enemy per frame
            break;
        }
    }
}

/// System that handles contact damage to the player from enemies
pub fn enemy_contact_damage_system(
    mut commands: Commands,
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    enemy_query: Query<(&EnemyStats, &Transform), With<Enemy>>,
    mut player_query: Query<(Entity, &Transform, &mut PlayerStats, Option<&mut InvincibilityTimer>), With<Player>>,
) {
    // Don't process if game is paused or god mode is enabled
    if debug_settings.is_paused() || debug_settings.god_mode {
        return;
    }

    let Ok((player_entity, player_transform, mut player_stats, invincibility_opt)) = player_query.get_single_mut() else {
        return;
    };

    // Check and tick invincibility timer
    if let Some(mut invincibility) = invincibility_opt {
        invincibility.timer.tick(time.delta());
        if invincibility.is_active() {
            return;
        }
    }

    let player_pos = player_transform.translation.truncate();

    for (enemy_stats, enemy_transform) in enemy_query.iter() {
        let enemy_pos = enemy_transform.translation.truncate();
        let distance = player_pos.distance(enemy_pos);

        if distance < ENEMY_CONTACT_RANGE {
            // Apply contact damage
            let damage = enemy_stats.base_damage * CONTACT_DAMAGE_MULTIPLIER * debug_settings.enemy_damage_multiplier as f64;
            player_stats.current_hp -= damage;

            // Add invincibility frames
            commands.entity(player_entity).insert(InvincibilityTimer::new(INVINCIBILITY_DURATION));

            // Only take contact damage from one enemy per frame
            break;
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

                    let proj_size = weapon_stats.projectile_size;
                    commands.spawn((
                        Projectile {
                            target: target_entity,
                            damage: weapon_stats.auto_damage,
                            crit_tier: CritTier::None, // Weapons don't crit (for now)
                            lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                            source_creature: None, // Weapon projectiles don't give creature XP
                            size: proj_size,
                            speed: projectile_speed,
                            penetration_remaining: weapon_stats.projectile_penetration,
                            enemies_hit: Vec::new(),
                            projectile_type: ProjectileType::Basic, // Weapons use basic projectiles
                        },
                        Velocity {
                            x: rotated_dir.x * projectile_speed,
                            y: rotated_dir.y * projectile_speed,
                        },
                        Sprite {
                            color: weapon_data.color.to_bevy_color().lighter(0.3),
                            custom_size: Some(Vec2::new(proj_size, proj_size)),
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

/// System to update the spatial grid with enemy positions
/// This should run before creature_attack_system for optimal performance
pub fn update_spatial_grid_system(
    mut spatial_grid: ResMut<SpatialGrid>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    // Clear the grid each frame
    spatial_grid.clear();

    // Insert all enemies into the grid
    for (entity, transform) in enemy_query.iter() {
        let pos = transform.translation.truncate();
        spatial_grid.insert(entity, pos);
    }
}

/// System to initialize projectile and damage number pools at startup
/// Pre-spawns hidden entities that can be reused
pub fn init_pools_system(
    mut commands: Commands,
    mut projectile_pool: ResMut<ProjectilePool>,
    mut damage_number_pool: ResMut<DamageNumberPool>,
) {
    use crate::resources::{PROJECTILE_POOL_SIZE, DAMAGE_NUMBER_POOL_SIZE};

    // Pre-spawn projectiles (hidden, off-screen)
    for _ in 0..PROJECTILE_POOL_SIZE {
        let entity = commands.spawn((
            Pooled,
            Projectile {
                target: Entity::PLACEHOLDER,
                damage: 0.0,
                crit_tier: CritTier::None,
                lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                source_creature: None,
                size: PROJECTILE_SIZE,
                speed: PROJECTILE_SPEED,
                penetration_remaining: 1,
                enemies_hit: Vec::new(),
                projectile_type: ProjectileType::Basic,
            },
            Velocity::default(),
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(PROJECTILE_SIZE, PROJECTILE_SIZE)),
                ..default()
            },
            Transform::from_translation(Vec3::new(-10000.0, -10000.0, 0.6)),
            Visibility::Hidden,
        )).id();
        projectile_pool.available.push(entity);
    }

    // Pre-spawn damage numbers (hidden, off-screen)
    for _ in 0..DAMAGE_NUMBER_POOL_SIZE {
        let entity = commands.spawn((
            Pooled,
            DamageNumber::new(),
            Text2d::new("0"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(Vec3::new(-10000.0, -10000.0, 10.0)),
            Visibility::Hidden,
        )).id();
        damage_number_pool.available.push(entity);
    }
}

/// System to re-initialize pools if they become empty (e.g., after game restart)
/// This runs every frame but only does work when pools are empty
pub fn init_pools_if_empty_system(
    mut commands: Commands,
    mut projectile_pool: ResMut<ProjectilePool>,
    mut damage_number_pool: ResMut<DamageNumberPool>,
) {
    use crate::resources::{PROJECTILE_POOL_SIZE, DAMAGE_NUMBER_POOL_SIZE};

    // Check if projectile pool needs re-initialization
    if projectile_pool.available.is_empty() && projectile_pool.active.is_empty() {
        // Pre-spawn projectiles (hidden, off-screen)
        for _ in 0..PROJECTILE_POOL_SIZE {
            let entity = commands.spawn((
                Pooled,
                Projectile {
                    target: Entity::PLACEHOLDER,
                    damage: 0.0,
                    crit_tier: CritTier::None,
                    lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                    source_creature: None,
                    size: PROJECTILE_SIZE,
                    speed: PROJECTILE_SPEED,
                    penetration_remaining: 1,
                    enemies_hit: Vec::new(),
                    projectile_type: ProjectileType::Basic,
                },
                Velocity::default(),
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(PROJECTILE_SIZE, PROJECTILE_SIZE)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(-10000.0, -10000.0, 0.6)),
                Visibility::Hidden,
            )).id();
            projectile_pool.available.push(entity);
        }
    }

    // Check if damage number pool needs re-initialization
    if damage_number_pool.available.is_empty() && damage_number_pool.active.is_empty() {
        // Pre-spawn damage numbers (hidden, off-screen)
        for _ in 0..DAMAGE_NUMBER_POOL_SIZE {
            let entity = commands.spawn((
                Pooled,
                DamageNumber::new(),
                Text2d::new("0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(-10000.0, -10000.0, 10.0)),
                Visibility::Hidden,
            )).id();
            damage_number_pool.available.push(entity);
        }
    }
}
