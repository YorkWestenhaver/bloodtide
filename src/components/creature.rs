use bevy::prelude::*;

/// Marker component for creature entities (player's minions)
#[derive(Component)]
pub struct Creature;

/// Creature color/element type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
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

/// Get a unique color for a creature based on its ID
/// Each creature has a distinct visual appearance even within the same color affinity
pub fn get_creature_color_by_id(creature_id: &str) -> Color {
    match creature_id {
        // Tier 1 - Lighter, more basic colors
        "fire_imp" => Color::srgb(1.0, 0.35, 0.2),          // Bright orange-red
        "ember_hound" => Color::srgb(1.0, 0.5, 0.15),       // Orange
        "fire_spirit" => Color::srgb(1.0, 0.6, 0.4),        // Peach/salmon

        // Tier 2 - Richer, more saturated colors
        "flame_fiend" => Color::srgb(0.9, 0.2, 0.1),        // Deep crimson
        "hellhound" => Color::srgb(0.85, 0.25, 0.0),        // Burnt orange
        "inferno_knight" => Color::srgb(0.7, 0.15, 0.15),   // Dark red (armored)
        "magma_elemental" => Color::srgb(0.8, 0.3, 0.0),    // Magma orange
        "greater_fire_spirit" => Color::srgb(1.0, 0.7, 0.5),// Bright peach

        // Tier 3 - Intense, dramatic colors
        "inferno_demon" => Color::srgb(0.6, 0.05, 0.1),     // Very dark red
        "hellhound_alpha" => Color::srgb(0.9, 0.15, 0.0),   // Intense orange-red
        "inferno_warlord" => Color::srgb(0.5, 0.1, 0.1),    // Maroon
        "phoenix" => Color::srgb(1.0, 0.55, 0.0),           // Brilliant orange

        // Tier 4 - Legendary, special colors
        "inferno_titan" => Color::srgb(0.4, 0.05, 0.05),    // Nearly black-red
        "eternal_phoenix" => Color::srgb(1.0, 0.75, 0.2),   // Golden-orange

        // Fallback - use affinity color
        _ => Color::srgb(1.0, 0.3, 0.2), // Default fire red
    }
}

/// Creature archetype/role
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
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
    pub kills_for_next_level: u32,
    pub max_level: u32,
    pub evolves_into: String,
    pub evolution_count: u32,
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
        kills_for_next_level: u32,
        max_level: u32,
        evolves_into: String,
        evolution_count: u32,
    ) -> Self {
        Self {
            id,
            name,
            color,
            tier,
            creature_type,
            level: 1,
            kills: 0,
            kills_for_next_level,
            max_level,
            evolves_into,
            evolution_count,
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

/// Projectile behavior type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ProjectileType {
    /// Standard square projectile
    #[default]
    Basic,
    /// Thin rectangle that rotates in travel direction (visual only, penetration handles mechanic)
    Piercing,
    /// On final hit, deals AoE damage to nearby enemies
    Explosive,
    /// Curves toward nearest enemy
    Homing,
    /// On hit, redirects toward nearby enemy (chain count = penetration)
    Chain,
}

impl ProjectileType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "basic" => ProjectileType::Basic,
            "piercing" => ProjectileType::Piercing,
            "explosive" => ProjectileType::Explosive,
            "homing" => ProjectileType::Homing,
            "chain" => ProjectileType::Chain,
            _ => ProjectileType::Basic,
        }
    }
}

/// Projectile configuration for creatures
/// Controls projectile count, spread, size, speed, penetration, and type
#[derive(Component, Clone, Debug)]
pub struct ProjectileConfig {
    /// Number of projectiles to fire per attack
    pub count: u32,
    /// Spread angle in radians (total arc for all projectiles)
    pub spread: f32,
    /// Projectile size in pixels
    pub size: f32,
    /// Projectile speed in pixels per second
    pub speed: f32,
    /// How many enemies the projectile can penetrate (hit) before despawning
    pub penetration: u32,
    /// Projectile behavior type
    pub projectile_type: ProjectileType,
}

impl Default for ProjectileConfig {
    fn default() -> Self {
        Self {
            count: 1,
            spread: 0.0,
            size: 8.0,
            speed: 500.0,
            penetration: 1,
            projectile_type: ProjectileType::Basic,
        }
    }
}

impl ProjectileConfig {
    pub fn new(count: u32, spread: f32, size: f32, speed: f32, penetration: u32, projectile_type: ProjectileType) -> Self {
        Self { count, spread, size, speed, penetration, projectile_type }
    }
}

/// Animation state for sprite-based creatures
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CreatureAnimationState {
    #[default]
    Idle,
    Walking,
    Dying,
    Dead,
}

/// Sprite animation controller for creatures with spritesheets
#[derive(Component)]
pub struct CreatureAnimation {
    /// Current animation state
    pub state: CreatureAnimationState,
    /// Timer for advancing animation frames
    pub frame_timer: Timer,
    /// Current frame index in the spritesheet
    pub current_frame: usize,
    /// Timer for ash pile persistence after death (only used in Dead state)
    pub ash_timer: Option<Timer>,
}

impl CreatureAnimation {
    /// Create a new animation in idle state (frame 0)
    pub fn new() -> Self {
        Self {
            state: CreatureAnimationState::Idle,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating), // 100ms for walk
            current_frame: 0,
            ash_timer: None,
        }
    }

    /// Transition to walking animation (frames 1-4)
    pub fn start_walking(&mut self) {
        if self.state != CreatureAnimationState::Dying && self.state != CreatureAnimationState::Dead {
            self.state = CreatureAnimationState::Walking;
            if self.current_frame < 1 || self.current_frame > 4 {
                self.current_frame = 1;
            }
            self.frame_timer = Timer::from_seconds(0.1, TimerMode::Repeating); // 100ms per frame
        }
    }

    /// Transition to idle animation (frame 0)
    pub fn go_idle(&mut self) {
        if self.state != CreatureAnimationState::Dying && self.state != CreatureAnimationState::Dead {
            self.state = CreatureAnimationState::Idle;
            self.current_frame = 0;
        }
    }

    /// Transition to dying animation (frames 5-6-7)
    pub fn start_dying(&mut self) {
        self.state = CreatureAnimationState::Dying;
        self.current_frame = 5;
        self.frame_timer = Timer::from_seconds(0.18, TimerMode::Repeating); // 180ms per frame
    }

    /// Transition to dead state (ash pile, frame 7)
    pub fn become_dead(&mut self, ash_duration: f32) {
        self.state = CreatureAnimationState::Dead;
        self.current_frame = 7;
        self.ash_timer = Some(Timer::from_seconds(ash_duration, TimerMode::Once));
    }
}

impl Default for CreatureAnimation {
    fn default() -> Self {
        Self::new()
    }
}

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
            10,   // kills_for_next_level
            10,   // max_level
            "evolved_test".to_string(), // evolves_into
            3,    // evolution_count
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
            10, 10, "".to_string(), 3,
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
            10, 10, "".to_string(), 3,
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
            10, 10, "".to_string(), 3,
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
            25,    // kills_for_next_level
            10,    // max_level
            "flame_fiend".to_string(), // evolves_into
            3,     // evolution_count
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
        assert_eq!(stats.kills_for_next_level, 25);
        assert_eq!(stats.max_level, 10);
        assert_eq!(stats.evolves_into, "flame_fiend");
        assert_eq!(stats.evolution_count, 3);
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

    // =========================================================================
    // ProjectileConfig Tests
    // =========================================================================

    #[test]
    fn projectile_config_default_values() {
        let config = ProjectileConfig::default();
        assert_eq!(config.count, 1);
        assert_eq!(config.spread, 0.0);
        assert_eq!(config.size, 8.0);
        assert_eq!(config.speed, 500.0);
        assert_eq!(config.penetration, 1);
        assert_eq!(config.projectile_type, ProjectileType::Basic);
    }

    #[test]
    fn projectile_config_new_preserves_values() {
        let config = ProjectileConfig::new(3, 0.5, 12.0, 600.0, 5, ProjectileType::Explosive);
        assert_eq!(config.count, 3);
        assert_eq!(config.spread, 0.5);
        assert_eq!(config.size, 12.0);
        assert_eq!(config.speed, 600.0);
        assert_eq!(config.penetration, 5);
        assert_eq!(config.projectile_type, ProjectileType::Explosive);
    }

    #[test]
    fn projectile_config_clone_works() {
        let config = ProjectileConfig::new(5, 1.0, 10.0, 400.0, 3, ProjectileType::Homing);
        let cloned = config.clone();
        assert_eq!(cloned.count, config.count);
        assert_eq!(cloned.spread, config.spread);
        assert_eq!(cloned.size, config.size);
        assert_eq!(cloned.speed, config.speed);
        assert_eq!(cloned.penetration, config.penetration);
        assert_eq!(cloned.projectile_type, config.projectile_type);
    }

    // =========================================================================
    // ProjectileType Tests
    // =========================================================================

    #[test]
    fn projectile_type_from_str_parses_all_types() {
        assert_eq!(ProjectileType::from_str("basic"), ProjectileType::Basic);
        assert_eq!(ProjectileType::from_str("piercing"), ProjectileType::Piercing);
        assert_eq!(ProjectileType::from_str("explosive"), ProjectileType::Explosive);
        assert_eq!(ProjectileType::from_str("homing"), ProjectileType::Homing);
        assert_eq!(ProjectileType::from_str("chain"), ProjectileType::Chain);
    }

    #[test]
    fn projectile_type_from_str_is_case_insensitive() {
        assert_eq!(ProjectileType::from_str("BASIC"), ProjectileType::Basic);
        assert_eq!(ProjectileType::from_str("Basic"), ProjectileType::Basic);
        assert_eq!(ProjectileType::from_str("EXPLOSIVE"), ProjectileType::Explosive);
        assert_eq!(ProjectileType::from_str("Homing"), ProjectileType::Homing);
    }

    #[test]
    fn projectile_type_from_str_defaults_to_basic_for_unknown() {
        assert_eq!(ProjectileType::from_str("unknown"), ProjectileType::Basic);
        assert_eq!(ProjectileType::from_str(""), ProjectileType::Basic);
        assert_eq!(ProjectileType::from_str("laser"), ProjectileType::Basic);
    }

    #[test]
    fn projectile_type_default_is_basic() {
        assert_eq!(ProjectileType::default(), ProjectileType::Basic);
    }
}
