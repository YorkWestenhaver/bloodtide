use bevy::prelude::*;

/// Marker component for the player entity
#[derive(Component)]
pub struct Player;

/// Velocity component for movement
#[derive(Component, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
