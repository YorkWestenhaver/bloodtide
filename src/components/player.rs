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

/// Player stats component tracking HP
#[derive(Component)]
pub struct PlayerStats {
    pub max_hp: f64,
    pub current_hp: f64,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_hp: 200.0,
            current_hp: 200.0,
        }
    }
}

/// Invincibility frames timer (prevents rapid HP loss)
#[derive(Component)]
pub struct InvincibilityTimer {
    pub timer: Timer,
}

impl InvincibilityTimer {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }

    pub fn is_active(&self) -> bool {
        !self.timer.finished()
    }
}

/// Player animation state for sprite-based player
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PlayerAnimationState {
    #[default]
    Idle,
    Walking,
    Dying,
    Dead,
}

/// Player animation controller
#[derive(Component)]
pub struct PlayerAnimation {
    pub state: PlayerAnimationState,
    pub frame_timer: Timer,
    pub current_frame: usize,
}

impl PlayerAnimation {
    // Frame indices for wizard_player_spritesheet (80x128, 6 frames)
    pub const FRAME_IDLE: usize = 0;
    pub const FRAME_WALK_START: usize = 1;
    pub const FRAME_WALK_END: usize = 2;
    pub const FRAME_DEATH_START: usize = 3;
    pub const FRAME_DEATH_END: usize = 5;

    pub const WALK_FRAME_DURATION_MS: u32 = 150;
    pub const DEATH_FRAME_DURATION_MS: u32 = 180;

    pub fn new() -> Self {
        Self {
            state: PlayerAnimationState::Idle,
            frame_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            current_frame: Self::FRAME_IDLE,
        }
    }

    pub fn start_walking(&mut self) {
        if self.state != PlayerAnimationState::Dying && self.state != PlayerAnimationState::Dead {
            self.state = PlayerAnimationState::Walking;
            self.current_frame = Self::FRAME_WALK_START;
            self.frame_timer = Timer::from_seconds(
                Self::WALK_FRAME_DURATION_MS as f32 / 1000.0,
                TimerMode::Repeating,
            );
        }
    }

    pub fn go_idle(&mut self) {
        if self.state != PlayerAnimationState::Dying && self.state != PlayerAnimationState::Dead {
            self.state = PlayerAnimationState::Idle;
            self.current_frame = Self::FRAME_IDLE;
        }
    }

    pub fn start_dying(&mut self) {
        self.state = PlayerAnimationState::Dying;
        self.current_frame = Self::FRAME_DEATH_START;
        self.frame_timer = Timer::from_seconds(
            Self::DEATH_FRAME_DURATION_MS as f32 / 1000.0,
            TimerMode::Repeating,
        );
    }

    /// Advance walk animation frame, cycling between frames 1-2
    pub fn advance_walk_frame(&mut self) {
        if self.current_frame >= Self::FRAME_WALK_END {
            self.current_frame = Self::FRAME_WALK_START;
        } else {
            self.current_frame += 1;
        }
    }

    /// Advance death animation frame. Returns true when animation is complete.
    pub fn advance_death_frame(&mut self) -> bool {
        if self.current_frame >= Self::FRAME_DEATH_END {
            true
        } else {
            self.current_frame += 1;
            self.current_frame >= Self::FRAME_DEATH_END
        }
    }

    pub fn become_dead(&mut self) {
        self.state = PlayerAnimationState::Dead;
    }
}
