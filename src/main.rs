use bevy::prelude::*;

mod components;
mod data;
mod math;
mod resources;
mod systems;

use components::{Player, Velocity};
use resources::{load_game_data, AffinityState, ArtifactBuffs, DeckCard, GameData, GameState, PlayerDeck};
use systems::{
    apply_velocity_system, camera_follow_system, creature_attack_system, creature_death_system,
    creature_evolution_system, creature_follow_system, creature_level_up_effect_system,
    creature_xp_system, damage_number_system, death_effect_system, enemy_attack_system,
    enemy_chase_system, enemy_death_system, enemy_spawn_system, evolution_effect_system,
    level_check_system, level_up_effect_system, player_movement_system, projectile_system,
    respawn_system, screen_shake_system, spawn_hp_bars_system, spawn_test_creature_system,
    spawn_ui_system, update_hp_bars_system, update_ui_system, weapon_attack_system,
    EnemySpawnTimer, RespawnQueue, ScreenShake, EvolutionReadyState,
    // UI Panel systems
    spawn_creature_panel_system, update_creature_panel_system,
    spawn_artifact_panel_system, update_artifact_panel_system,
    spawn_affinity_display_system, update_affinity_display_system,
    show_card_roll_popup_system, card_roll_popup_update_system,
    show_wave_announcement_system, wave_announcement_update_system,
    CardRollState, WaveAnnouncementState, DamageNumberOffsets,
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
        .init_resource::<ScreenShake>()
        .init_resource::<ArtifactBuffs>()
        .init_resource::<AffinityState>()
        .init_resource::<CardRollState>()
        .init_resource::<WaveAnnouncementState>()
        .init_resource::<DamageNumberOffsets>()
        .init_resource::<EvolutionReadyState>()
        .add_systems(Startup, (
            setup,
            spawn_ui_system,
            spawn_creature_panel_system,
            spawn_artifact_panel_system,
            spawn_affinity_display_system,
        ))
        // Input and spawning systems
        .add_systems(Update, (
            player_movement_system,
            spawn_test_creature_system,
            enemy_spawn_system,
            respawn_system,
        ).chain())
        // AI and movement systems
        .add_systems(Update, (
            creature_follow_system,
            enemy_chase_system,
            apply_velocity_system,
        ).chain().after(player_movement_system))
        // Combat systems
        .add_systems(Update, (
            creature_attack_system,
            enemy_attack_system,
            weapon_attack_system,
            projectile_system,
            damage_number_system,
        ).chain().after(apply_velocity_system))
        // Death and effects systems
        .add_systems(Update, (
            enemy_death_system,
            creature_death_system,
            death_effect_system,
        ).chain().after(projectile_system))
        // Creature XP and evolution
        .add_systems(Update, (
            creature_xp_system,
            creature_level_up_effect_system,
            creature_evolution_system,
            evolution_effect_system,
        ).chain().after(enemy_death_system))
        // HP bars and leveling
        .add_systems(Update, (
            spawn_hp_bars_system,
            update_hp_bars_system,
            level_check_system,
            level_up_effect_system,
        ).chain().after(creature_xp_system))
        // UI panel updates
        .add_systems(Update, (
            update_creature_panel_system,
            update_artifact_panel_system,
            update_affinity_display_system,
            show_card_roll_popup_system,
            card_roll_popup_update_system,
            show_wave_announcement_system,
            wave_announcement_update_system,
        ).after(level_up_effect_system))
        // UI and camera (run last)
        .add_systems(Update, (
            update_ui_system,
            camera_follow_system,
            screen_shake_system,
        ).chain().after(update_creature_panel_system))
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
