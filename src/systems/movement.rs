use bevy::prelude::*;

use crate::components::{Player, Velocity};
use crate::resources::DebugSettings;

/// Player movement speed in pixels per second
pub const PLAYER_SPEED: f32 = 300.0;

/// Read keyboard input and update player velocity
pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    // Don't process movement if game is paused
    if debug_settings.is_paused() {
        for mut velocity in query.iter_mut() {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
        return;
    }

    for mut velocity in query.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        // Normalize to prevent faster diagonal movement
        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        // Apply debug settings speed multiplier
        let speed = PLAYER_SPEED * debug_settings.player_speed_multiplier;
        velocity.x = direction.x * speed;
        velocity.y = direction.y * speed;
    }
}

/// Apply velocity to transform for all entities with Velocity component
pub fn apply_velocity_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    // Don't apply velocity if game is paused
    if debug_settings.is_paused() {
        return;
    }

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

/// Camera follows the player
pub fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut camera_transform in camera_query.iter_mut() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}
