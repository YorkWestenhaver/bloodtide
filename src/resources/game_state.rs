use bevy::prelude::*;

/// Global game state resource tracking progress through a run
#[derive(Resource)]
pub struct GameState {
    pub kill_count: u32,
    pub total_kills: u32,
    pub current_level: u32,
    pub current_wave: u32,
    pub kills_for_next_level: u32,
    pub kills_at_wave_start: u32,
    pub level_up_pending: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            kill_count: 0,
            total_kills: 0,
            current_level: 1,
            current_wave: 1,
            kills_for_next_level: 25,
            kills_at_wave_start: 0,
            level_up_pending: false,
        }
    }
}
