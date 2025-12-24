use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod components;
mod data;
mod math;
mod resources;
mod systems;

use components::{Player, PlayerStats, PlayerAnimation, Velocity};
use resources::{load_game_data, AffinityState, ArtifactBuffs, CreatureSprites, DeathSprites, PlayerSprites, DebugSettings, Director, GameData, GameState, GameOverState, GamePhase, PlayerDeck, DeckBuilderState, SpatialGrid, ProjectilePool, DamageNumberPool, ChunkManager};
use systems::{
    apply_velocity_system, camera_follow_system, creature_attack_system, creature_death_animation_system, creature_death_system,
    creature_evolution_system, creature_follow_system, creature_level_up_effect_system,
    creature_xp_system, damage_number_system, death_animation_system, death_effect_system,
    blood_cleanup_system, creature_animation_system, enemy_animation_system, enemy_attack_system,
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
    // Pooling systems
    init_pools_system, init_pools_if_empty_system,
    // Deck builder systems
    spawn_deck_builder_system, deck_builder_visibility_system, deck_builder_update_cards_system,
    deck_builder_available_cards_system, deck_builder_tab_system, deck_builder_button_system,
    deck_builder_add_card_system, deck_builder_start_run_system, deck_builder_clear_deck_system,
    deck_builder_footer_system, deck_builder_weapon_select_system,
    // Tilemap systems
    load_tilemap_assets, chunk_loading_system,
    // Player systems
    player_animation_system,
    enemy_contact_damage_system, enemy_attack_player_system,
    spawn_player_hp_bar_system, update_player_hp_bar_system,
    update_player_hp_hud_system,
    player_death_system, player_death_animation_system,
    // Game over systems
    spawn_game_over_ui_system, game_over_visibility_system,
    game_over_restart_button_system, game_over_deck_builder_button_system,
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

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bloodtide".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TilemapPlugin)
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
        .init_resource::<ChunkManager>()
        .init_resource::<GameOverState>()
        .add_systems(Startup, (
            setup,
            spawn_ui_system,
            spawn_creature_panel_system,
            spawn_artifact_panel_system,
            spawn_affinity_display_system,
            spawn_debug_menu_system,
            spawn_pause_menu_system,
            spawn_deck_builder_system,
            spawn_game_over_ui_system,
            init_pools_system,
            load_death_sprites,
            load_creature_sprites,
            load_player_sprites,
            load_tilemap_assets,
        ))
        // Player sprite initialization (runs once when sprites are loaded)
        .add_systems(Update, init_player_sprite_system)
        // Director update (runs early)
        .add_systems(Update, director_update_system)
        // Tilemap chunk loading (runs early, based on player position)
        .add_systems(Update, chunk_loading_system.after(director_update_system))
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
            enemy_animation_system,    // Update enemy sprite animations based on velocity
            creature_animation_system, // Update creature sprite animations based on velocity
            player_animation_system,   // Update player sprite animations based on velocity
        ).chain().after(player_movement_system))
        // Pool re-initialization (needed after game restart)
        .add_systems(Update, init_pools_if_empty_system.after(apply_velocity_system))
        // Combat systems (spatial grid updates first for efficient enemy lookups)
        .add_systems(Update, (
            update_spatial_grid_system,
            creature_attack_system,
            enemy_attack_system,
            enemy_attack_player_system,  // Enemies attack player
            enemy_contact_damage_system, // Contact damage to player
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
            creature_death_animation_system,
            player_death_system,           // Check for player death
            player_death_animation_system, // Animate player death
            death_effect_system,
            death_animation_system,
            blood_cleanup_system,
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
            spawn_player_hp_bar_system,    // Player HP bar above head
            update_player_hp_bar_system,   // Update player HP bar
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
            update_player_hp_hud_system,  // Player HP in HUD
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
            deck_builder_weapon_select_system,
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
        // Game over UI systems
        .add_systems(Update, (
            game_over_visibility_system,
            game_over_restart_button_system,
            game_over_deck_builder_button_system,
        ).after(player_death_animation_system))
        .run();
}

/// Load sprite animation assets and create texture atlases
fn load_death_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load the sprite images
    // goblin_spritesheet.png: 384x80 (6 frames at 64x80 each, exported at 2x from 32x40 SVG)
    let goblin_spritesheet: Handle<Image> = asset_server.load("sprites/enemies/goblin_spritesheet.png");
    // blood_splatters.png: 128x32 (4 variants at 32x32 each, exported at 2x from 16x16 SVG)
    let blood_splatters: Handle<Image> = asset_server.load("sprites/effects/blood_splatters.png");

    // Create texture atlas layouts
    // goblin_spritesheet: 6 frames (idle, walk1, walk2, death1, death2, death3) at 64x80 each
    let goblin_layout = TextureAtlasLayout::from_grid(UVec2::new(64, 80), 6, 1, None, None);
    let goblin_atlas = texture_atlas_layouts.add(goblin_layout);

    // blood_splatters: 4 variants at 32x32 each
    let blood_layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 4, 1, None, None);
    let blood_atlas = texture_atlas_layouts.add(blood_layout);

    // Insert the resource
    commands.insert_resource(DeathSprites {
        goblin_spritesheet,
        blood_splatters,
        goblin_atlas,
        blood_atlas,
    });
}

/// Load creature sprite animation assets and create texture atlases
fn load_creature_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Fire creature evolution line - all have 8 frames (idle, turn, walk1-2, death1-4)
    // Sprites are exported at 2x resolution

    // Fire Imp (Tier 1): 64x80 per frame (128x160 at 2x)
    let fire_imp_spritesheet: Handle<Image> = asset_server.load("sprites/creatures/fire_imp_spritesheet.png");
    let fire_imp_layout = TextureAtlasLayout::from_grid(UVec2::new(128, 160), 8, 1, None, None);
    let fire_imp_atlas = texture_atlas_layouts.add(fire_imp_layout);

    // Flame Fiend (Tier 2): 64x96 per frame (128x192 at 2x)
    let flame_fiend_spritesheet: Handle<Image> = asset_server.load("sprites/creatures/flame_fiend_spritesheet.png");
    let flame_fiend_layout = TextureAtlasLayout::from_grid(UVec2::new(128, 192), 8, 1, None, None);
    let flame_fiend_atlas = texture_atlas_layouts.add(flame_fiend_layout);

    // Inferno Demon (Tier 3): 64x112 per frame (128x224 at 2x)
    let inferno_demon_spritesheet: Handle<Image> = asset_server.load("sprites/creatures/inferno_demon_spritesheet.png");
    let inferno_demon_layout = TextureAtlasLayout::from_grid(UVec2::new(128, 224), 8, 1, None, None);
    let inferno_demon_atlas = texture_atlas_layouts.add(inferno_demon_layout);

    // Flame projectile sprite
    let flame_projectile: Handle<Image> = asset_server.load("sprites/projectiles/flame_small.png");

    commands.insert_resource(CreatureSprites {
        fire_imp_spritesheet,
        fire_imp_atlas,
        flame_fiend_spritesheet,
        flame_fiend_atlas,
        inferno_demon_spritesheet,
        inferno_demon_atlas,
        flame_projectile,
    });
}

/// Load player sprite animation assets and create texture atlases
fn load_player_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Wizard player: 80x128 per frame, 6 frames (idle, walk1, walk2, death1, death2, death3)
    // Total spritesheet: 480x128
    let wizard_spritesheet: Handle<Image> = asset_server.load("sprites/creatures/wizard_player_spritesheet.png");
    let wizard_layout = TextureAtlasLayout::from_grid(UVec2::new(80, 128), 6, 1, None, None);
    let wizard_atlas = texture_atlas_layouts.add(wizard_layout);

    commands.insert_resource(PlayerSprites {
        wizard_spritesheet,
        wizard_atlas,
    });
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Ground tilemap is now handled by chunk_loading_system

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

    // Spawn player with stats and animation (sprite added by init_player_sprite_system)
    commands.spawn((
        Player,
        PlayerStats::default(),
        PlayerAnimation::new(),
        Velocity::default(),
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(48.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0), // Above background
    ));
}

/// Marker component indicating player sprite has been initialized
#[derive(Component)]
struct PlayerSpriteInitialized;

/// Initialize player sprite when PlayerSprites resource is available
/// This runs once to replace the placeholder sprite with the actual wizard sprite
fn init_player_sprite_system(
    player_sprites: Option<Res<PlayerSprites>>,
    mut player_query: Query<(Entity, &mut Sprite), (With<Player>, Without<PlayerSpriteInitialized>)>,
    mut commands: Commands,
) {
    // Only run if sprites are loaded and player doesn't have the marker yet
    let Some(sprites) = player_sprites else { return };

    for (entity, mut sprite) in player_query.iter_mut() {
        // Add the sprite atlas to the player
        sprite.image = sprites.wizard_spritesheet.clone();
        sprite.custom_size = None; // Use actual sprite size
        sprite.texture_atlas = Some(bevy::sprite::TextureAtlas {
            layout: sprites.wizard_atlas.clone(),
            index: 0, // Idle frame
        });
        sprite.color = Color::WHITE; // Reset to white (no tint)

        // Scale player sprite (80x128 is quite large, scale to 0.5) and mark as initialized
        commands.entity(entity)
            .insert(Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(0.5)))
            .insert(PlayerSpriteInitialized);
    }
}
