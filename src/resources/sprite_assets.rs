use bevy::prelude::*;

/// Resource holding handles to sprite animation assets
#[derive(Resource)]
pub struct DeathSprites {
    /// Handle to the unified goblin sprite sheet (6 frames: idle, walk1, walk2, death1, death2, death3)
    pub goblin_spritesheet: Handle<Image>,
    /// Handle to the blood splatters sprite sheet image
    pub blood_splatters: Handle<Image>,
    /// Texture atlas layout for goblin spritesheet (6 frames, 64x80 each at 2x export)
    pub goblin_atlas: Handle<TextureAtlasLayout>,
    /// Texture atlas layout for blood splatters (4 variants, 32x32 each at 2x export)
    pub blood_atlas: Handle<TextureAtlasLayout>,
}

/// Resource holding handles to creature sprite assets
///
/// Fire creature evolution line:
/// - Fire Imp (T1): 64x80 per frame
/// - Flame Fiend (T2): 64x96 per frame
/// - Inferno Demon (T3): 64x112 per frame
///
/// All use 8 frames: idle, turn, walk1, walk2, death1-4
#[derive(Resource)]
pub struct CreatureSprites {
    // Fire Imp (Tier 1)
    /// Handle to the Fire Imp sprite sheet (8 frames, 64x80 each)
    pub fire_imp_spritesheet: Handle<Image>,
    /// Texture atlas layout for Fire Imp
    pub fire_imp_atlas: Handle<TextureAtlasLayout>,

    // Flame Fiend (Tier 2)
    /// Handle to the Flame Fiend sprite sheet (8 frames, 64x96 each)
    pub flame_fiend_spritesheet: Handle<Image>,
    /// Texture atlas layout for Flame Fiend
    pub flame_fiend_atlas: Handle<TextureAtlasLayout>,

    // Inferno Demon (Tier 3)
    /// Handle to the Inferno Demon sprite sheet (8 frames, 64x112 each)
    pub inferno_demon_spritesheet: Handle<Image>,
    /// Texture atlas layout for Inferno Demon
    pub inferno_demon_atlas: Handle<TextureAtlasLayout>,

    // Projectile
    /// Handle to the flame projectile sprite
    pub flame_projectile: Handle<Image>,
}
