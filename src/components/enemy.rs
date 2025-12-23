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

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // EnemyClass Tests
    // =========================================================================

    #[test]
    fn enemy_class_from_str_parses_all_classes() {
        assert_eq!(EnemyClass::from_str("fodder"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("elite"), EnemyClass::Elite);
        assert_eq!(EnemyClass::from_str("miniboss"), EnemyClass::Miniboss);
        assert_eq!(EnemyClass::from_str("boss"), EnemyClass::Boss);
    }

    #[test]
    fn enemy_class_from_str_is_case_insensitive() {
        assert_eq!(EnemyClass::from_str("FODDER"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("Fodder"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("ELITE"), EnemyClass::Elite);
        assert_eq!(EnemyClass::from_str("Elite"), EnemyClass::Elite);
        assert_eq!(EnemyClass::from_str("BOSS"), EnemyClass::Boss);
    }

    #[test]
    fn enemy_class_from_str_defaults_to_fodder_for_unknown() {
        assert_eq!(EnemyClass::from_str("unknown"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str(""), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("legendary"), EnemyClass::Fodder);
    }

    #[test]
    fn enemy_class_default_is_fodder() {
        assert_eq!(EnemyClass::default(), EnemyClass::Fodder);
    }

    // =========================================================================
    // EnemyType Tests
    // =========================================================================

    #[test]
    fn enemy_type_from_str_parses_all_types() {
        assert_eq!(EnemyType::from_str("melee"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("ranged"), EnemyType::Ranged);
        assert_eq!(EnemyType::from_str("fast"), EnemyType::Fast);
        assert_eq!(EnemyType::from_str("tank"), EnemyType::Tank);
        assert_eq!(EnemyType::from_str("healer"), EnemyType::Healer);
        assert_eq!(EnemyType::from_str("commander"), EnemyType::Commander);
    }

    #[test]
    fn enemy_type_from_str_is_case_insensitive() {
        assert_eq!(EnemyType::from_str("MELEE"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("Melee"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("RANGED"), EnemyType::Ranged);
        assert_eq!(EnemyType::from_str("TANK"), EnemyType::Tank);
    }

    #[test]
    fn enemy_type_from_str_defaults_to_melee_for_unknown() {
        assert_eq!(EnemyType::from_str("unknown"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str(""), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("assassin"), EnemyType::Melee);
    }

    #[test]
    fn enemy_type_default_is_melee() {
        assert_eq!(EnemyType::default(), EnemyType::Melee);
    }

    // =========================================================================
    // EnemyStats Tests
    // =========================================================================

    #[test]
    fn enemy_stats_new_sets_current_hp_equal_to_base_hp() {
        let stats = EnemyStats::new(
            "goblin".to_string(),
            "Goblin".to_string(),
            EnemyClass::Fodder,
            EnemyType::Melee,
            30.0,  // base_hp
            5.0,   // base_damage
            1.0,   // attack_speed
            80.0,  // movement_speed
            40.0,  // attack_range
        );
        assert_eq!(stats.current_hp, 30.0);
        assert_eq!(stats.current_hp, stats.base_hp);
    }

    #[test]
    fn enemy_stats_new_preserves_all_input_values() {
        let stats = EnemyStats::new(
            "orc_warrior".to_string(),
            "Orc Warrior".to_string(),
            EnemyClass::Elite,
            EnemyType::Tank,
            200.0, // base_hp
            15.0,  // base_damage
            0.8,   // attack_speed
            60.0,  // movement_speed
            50.0,  // attack_range
        );
        assert_eq!(stats.id, "orc_warrior");
        assert_eq!(stats.name, "Orc Warrior");
        assert_eq!(stats.enemy_class, EnemyClass::Elite);
        assert_eq!(stats.enemy_type, EnemyType::Tank);
        assert_eq!(stats.base_hp, 200.0);
        assert_eq!(stats.base_damage, 15.0);
        assert_eq!(stats.attack_speed, 0.8);
        assert_eq!(stats.movement_speed, 60.0);
        assert_eq!(stats.attack_range, 50.0);
    }

    // =========================================================================
    // EnemyAttackTimer Tests
    // =========================================================================

    #[test]
    fn enemy_attack_timer_calculates_duration_from_attack_speed() {
        // attack_speed = 1.0 means 1 attack per second, so duration = 1.0
        let timer = EnemyAttackTimer::new(1.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);

        // attack_speed = 2.0 means 2 attacks per second, so duration = 0.5
        let timer = EnemyAttackTimer::new(2.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 0.5);

        // attack_speed = 0.5 means 0.5 attacks per second, so duration = 2.0
        let timer = EnemyAttackTimer::new(0.5);
        assert_eq!(timer.timer.duration().as_secs_f32(), 2.0);
    }

    #[test]
    fn enemy_attack_timer_defaults_to_1_second_for_zero_attack_speed() {
        let timer = EnemyAttackTimer::new(0.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);
    }

    #[test]
    fn enemy_attack_timer_defaults_to_1_second_for_negative_attack_speed() {
        let timer = EnemyAttackTimer::new(-1.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);
    }

    #[test]
    fn enemy_attack_timer_is_repeating() {
        let timer = EnemyAttackTimer::new(1.0);
        assert_eq!(timer.timer.mode(), TimerMode::Repeating);
    }
}
