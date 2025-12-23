use bevy::prelude::*;

/// State of the debug/pause menus
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MenuState {
    #[default]
    Closed,
    DebugMenuOpen,
    PauseMenuOpen,
}

/// Debug settings resource with all tunable values for real-time game adjustments
#[derive(Resource)]
pub struct DebugSettings {
    // Speed multipliers
    pub player_speed_multiplier: f32,
    pub creature_speed_multiplier: f32,
    pub enemy_speed_multiplier: f32,

    // Damage multipliers
    pub creature_damage_multiplier: f32,
    pub enemy_damage_multiplier: f32,

    // Spawn rate
    pub enemy_spawn_rate_multiplier: f32,

    // Crit bonuses (added to base crit chance)
    pub crit_t1_bonus: f32,
    pub crit_t2_bonus: f32,
    pub crit_t3_bonus: f32,

    // Projectile settings
    pub projectile_count_bonus: i32,      // Added to base projectile count
    pub projectile_size_multiplier: f32,  // Multiplied by base size
    pub projectile_speed_multiplier: f32, // Multiplied by base speed
    pub attack_speed_multiplier: f32,     // Multiplied by attack speed

    // Overrides (None = use normal, Some(X) = force to X)
    pub current_wave_override: Option<u32>,
    pub current_level_override: Option<u32>,

    // Toggles
    pub god_mode: bool,      // Creatures can't die
    pub show_fps: bool,      // Display FPS in corner
    pub show_enemy_count: bool, // Display enemy count in HUD

    // Menu state
    pub menu_state: MenuState,
    pub menu_toggle_mode: bool, // true = toggle on press, false = hold to open

    // Animation state
    pub menu_slide_progress: f32, // 0.0 = closed, 1.0 = fully open
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            player_speed_multiplier: 1.0,
            creature_speed_multiplier: 1.0,
            enemy_speed_multiplier: 1.0,
            creature_damage_multiplier: 1.0,
            enemy_damage_multiplier: 1.0,
            enemy_spawn_rate_multiplier: 1.0,
            crit_t1_bonus: 0.0,
            crit_t2_bonus: 0.0,
            crit_t3_bonus: 0.0,
            projectile_count_bonus: 0,
            projectile_size_multiplier: 1.0,
            projectile_speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            current_wave_override: None,
            current_level_override: None,
            god_mode: false,
            show_fps: true,
            show_enemy_count: true,
            menu_state: MenuState::Closed,
            menu_toggle_mode: true,
            menu_slide_progress: 0.0,
        }
    }
}

impl DebugSettings {
    /// Reset all settings to their default values
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// Check if game should be paused (paused when any menu is open)
    pub fn is_paused(&self) -> bool {
        self.menu_state != MenuState::Closed
    }

    /// Check if any menu is open
    pub fn is_menu_open(&self) -> bool {
        self.menu_state != MenuState::Closed
    }
}

/// Slider range definitions for debug settings
pub struct SliderRange {
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl SliderRange {
    pub const SPEED: SliderRange = SliderRange { min: 0.1, max: 5.0, step: 0.1 };
    pub const DAMAGE: SliderRange = SliderRange { min: 0.1, max: 10.0, step: 0.1 };
    pub const CRIT: SliderRange = SliderRange { min: 0.0, max: 100.0, step: 1.0 };
    pub const WAVE_LEVEL: SliderRange = SliderRange { min: 1.0, max: 100.0, step: 1.0 };
    pub const PROJECTILE_COUNT: SliderRange = SliderRange { min: -3.0, max: 10.0, step: 1.0 };
    pub const PROJECTILE_SIZE: SliderRange = SliderRange { min: 0.25, max: 4.0, step: 0.25 };
    pub const PROJECTILE_SPEED: SliderRange = SliderRange { min: 0.25, max: 3.0, step: 0.25 };
    pub const ATTACK_SPEED: SliderRange = SliderRange { min: 0.1, max: 5.0, step: 0.1 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_neutral() {
        let settings = DebugSettings::default();
        assert_eq!(settings.player_speed_multiplier, 1.0);
        assert_eq!(settings.creature_speed_multiplier, 1.0);
        assert_eq!(settings.creature_damage_multiplier, 1.0);
        assert_eq!(settings.enemy_damage_multiplier, 1.0);
        assert_eq!(settings.enemy_speed_multiplier, 1.0);
        assert_eq!(settings.enemy_spawn_rate_multiplier, 1.0);
        assert_eq!(settings.crit_t1_bonus, 0.0);
        assert_eq!(settings.crit_t2_bonus, 0.0);
        assert_eq!(settings.crit_t3_bonus, 0.0);
        assert_eq!(settings.projectile_count_bonus, 0);
        assert_eq!(settings.projectile_size_multiplier, 1.0);
        assert_eq!(settings.projectile_speed_multiplier, 1.0);
        assert_eq!(settings.attack_speed_multiplier, 1.0);
    }

    #[test]
    fn default_overrides_are_none() {
        let settings = DebugSettings::default();
        assert!(settings.current_wave_override.is_none());
        assert!(settings.current_level_override.is_none());
    }

    #[test]
    fn default_god_mode_is_off() {
        let settings = DebugSettings::default();
        assert!(!settings.god_mode);
    }

    #[test]
    fn default_menu_is_closed() {
        let settings = DebugSettings::default();
        assert_eq!(settings.menu_state, MenuState::Closed);
        assert!(!settings.is_paused());
        assert!(!settings.is_menu_open());
    }

    #[test]
    fn pause_menu_pauses_game() {
        let mut settings = DebugSettings::default();
        settings.menu_state = MenuState::PauseMenuOpen;
        assert!(settings.is_paused());
        assert!(settings.is_menu_open());
    }

    #[test]
    fn debug_menu_pauses_game() {
        let mut settings = DebugSettings::default();
        settings.menu_state = MenuState::DebugMenuOpen;
        assert!(settings.is_paused());
        assert!(settings.is_menu_open());
    }

    #[test]
    fn reset_to_defaults_works() {
        let mut settings = DebugSettings::default();
        settings.player_speed_multiplier = 5.0;
        settings.god_mode = true;
        settings.crit_t1_bonus = 50.0;
        settings.current_wave_override = Some(10);

        settings.reset_to_defaults();

        assert_eq!(settings.player_speed_multiplier, 1.0);
        assert!(!settings.god_mode);
        assert_eq!(settings.crit_t1_bonus, 0.0);
        assert!(settings.current_wave_override.is_none());
    }

    #[test]
    fn slider_ranges_are_valid() {
        assert!(SliderRange::SPEED.min < SliderRange::SPEED.max);
        assert!(SliderRange::DAMAGE.min < SliderRange::DAMAGE.max);
        assert!(SliderRange::CRIT.min < SliderRange::CRIT.max);
        assert!(SliderRange::WAVE_LEVEL.min < SliderRange::WAVE_LEVEL.max);
        assert!(SliderRange::PROJECTILE_COUNT.min < SliderRange::PROJECTILE_COUNT.max);
        assert!(SliderRange::PROJECTILE_SIZE.min < SliderRange::PROJECTILE_SIZE.max);
        assert!(SliderRange::PROJECTILE_SPEED.min < SliderRange::PROJECTILE_SPEED.max);
        assert!(SliderRange::ATTACK_SPEED.min < SliderRange::ATTACK_SPEED.max);
    }
}
