use bevy::prelude::*;

mod components;
mod data;
mod resources;
mod systems;

use components::{Player, Velocity};
use resources::{load_game_data, DeckCard, GameData, GameState, PlayerDeck};
use systems::{
    apply_velocity_system, camera_follow_system, creature_attack_system, creature_death_system,
    creature_follow_system, death_effect_system, enemy_attack_system, enemy_chase_system,
    enemy_death_system, enemy_spawn_system, level_check_system, level_up_effect_system,
    player_movement_system, projectile_system, respawn_system, spawn_hp_bars_system,
    spawn_test_creature_system, spawn_ui_system, update_hp_bars_system, update_ui_system,
    EnemySpawnTimer, RespawnQueue,
};

fn main() {
    // Load game data before starting Bevy
    let game_data = match load_game_data() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to load game data: {}", e);
            std::process::exit(1);
        }
    };

    // Print loaded data counts
    println!(
        "Loaded {} creatures, {} weapons, {} artifacts, {} enemies, {} affinity colors",
        game_data.creatures.len(),
        game_data.weapons.len(),
        game_data.artifacts.len(),
        game_data.enemies.len(),
        game_data.affinity_colors.len()
    );

    // Create starter deck
    let starter_deck = PlayerDeck::new(vec![
        DeckCard::creature("fire_imp", 25.0),
        DeckCard::creature("ember_hound", 15.0),
        DeckCard::weapon("ember_staff", 15.0),
        DeckCard::artifact("molten_core", 10.0),
    ]);
    println!(
        "Starter deck: {} cards, total weight: {}",
        starter_deck.cards.len(),
        starter_deck.total_weight
    );

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bloodtide".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(game_data)
        .insert_resource(starter_deck)
        .init_resource::<EnemySpawnTimer>()
        .init_resource::<GameState>()
        .init_resource::<RespawnQueue>()
        .add_systems(Startup, (setup, spawn_ui_system))
        .add_systems(Update, (
            player_movement_system,
            spawn_test_creature_system,
            enemy_spawn_system,
            respawn_system,
            creature_follow_system,
            enemy_chase_system,
            creature_attack_system,
            enemy_attack_system,
            apply_velocity_system,
            projectile_system,
            enemy_death_system,
            creature_death_system,
            death_effect_system,
            spawn_hp_bars_system,
            update_hp_bars_system,
            level_check_system,
            level_up_effect_system,
            update_ui_system,
            camera_follow_system,
        ).chain())
        .run();
}

fn setup(mut commands: Commands, game_data: Res<GameData>) {
    // Log game data loaded
    println!(
        "Game initialized with {} creatures, {} weapons, {} artifacts, {} enemies",
        game_data.creatures.len(),
        game_data.weapons.len(),
        game_data.artifacts.len(),
        game_data.enemies.len()
    );

    // Spawn camera
    commands.spawn(Camera2d);

    // Spawn background grid for visual reference
    let grid_size = 100.0; // Size of each grid cell
    let grid_count = 40; // Number of cells in each direction from center
    let dark_color = Color::srgb(0.1, 0.1, 0.15);
    let light_color = Color::srgb(0.15, 0.15, 0.2);

    for x in -grid_count..=grid_count {
        for y in -grid_count..=grid_count {
            let is_dark = (x + y) % 2 == 0;
            let color = if is_dark { dark_color } else { light_color };

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(grid_size, grid_size)),
                    ..default()
                },
                Transform::from_xyz(
                    x as f32 * grid_size,
                    y as f32 * grid_size,
                    -1.0, // Behind player
                ),
            ));
        }
    }

    // Spawn origin marker (red cross) for reference
    // Horizontal bar
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.1, 0.1),
            custom_size: Some(Vec2::new(200.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -0.5),
    ));
    // Vertical bar
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.1, 0.1),
            custom_size: Some(Vec2::new(10.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -0.5),
    ));

    // Spawn player (white square, 48x48 pixels, centered)
    commands.spawn((
        Player,
        Velocity::default(),
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(48.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0), // Above background
    ));

    println!("Player spawned at origin. Use WASD or arrow keys to move.");
    println!("Press SPACE to spawn Fire Imps!");
    println!("Goblins spawn every 1.5 seconds and chase you!");
    println!("Fire Imps auto-attack nearby goblins!");
}
