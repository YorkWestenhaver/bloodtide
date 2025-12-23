use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{CreatureColor, CreatureType};
use crate::resources::GameData;

/// Bonus stats that can be applied to creatures
#[derive(Clone, Debug, Default)]
pub struct StatBonuses {
    pub damage_bonus: f64,
    pub attack_speed_bonus: f64,
    pub hp_bonus: f64,
    pub crit_t1_bonus: f64,
    pub crit_t2_bonus: f64,
    pub crit_t3_bonus: f64,
}

impl StatBonuses {
    /// Add another set of bonuses to this one
    pub fn add(&mut self, other: &StatBonuses) {
        self.damage_bonus += other.damage_bonus;
        self.attack_speed_bonus += other.attack_speed_bonus;
        self.hp_bonus += other.hp_bonus;
        self.crit_t1_bonus += other.crit_t1_bonus;
        self.crit_t2_bonus += other.crit_t2_bonus;
        self.crit_t3_bonus += other.crit_t3_bonus;
    }
}

/// Resource tracking all active artifact effects
#[derive(Resource, Debug, Default)]
pub struct ArtifactBuffs {
    /// Global bonuses that apply to all creatures
    pub global: StatBonuses,
    /// Bonuses that apply to creatures of a specific color
    pub color_bonuses: HashMap<CreatureColor, StatBonuses>,
    /// Bonuses that apply to creatures of a specific type
    pub type_bonuses: HashMap<CreatureType, StatBonuses>,
    /// Bonuses that apply to specific creatures (by id)
    pub creature_bonuses: HashMap<String, StatBonuses>,
    /// List of acquired artifact ids (for UI display)
    pub acquired_artifacts: Vec<String>,
}

impl ArtifactBuffs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply an artifact's bonuses based on its target scope
    pub fn apply_artifact(&mut self, game_data: &GameData, artifact_id: &str) {
        // Find the artifact data
        let Some(artifact) = game_data.artifacts.iter().find(|a| a.id == artifact_id) else {
            return;
        };

        // Create bonuses from artifact data
        let bonuses = StatBonuses {
            damage_bonus: artifact.damage_bonus,
            attack_speed_bonus: artifact.attack_speed_bonus,
            hp_bonus: artifact.hp_bonus,
            crit_t1_bonus: artifact.crit_t1_bonus,
            crit_t2_bonus: artifact.crit_t2_bonus,
            crit_t3_bonus: artifact.crit_t3_bonus,
        };

        // Apply to appropriate bucket based on target_scope
        match artifact.target_scope.as_str() {
            "global" => {
                self.global.add(&bonuses);
            }
            "color" => {
                let color = CreatureColor::from_str(&artifact.target_color);
                self.color_bonuses
                    .entry(color)
                    .or_default()
                    .add(&bonuses);
            }
            "type" => {
                let creature_type = CreatureType::from_str(&artifact.target_type);
                self.type_bonuses
                    .entry(creature_type)
                    .or_default()
                    .add(&bonuses);
            }
            "creature" => {
                self.creature_bonuses
                    .entry(artifact.target_creature.clone())
                    .or_default()
                    .add(&bonuses);
            }
            _ => {
                // Default to global for unknown scopes
                self.global.add(&bonuses);
            }
        }

        // Track the acquired artifact
        self.acquired_artifacts.push(artifact_id.to_string());
    }

    /// Get total combined bonuses for a specific creature
    pub fn get_total_bonuses(
        &self,
        creature_id: &str,
        color: CreatureColor,
        creature_type: CreatureType,
    ) -> StatBonuses {
        let mut total = StatBonuses::default();

        // Add global bonuses
        total.add(&self.global);

        // Add color-specific bonuses
        if let Some(color_bonus) = self.color_bonuses.get(&color) {
            total.add(color_bonus);
        }

        // Add type-specific bonuses
        if let Some(type_bonus) = self.type_bonuses.get(&creature_type) {
            total.add(type_bonus);
        }

        // Add creature-specific bonuses
        if let Some(creature_bonus) = self.creature_bonuses.get(creature_id) {
            total.add(creature_bonus);
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_bonuses_default_is_zero() {
        let bonuses = StatBonuses::default();
        assert_eq!(bonuses.damage_bonus, 0.0);
        assert_eq!(bonuses.attack_speed_bonus, 0.0);
        assert_eq!(bonuses.hp_bonus, 0.0);
        assert_eq!(bonuses.crit_t1_bonus, 0.0);
        assert_eq!(bonuses.crit_t2_bonus, 0.0);
        assert_eq!(bonuses.crit_t3_bonus, 0.0);
    }

    #[test]
    fn stat_bonuses_add_accumulates() {
        let mut a = StatBonuses {
            damage_bonus: 10.0,
            attack_speed_bonus: 5.0,
            hp_bonus: 20.0,
            crit_t1_bonus: 1.0,
            crit_t2_bonus: 0.5,
            crit_t3_bonus: 0.1,
        };
        let b = StatBonuses {
            damage_bonus: 15.0,
            attack_speed_bonus: 10.0,
            hp_bonus: 30.0,
            crit_t1_bonus: 2.0,
            crit_t2_bonus: 1.0,
            crit_t3_bonus: 0.2,
        };
        a.add(&b);

        assert_eq!(a.damage_bonus, 25.0);
        assert_eq!(a.attack_speed_bonus, 15.0);
        assert_eq!(a.hp_bonus, 50.0);
        assert_eq!(a.crit_t1_bonus, 3.0);
        assert_eq!(a.crit_t2_bonus, 1.5);
        // Use approximate comparison for floating point
        assert!((a.crit_t3_bonus - 0.3).abs() < 0.0001);
    }

    #[test]
    fn artifact_buffs_default_is_empty() {
        let buffs = ArtifactBuffs::default();
        assert_eq!(buffs.global.damage_bonus, 0.0);
        assert!(buffs.color_bonuses.is_empty());
        assert!(buffs.type_bonuses.is_empty());
        assert!(buffs.creature_bonuses.is_empty());
        assert!(buffs.acquired_artifacts.is_empty());
    }

    #[test]
    fn get_total_bonuses_with_no_bonuses_returns_zeros() {
        let buffs = ArtifactBuffs::default();
        let total = buffs.get_total_bonuses("fire_imp", CreatureColor::Red, CreatureType::Ranged);
        assert_eq!(total.damage_bonus, 0.0);
        assert_eq!(total.hp_bonus, 0.0);
    }

    #[test]
    fn get_total_bonuses_includes_global() {
        let mut buffs = ArtifactBuffs::default();
        buffs.global.damage_bonus = 10.0;

        let total = buffs.get_total_bonuses("fire_imp", CreatureColor::Red, CreatureType::Ranged);
        assert_eq!(total.damage_bonus, 10.0);
    }

    #[test]
    fn get_total_bonuses_includes_matching_color() {
        let mut buffs = ArtifactBuffs::default();
        buffs.global.damage_bonus = 10.0;
        buffs.color_bonuses.insert(
            CreatureColor::Red,
            StatBonuses {
                damage_bonus: 15.0,
                ..Default::default()
            },
        );
        buffs.color_bonuses.insert(
            CreatureColor::Blue,
            StatBonuses {
                damage_bonus: 20.0,
                ..Default::default()
            },
        );

        let red_total =
            buffs.get_total_bonuses("fire_imp", CreatureColor::Red, CreatureType::Ranged);
        assert_eq!(red_total.damage_bonus, 25.0); // 10 + 15

        let blue_total =
            buffs.get_total_bonuses("ice_sprite", CreatureColor::Blue, CreatureType::Ranged);
        assert_eq!(blue_total.damage_bonus, 30.0); // 10 + 20
    }

    #[test]
    fn get_total_bonuses_includes_matching_type() {
        let mut buffs = ArtifactBuffs::default();
        buffs.type_bonuses.insert(
            CreatureType::Ranged,
            StatBonuses {
                damage_bonus: 12.0,
                ..Default::default()
            },
        );

        let ranged_total =
            buffs.get_total_bonuses("fire_imp", CreatureColor::Red, CreatureType::Ranged);
        assert_eq!(ranged_total.damage_bonus, 12.0);

        let melee_total =
            buffs.get_total_bonuses("ember_hound", CreatureColor::Red, CreatureType::Melee);
        assert_eq!(melee_total.damage_bonus, 0.0);
    }

    #[test]
    fn get_total_bonuses_includes_matching_creature_id() {
        let mut buffs = ArtifactBuffs::default();
        buffs.creature_bonuses.insert(
            "fire_imp".to_string(),
            StatBonuses {
                damage_bonus: 50.0,
                ..Default::default()
            },
        );

        let fire_imp_total =
            buffs.get_total_bonuses("fire_imp", CreatureColor::Red, CreatureType::Ranged);
        assert_eq!(fire_imp_total.damage_bonus, 50.0);

        let other_total =
            buffs.get_total_bonuses("ember_hound", CreatureColor::Red, CreatureType::Melee);
        assert_eq!(other_total.damage_bonus, 0.0);
    }

    #[test]
    fn get_total_bonuses_combines_all_sources() {
        let mut buffs = ArtifactBuffs::default();
        buffs.global.damage_bonus = 10.0;
        buffs.color_bonuses.insert(
            CreatureColor::Red,
            StatBonuses {
                damage_bonus: 15.0,
                ..Default::default()
            },
        );
        buffs.type_bonuses.insert(
            CreatureType::Ranged,
            StatBonuses {
                damage_bonus: 20.0,
                ..Default::default()
            },
        );
        buffs.creature_bonuses.insert(
            "fire_imp".to_string(),
            StatBonuses {
                damage_bonus: 25.0,
                ..Default::default()
            },
        );

        let total = buffs.get_total_bonuses("fire_imp", CreatureColor::Red, CreatureType::Ranged);
        // 10 (global) + 15 (red) + 20 (ranged) + 25 (fire_imp) = 70
        assert_eq!(total.damage_bonus, 70.0);
    }
}
