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
