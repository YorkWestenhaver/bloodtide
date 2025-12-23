use bevy::prelude::*;

use crate::components::CreatureColor;

/// Marker component for weapon entities
#[derive(Component)]
pub struct Weapon;

/// Weapon identification and affinity data
#[derive(Component, Clone, Debug)]
pub struct WeaponData {
    pub id: String,
    pub name: String,
    pub color: CreatureColor,
    pub tier: u8,
    pub affinity_amount: f64,
}

impl WeaponData {
    pub fn new(
        id: String,
        name: String,
        color: CreatureColor,
        tier: u8,
        affinity_amount: f64,
    ) -> Self {
        Self {
            id,
            name,
            color,
            tier,
            affinity_amount,
        }
    }
}

/// Weapon combat stats
#[derive(Component, Clone, Debug)]
pub struct WeaponStats {
    pub auto_damage: f64,
    pub auto_speed: f64,
    pub auto_range: f64,
    pub projectile_count: u32,
    pub projectile_pattern: String,
    pub projectile_speed: f64,
    pub projectile_size: f32,
    pub projectile_penetration: u32,
}

impl WeaponStats {
    pub fn new(
        auto_damage: f64,
        auto_speed: f64,
        auto_range: f64,
        projectile_count: u32,
        projectile_pattern: String,
        projectile_speed: f64,
        projectile_size: f32,
        projectile_penetration: u32,
    ) -> Self {
        Self {
            auto_damage,
            auto_speed,
            auto_range,
            projectile_count,
            projectile_pattern,
            projectile_speed,
            projectile_size,
            projectile_penetration,
        }
    }
}

/// Weapon attack timer component
#[derive(Component)]
pub struct WeaponAttackTimer {
    pub timer: Timer,
}

impl WeaponAttackTimer {
    pub fn new(attack_speed: f64) -> Self {
        // Attack speed is attacks per second, so timer duration = 1 / attack_speed
        let duration = if attack_speed > 0.0 {
            1.0 / attack_speed
        } else {
            1.0 // Default to 1 second if speed is 0 or negative
        };

        Self {
            timer: Timer::from_seconds(duration as f32, TimerMode::Repeating),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_data_new_preserves_values() {
        let data = WeaponData::new(
            "ember_staff".to_string(),
            "Ember Staff".to_string(),
            CreatureColor::Red,
            1,
            10.0,
        );
        assert_eq!(data.id, "ember_staff");
        assert_eq!(data.name, "Ember Staff");
        assert_eq!(data.color, CreatureColor::Red);
        assert_eq!(data.tier, 1);
        assert_eq!(data.affinity_amount, 10.0);
    }

    #[test]
    fn weapon_stats_new_preserves_values() {
        let stats = WeaponStats::new(8.0, 1.5, 250.0, 1, "single".to_string(), 300.0, 10.0, 1);
        assert_eq!(stats.auto_damage, 8.0);
        assert_eq!(stats.auto_speed, 1.5);
        assert_eq!(stats.auto_range, 250.0);
        assert_eq!(stats.projectile_count, 1);
        assert_eq!(stats.projectile_pattern, "single");
        assert_eq!(stats.projectile_speed, 300.0);
        assert_eq!(stats.projectile_size, 10.0);
        assert_eq!(stats.projectile_penetration, 1);
    }

    #[test]
    fn weapon_attack_timer_calculates_duration_from_attack_speed() {
        // 2.0 attacks per second = 0.5 second timer
        let timer = WeaponAttackTimer::new(2.0);
        assert!((timer.timer.duration().as_secs_f32() - 0.5).abs() < 0.001);
    }

    #[test]
    fn weapon_attack_timer_defaults_to_1_second_for_zero_attack_speed() {
        let timer = WeaponAttackTimer::new(0.0);
        assert!((timer.timer.duration().as_secs_f32() - 1.0).abs() < 0.001);
    }

    #[test]
    fn weapon_attack_timer_is_repeating() {
        let timer = WeaponAttackTimer::new(1.0);
        assert_eq!(timer.timer.mode(), TimerMode::Repeating);
    }
}
