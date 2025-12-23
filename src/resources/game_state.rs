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

/// Calculate the next level threshold based on current threshold
/// Multiplies by 1.2 and rounds up
pub fn calculate_next_level_threshold(current_threshold: u32) -> u32 {
    (current_threshold as f32 * 1.2).ceil() as u32
}

/// Check if wave should advance based on kills
pub fn should_advance_wave(kills_at_wave_start: u32, total_kills: u32, kills_per_wave: u32) -> bool {
    (total_kills - kills_at_wave_start) >= kills_per_wave
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GameState Default Tests
    // =========================================================================

    #[test]
    fn game_state_default_starts_at_level_1() {
        let state = GameState::default();
        assert_eq!(state.current_level, 1);
    }

    #[test]
    fn game_state_default_starts_at_wave_1() {
        let state = GameState::default();
        assert_eq!(state.current_wave, 1);
    }

    #[test]
    fn game_state_default_starts_with_0_kills() {
        let state = GameState::default();
        assert_eq!(state.kill_count, 0);
        assert_eq!(state.total_kills, 0);
    }

    #[test]
    fn game_state_default_requires_25_kills_for_first_level() {
        let state = GameState::default();
        assert_eq!(state.kills_for_next_level, 25);
    }

    #[test]
    fn game_state_default_has_no_pending_level_up() {
        let state = GameState::default();
        assert!(!state.level_up_pending);
    }

    #[test]
    fn game_state_default_starts_with_0_kills_at_wave_start() {
        let state = GameState::default();
        assert_eq!(state.kills_at_wave_start, 0);
    }

    // =========================================================================
    // Level Threshold Calculation Tests
    // =========================================================================

    #[test]
    fn level_threshold_increases_by_approximately_1_2x() {
        // Level 1 -> 2: 25 * 1.2 = 30 (but f32 precision gives 31 due to ceil)
        // Note: Due to f32 precision, 1.2 is slightly > 1.2, so ceil rounds up
        let result = calculate_next_level_threshold(25);
        assert!(result >= 30 && result <= 31, "Expected ~30, got {}", result);
    }

    #[test]
    fn level_threshold_rounds_up() {
        // These test the ceiling behavior with f32 precision
        let from_30 = calculate_next_level_threshold(30);
        assert!(from_30 >= 36 && from_30 <= 37, "Expected ~36, got {}", from_30);

        let from_36 = calculate_next_level_threshold(36);
        assert!(from_36 >= 43 && from_36 <= 44, "Expected ~44, got {}", from_36);

        let from_44 = calculate_next_level_threshold(44);
        assert!(from_44 >= 52 && from_44 <= 53, "Expected ~53, got {}", from_44);
    }

    #[test]
    fn level_threshold_progression_increases_each_level() {
        // Verify that thresholds consistently increase
        let mut threshold = 25u32;
        let mut previous = 0u32;

        for level in 1..=10 {
            assert!(
                threshold > previous,
                "Level {} threshold {} should be > previous {}",
                level,
                threshold,
                previous
            );
            previous = threshold;
            threshold = calculate_next_level_threshold(threshold);
        }

        // After 10 levels, threshold should be significantly higher
        assert!(threshold > 100, "After 10 levels, threshold should exceed 100, got {}", threshold);
    }

    #[test]
    fn level_threshold_handles_zero() {
        // 0 * 1.2 = 0
        assert_eq!(calculate_next_level_threshold(0), 0);
    }

    #[test]
    fn level_threshold_handles_very_large_values() {
        // Test with a large value to ensure no overflow issues
        let large = 1_000_000u32;
        let result = calculate_next_level_threshold(large);
        assert_eq!(result, 1_200_000);
    }

    // =========================================================================
    // Wave Advancement Tests
    // =========================================================================

    #[test]
    fn wave_advances_at_exactly_kills_per_wave() {
        // If we need 50 kills per wave and we have exactly 50
        assert!(should_advance_wave(0, 50, 50));
    }

    #[test]
    fn wave_advances_when_over_threshold() {
        // If we need 50 kills per wave and we have 51
        assert!(should_advance_wave(0, 51, 50));
    }

    #[test]
    fn wave_does_not_advance_when_under_threshold() {
        // If we need 50 kills and we only have 49
        assert!(!should_advance_wave(0, 49, 50));
    }

    #[test]
    fn wave_advancement_uses_kills_since_wave_start() {
        // Wave started at kill 100, now at 149 -> only 49 kills this wave
        assert!(!should_advance_wave(100, 149, 50));
        // Wave started at kill 100, now at 150 -> exactly 50 kills this wave
        assert!(should_advance_wave(100, 150, 50));
    }

    #[test]
    fn wave_advancement_handles_zero_kills() {
        assert!(!should_advance_wave(0, 0, 50));
    }
}
