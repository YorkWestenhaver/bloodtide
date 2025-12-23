use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Chunk size in tiles (16x16 tiles per chunk)
pub const CHUNK_SIZE: u32 = 16;

/// Tile size in pixels
pub const TILE_SIZE: f32 = 32.0;

/// Detail sprite size in pixels
pub const DETAIL_SIZE: f32 = 16.0;

/// Load chunks within this radius of the player (in chunks)
pub const LOAD_RADIUS: i32 = 4;

/// Unload chunks beyond this radius (in chunks)
pub const UNLOAD_RADIUS: i32 = 6;

/// Z-position for ground tilemap
pub const GROUND_Z: f32 = -10.0;

/// Z-position for detail overlays
pub const DETAIL_Z: f32 = -9.0;

/// Biome types matching tileset_schema.json
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Dirt,
    Rocky,
    Gravel,
    Stone,
    Battlefield,
    Fire,
}

impl BiomeType {
    /// Get base tile indices for this biome
    pub fn base_tiles(&self) -> &[u32] {
        match self {
            BiomeType::Dirt => &[0, 1, 2, 3],
            BiomeType::Rocky => &[8, 9],
            BiomeType::Gravel => &[10, 11],
            BiomeType::Stone => &[12],
            BiomeType::Battlefield => &[0, 1, 2],
            BiomeType::Fire => &[2, 3],
        }
    }

    /// Get accent tile indices for this biome
    pub fn accent_tiles(&self) -> &[u32] {
        match self {
            BiomeType::Dirt => &[4, 5, 6, 7],
            BiomeType::Rocky => &[10, 11],
            BiomeType::Gravel => &[8, 9],
            BiomeType::Stone => &[13],
            BiomeType::Battlefield => &[14],
            BiomeType::Fire => &[15],
        }
    }

    /// Chance of accent tile appearing (0.0 - 1.0)
    pub fn accent_chance(&self) -> f32 {
        match self {
            BiomeType::Dirt => 0.15,
            BiomeType::Rocky => 0.20,
            BiomeType::Gravel => 0.10,
            BiomeType::Stone => 0.25,
            BiomeType::Battlefield => 0.30,
            BiomeType::Fire => 0.40,
        }
    }

    /// Get detail sprite categories for this biome
    pub fn detail_categories(&self) -> &[DetailCategory] {
        match self {
            BiomeType::Dirt => &[DetailCategory::Rocks, DetailCategory::Cracks, DetailCategory::Vegetation],
            BiomeType::Rocky => &[DetailCategory::Rocks, DetailCategory::Cracks],
            BiomeType::Gravel => &[DetailCategory::Rocks, DetailCategory::Terrain],
            BiomeType::Stone => &[DetailCategory::Rocks, DetailCategory::Cracks],
            BiomeType::Battlefield => &[DetailCategory::Bones, DetailCategory::Blood, DetailCategory::Rocks],
            BiomeType::Fire => &[DetailCategory::Fire, DetailCategory::Cracks],
        }
    }
}

/// Detail sprite categories from tileset_schema.json
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailCategory {
    Rocks,
    Terrain,
    Bones,
    Cracks,
    Blood,
    Fire,
    Vegetation,
}

impl DetailCategory {
    /// Get sprite indices for this category (row 0 = 0-7, row 1 = 8-15)
    pub fn sprite_indices(&self) -> &[u32] {
        match self {
            DetailCategory::Rocks => &[0, 1, 2, 3, 4],
            DetailCategory::Terrain => &[5, 6, 7],
            DetailCategory::Bones => &[8, 9, 10],
            DetailCategory::Cracks => &[11, 12],
            DetailCategory::Blood => &[13],
            DetailCategory::Fire => &[14],
            DetailCategory::Vegetation => &[15],
        }
    }
}

/// Marker component for ground chunk entities
#[derive(Component)]
pub struct GroundChunk {
    pub chunk_coords: (i32, i32),
}

/// Marker component for detail overlay sprites
#[derive(Component)]
pub struct DetailOverlay {
    pub chunk_coords: (i32, i32),
}

/// Holds loaded tilemap texture assets
#[derive(Resource)]
pub struct TilemapAssets {
    pub ground_tileset: Handle<Image>,
    pub ground_atlas: Handle<TextureAtlasLayout>,
    pub detail_tileset: Handle<Image>,
    pub detail_atlas: Handle<TextureAtlasLayout>,
}

/// Manages which chunks are loaded and world generation seed
#[derive(Resource)]
pub struct ChunkManager {
    /// Set of currently loaded chunk coordinates
    pub loaded_chunks: HashSet<(i32, i32)>,
    /// Entity ID for each loaded chunk (for despawning)
    pub chunk_entities: HashMap<(i32, i32), Entity>,
    /// World seed for reproducible generation
    pub seed: u64,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            loaded_chunks: HashSet::new(),
            chunk_entities: HashMap::new(),
            seed: 42, // Default seed, can be randomized per run
        }
    }
}

impl ChunkManager {
    /// Convert world position to chunk coordinates
    pub fn world_to_chunk(world_pos: Vec2) -> (i32, i32) {
        let chunk_world_size = CHUNK_SIZE as f32 * TILE_SIZE;
        (
            (world_pos.x / chunk_world_size).floor() as i32,
            (world_pos.y / chunk_world_size).floor() as i32,
        )
    }

    /// Convert chunk coordinates to world position (chunk origin)
    pub fn chunk_to_world(chunk_coords: (i32, i32)) -> Vec2 {
        let chunk_world_size = CHUNK_SIZE as f32 * TILE_SIZE;
        Vec2::new(
            chunk_coords.0 as f32 * chunk_world_size,
            chunk_coords.1 as f32 * chunk_world_size,
        )
    }

    /// Get chunks that should be loaded around a position
    pub fn chunks_in_load_radius(center_chunk: (i32, i32)) -> Vec<(i32, i32)> {
        let mut chunks = Vec::new();
        for dx in -LOAD_RADIUS..=LOAD_RADIUS {
            for dy in -LOAD_RADIUS..=LOAD_RADIUS {
                chunks.push((center_chunk.0 + dx, center_chunk.1 + dy));
            }
        }
        chunks
    }

    /// Check if a chunk is beyond unload radius from center
    pub fn is_beyond_unload_radius(chunk: (i32, i32), center_chunk: (i32, i32)) -> bool {
        let dx = (chunk.0 - center_chunk.0).abs();
        let dy = (chunk.1 - center_chunk.1).abs();
        dx > UNLOAD_RADIUS || dy > UNLOAD_RADIUS
    }
}
