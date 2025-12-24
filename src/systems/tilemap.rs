use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use noise::{NoiseFn, Perlin, Fbm, MultiFractal};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::components::Player;
use crate::resources::{
    BiomeType, ChunkManager, DetailCategory, DetailOverlay, GroundChunk,
    TilemapAssets, CHUNK_SIZE, DETAIL_SIZE, DETAIL_Z, GROUND_Z, TILE_SIZE,
};

/// Noise frequency for biome generation (lower = larger features)
const BIOME_FREQUENCY: f64 = 0.035;

/// Domain warping frequency (creates swirl patterns)
const WARP_FREQUENCY: f64 = 0.02;

/// Domain warping strength in tiles
const WARP_STRENGTH: f64 = 25.0;

/// Density of detail overlay sprites (0.0 - 1.0)
const DETAIL_DENSITY: f64 = 0.05;

/// Load tilemap assets on startup
pub fn load_tilemap_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Load ground tileset (128x128, 4x4 tiles of 32x32 each)
    let ground_tileset = asset_server.load("sprites/tiles/ground_tileset.png");
    let ground_layout = TextureAtlasLayout::from_grid(
        UVec2::new(32, 32),
        4, 4,
        None, None,
    );
    let ground_atlas = texture_atlas_layouts.add(ground_layout);

    // Load detail tileset (128x32, 8x2 sprites of 16x16 each)
    let detail_tileset = asset_server.load("sprites/tiles/ground_details.png");
    let detail_layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        8, 2,
        None, None,
    );
    let detail_atlas = texture_atlas_layouts.add(detail_layout);

    commands.insert_resource(TilemapAssets {
        ground_tileset,
        ground_atlas,
        detail_tileset,
        detail_atlas,
    });
}

/// System to load/unload chunks based on player position
pub fn chunk_loading_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut chunk_manager: ResMut<ChunkManager>,
    tilemap_assets: Option<Res<TilemapAssets>>,
    chunk_query: Query<Entity, With<GroundChunk>>,
    detail_query: Query<(Entity, &DetailOverlay)>,
) {
    // Wait for assets to be loaded
    let Some(assets) = tilemap_assets else {
        return;
    };

    // Get player position
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let player_chunk = ChunkManager::world_to_chunk(player_pos);

    // Get chunks that should be loaded
    let chunks_to_load = ChunkManager::chunks_in_load_radius(player_chunk);

    // Load missing chunks
    for chunk_coords in chunks_to_load {
        if !chunk_manager.loaded_chunks.contains(&chunk_coords) {
            spawn_chunk(&mut commands, &assets, &mut chunk_manager, chunk_coords);
        }
    }

    // Unload distant chunks
    let chunks_to_unload: Vec<(i32, i32)> = chunk_manager
        .loaded_chunks
        .iter()
        .filter(|&&chunk| ChunkManager::is_beyond_unload_radius(chunk, player_chunk))
        .copied()
        .collect();

    for chunk_coords in chunks_to_unload {
        despawn_chunk(&mut commands, &mut chunk_manager, chunk_coords, &detail_query);
    }
}

/// Spawn a chunk at the given coordinates
fn spawn_chunk(
    commands: &mut Commands,
    assets: &TilemapAssets,
    chunk_manager: &mut ChunkManager,
    chunk_coords: (i32, i32),
) {
    let seed = chunk_manager.seed;
    let chunk_world_pos = ChunkManager::chunk_to_world(chunk_coords);

    // Create noise generators with seed
    let elevation_noise = create_fbm_noise(seed);
    let moisture_noise = create_fbm_noise(seed.wrapping_add(12345));
    let warp_noise_x = Perlin::new(seed.wrapping_add(54321) as u32);
    let warp_noise_y = Perlin::new(seed.wrapping_add(98765) as u32);

    // Create tilemap
    let map_size = TilemapSize { x: CHUNK_SIZE, y: CHUNK_SIZE };
    let tile_size = TilemapTileSize { x: TILE_SIZE, y: TILE_SIZE };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    // Spawn tilemap entity first
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    // Position-seeded RNG for tile variation
    let mut rng = create_position_rng(seed, chunk_coords);

    // Generate tiles for this chunk
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            // World tile position
            let world_tile_x = chunk_coords.0 * CHUNK_SIZE as i32 + x as i32;
            let world_tile_y = chunk_coords.1 * CHUNK_SIZE as i32 + y as i32;

            // Sample noise with domain warping for swirl patterns
            let warp_x = warp_noise_x.get([
                world_tile_x as f64 * WARP_FREQUENCY,
                world_tile_y as f64 * WARP_FREQUENCY,
            ]) * WARP_STRENGTH;
            let warp_y = warp_noise_y.get([
                world_tile_x as f64 * WARP_FREQUENCY,
                world_tile_y as f64 * WARP_FREQUENCY,
            ]) * WARP_STRENGTH;

            let sample_x = (world_tile_x as f64 + warp_x) * BIOME_FREQUENCY;
            let sample_y = (world_tile_y as f64 + warp_y) * BIOME_FREQUENCY;

            let elevation = elevation_noise.get([sample_x, sample_y]);
            let moisture = moisture_noise.get([sample_x * 1.3, sample_y * 1.3]);

            // Determine biome from elevation + moisture
            let biome = get_biome(elevation, moisture);

            // Pick tile index
            let tile_index = pick_tile_index(&mut rng, biome);

            // Spawn tile entity
            let tile_pos = TilePos { x, y };
            let tile_entity = commands.spawn(TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(tile_index),
                tilemap_id: TilemapId(tilemap_entity),
                ..default()
            }).id();

            tile_storage.set(&tile_pos, tile_entity);

            // Maybe spawn detail overlay
            if rng.gen::<f64>() < DETAIL_DENSITY {
                spawn_detail_overlay(
                    commands,
                    assets,
                    chunk_coords,
                    world_tile_x,
                    world_tile_y,
                    biome,
                    &mut rng,
                );
            }
        }
    }

    // Insert tilemap bundle
    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(assets.ground_tileset.clone()),
            tile_size,
            transform: Transform::from_translation(Vec3::new(
                chunk_world_pos.x,
                chunk_world_pos.y,
                GROUND_Z,
            )),
            ..default()
        },
        GroundChunk { chunk_coords },
    ));

    chunk_manager.loaded_chunks.insert(chunk_coords);
    chunk_manager.chunk_entities.insert(chunk_coords, tilemap_entity);
}

/// Despawn a chunk and its details
fn despawn_chunk(
    commands: &mut Commands,
    chunk_manager: &mut ChunkManager,
    chunk_coords: (i32, i32),
    detail_query: &Query<(Entity, &DetailOverlay)>,
) {
    // Despawn tilemap entity
    if let Some(entity) = chunk_manager.chunk_entities.remove(&chunk_coords) {
        commands.entity(entity).despawn_recursive();
    }

    // Despawn detail overlays for this chunk
    for (entity, detail) in detail_query.iter() {
        if detail.chunk_coords == chunk_coords {
            commands.entity(entity).despawn();
        }
    }

    chunk_manager.loaded_chunks.remove(&chunk_coords);
}

/// Spawn a detail overlay sprite
fn spawn_detail_overlay(
    commands: &mut Commands,
    assets: &TilemapAssets,
    chunk_coords: (i32, i32),
    world_tile_x: i32,
    world_tile_y: i32,
    biome: BiomeType,
    rng: &mut StdRng,
) {
    // Pick a detail category appropriate for this biome
    let categories = biome.detail_categories();
    if categories.is_empty() {
        return;
    }

    let category = categories[rng.gen_range(0..categories.len())];
    let sprite_indices = category.sprite_indices();
    if sprite_indices.is_empty() {
        return;
    }

    let sprite_index = sprite_indices[rng.gen_range(0..sprite_indices.len())];

    // Random offset within tile for organic placement
    let offset_x = rng.gen_range(-8.0..8.0);
    let offset_y = rng.gen_range(-8.0..8.0);

    let world_x = world_tile_x as f32 * TILE_SIZE + TILE_SIZE / 2.0 + offset_x;
    let world_y = world_tile_y as f32 * TILE_SIZE + TILE_SIZE / 2.0 + offset_y;

    commands.spawn((
        Sprite::from_atlas_image(
            assets.detail_tileset.clone(),
            TextureAtlas {
                layout: assets.detail_atlas.clone(),
                index: sprite_index as usize,
            },
        ),
        Transform::from_translation(Vec3::new(world_x, world_y, DETAIL_Z)),
        DetailOverlay { chunk_coords },
    ));
}

/// Create an Fbm noise generator for terrain
fn create_fbm_noise(seed: u64) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(seed as u32)
        .set_octaves(4)
        .set_frequency(1.0)
        .set_lacunarity(2.0)
        .set_persistence(0.5)
}

/// Create a position-seeded RNG for reproducible generation
fn create_position_rng(seed: u64, chunk_coords: (i32, i32)) -> StdRng {
    // Combine seed with chunk position for unique but reproducible per-chunk randomness
    // Use wrapping operations throughout to handle negative coordinates safely
    let x_hash = (chunk_coords.0 as i64).wrapping_mul(73856093) as u64;
    let y_hash = (chunk_coords.1 as i64).wrapping_mul(19349663) as u64;
    let combined_seed = seed.wrapping_add(x_hash).wrapping_add(y_hash);
    StdRng::seed_from_u64(combined_seed)
}

/// Determine biome from elevation and moisture values
fn get_biome(elevation: f64, moisture: f64) -> BiomeType {
    // Noise returns roughly -1.0 to 1.0, normalize to 0.0 to 1.0
    let e = (elevation + 1.0) / 2.0;
    let m = (moisture + 1.0) / 2.0;

    // Biome lookup based on elevation and moisture
    if e > 0.7 {
        // High elevation: rocky or stone
        if m > 0.5 {
            BiomeType::Stone
        } else {
            BiomeType::Rocky
        }
    } else if e < 0.3 {
        // Low elevation: fire or battlefield
        if m < 0.4 {
            BiomeType::Fire
        } else {
            BiomeType::Battlefield
        }
    } else {
        // Middle elevation: dirt or gravel
        if m > 0.6 {
            BiomeType::Gravel
        } else {
            BiomeType::Dirt
        }
    }
}

/// Pick a tile index for the given biome
fn pick_tile_index(rng: &mut StdRng, biome: BiomeType) -> u32 {
    let roll: f32 = rng.gen();

    if roll < biome.accent_chance() {
        // Pick an accent tile
        let accents = biome.accent_tiles();
        accents[rng.gen_range(0..accents.len())]
    } else {
        // Pick a base tile
        let bases = biome.base_tiles();
        bases[rng.gen_range(0..bases.len())]
    }
}
