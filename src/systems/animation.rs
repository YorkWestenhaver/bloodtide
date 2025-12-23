use bevy::prelude::*;

use crate::components::{AnimationState, Enemy, SpriteAnimation, Velocity};
use crate::resources::DebugSettings;

/// Velocity threshold below which enemy is considered idle
const WALK_THRESHOLD: f32 = 10.0;

/// System that updates enemy sprite animations based on movement
pub fn enemy_animation_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<(&Velocity, &mut SpriteAnimation, &mut Sprite), With<Enemy>>,
) {
    // Don't animate if paused
    if debug_settings.is_paused() {
        return;
    }

    for (velocity, mut anim, mut sprite) in query.iter_mut() {
        // Skip dying enemies - they're handled by death_animation_system
        if anim.state == AnimationState::Dying {
            continue;
        }

        let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        let is_moving = speed > WALK_THRESHOLD;

        // State transitions
        match anim.state {
            AnimationState::Idle if is_moving => {
                anim.start_walking();
            }
            AnimationState::Walking if !is_moving => {
                anim.go_idle();
            }
            AnimationState::Walking => {
                // Advance walk animation
                anim.frame_timer.tick(time.delta());
                if anim.frame_timer.just_finished() {
                    // Toggle between frames 1 and 2
                    anim.current_frame = if anim.current_frame == 1 { 2 } else { 1 };
                }
            }
            _ => {}
        }

        // Update sprite atlas index
        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = anim.current_frame;
        }
    }
}
