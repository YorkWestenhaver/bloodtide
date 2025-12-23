use serde::Deserialize;

// =============================================================================
// CREATURE DATA
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct Creature {
    pub id: String,
    pub name: String,
    pub color: String,
    pub tier: u8,
    pub creature_type: String,
    pub base_damage: f64,
    pub attack_speed: f64,
    pub base_hp: f64,
    pub movement_speed: f64,
    pub attack_range: f64,
    pub crit_t1: f64,
    pub crit_t2: f64,
    pub crit_t3: f64,
    pub evolves_from: String,
    pub evolves_into: String,
    pub evolution_count: u32,
    pub kills_per_level: Vec<u32>,
    pub max_level: u32,
    pub abilities: Vec<String>,
    pub respawn_time: f64,
    pub description: String,
    // Projectile configuration (optional, defaults to 1/0.0/8.0/500.0)
    #[serde(default = "default_projectile_count")]
    pub projectile_count: u32,
    #[serde(default)]
    pub projectile_spread: f32,
    #[serde(default = "default_projectile_size")]
    pub projectile_size: f32,
    #[serde(default = "default_projectile_speed")]
    pub projectile_speed: f32,
}

fn default_projectile_count() -> u32 { 1 }
fn default_projectile_size() -> f32 { 8.0 }
fn default_projectile_speed() -> f32 { 500.0 }

#[derive(Debug, Clone, Deserialize)]
pub struct CreaturesFile {
    pub creatures: Vec<Creature>,
}

// =============================================================================
// WEAPON DATA
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub color: String,
    pub tier: u8,
    pub affinity_amount: f64,
    pub auto_damage: f64,
    pub auto_speed: f64,
    pub auto_range: f64,
    pub projectile_count: u32,
    pub projectile_pattern: String,
    pub projectile_speed: f64,
    pub evolves_from: Vec<String>,
    pub evolves_into: String,
    pub evolution_recipe: Vec<String>,
    pub passive_effect: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeaponsFile {
    pub weapons: Vec<Weapon>,
}

// =============================================================================
// ARTIFACT DATA
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub tier: u8,
    pub target_scope: String,
    pub target_color: String,
    pub target_type: String,
    pub target_creature: String,
    pub damage_bonus: f64,
    pub attack_speed_bonus: f64,
    pub hp_bonus: f64,
    pub crit_t1_bonus: f64,
    pub crit_t2_bonus: f64,
    pub crit_t3_bonus: f64,
    pub crit_damage_bonus: f64,
    pub special_effect: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactsFile {
    pub artifacts: Vec<Artifact>,
}

// =============================================================================
// ENEMY DATA
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct Enemy {
    pub id: String,
    pub name: String,
    pub enemy_class: String,
    pub enemy_type: String,
    pub color_resist: String,
    pub color_weak: String,
    pub base_hp: f64,
    pub base_damage: f64,
    pub attack_speed: f64,
    pub movement_speed: f64,
    pub attack_range: f64,
    pub ai_type: String,
    pub targets_creatures: bool,
    pub min_wave: u32,
    pub spawn_weight: f64,
    pub group_size_min: u32,
    pub group_size_max: u32,
    pub xp_value: u32,
    pub phases: u32,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnemiesFile {
    pub enemies: Vec<Enemy>,
}

// =============================================================================
// AFFINITY DATA
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct AffinityThreshold {
    pub min: u32,
    pub damage_bonus: f64,
    pub attack_speed_bonus: f64,
    pub hp_bonus: f64,
    pub crit_t1_bonus: f64,
    pub crit_t2_unlock: bool,
    pub crit_t3_unlock: bool,
    pub special: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AffinityColor {
    pub color: String,
    pub overflow_bonus_per_point: f64,
    pub thresholds: Vec<AffinityThreshold>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AffinityFile {
    pub affinity_colors: Vec<AffinityColor>,
}
