use bevy::prelude::*;

/// Marker component for enemy entities
#[derive(Component)]
pub struct Enemy;

/// Enemy class/tier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnemyClass {
    #[default]
    Fodder,
    Elite,
    Miniboss,
    Boss,
}

impl EnemyClass {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fodder" => EnemyClass::Fodder,
            "elite" => EnemyClass::Elite,
            "miniboss" => EnemyClass::Miniboss,
            "boss" => EnemyClass::Boss,
            _ => EnemyClass::Fodder,
        }
    }
}

/// Enemy behavior type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnemyType {
    #[default]
    Melee,
    Ranged,
    Fast,
    Tank,
    Healer,
    Commander,
}

impl EnemyType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "melee" => EnemyType::Melee,
            "ranged" => EnemyType::Ranged,
            "fast" => EnemyType::Fast,
            "tank" => EnemyType::Tank,
            "healer" => EnemyType::Healer,
            "commander" => EnemyType::Commander,
            _ => EnemyType::Melee,
        }
    }
}

/// Runtime data for an enemy entity
#[derive(Component, Clone, Debug)]
pub struct EnemyStats {
    pub id: String,
    pub name: String,
    pub enemy_class: EnemyClass,
    pub enemy_type: EnemyType,
    pub base_hp: f64,
    pub current_hp: f64,
    pub base_damage: f64,
    pub attack_speed: f64,
    pub movement_speed: f64,
    pub attack_range: f64,
}

impl EnemyStats {
    pub fn new(
        id: String,
        name: String,
        enemy_class: EnemyClass,
        enemy_type: EnemyType,
        base_hp: f64,
        base_damage: f64,
        attack_speed: f64,
        movement_speed: f64,
        attack_range: f64,
    ) -> Self {
        Self {
            id,
            name,
            enemy_class,
            enemy_type,
            base_hp,
            current_hp: base_hp,
            base_damage,
            attack_speed,
            movement_speed,
            attack_range,
        }
    }
}

/// Attack cooldown timer for enemies
#[derive(Component)]
pub struct EnemyAttackTimer {
    pub timer: Timer,
}

impl EnemyAttackTimer {
    pub fn new(attack_speed: f64) -> Self {
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
