use bevy::prelude::*;

use crate::components::CreatureColor;
use crate::resources::GameData;

/// Resource tracking current affinity values for each color
#[derive(Resource, Debug, Default)]
pub struct AffinityState {
    pub red: f64,
    pub blue: f64,
    pub green: f64,
    pub white: f64,
    pub black: f64,
    pub colorless: f64,
}

impl AffinityState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get affinity value for a specific color
    pub fn get(&self, color: CreatureColor) -> f64 {
        match color {
            CreatureColor::Red => self.red,
            CreatureColor::Blue => self.blue,
            CreatureColor::Green => self.green,
            CreatureColor::White => self.white,
            CreatureColor::Black => self.black,
            CreatureColor::Colorless => self.colorless,
        }
    }

    /// Add affinity for a specific color
    pub fn add(&mut self, color: CreatureColor, amount: f64) {
        match color {
            CreatureColor::Red => self.red += amount,
            CreatureColor::Blue => self.blue += amount,
            CreatureColor::Green => self.green += amount,
            CreatureColor::White => self.white += amount,
            CreatureColor::Black => self.black += amount,
            CreatureColor::Colorless => self.colorless += amount,
        }
    }

    /// Remove affinity for a specific color
    pub fn remove(&mut self, color: CreatureColor, amount: f64) {
        match color {
            CreatureColor::Red => self.red = (self.red - amount).max(0.0),
            CreatureColor::Blue => self.blue = (self.blue - amount).max(0.0),
            CreatureColor::Green => self.green = (self.green - amount).max(0.0),
            CreatureColor::White => self.white = (self.white - amount).max(0.0),
            CreatureColor::Black => self.black = (self.black - amount).max(0.0),
            CreatureColor::Colorless => self.colorless = (self.colorless - amount).max(0.0),
        }
    }
}

/// Bonuses from affinity thresholds
#[derive(Clone, Debug, Default)]
pub struct AffinityBonus {
    pub damage_bonus: f64,
    pub attack_speed_bonus: f64,
    pub hp_bonus: f64,
    pub crit_t1_bonus: f64,
    pub crit_t2_unlock: bool,
    pub crit_t3_unlock: bool,
    pub special: String,
}

/// Get affinity bonuses for a creature based on its color and current affinity
pub fn get_affinity_bonuses(game_data: &GameData, color: CreatureColor, affinity_state: &AffinityState) -> AffinityBonus {
    let color_str = match color {
        CreatureColor::Red => "red",
        CreatureColor::Blue => "blue",
        CreatureColor::Green => "green",
        CreatureColor::White => "white",
        CreatureColor::Black => "black",
        CreatureColor::Colorless => "colorless",
    };

    let current_affinity = affinity_state.get(color);

    // Find the affinity color data
    let affinity_color = game_data.affinity_colors.iter().find(|ac| ac.color == color_str);

    let Some(affinity_color) = affinity_color else {
        return AffinityBonus::default();
    };

    // Find the highest threshold that is met
    let mut best_threshold = None;

    for threshold in &affinity_color.thresholds {
        if current_affinity >= threshold.min as f64 {
            best_threshold = Some(threshold);
        }
    }

    match best_threshold {
        Some(threshold) => AffinityBonus {
            damage_bonus: threshold.damage_bonus,
            attack_speed_bonus: threshold.attack_speed_bonus,
            hp_bonus: threshold.hp_bonus,
            crit_t1_bonus: threshold.crit_t1_bonus,
            crit_t2_unlock: threshold.crit_t2_unlock,
            crit_t3_unlock: threshold.crit_t3_unlock,
            special: threshold.special.clone(),
        },
        None => AffinityBonus::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affinity_state_default_is_zero() {
        let state = AffinityState::default();
        assert_eq!(state.red, 0.0);
        assert_eq!(state.blue, 0.0);
        assert_eq!(state.green, 0.0);
        assert_eq!(state.white, 0.0);
        assert_eq!(state.black, 0.0);
        assert_eq!(state.colorless, 0.0);
    }

    #[test]
    fn affinity_state_get_returns_correct_value() {
        let mut state = AffinityState::default();
        state.red = 10.0;
        state.blue = 20.0;

        assert_eq!(state.get(CreatureColor::Red), 10.0);
        assert_eq!(state.get(CreatureColor::Blue), 20.0);
        assert_eq!(state.get(CreatureColor::Green), 0.0);
    }

    #[test]
    fn affinity_state_add_increases_value() {
        let mut state = AffinityState::default();
        state.add(CreatureColor::Red, 15.0);
        state.add(CreatureColor::Red, 10.0);

        assert_eq!(state.red, 25.0);
    }

    #[test]
    fn affinity_state_remove_decreases_value() {
        let mut state = AffinityState::default();
        state.red = 30.0;
        state.remove(CreatureColor::Red, 10.0);

        assert_eq!(state.red, 20.0);
    }

    #[test]
    fn affinity_state_remove_clamps_to_zero() {
        let mut state = AffinityState::default();
        state.red = 5.0;
        state.remove(CreatureColor::Red, 10.0);

        assert_eq!(state.red, 0.0);
    }

    #[test]
    fn affinity_bonus_default_is_zero() {
        let bonus = AffinityBonus::default();
        assert_eq!(bonus.damage_bonus, 0.0);
        assert_eq!(bonus.attack_speed_bonus, 0.0);
        assert_eq!(bonus.hp_bonus, 0.0);
        assert_eq!(bonus.crit_t1_bonus, 0.0);
        assert!(!bonus.crit_t2_unlock);
        assert!(!bonus.crit_t3_unlock);
        assert!(bonus.special.is_empty());
    }
}
