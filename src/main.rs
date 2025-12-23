use bevy::prelude::*;

mod components;
mod data;
mod math;
mod resources;
mod systems;

use components::{Player, Velocity};
use resources::{load_game_data, AffinityState, ArtifactBuffs, DebugSettings, Director, GameData, GameState, GamePhase, PlayerDeck, DeckBuilderState, SpatialGrid, ProjectilePool, DamageNumberPool};
use systems::{
    apply_velocity_system, camera_follow_system, creature_attack_system, creature_death_system,
    creature_evolution_system, creature_follow_system, creature_level_up_effect_system,
    creature_xp_system, damage_number_system, death_effect_system, enemy_attack_system,
    enemy_chase_system, enemy_death_system, enemy_spawn_system, evolution_effect_system,
    level_check_system, level_up_effect_system, player_movement_system, projectile_system,
    respawn_system, screen_shake_system, spawn_hp_bars_system, spawn_test_creature_system,
    spawn_ui_system, update_hp_bars_system, update_level_labels_system, update_tier_borders_system,
    update_ui_system, weapon_attack_system,
    EnemySpawnTimer, RespawnQueue, ScreenShake, EvolutionReadyState,
    // Projectile type systems
    homing_projectile_system, piercing_rotation_system, explosion_effect_system, chain_effect_system,
    // Director systems
    director_update_system, enemy_cleanup_system,
    // UI Panel systems
    spawn_creature_panel_system, update_creature_panel_system,
    spawn_artifact_panel_system, update_artifact_panel_system,
    spawn_affinity_display_system, update_affinity_display_system, update_weapon_stats_display_system,
    show_card_roll_popup_system, card_roll_popup_update_system,
    show_wave_announcement_system, wave_announcement_update_system,
    CardRollState, WaveAnnouncementState, DamageNumberOffsets,
    // Tooltip systems
    tooltip_hover_system, tooltip_spawn_system, tooltip_position_system,
    tooltip_settings_change_system, TooltipState,
    // Debug menu systems
    spawn_debug_menu_system, spawn_pause_menu_system,
    debug_menu_input_system, debug_menu_animation_system, pause_menu_visibility_system,
    slider_interaction_system, slider_fill_update_system, slider_value_text_system,
    checkbox_interaction_system, checkbox_indicator_system, toggle_mode_checkbox_system,
    reset_button_system, resume_button_system, restart_button_system, quit_button_system,
    main_menu_button_system,
    evolution_keybind_capture_system, evolution_keybind_text_system,
    // Leveling systems (Phase 21E)
    card_roll_queue_system, screen_flash_system, level_up_text_system, level_up_particle_system,
    kill_rate_system, CardRollQueue,
    // Spatial grid system
    update_spatial_grid_system,
    // Pooling system
    init_pools_system,
    // Deck builder systems
    spawn_deck_builder_system, deck_builder_visibility_system, deck_builder_update_cards_system,
    deck_builder_available_cards_system, deck_builder_tab_system, deck_builder_button_system,
    deck_builder_add_card_system, deck_builder_start_run_system, deck_builder_clear_deck_system,
    deck_builder_footer_system,
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
        .init_resource::<PlayerDeck>()  // Empty deck, will be populated from DeckBuilder
        .init_resource::<DeckBuilderState>()  // Deck builder with default starter cards
        .init_resource::<GamePhase>()  // Starts in DeckBuilder phase
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
        .init_resource::<Director>()
        .init_resource::<DebugSettings>()
        .init_resource::<TooltipState>()
        .init_resource::<CardRollQueue>()
        .init_resource::<SpatialGrid>()
        .init_resource::<ProjectilePool>()
        .init_resource::<DamageNumberPool>()
        .add_systems(Startup, (
            setup,
            spawn_ui_system,
            spawn_creature_panel_system,
            spawn_artifact_panel_system,
            spawn_affinity_display_system,
            spawn_debug_menu_system,
            spawn_pause_menu_system,
            spawn_deck_builder_system,
            init_pools_system,
        ))
        // Director update (runs early)
        .add_systems(Update, director_update_system)
        // Input and spawning systems
        .add_systems(Update, (
            player_movement_system,
            spawn_test_creature_system,
            enemy_spawn_system,
            enemy_cleanup_system,
            respawn_system,
        ).chain().after(director_update_system))
        // AI and movement systems
        .add_systems(Update, (
            creature_follow_system,
            enemy_chase_system,
            apply_velocity_system,
        ).chain().after(player_movement_system))
        // Combat systems (spatial grid updates first for efficient enemy lookups)
        .add_systems(Update, (
            update_spatial_grid_system,
            creature_attack_system,
            enemy_attack_system,
            weapon_attack_system,
            homing_projectile_system,  // Run homing before projectile movement/collision
            projectile_system,
            piercing_rotation_system,  // Rotate piercing projectiles after collision
            explosion_effect_system,
            chain_effect_system,
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
        // HP bars, level labels, tier borders and leveling
        .add_systems(Update, (
            spawn_hp_bars_system,
            update_hp_bars_system,
            update_level_labels_system,
            update_tier_borders_system,
            level_check_system,
            level_up_effect_system,
            card_roll_queue_system,
            screen_flash_system,
            level_up_text_system,
            level_up_particle_system,
        ).chain().after(creature_xp_system))
        // UI panel updates
        .add_systems(Update, (
            update_creature_panel_system,
            update_artifact_panel_system,
            update_weapon_stats_display_system,
            update_affinity_display_system,
            show_card_roll_popup_system,
            card_roll_popup_update_system,
            show_wave_announcement_system,
            wave_announcement_update_system,
        ).after(level_up_effect_system))
        // UI and camera (run last)
        .add_systems(Update, (
            kill_rate_system,
            update_ui_system,
            camera_follow_system,
            screen_shake_system,
        ).chain().after(update_creature_panel_system))
        // Debug menu systems (run very early and always)
        .add_systems(Update, debug_menu_input_system.before(director_update_system))
        .add_systems(Update, (
            debug_menu_animation_system,
            pause_menu_visibility_system,
            slider_interaction_system,
            slider_fill_update_system,
            slider_value_text_system,
            checkbox_interaction_system,
            checkbox_indicator_system,
            toggle_mode_checkbox_system,
            reset_button_system,
            resume_button_system,
            restart_button_system,
            quit_button_system,
            main_menu_button_system,
            evolution_keybind_capture_system,
            evolution_keybind_text_system,
        ).after(debug_menu_input_system))
        // Deck builder systems (run early, before director)
        .add_systems(Update, (
            deck_builder_visibility_system,
            deck_builder_tab_system,
            deck_builder_button_system,
            deck_builder_add_card_system,
            deck_builder_start_run_system,
            deck_builder_clear_deck_system,
            deck_builder_update_cards_system,
            deck_builder_available_cards_system,
            deck_builder_footer_system,
        ).chain().before(director_update_system))
        // Tooltip systems (run after UI updates)
        .add_systems(Update, (
            tooltip_hover_system,
            tooltip_spawn_system,
            tooltip_position_system,
            tooltip_settings_change_system,
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
