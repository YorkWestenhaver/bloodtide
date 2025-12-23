use bevy::prelude::*;
use rand::Rng;

use crate::components::{
    AttackRange, AttackTimer, Creature, CreatureColor, CreatureStats, CreatureType, Enemy,
    EnemyAttackTimer, EnemyClass, EnemyStats, EnemyType, Player, Velocity,
};
use crate::resources::{GameData, GameState};
use crate::systems::death::RespawnQueue;

/// Size of creature sprites in pixels
pub const CREATURE_SIZE: f32 = 32.0;

/// Size of enemy sprites in pixels
pub const ENEMY_SIZE: f32 = 28.0;

/// Enemy spawn interval in seconds
pub const ENEMY_SPAWN_INTERVAL: f32 = 1.5;

/// Minimum distance from player to spawn enemies
pub const ENEMY_SPAWN_MIN_DISTANCE: f32 = 600.0;

/// Maximum distance from player to spawn enemies
pub const ENEMY_SPAWN_MAX_DISTANCE: f32 = 800.0;

/// Resource for tracking enemy spawn timing
#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: Timer,
}

impl Default for EnemySpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(ENEMY_SPAWN_INTERVAL, TimerMode::Repeating),
        }
    }
}

/// Spawn a creature by ID from the game data
pub fn spawn_creature(
    commands: &mut Commands,
    game_data: &GameData,
    creature_id: &str,
    position: Vec3,
) -> Option<Entity> {
    // Find creature data by ID
    let creature_data = game_data.creatures.iter().find(|c| c.id == creature_id)?;

    let color = CreatureColor::from_str(&creature_data.color);
    let creature_type = CreatureType::from_str(&creature_data.creature_type);

    let stats = CreatureStats::new(
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
    );

    // Determine attack range based on creature type
    let attack_range = match creature_type {
        CreatureType::Ranged => creature_data.attack_range as f32,
        CreatureType::Support => creature_data.attack_range as f32,
        CreatureType::Melee => 50.0,
        CreatureType::Assassin => 60.0,
    };

    let entity = commands
        .spawn((
            Creature,
            stats.clone(),
            Velocity::default(),
            AttackTimer::new(creature_data.attack_speed),
            AttackRange(attack_range),
            Sprite {
                color: color.to_bevy_color(),
                custom_size: Some(Vec2::new(CREATURE_SIZE, CREATURE_SIZE)),
                ..default()
            },
            Transform::from_translation(position),
        ))
        .id();

    println!(
        "Spawned {} (Tier {} {}, range: {:.0}) at ({:.0}, {:.0})",
        stats.name, stats.tier, creature_data.creature_type, attack_range, position.x, position.y
    );

    Some(entity)
}

/// Get color for an enemy based on its ID
fn get_enemy_color(enemy_id: &str) -> Color {
    match enemy_id {
        "goblin" => Color::srgb(0.2, 0.7, 0.3),           // Green
        "goblin_archer" => Color::srgb(0.15, 0.5, 0.2),   // Dark green
        "wolf" => Color::srgb(0.5, 0.5, 0.55),            // Gray
        "skeleton" => Color::srgb(0.9, 0.9, 0.85),        // Bone white
        "bat_swarm" => Color::srgb(0.3, 0.2, 0.3),        // Dark purple
        "slime" => Color::srgb(0.3, 0.8, 0.5),            // Light green
        "orc_warrior" => Color::srgb(0.4, 0.6, 0.3),      // Olive green
        _ => Color::srgb(0.6, 0.3, 0.3),                  // Default reddish
    }
}

/// Spawn an enemy by ID from the game data
pub fn spawn_enemy(
    commands: &mut Commands,
    game_data: &GameData,
    enemy_id: &str,
    position: Vec3,
) -> Option<Entity> {
    // Find enemy data by ID
    let enemy_data = game_data.enemies.iter().find(|e| e.id == enemy_id)?;

    let enemy_class = EnemyClass::from_str(&enemy_data.enemy_class);
    let enemy_type = EnemyType::from_str(&enemy_data.enemy_type);

    let stats = EnemyStats::new(
        enemy_data.id.clone(),
        enemy_data.name.clone(),
        enemy_class,
        enemy_type,
        enemy_data.base_hp,
        enemy_data.base_damage,
        enemy_data.attack_speed,
        enemy_data.movement_speed,
        enemy_data.attack_range,
    );

    // Get color based on enemy type
    let enemy_color = get_enemy_color(enemy_id);

    let entity = commands
        .spawn((
            Enemy,
            stats.clone(),
            Velocity::default(),
            EnemyAttackTimer::new(enemy_data.attack_speed),
            Sprite {
                color: enemy_color,
                custom_size: Some(Vec2::new(ENEMY_SIZE, ENEMY_SIZE)),
                ..default()
            },
            Transform::from_translation(position),
        ))
        .id();

    println!(
        "Spawned {} ({} {}, HP: {:.0}, Speed: {:.0}) at ({:.0}, {:.0})",
        stats.name, enemy_data.enemy_class, enemy_data.enemy_type,
        enemy_data.base_hp, enemy_data.movement_speed,
        position.x, position.y
    );

    Some(entity)
}

/// System to spawn a test creature (Fire Imp) when spacebar is pressed
pub fn spawn_test_creature_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_data: Res<GameData>,
    player_query: Query<&Transform, With<Player>>,
    creature_query: Query<&Creature>,
) {
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

            spawn_creature(&mut commands, &game_data, "fire_imp", spawn_pos);
        }
    }
}

/// Kills needed to advance to the next wave
pub const KILLS_PER_WAVE: u32 = 50;

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

/// System to spawn enemies periodically
pub fn enemy_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    mut game_state: ResMut<GameState>,
    game_data: Res<GameData>,
    player_query: Query<&Transform, With<Player>>,
) {
    spawn_timer.timer.tick(time.delta());

    // Check for wave advancement based on kills
    let kills_this_wave = game_state.total_kills - game_state.kills_at_wave_start;
    if kills_this_wave >= KILLS_PER_WAVE {
        game_state.current_wave += 1;
        game_state.kills_at_wave_start = game_state.total_kills;
        println!(
            "========== WAVE {} STARTED! ==========",
            game_state.current_wave
        );
    }

    if spawn_timer.timer.just_finished() {
        if let Ok(player_transform) = player_query.get_single() {
            let mut rng = rand::thread_rng();

            // Random angle around player
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;

            // Random distance between min and max spawn distance
            let distance =
                rng.gen::<f32>() * (ENEMY_SPAWN_MAX_DISTANCE - ENEMY_SPAWN_MIN_DISTANCE)
                    + ENEMY_SPAWN_MIN_DISTANCE;

            let spawn_pos = Vec3::new(
                player_transform.translation.x + angle.cos() * distance,
                player_transform.translation.y + angle.sin() * distance,
                0.3, // Below creatures and player
            );

            // Select enemy based on current wave
            let enemy_id = select_enemy_for_wave(game_state.current_wave);
            spawn_enemy(&mut commands, &game_data, enemy_id, spawn_pos);
        }
    }
}

/// System to handle creature respawns from the respawn queue
pub fn respawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut respawn_queue: ResMut<RespawnQueue>,
    game_data: Res<GameData>,
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
            if spawn_creature(&mut commands, &game_data, &entry.creature_id, spawn_pos).is_some() {
                println!(
                    "{} respawned!",
                    entry.creature_id
                );
            }

            completed_indices.push(index);
        }
    }

    // Remove completed entries (in reverse order to preserve indices)
    for index in completed_indices.into_iter().rev() {
        respawn_queue.entries.remove(index);
    }
}
