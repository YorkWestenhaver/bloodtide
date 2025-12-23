use bevy::prelude::*;

/// Marker component for creature entities (player's minions)
#[derive(Component)]
pub struct Creature;

/// Creature color/element type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreatureColor {
    #[default]
    Red,
    Blue,
    Green,
    White,
    Black,
    Colorless,
}

impl CreatureColor {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "red" => CreatureColor::Red,
            "blue" => CreatureColor::Blue,
            "green" => CreatureColor::Green,
            "white" => CreatureColor::White,
            "black" => CreatureColor::Black,
            _ => CreatureColor::Colorless,
        }
    }

    /// Get the display color for this creature color
    pub fn to_bevy_color(&self) -> Color {
        match self {
            CreatureColor::Red => Color::srgb(1.0, 0.3, 0.2),      // Fire red
            CreatureColor::Blue => Color::srgb(0.2, 0.4, 1.0),     // Ice blue
            CreatureColor::Green => Color::srgb(0.2, 0.8, 0.3),    // Nature green
            CreatureColor::White => Color::srgb(0.95, 0.95, 0.9),  // Holy white
            CreatureColor::Black => Color::srgb(0.3, 0.1, 0.3),    // Dark purple
            CreatureColor::Colorless => Color::srgb(0.7, 0.7, 0.8), // Gray
        }
    }
}

/// Creature archetype/role
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreatureType {
    #[default]
    Melee,
    Ranged,
    Support,
    Assassin,
}

impl CreatureType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "melee" => CreatureType::Melee,
            "ranged" => CreatureType::Ranged,
            "support" => CreatureType::Support,
            "assassin" => CreatureType::Assassin,
            _ => CreatureType::Melee,
        }
    }
}

/// Runtime data for a creature entity
#[derive(Component, Clone, Debug)]
pub struct CreatureStats {
    pub id: String,
    pub name: String,
    pub color: CreatureColor,
    pub tier: u8,
    pub creature_type: CreatureType,
    pub level: u32,
    pub kills: u32,
    // Combat stats
    pub base_damage: f64,
    pub attack_speed: f64,
    pub base_hp: f64,
    pub max_hp: f64,
    pub current_hp: f64,
    pub movement_speed: f64,
    pub attack_range: f64,
    // Crit stats (stored for future use)
    pub crit_t1: f64,
    pub crit_t2: f64,
    pub crit_t3: f64,
}

impl CreatureStats {
    pub fn new(
        id: String,
        name: String,
        color: CreatureColor,
        tier: u8,
        creature_type: CreatureType,
        base_damage: f64,
        attack_speed: f64,
        base_hp: f64,
        movement_speed: f64,
        attack_range: f64,
        crit_t1: f64,
        crit_t2: f64,
        crit_t3: f64,
    ) -> Self {
        Self {
            id,
            name,
            color,
            tier,
            creature_type,
            level: 1,
            kills: 0,
            base_damage,
            attack_speed,
            base_hp,
            max_hp: base_hp,
            current_hp: base_hp,
            movement_speed,
            attack_range,
            crit_t1,
            crit_t2,
            crit_t3,
        }
    }
}

/// Attack cooldown timer for creatures
#[derive(Component)]
pub struct AttackTimer {
    pub timer: Timer,
}

impl AttackTimer {
    pub fn new(attack_speed: f64) -> Self {
        // attack_speed is attacks per second, so duration = 1.0 / attack_speed
        let duration = if attack_speed > 0.0 {
            1.0 / attack_speed
        } else {
            1.0
        };
        Self {
            timer: Timer::from_seconds(duration as f32, TimerMode::Repeating),
        }
    }
}

/// Attack range in pixels
#[derive(Component)]
pub struct AttackRange(pub f32);

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CreatureColor Tests
    // =========================================================================

    #[test]
    fn creature_color_from_str_parses_all_colors() {
        assert_eq!(CreatureColor::from_str("red"), CreatureColor::Red);
        assert_eq!(CreatureColor::from_str("blue"), CreatureColor::Blue);
        assert_eq!(CreatureColor::from_str("green"), CreatureColor::Green);
        assert_eq!(CreatureColor::from_str("white"), CreatureColor::White);
        assert_eq!(CreatureColor::from_str("black"), CreatureColor::Black);
    }

    #[test]
    fn creature_color_from_str_is_case_insensitive() {
        assert_eq!(CreatureColor::from_str("RED"), CreatureColor::Red);
        assert_eq!(CreatureColor::from_str("Red"), CreatureColor::Red);
        assert_eq!(CreatureColor::from_str("rEd"), CreatureColor::Red);
        assert_eq!(CreatureColor::from_str("BLUE"), CreatureColor::Blue);
        assert_eq!(CreatureColor::from_str("Blue"), CreatureColor::Blue);
    }

    #[test]
    fn creature_color_from_str_defaults_to_colorless_for_unknown() {
        assert_eq!(CreatureColor::from_str("unknown"), CreatureColor::Colorless);
        assert_eq!(CreatureColor::from_str(""), CreatureColor::Colorless);
        assert_eq!(CreatureColor::from_str("purple"), CreatureColor::Colorless);
        assert_eq!(CreatureColor::from_str("yellow"), CreatureColor::Colorless);
    }

    #[test]
    fn creature_color_to_bevy_color_returns_correct_rgb() {
        // Red should be reddish
        let red = CreatureColor::Red.to_bevy_color();
        assert_eq!(red, Color::srgb(1.0, 0.3, 0.2));

        // Blue should be bluish
        let blue = CreatureColor::Blue.to_bevy_color();
        assert_eq!(blue, Color::srgb(0.2, 0.4, 1.0));

        // Green should be greenish
        let green = CreatureColor::Green.to_bevy_color();
        assert_eq!(green, Color::srgb(0.2, 0.8, 0.3));

        // White should be light
        let white = CreatureColor::White.to_bevy_color();
        assert_eq!(white, Color::srgb(0.95, 0.95, 0.9));

        // Black should be dark purple
        let black = CreatureColor::Black.to_bevy_color();
        assert_eq!(black, Color::srgb(0.3, 0.1, 0.3));

        // Colorless should be gray
        let colorless = CreatureColor::Colorless.to_bevy_color();
        assert_eq!(colorless, Color::srgb(0.7, 0.7, 0.8));
    }

    #[test]
    fn creature_color_default_is_red() {
        assert_eq!(CreatureColor::default(), CreatureColor::Red);
    }

    #[test]
    fn creature_color_equality_works() {
        assert_eq!(CreatureColor::Red, CreatureColor::Red);
        assert_ne!(CreatureColor::Red, CreatureColor::Blue);
    }

    // =========================================================================
    // CreatureType Tests
    // =========================================================================

    #[test]
    fn creature_type_from_str_parses_all_types() {
        assert_eq!(CreatureType::from_str("melee"), CreatureType::Melee);
        assert_eq!(CreatureType::from_str("ranged"), CreatureType::Ranged);
        assert_eq!(CreatureType::from_str("support"), CreatureType::Support);
        assert_eq!(CreatureType::from_str("assassin"), CreatureType::Assassin);
    }

    #[test]
    fn creature_type_from_str_is_case_insensitive() {
        assert_eq!(CreatureType::from_str("MELEE"), CreatureType::Melee);
        assert_eq!(CreatureType::from_str("Melee"), CreatureType::Melee);
        assert_eq!(CreatureType::from_str("RANGED"), CreatureType::Ranged);
        assert_eq!(CreatureType::from_str("Ranged"), CreatureType::Ranged);
    }

    #[test]
    fn creature_type_from_str_defaults_to_melee_for_unknown() {
        assert_eq!(CreatureType::from_str("unknown"), CreatureType::Melee);
        assert_eq!(CreatureType::from_str(""), CreatureType::Melee);
        assert_eq!(CreatureType::from_str("tank"), CreatureType::Melee);
    }

    #[test]
    fn creature_type_default_is_melee() {
        assert_eq!(CreatureType::default(), CreatureType::Melee);
    }

    // =========================================================================
    // CreatureStats Tests
    // =========================================================================

    #[test]
    fn creature_stats_new_initializes_level_to_1() {
        let stats = CreatureStats::new(
            "test".to_string(),
            "Test Creature".to_string(),
            CreatureColor::Red,
            1,
            CreatureType::Ranged,
            15.0, // base_damage
            1.0,  // attack_speed
            100.0, // base_hp
            100.0, // movement_speed
            200.0, // attack_range
            5.0,  // crit_t1
            0.0,  // crit_t2
            0.0,  // crit_t3
        );
        assert_eq!(stats.level, 1);
    }

    #[test]
    fn creature_stats_new_initializes_kills_to_0() {
        let stats = CreatureStats::new(
            "test".to_string(),
            "Test Creature".to_string(),
            CreatureColor::Red,
            1,
            CreatureType::Ranged,
            15.0, 1.0, 100.0, 100.0, 200.0, 5.0, 0.0, 0.0,
        );
        assert_eq!(stats.kills, 0);
    }

    #[test]
    fn creature_stats_new_sets_max_hp_equal_to_base_hp() {
        let stats = CreatureStats::new(
            "test".to_string(),
            "Test Creature".to_string(),
            CreatureColor::Red,
            1,
            CreatureType::Ranged,
            15.0, 1.0, 100.0, 100.0, 200.0, 5.0, 0.0, 0.0,
        );
        assert_eq!(stats.max_hp, 100.0);
        assert_eq!(stats.max_hp, stats.base_hp);
    }

    #[test]
    fn creature_stats_new_sets_current_hp_equal_to_base_hp() {
        let stats = CreatureStats::new(
            "test".to_string(),
            "Test Creature".to_string(),
            CreatureColor::Red,
            1,
            CreatureType::Ranged,
            15.0, 1.0, 100.0, 100.0, 200.0, 5.0, 0.0, 0.0,
        );
        assert_eq!(stats.current_hp, 100.0);
        assert_eq!(stats.current_hp, stats.base_hp);
    }

    #[test]
    fn creature_stats_new_preserves_all_input_values() {
        let stats = CreatureStats::new(
            "fire_imp".to_string(),
            "Fire Imp".to_string(),
            CreatureColor::Red,
            2,
            CreatureType::Ranged,
            25.5,  // base_damage
            1.5,   // attack_speed
            75.0,  // base_hp
            120.0, // movement_speed
            250.0, // attack_range
            10.0,  // crit_t1
            5.0,   // crit_t2
            1.0,   // crit_t3
        );
        assert_eq!(stats.id, "fire_imp");
        assert_eq!(stats.name, "Fire Imp");
        assert_eq!(stats.color, CreatureColor::Red);
        assert_eq!(stats.tier, 2);
        assert_eq!(stats.creature_type, CreatureType::Ranged);
        assert_eq!(stats.base_damage, 25.5);
        assert_eq!(stats.attack_speed, 1.5);
        assert_eq!(stats.base_hp, 75.0);
        assert_eq!(stats.movement_speed, 120.0);
        assert_eq!(stats.attack_range, 250.0);
        assert_eq!(stats.crit_t1, 10.0);
        assert_eq!(stats.crit_t2, 5.0);
        assert_eq!(stats.crit_t3, 1.0);
    }

    // =========================================================================
    // AttackTimer Tests
    // =========================================================================

    #[test]
    fn attack_timer_calculates_duration_from_attack_speed() {
        // attack_speed = 1.0 means 1 attack per second, so duration = 1.0
        let timer = AttackTimer::new(1.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);

        // attack_speed = 2.0 means 2 attacks per second, so duration = 0.5
        let timer = AttackTimer::new(2.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 0.5);

        // attack_speed = 0.5 means 0.5 attacks per second, so duration = 2.0
        let timer = AttackTimer::new(0.5);
        assert_eq!(timer.timer.duration().as_secs_f32(), 2.0);
    }

    #[test]
    fn attack_timer_defaults_to_1_second_for_zero_attack_speed() {
        let timer = AttackTimer::new(0.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);
    }

    #[test]
    fn attack_timer_defaults_to_1_second_for_negative_attack_speed() {
        let timer = AttackTimer::new(-1.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);
    }

    #[test]
    fn attack_timer_is_repeating() {
        let timer = AttackTimer::new(1.0);
        assert_eq!(timer.timer.mode(), TimerMode::Repeating);
    }

    // =========================================================================
    // AttackRange Tests
    // =========================================================================

    #[test]
    fn attack_range_stores_value() {
        let range = AttackRange(150.0);
        assert_eq!(range.0, 150.0);
    }
}
