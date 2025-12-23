use bevy::prelude::*;

/// Resource holding handles to sprite animation assets
#[derive(Resource)]
pub struct DeathSprites {
    /// Handle to the unified imp sprite sheet (6 frames: idle, walk1, walk2, death1, death2, death3)
    pub imp_spritesheet: Handle<Image>,
    /// Handle to the blood splatters sprite sheet image
    pub blood_splatters: Handle<Image>,
    /// Texture atlas layout for imp spritesheet (6 frames, 64x80 each at 2x export)
    pub imp_atlas: Handle<TextureAtlasLayout>,
    /// Texture atlas layout for blood splatters (4 variants, 32x32 each at 2x export)
    pub blood_atlas: Handle<TextureAtlasLayout>,
}
