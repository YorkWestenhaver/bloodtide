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
#[derive(Resource)]
pub struct CreatureSprites {
    /// Handle to the Fire Imp sprite sheet (8 frames: idle, walk1-4, death1-3)
    pub fire_imp_spritesheet: Handle<Image>,
    /// Texture atlas layout for Fire Imp (8 frames, 128x160 each at 2x export, logical 64x80)
    pub fire_imp_atlas: Handle<TextureAtlasLayout>,
    /// Handle to the flame projectile sprite
    pub flame_projectile: Handle<Image>,
}
