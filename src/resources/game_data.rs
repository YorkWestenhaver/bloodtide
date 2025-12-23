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
