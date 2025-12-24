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

/// Resource holding handles to player sprite assets
///
/// Wizard player: 80x128 per frame, 6 frames total
/// - Frame 0: idle
/// - Frames 1-2: walk cycle
/// - Frames 3-5: death animation
#[derive(Resource)]
pub struct PlayerSprites {
    /// Handle to the wizard player sprite sheet (6 frames, 80x128 each)
    pub wizard_spritesheet: Handle<Image>,
    /// Texture atlas layout for wizard player
    pub wizard_atlas: Handle<TextureAtlasLayout>,
}

/// Resource holding handles to boss sprite assets
///
/// Goblin King: 128x192 per frame (64x96 at 2x export), 12 frames total
/// Animation layout:
/// - Frame 0: idle
/// - Frames 1-2: walk cycle
/// - Frames 3-4: charge attack (windup, dash)
/// - Frames 5-6: sword swipe (windup, strike)
/// - Frames 7-8: ground pound (windup, impact)
/// - Frames 9-11: death animation
#[derive(Resource)]
pub struct BossSprites {
    /// Handle to the Goblin King sprite sheet (12 frames, 128x192 each at 2x)
    pub goblin_king_spritesheet: Handle<Image>,
    /// Texture atlas layout for Goblin King
    pub goblin_king_atlas: Handle<TextureAtlasLayout>,
}
