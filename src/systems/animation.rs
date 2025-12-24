use bevy::prelude::*;

use crate::components::{
    AnimationState, Creature, CreatureAnimation, CreatureAnimationState, CreatureFacing,
    Enemy, Player, PlayerAnimation, PlayerAnimationState, SpriteAnimation, Velocity,
    // Boss animation
    GoblinKing, GoblinKingAnimation, GoblinKingAnimState, BossAttackState,
};
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

/// System that updates creature sprite animations based on movement
///
/// Animation frame layout (per fire_creatures_schema.json):
/// - Frame 0: Idle (front-facing)
/// - Frame 1: Turn (transition to side view, 100ms)
/// - Frame 2-3: Walk cycle (side view, 150ms each)
/// - Frame 4-7: Death animation (handled by death_animation_system)
///
/// State machine:
/// Idle → Turning (when movement starts)
/// Turning → Walking (after 100ms turn animation)
/// Walking → Idle (when movement stops)
pub fn creature_animation_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<(&Velocity, &mut CreatureAnimation, Option<&mut CreatureFacing>, &mut Sprite), With<Creature>>,
) {
    // Don't animate if paused
    if debug_settings.is_paused() {
        return;
    }

    for (velocity, mut anim, facing, mut sprite) in query.iter_mut() {
        // Skip dying/dead creatures - they're handled by death_animation_system
        if anim.state == CreatureAnimationState::Dying || anim.state == CreatureAnimationState::Dead {
            continue;
        }

        let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        let is_moving = speed > WALK_THRESHOLD;

        // Update facing direction based on velocity (for sprite flipping)
        if let Some(mut facing) = facing {
            if velocity.x.abs() > 1.0 {
                let new_facing = if velocity.x > 0.0 {
                    CreatureFacing::Right
                } else {
                    CreatureFacing::Left
                };
                if *facing != new_facing {
                    *facing = new_facing;
                }
            }

            // Apply sprite flip based on facing direction
            // Walk/turn animations face right by default, flip for left
            match anim.state {
                CreatureAnimationState::Walking | CreatureAnimationState::Turning => {
                    sprite.flip_x = *facing == CreatureFacing::Left;
                }
                _ => {
                    // Idle is front-facing, no flip needed
                    sprite.flip_x = false;
                }
            }
        }

        // State transitions and animation updates
        match anim.state {
            CreatureAnimationState::Idle if is_moving => {
                // Start turn transition before walking
                anim.start_turning();
            }
            CreatureAnimationState::Turning => {
                // Wait for turn animation to complete
                anim.frame_timer.tick(time.delta());
                if anim.frame_timer.finished() {
                    // Turn complete, start walking
                    anim.start_walking();
                }
            }
            CreatureAnimationState::Walking if !is_moving => {
                // Stop and return to idle
                anim.go_idle();
            }
            CreatureAnimationState::Walking => {
                // Advance walk animation through frames 2-3
                anim.frame_timer.tick(time.delta());
                if anim.frame_timer.just_finished() {
                    anim.advance_walk_frame();
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

/// System that updates player sprite animations based on movement
///
/// Animation frame layout (per wizard_player_schema.json):
/// - Frame 0: Idle (standing neutral)
/// - Frame 1-2: Walk cycle (150ms each)
/// - Frame 3-5: Death animation (180ms each, handled by death system)
///
/// State machine:
/// Idle → Walking (when movement starts)
/// Walking → Idle (when movement stops)
/// Any → Dying → Dead (when HP <= 0)
pub fn player_animation_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<(&Velocity, &mut PlayerAnimation, &mut Sprite), With<Player>>,
) {
    // Don't animate if paused
    if debug_settings.is_paused() {
        return;
    }

    for (velocity, mut anim, mut sprite) in query.iter_mut() {
        // Skip dying/dead players - they're handled by player_death_animation_system
        if anim.state == PlayerAnimationState::Dying || anim.state == PlayerAnimationState::Dead {
            continue;
        }

        let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        let is_moving = speed > WALK_THRESHOLD;

        // State transitions and animation updates
        match anim.state {
            PlayerAnimationState::Idle if is_moving => {
                anim.start_walking();
            }
            PlayerAnimationState::Walking if !is_moving => {
                anim.go_idle();
            }
            PlayerAnimationState::Walking => {
                // Advance walk animation through frames 1-2
                anim.frame_timer.tick(time.delta());
                if anim.frame_timer.just_finished() {
                    anim.advance_walk_frame();
                }
            }
            _ => {}
        }

        // Update sprite atlas index
        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = anim.current_frame;
        }

        // Flip sprite based on horizontal velocity
        if velocity.x.abs() > 1.0 {
            sprite.flip_x = velocity.x < 0.0;
        }
    }
}

/// System that updates Goblin King boss sprite animations
///
/// Animation frame layout (12 frames at 128x192 each):
/// - Frame 0: idle
/// - Frames 1-2: walk cycle
/// - Frames 3-4: charge attack (windup, dash)
/// - Frames 5-6: sword swipe / slam (windup, strike)
/// - Frames 7-8: ground pound (windup, impact)
/// - Frames 9-11: death animation
pub fn goblin_king_animation_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<
        (&Velocity, &BossAttackState, &mut GoblinKingAnimation, &mut Sprite),
        With<GoblinKing>,
    >,
) {
    // Don't animate if paused
    if debug_settings.is_paused() {
        return;
    }

    for (velocity, attack_state, mut anim, mut sprite) in query.iter_mut() {
        // Skip dead bosses
        if anim.state == GoblinKingAnimState::Dead {
            continue;
        }

        // Sync animation state with attack state
        match *attack_state {
            BossAttackState::Idle => {
                let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
                let is_moving = speed > WALK_THRESHOLD;

                match anim.state {
                    GoblinKingAnimState::Idle if is_moving => {
                        anim.start_walking();
                    }
                    GoblinKingAnimState::Walking if !is_moving => {
                        anim.go_idle();
                    }
                    GoblinKingAnimState::Walking => {
                        // Advance walk animation
                        anim.frame_timer.tick(time.delta());
                        if anim.frame_timer.just_finished() {
                            anim.advance_walk_frame();
                        }
                    }
                    // If we were in an attack state and now idle, return to normal
                    GoblinKingAnimState::SwipeStrike
                    | GoblinKingAnimState::PoundImpact
                    | GoblinKingAnimState::ChargeDash => {
                        if is_moving {
                            anim.start_walking();
                        } else {
                            anim.go_idle();
                        }
                    }
                    _ => {}
                }
            }
            BossAttackState::WindingUpSlam => {
                // Use sword swipe for slam attack (frames 5-6)
                if anim.state != GoblinKingAnimState::SwipeWindup {
                    anim.start_swipe_windup();
                } else {
                    anim.frame_timer.tick(time.delta());
                }
            }
            BossAttackState::Slamming => {
                if anim.state != GoblinKingAnimState::SwipeStrike {
                    anim.start_swipe_strike();
                }
            }
            BossAttackState::ChargingUp => {
                // Charge windup (frame 3)
                if anim.state != GoblinKingAnimState::ChargeWindup {
                    anim.start_charge_windup();
                } else {
                    anim.frame_timer.tick(time.delta());
                }
            }
            BossAttackState::Charging => {
                // Charge dash (frame 4)
                if anim.state != GoblinKingAnimState::ChargeDash {
                    anim.start_charge_dash();
                }
            }
            BossAttackState::Summoning => {
                // Use ground pound for summoning visual (frames 7-8)
                if anim.state != GoblinKingAnimState::PoundWindup
                    && anim.state != GoblinKingAnimState::PoundImpact
                {
                    anim.start_pound_windup();
                } else {
                    anim.frame_timer.tick(time.delta());
                    if anim.frame_timer.just_finished() && anim.state == GoblinKingAnimState::PoundWindup {
                        anim.start_pound_impact();
                    }
                }
            }
        }

        // Update sprite atlas index
        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = anim.current_frame;
        }

        // Flip sprite based on horizontal velocity (when moving)
        if velocity.x.abs() > 1.0 {
            sprite.flip_x = velocity.x < 0.0;
        }
    }
}
