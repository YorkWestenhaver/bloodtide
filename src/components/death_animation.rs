use bevy::prelude::*;

/// Component for animated death sequences
/// Death animation uses frames 3-4-5 of the unified imp spritesheet:
/// - Frame 3: death1 - hit recoil, eyes flash white
/// - Frame 4: death2 - body splitting, blood spray
/// - Frame 5: death3 - corpse chunks in blood pool with guts
#[derive(Component)]
pub struct DeathAnimation {
    /// Total animation duration (3 frames at 120ms each = 360ms)
    pub timer: Timer,
    /// Current frame index in spritesheet (3-5 for death)
    pub current_frame: usize,
    /// Timer for advancing frames (120ms per frame)
    pub frame_timer: Timer,
    /// Enemy type for future extensibility (different animations per enemy)
    pub enemy_type: String,
    /// Position where blood should spawn when animation completes
    pub death_position: Vec3,
}

impl DeathAnimation {
    pub fn new(enemy_type: String, position: Vec3) -> Self {
        Self {
            // 3 frames at 0.12s each = 0.36s total
            timer: Timer::from_seconds(0.36, TimerMode::Once),
            current_frame: 3, // Start at frame 3 (death1)
            frame_timer: Timer::from_seconds(0.12, TimerMode::Repeating),
            enemy_type,
            death_position: position,
        }
    }
}

/// Component for blood splatter decals left on the ground
#[derive(Component)]
pub struct BloodSplatter {
    /// Lifetime timer (30 seconds, fades in last 15 seconds - 50% of lifetime)
    pub lifetime: Timer,
    /// Which splatter variant (0-3) for visual variety
    pub variant: usize,
}

impl BloodSplatter {
    pub fn new(variant: usize) -> Self {
        Self {
            lifetime: Timer::from_seconds(30.0, TimerMode::Once),
            variant,
        }
    }
}
