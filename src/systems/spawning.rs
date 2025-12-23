use bevy::prelude::*;
use rand::Rng;

use crate::components::{
    AttackRange, AttackTimer, Creature, CreatureColor, CreatureStats, CreatureType, Enemy,
    EnemyAttackTimer, EnemyClass, EnemyStats, EnemyType, Player, ProjectileConfig, ProjectileType,
    SpriteAnimation, Velocity, Weapon, WeaponAttackTimer, WeaponData, WeaponStats,
    get_creature_color_by_id,
};
use crate::resources::{AffinityState, ArtifactBuffs, DeathSprites, DebugSettings, Director, GameData, GameState};
use crate::systems::death::RespawnQueue;

/// Size of creature sprites in pixels
pub const CREATURE_SIZE: f32 = 32.0;

/// Size of enemy sprites in pixels
pub const ENEMY_SIZE: f32 = 28.0;

/// Base spawn interval in seconds (will be modified by Director)
pub const BASE_SPAWN_INTERVAL: f32 = 0.2;

/// Minimum distance from player to spawn enemies
pub const ENEMY_SPAWN_MIN_DISTANCE: f32 = 600.0;

/// Maximum distance from player to spawn enemies
pub const ENEMY_SPAWN_MAX_DISTANCE: f32 = 900.0;

/// Distance at which enemies are despawned (cleanup)
pub const ENEMY_DESPAWN_DISTANCE: f32 = 2500.0;

/// Minimum enemies spawned per second (floor)
pub const MIN_ENEMIES_PER_SECOND: u32 = 15;

/// Maximum enemies allowed on screen at once (performance cap)
pub const MAX_ENEMIES: u32 = 2000;

/// Kills needed to advance to the next wave
pub const KILLS_PER_WAVE: u32 = 50;

/// Resource for tracking enemy spawn timing
#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: Timer,
    /// Tracks if we need to update timer duration
    pub last_interval: f32,
}

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(BASE_SPAWN_INTERVAL, TimerMode::Repeating),
            last_interval: BASE_SPAWN_INTERVAL,
        }
    }
}

/// Spawn a creature by ID from the game data
pub fn spawn_creature(
    commands: &mut Commands,
    game_data: &GameData,
    artifact_buffs: &ArtifactBuffs,
    creature_id: &str,
    position: Vec3,
) -> Option<Entity> {
    // Find creature data by ID
    let creature_data = game_data.creatures.iter().find(|c| c.id == creature_id)?;

    let color = CreatureColor::from_str(&creature_data.color);
    let creature_type = CreatureType::from_str(&creature_data.creature_type);

    // Get artifact bonuses for this creature
    let bonuses = artifact_buffs.get_total_bonuses(creature_id, color, creature_type);

    // Apply HP bonus to base HP
    let modified_hp = creature_data.base_hp * (1.0 + bonuses.hp_bonus / 100.0);

    // Apply attack speed bonus
    let modified_attack_speed = creature_data.attack_speed * (1.0 + bonuses.attack_speed_bonus / 100.0);

    // Get first kills_per_level threshold, or default to 10
    let kills_for_next_level = creature_data.kills_per_level.first().copied().unwrap_or(10);

    let mut stats = CreatureStats::new(
        creature_data.id.clone(),
        creature_data.name.clone(),
        color,
        creature_data.tier,
        creature_type,
        creature_data.base_damage,
        creature_data.attack_speed,
        creature_data.base_hp,
        creature_data.movement_speed,
        creature_data.attack_range,
        creature_data.crit_t1,
        creature_data.crit_t2,
        creature_data.crit_t3,
        kills_for_next_level,
        creature_data.max_level,
        creature_data.evolves_into.clone(),
        creature_data.evolution_count,
    );

    // Apply HP bonuses to the stats
    stats.max_hp = modified_hp;
    stats.current_hp = modified_hp;

    // Determine attack range based on creature type
    let attack_range = match creature_type {
        CreatureType::Ranged => creature_data.attack_range as f32,
        CreatureType::Support => creature_data.attack_range as f32,
        CreatureType::Melee => 50.0,
        CreatureType::Assassin => 60.0,
    };

    // Create projectile config from TOML data
    let projectile_config = ProjectileConfig::new(
        creature_data.projectile_count,
        creature_data.projectile_spread,
        creature_data.projectile_size,
        creature_data.projectile_speed,
        creature_data.projectile_penetration,
        ProjectileType::from_str(&creature_data.projectile_type),
    );

    // Get unique color for this specific creature type
    let creature_color = get_creature_color_by_id(&creature_data.id);

    let entity = commands
        .spawn((
            Creature,
            stats.clone(),
            Velocity::default(),
            AttackTimer::new(modified_attack_speed),
            AttackRange(attack_range),
            projectile_config,
            Sprite {
                color: creature_color,
                custom_size: Some(Vec2::new(CREATURE_SIZE, CREATURE_SIZE)),
                ..default()
            },
            Transform::from_translation(position),
        ))
        .id();

    Some(entity)
}

/// Spawn a weapon by ID from the game data
/// Weapons are invisible entities that auto-attack and provide affinity
pub fn spawn_weapon(
    commands: &mut Commands,
    game_data: &GameData,
    affinity_state: &mut AffinityState,
    weapon_id: &str,
) -> Option<Entity> {
    // Find weapon data by ID
    let weapon_data = game_data.weapons.iter().find(|w| w.id == weapon_id)?;

    let color = CreatureColor::from_str(&weapon_data.color);

    let data = WeaponData::new(
        weapon_data.id.clone(),
        weapon_data.name.clone(),
        color,
        weapon_data.tier,
        weapon_data.affinity_amount,
    );

    let stats = WeaponStats::new(
        weapon_data.auto_damage,
        weapon_data.auto_speed,
        weapon_data.auto_range,
        weapon_data.projectile_count,
        weapon_data.projectile_pattern.clone(),
        weapon_data.projectile_speed,
        weapon_data.projectile_size,
        weapon_data.projectile_penetration,
    );

    // Add affinity for this weapon's color
    affinity_state.add(color, weapon_data.affinity_amount);

    // Spawn weapon entity (no visible sprite)
    let entity = commands
        .spawn((
            Weapon,
            data.clone(),
            stats,
            WeaponAttackTimer::new(weapon_data.auto_speed),
        ))
        .id();

    Some(entity)
}

/// Check and handle weapon evolution
/// Returns Some(evolved_weapon_id) if evolution occurred
pub fn try_weapon_evolution(
    commands: &mut Commands,
    game_data: &GameData,
    affinity_state: &mut AffinityState,
    weapon_query: &Query<(Entity, &WeaponData)>,
) -> Option<String> {
    // Get list of all current weapons
    let weapons: Vec<(Entity, String, CreatureColor, f64)> = weapon_query
        .iter()
        .map(|(entity, data)| (entity, data.id.clone(), data.color, data.affinity_amount))
        .collect();

    // Check each weapon's evolution recipe
    for weapon in &game_data.weapons {
        if weapon.evolution_recipe.is_empty() {
            continue;
        }

        // Count how many of each required weapon we have
        let mut recipe_met = true;
        let mut recipe_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for required in &weapon.evolution_recipe {
            *recipe_counts.entry(required.clone()).or_insert(0) += 1;
        }

        let mut weapons_to_consume: Vec<Entity> = Vec::new();

        for (required_id, required_count) in &recipe_counts {
            let matching: Vec<_> = weapons
                .iter()
                .filter(|(entity, id, _, _)| {
                    id == required_id && !weapons_to_consume.contains(entity)
                })
                .take(*required_count)
                .collect();

            if matching.len() < *required_count {
                recipe_met = false;
                break;
            }

            for (entity, _, _, _) in matching {
                weapons_to_consume.push(*entity);
            }
        }

        if recipe_met && !weapons_to_consume.is_empty() {
            // Remove affinity from consumed weapons
            for &entity in &weapons_to_consume {
                if let Some((_, _, color, affinity)) = weapons.iter().find(|(e, _, _, _)| *e == entity) {
                    affinity_state.remove(*color, *affinity);
                }
                commands.entity(entity).despawn();
            }

            // Spawn evolved weapon
            let evolved_id = &weapon.id;
            if spawn_weapon(commands, game_data, affinity_state, evolved_id).is_some() {
                return Some(evolved_id.clone());
            }
        }
    }

    None
}

/// Get color for an enemy based on its ID and whether it's elite
fn get_enemy_color(enemy_id: &str, is_elite: bool) -> Color {
    let base_color = match enemy_id {
        "goblin" => Color::srgb(0.2, 0.7, 0.3),           // Green
        "goblin_archer" => Color::srgb(0.15, 0.5, 0.2),   // Dark green
        "wolf" => Color::srgb(0.5, 0.5, 0.55),            // Gray
        "skeleton" => Color::srgb(0.9, 0.9, 0.85),        // Bone white
        "bat_swarm" => Color::srgb(0.3, 0.2, 0.3),        // Dark purple
        "slime" => Color::srgb(0.3, 0.8, 0.5),            // Light green
        "orc_warrior" => Color::srgb(0.4, 0.6, 0.3),      // Olive green
        _ => Color::srgb(0.6, 0.3, 0.3),                  // Default reddish
    };

    if is_elite {
        // Make elites brighter/more saturated
        let Srgba { red, green, blue, alpha } = base_color.to_srgba();
        Color::srgba(
            (red * 1.3).min(1.0),
            (green * 1.3).min(1.0),
            (blue * 1.3).min(1.0),
            alpha
        )
    } else {
        base_color
    }
}

/// Spawn an enemy by ID with wave scaling
pub fn spawn_enemy_scaled(
    commands: &mut Commands,
    game_data: &GameData,
    death_sprites: Option<&DeathSprites>,
    enemy_id: &str,
    position: Vec3,
    wave: u32,
    is_elite: bool,
) -> Option<Entity> {
    // Find enemy data by ID
    let enemy_data = game_data.enemies.iter().find(|e| e.id == enemy_id)?;

    let enemy_class = EnemyClass::from_str(&enemy_data.enemy_class);
    let enemy_type = EnemyType::from_str(&enemy_data.enemy_type);

    // Apply wave HP scaling
    let hp_scale = Director::get_hp_scale(wave);
    let scaled_hp = enemy_data.base_hp * hp_scale;

    // Elites get 3x HP and 1.5x damage
    let (final_hp, final_damage) = if is_elite {
        (scaled_hp * 3.0, enemy_data.base_damage * 1.5)
    } else {
        (scaled_hp, enemy_data.base_damage)
    };

    let stats = EnemyStats::new(
        enemy_data.id.clone(),
        if is_elite {
            format!("Elite {}", enemy_data.name)
        } else {
            enemy_data.name.clone()
        },
        enemy_class,
        enemy_type,
        final_hp,
        final_damage,
        enemy_data.attack_speed,
        enemy_data.movement_speed,
        enemy_data.attack_range,
    );

    // Elites are slightly larger (scale factor for sprite)
    let scale = if is_elite { 0.5 } else { 0.4 };

    // Use imp spritesheet if available, otherwise fall back to colored square
    let entity = if let Some(sprites) = death_sprites {
        commands
            .spawn((
                Enemy,
                stats,
                Velocity::default(),
                EnemyAttackTimer::new(enemy_data.attack_speed),
                SpriteAnimation::new(), // Start in idle state (frame 0)
                Sprite::from_atlas_image(
                    sprites.imp_spritesheet.clone(),
                    bevy::sprite::TextureAtlas {
                        layout: sprites.imp_atlas.clone(),
                        index: 0, // Frame 0 = idle
                    },
                ),
                Transform::from_translation(position).with_scale(Vec3::splat(scale)),
            ))
            .id()
    } else {
        // Fallback: colored square (no sprites loaded)
        let enemy_color = get_enemy_color(enemy_id, is_elite);
        let size = if is_elite { ENEMY_SIZE * 1.3 } else { ENEMY_SIZE };
        commands
            .spawn((
                Enemy,
                stats,
                Velocity::default(),
                EnemyAttackTimer::new(enemy_data.attack_speed),
                Sprite {
                    color: enemy_color,
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                Transform::from_translation(position),
            ))
            .id()
    };

    Some(entity)
}

/// Spawn an enemy by ID from the game data (legacy function for compatibility)
pub fn spawn_enemy(
    commands: &mut Commands,
    game_data: &GameData,
    enemy_id: &str,
    position: Vec3,
) -> Option<Entity> {
    spawn_enemy_scaled(commands, game_data, None, enemy_id, position, 1, false)
}

/// System to spawn a test creature (Fire Imp) when spacebar is pressed
pub fn spawn_test_creature_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_data: Res<GameData>,
    artifact_buffs: Res<ArtifactBuffs>,
    game_phase: Res<crate::resources::GamePhase>,
    player_query: Query<&Transform, With<Player>>,
    creature_query: Query<&Creature>,
) {
    // Only allow spawning during gameplay
    if *game_phase != crate::resources::GamePhase::Playing {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.get_single() {
            // Count existing creatures for offset calculation
            let creature_count = creature_query.iter().count();

            // Offset position around the player in a circle pattern
            let angle = creature_count as f32 * 0.8; // Spread creatures around
            let offset_distance = 80.0;
            let offset_x = angle.cos() * offset_distance;
            let offset_y = angle.sin() * offset_distance;

            let spawn_pos = Vec3::new(
                player_transform.translation.x + offset_x,
                player_transform.translation.y + offset_y,
                0.5, // Above background, below player
            );

            spawn_creature(&mut commands, &game_data, &artifact_buffs, "fire_imp", spawn_pos);
        }
    }
}

/// Select which enemy to spawn based on current wave
fn select_enemy_for_wave(wave: u32) -> &'static str {
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();

    match wave {
        1..=5 => "goblin",
        6..=10 => {
            if roll < 0.20 {
                "goblin_archer"
            } else {
                "goblin"
            }
        }
        11..=14 => {
            if roll < 0.15 {
                "wolf"
            } else if roll < 0.35 {
                "goblin_archer"
            } else {
                "goblin"
            }
        }
        _ => {
            // Wave 15+: More variety
            if roll < 0.15 {
                "wolf"
            } else if roll < 0.30 {
                "goblin_archer"
            } else if roll < 0.40 {
                "skeleton"
            } else {
                "goblin"
            }
        }
    }
}

/// MASSIVE HORDE enemy spawn system
/// Spawns enemies in large batches from multiple directions
pub fn enemy_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    mut game_state: ResMut<GameState>,
    mut director: ResMut<Director>,
    debug_settings: Res<DebugSettings>,
    game_phase: Res<crate::resources::GamePhase>,
    game_data: Res<GameData>,
    death_sprites: Option<Res<DeathSprites>>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Enemy>,
) {
    // Don't spawn if game is paused or not in playing phase
    if debug_settings.is_paused() || *game_phase != crate::resources::GamePhase::Playing {
        return;
    }

    // Update enemy count in director
    director.enemies_alive = enemy_query.iter().count() as u32;

    // Don't spawn if at enemy cap (performance limit, configurable via debug menu)
    if director.enemies_alive >= debug_settings.max_enemies {
        return;
    }

    // Apply wave/level overrides from debug settings
    if let Some(wave_override) = debug_settings.current_wave_override {
        if game_state.current_wave != wave_override {
            game_state.current_wave = wave_override;
            game_state.kills_at_wave_start = game_state.total_kills;
        }
    }
    if let Some(level_override) = debug_settings.current_level_override {
        if game_state.current_level != level_override {
            game_state.current_level = level_override;
        }
    }

    // Check for wave advancement based on kills (only if not overridden)
    if debug_settings.current_wave_override.is_none() {
        let kills_this_wave = game_state.total_kills - game_state.kills_at_wave_start;
        if kills_this_wave >= KILLS_PER_WAVE {
            game_state.current_wave += 1;
            game_state.kills_at_wave_start = game_state.total_kills;
        }
    }

    // Update spawn interval based on Director and debug spawn rate multiplier
    let base_interval = director.get_spawn_interval(game_state.current_wave);
    // Higher multiplier = faster spawns (divide by multiplier)
    let new_interval = base_interval / debug_settings.enemy_spawn_rate_multiplier;
    if (new_interval - spawn_timer.last_interval).abs() > 0.01 {
        spawn_timer.timer.set_duration(std::time::Duration::from_secs_f32(new_interval.max(0.05)));
        spawn_timer.last_interval = new_interval;
    }

    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        if let Ok(player_transform) = player_query.get_single() {
            let mut rng = rand::thread_rng();
            let player_pos = player_transform.translation;

            // Get spawn counts for this wave
            let (min_spawn, max_spawn) = Director::get_enemies_per_spawn(game_state.current_wave);
            let enemies_to_spawn = rng.gen_range(min_spawn..=max_spawn);

            // Apply performance throttle and spawn rate multiplier
            let throttled_spawn = ((enemies_to_spawn as f32)
                * director.performance_throttle
                * debug_settings.enemy_spawn_rate_multiplier) as u32;
            let final_spawn_count = throttled_spawn.max(MIN_ENEMIES_PER_SECOND / 5); // Minimum floor

            // Spawn from 2-4 cluster points
            let cluster_count = rng.gen_range(2..=4);
            let enemies_per_cluster = final_spawn_count / cluster_count;

            // Get elite chance for this wave
            let elite_chance = Director::get_elite_chance(game_state.current_wave);

            for _ in 0..cluster_count {
                // Random cluster center angle
                let cluster_angle = rng.gen::<f32>() * std::f32::consts::TAU;

                // Random distance for cluster center
                let cluster_distance = rng.gen::<f32>() * (ENEMY_SPAWN_MAX_DISTANCE - ENEMY_SPAWN_MIN_DISTANCE)
                    + ENEMY_SPAWN_MIN_DISTANCE;

                let cluster_center = Vec2::new(
                    player_pos.x + cluster_angle.cos() * cluster_distance,
                    player_pos.y + cluster_angle.sin() * cluster_distance,
                );

                // Spawn enemies in a tight cluster
                for _ in 0..enemies_per_cluster {
                    // Small random offset within cluster (50-80 pixel radius)
                    let offset_angle = rng.gen::<f32>() * std::f32::consts::TAU;
                    let offset_dist = rng.gen::<f32>() * 80.0;

                    let spawn_pos = Vec3::new(
                        cluster_center.x + offset_angle.cos() * offset_dist,
                        cluster_center.y + offset_angle.sin() * offset_dist,
                        0.3, // Below creatures and player
                    );

                    // Check if elite
                    let is_elite = rng.gen::<f32>() < elite_chance;

                    // Select enemy based on current wave
                    let enemy_id = select_enemy_for_wave(game_state.current_wave);

                    spawn_enemy_scaled(
                        &mut commands,
                        &game_data,
                        death_sprites.as_deref(),
                        enemy_id,
                        spawn_pos,
                        game_state.current_wave,
                        is_elite,
                    );
                }
            }
        }
    }
}

/// System to despawn enemies that are too far from player (cleanup)
pub fn enemy_cleanup_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (entity, transform) in enemy_query.iter() {
        let enemy_pos = transform.translation.truncate();
        let distance = player_pos.distance(enemy_pos);

        if distance > ENEMY_DESPAWN_DISTANCE {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System to update Director metrics
pub fn director_update_system(
    time: Res<Time>,
    mut director: ResMut<Director>,
    game_phase: Res<crate::resources::GamePhase>,
    creature_query: Query<&CreatureStats, With<Creature>>,
    enemy_query: Query<&Enemy>,
) {
    // Don't update director when not playing
    if *game_phase != crate::resources::GamePhase::Playing {
        return;
    }
    // Update creature count and HP
    let mut total_hp = 0.0;
    let mut total_max_hp = 0.0;
    let mut creature_count = 0u32;

    for stats in creature_query.iter() {
        creature_count += 1;
        total_hp += stats.current_hp;
        total_max_hp += stats.max_hp;
    }

    director.creature_count = creature_count;
    director.total_creature_hp_percent = if total_max_hp > 0.0 {
        total_hp / total_max_hp
    } else {
        1.0
    };

    // Update enemy count
    director.enemies_alive = enemy_query.iter().count() as u32;

    // Calculate stress
    director.calculate_stress();

    // Update DPS (would need damage events to track properly)
    director.update_dps(time.elapsed_secs());

    // Update FPS (simple approximation)
    let fps = 1.0 / time.delta_secs();
    director.update_performance(fps, time.delta_secs());
}

/// System to handle creature respawns from the respawn queue
pub fn respawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut respawn_queue: ResMut<RespawnQueue>,
    game_data: Res<GameData>,
    artifact_buffs: Res<ArtifactBuffs>,
    player_query: Query<&Transform, With<Player>>,
    creature_query: Query<&Creature>,
) {
    // Get current player position for spawn offset calculation
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or(Vec3::ZERO);

    // Count existing creatures for offset calculation
    let creature_count = creature_query.iter().count();

    // Track which entries to remove (completed respawns)
    let mut completed_indices = Vec::new();

    // Tick all timers and check for completed respawns
    for (index, entry) in respawn_queue.entries.iter_mut().enumerate() {
        entry.timer.tick(time.delta());

        if entry.timer.just_finished() {
            // Calculate spawn position around player
            let offset_index = creature_count + completed_indices.len();
            let angle = offset_index as f32 * 0.8;
            let offset_distance = 80.0;
            let offset_x = angle.cos() * offset_distance;
            let offset_y = angle.sin() * offset_distance;

            let spawn_pos = Vec3::new(
                player_pos.x + offset_x,
                player_pos.y + offset_y,
                0.5,
            );

            // Spawn the creature
            spawn_creature(&mut commands, &game_data, &artifact_buffs, &entry.creature_id, spawn_pos);

            completed_indices.push(index);
        }
    }

    // Remove completed entries (in reverse order to preserve indices)
    for index in completed_indices.into_iter().rev() {
        respawn_queue.entries.remove(index);
    }
}
