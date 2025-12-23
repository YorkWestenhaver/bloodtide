use bevy::prelude::*;
use std::fs;
use std::path::Path;

use crate::data::{
    AffinityColor, AffinityFile, Artifact, ArtifactsFile, Creature, CreaturesFile, EnemiesFile,
    Enemy, Weapon, WeaponsFile,
};

#[derive(Resource)]
pub struct GameData {
    pub creatures: Vec<Creature>,
    pub weapons: Vec<Weapon>,
    pub artifacts: Vec<Artifact>,
    pub enemies: Vec<Enemy>,
    pub affinity_colors: Vec<AffinityColor>,
}

impl GameData {
    pub fn new() -> Self {
        Self {
            creatures: Vec::new(),
            weapons: Vec::new(),
            artifacts: Vec::new(),
            enemies: Vec::new(),
            affinity_colors: Vec::new(),
        }
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self::new()
    }
}

/// Load all game data from TOML files in the assets/data directory
pub fn load_game_data() -> Result<GameData, String> {
    let base_path = Path::new("assets/data");

    // Load creatures
    let creatures_path = base_path.join("creatures.toml");
    let creatures_content = fs::read_to_string(&creatures_path)
        .map_err(|e| format!("Failed to read creatures.toml: {}", e))?;
    let creatures_file: CreaturesFile = toml::from_str(&creatures_content)
        .map_err(|e| format!("Failed to parse creatures.toml: {}", e))?;

    // Load weapons
    let weapons_path = base_path.join("weapons.toml");
    let weapons_content = fs::read_to_string(&weapons_path)
        .map_err(|e| format!("Failed to read weapons.toml: {}", e))?;
    let weapons_file: WeaponsFile = toml::from_str(&weapons_content)
        .map_err(|e| format!("Failed to parse weapons.toml: {}", e))?;

    // Load artifacts
    let artifacts_path = base_path.join("artifacts.toml");
    let artifacts_content = fs::read_to_string(&artifacts_path)
        .map_err(|e| format!("Failed to read artifacts.toml: {}", e))?;
    let artifacts_file: ArtifactsFile = toml::from_str(&artifacts_content)
        .map_err(|e| format!("Failed to parse artifacts.toml: {}", e))?;

    // Load enemies
    let enemies_path = base_path.join("enemies.toml");
    let enemies_content = fs::read_to_string(&enemies_path)
        .map_err(|e| format!("Failed to read enemies.toml: {}", e))?;
    let enemies_file: EnemiesFile = toml::from_str(&enemies_content)
        .map_err(|e| format!("Failed to parse enemies.toml: {}", e))?;

    // Load affinity
    let affinity_path = base_path.join("affinity.toml");
    let affinity_content = fs::read_to_string(&affinity_path)
        .map_err(|e| format!("Failed to read affinity.toml: {}", e))?;
    let affinity_file: AffinityFile = toml::from_str(&affinity_content)
        .map_err(|e| format!("Failed to parse affinity.toml: {}", e))?;

    Ok(GameData {
        creatures: creatures_file.creatures,
        weapons: weapons_file.weapons,
        artifacts: artifacts_file.artifacts,
        enemies: enemies_file.enemies,
        affinity_colors: affinity_file.affinity_colors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GameData Basic Tests
    // =========================================================================

    #[test]
    fn game_data_new_creates_empty_collections() {
        let data = GameData::new();
        assert!(data.creatures.is_empty());
        assert!(data.weapons.is_empty());
        assert!(data.artifacts.is_empty());
        assert!(data.enemies.is_empty());
        assert!(data.affinity_colors.is_empty());
    }

    #[test]
    fn game_data_default_is_same_as_new() {
        let new_data = GameData::new();
        let default_data = GameData::default();
        assert_eq!(new_data.creatures.len(), default_data.creatures.len());
        assert_eq!(new_data.weapons.len(), default_data.weapons.len());
        assert_eq!(new_data.artifacts.len(), default_data.artifacts.len());
        assert_eq!(new_data.enemies.len(), default_data.enemies.len());
        assert_eq!(new_data.affinity_colors.len(), default_data.affinity_colors.len());
    }

    // =========================================================================
    // TOML Loading Integration Tests
    // Note: These tests require the actual TOML files to exist
    // =========================================================================

    #[test]
    fn load_game_data_succeeds() {
        // This test requires running from the project root
        let result = load_game_data();
        assert!(result.is_ok(), "Failed to load game data: {:?}", result.err());
    }

    #[test]
    fn loaded_creatures_are_not_empty() {
        let data = load_game_data().expect("Failed to load game data");
        assert!(!data.creatures.is_empty(), "No creatures were loaded");
    }

    #[test]
    fn loaded_weapons_are_not_empty() {
        let data = load_game_data().expect("Failed to load game data");
        assert!(!data.weapons.is_empty(), "No weapons were loaded");
    }

    #[test]
    fn loaded_artifacts_are_not_empty() {
        let data = load_game_data().expect("Failed to load game data");
        assert!(!data.artifacts.is_empty(), "No artifacts were loaded");
    }

    #[test]
    fn loaded_enemies_are_not_empty() {
        let data = load_game_data().expect("Failed to load game data");
        assert!(!data.enemies.is_empty(), "No enemies were loaded");
    }

    #[test]
    fn loaded_affinity_colors_are_not_empty() {
        let data = load_game_data().expect("Failed to load game data");
        assert!(!data.affinity_colors.is_empty(), "No affinity colors were loaded");
    }

    // =========================================================================
    // Creature Data Validation Tests
    // =========================================================================

    #[test]
    fn fire_imp_exists_with_correct_stats() {
        let data = load_game_data().expect("Failed to load game data");
        let fire_imp = data.creatures.iter().find(|c| c.id == "fire_imp");

        assert!(fire_imp.is_some(), "fire_imp creature not found");
        let fire_imp = fire_imp.unwrap();

        assert_eq!(fire_imp.name, "Fire Imp");
        assert_eq!(fire_imp.color, "red");
        assert_eq!(fire_imp.tier, 1);
        assert_eq!(fire_imp.creature_type, "ranged");
        assert!(fire_imp.base_damage > 0.0);
        assert!(fire_imp.attack_speed > 0.0);
        assert!(fire_imp.base_hp > 0.0);
    }

    #[test]
    fn all_creatures_have_valid_ids() {
        let data = load_game_data().expect("Failed to load game data");
        for creature in &data.creatures {
            assert!(!creature.id.is_empty(), "Creature has empty id");
            assert!(!creature.name.is_empty(), "Creature {} has empty name", creature.id);
        }
    }

    #[test]
    fn all_creatures_have_valid_stats() {
        let data = load_game_data().expect("Failed to load game data");
        for creature in &data.creatures {
            assert!(creature.base_hp > 0.0, "Creature {} has invalid base_hp", creature.id);
            assert!(creature.attack_speed > 0.0, "Creature {} has invalid attack_speed", creature.id);
            assert!(creature.tier >= 1 && creature.tier <= 5, "Creature {} has invalid tier", creature.id);
        }
    }

    // =========================================================================
    // Enemy Data Validation Tests
    // =========================================================================

    #[test]
    fn goblin_exists_with_correct_stats() {
        let data = load_game_data().expect("Failed to load game data");
        let goblin = data.enemies.iter().find(|e| e.id == "goblin");

        assert!(goblin.is_some(), "goblin enemy not found");
        let goblin = goblin.unwrap();

        assert_eq!(goblin.name, "Goblin");
        assert_eq!(goblin.enemy_class, "fodder");
        assert_eq!(goblin.enemy_type, "melee");
        assert!(goblin.base_hp > 0.0);
        assert!(goblin.base_damage > 0.0);
    }

    #[test]
    fn all_enemies_have_valid_ids() {
        let data = load_game_data().expect("Failed to load game data");
        for enemy in &data.enemies {
            assert!(!enemy.id.is_empty(), "Enemy has empty id");
            assert!(!enemy.name.is_empty(), "Enemy {} has empty name", enemy.id);
        }
    }

    #[test]
    fn all_enemies_have_valid_stats() {
        let data = load_game_data().expect("Failed to load game data");
        for enemy in &data.enemies {
            assert!(enemy.base_hp > 0.0, "Enemy {} has invalid base_hp", enemy.id);
            assert!(enemy.movement_speed > 0.0, "Enemy {} has invalid movement_speed", enemy.id);
        }
    }

    // =========================================================================
    // Weapon Data Validation Tests
    // =========================================================================

    #[test]
    fn ember_staff_exists() {
        let data = load_game_data().expect("Failed to load game data");
        let ember_staff = data.weapons.iter().find(|w| w.id == "ember_staff");
        assert!(ember_staff.is_some(), "ember_staff weapon not found");
    }

    #[test]
    fn all_weapons_have_valid_ids() {
        let data = load_game_data().expect("Failed to load game data");
        for weapon in &data.weapons {
            assert!(!weapon.id.is_empty(), "Weapon has empty id");
            assert!(!weapon.name.is_empty(), "Weapon {} has empty name", weapon.id);
        }
    }

    // =========================================================================
    // Artifact Data Validation Tests
    // =========================================================================

    #[test]
    fn molten_core_exists() {
        let data = load_game_data().expect("Failed to load game data");
        let molten_core = data.artifacts.iter().find(|a| a.id == "molten_core");
        assert!(molten_core.is_some(), "molten_core artifact not found");
    }

    #[test]
    fn all_artifacts_have_valid_ids() {
        let data = load_game_data().expect("Failed to load game data");
        for artifact in &data.artifacts {
            assert!(!artifact.id.is_empty(), "Artifact has empty id");
            assert!(!artifact.name.is_empty(), "Artifact {} has empty name", artifact.id);
        }
    }

    // =========================================================================
    // Affinity Data Validation Tests
    // =========================================================================

    #[test]
    fn red_affinity_exists() {
        let data = load_game_data().expect("Failed to load game data");
        let red = data.affinity_colors.iter().find(|a| a.color == "red");
        assert!(red.is_some(), "red affinity color not found");
    }

    #[test]
    fn affinity_colors_have_thresholds() {
        let data = load_game_data().expect("Failed to load game data");
        for affinity in &data.affinity_colors {
            assert!(
                !affinity.thresholds.is_empty(),
                "Affinity {} has no thresholds",
                affinity.color
            );
        }
    }
}
