# Bloodtide: Technical Specification
## For Claude Code Implementation

**Status Legend:** [IMPLEMENTED] | [PARTIAL] | [PLANNED]

---

# 1. Project Structure [IMPLEMENTED]

```
bloodtide/
├── Cargo.toml
├── src/
│   ├── main.rs                     # App init, system ordering, plugin setup
│   ├── components/
│   │   ├── mod.rs
│   │   ├── creature.rs             # CreatureStats, CreatureColor, CreatureType, ProjectileConfig
│   │   ├── enemy.rs                # EnemyStats, EnemyClass, EnemyType, animation state
│   │   ├── weapon.rs               # WeaponData, WeaponStats, WeaponAttackTimer
│   │   ├── player.rs               # Player marker component
│   │   └── death_animation.rs      # Death animation components
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── combat.rs               # Damage calculation, projectiles, attacks (47KB)
│   │   ├── spawning.rs             # Creature/enemy/weapon spawning
│   │   ├── leveling.rs             # Player level-ups, card rolling, effects
│   │   ├── creature_xp.rs          # Per-creature XP, evolution triggers
│   │   ├── deck_builder_ui.rs      # Pre-run deck composition UI
│   │   ├── debug_menu.rs           # Pause menu, debug controls
│   │   ├── ui_panels.rs            # HUD, creature panel, artifact panel
│   │   ├── ui.rs                   # Base UI setup
│   │   ├── ai.rs                   # Creature formation, enemy chase
│   │   ├── death.rs                # Death handling, respawn queue
│   │   ├── death_animation.rs      # Death visual effects
│   │   ├── animation.rs            # Sprite animation system
│   │   ├── hp_bars.rs              # Health bar rendering
│   │   ├── movement.rs             # Player/entity movement
│   │   ├── tilemap.rs              # Procedural tilemap chunks
│   │   └── tooltips.rs             # Hover tooltips
│   ├── resources/
│   │   ├── mod.rs
│   │   ├── game_data.rs            # Loaded TOML data container
│   │   ├── game_state.rs           # Kills, wave, level tracking
│   │   ├── director.rs             # Adaptive spawn AI
│   │   ├── deck.rs                 # Probability deck, card rolling
│   │   ├── deck_builder.rs         # Deck builder state
│   │   ├── affinity.rs             # Color affinity + thresholds
│   │   ├── artifact_buffs.rs       # Equipment bonus stacking
│   │   ├── spatial.rs              # Spatial grid for O(1) lookups
│   │   ├── pools.rs                # Object pooling (projectiles, damage numbers)
│   │   ├── debug_settings.rs       # GamePhase, debug toggles
│   │   ├── sprite_assets.rs        # Asset handles
│   │   └── tilemap.rs              # Tilemap chunk state
│   ├── math/
│   │   ├── mod.rs
│   │   └── crit.rs                 # 3-tier crit calculation
│   └── data/
│       └── mod.rs                  # TOML struct definitions
├── assets/
│   ├── data/
│   │   ├── creatures.toml          # 14 creatures (T1-T4)
│   │   ├── weapons.toml            # 13 weapons (T1-T4)
│   │   ├── artifacts.toml          # Stat buff items
│   │   ├── enemies.toml            # Enemy types
│   │   └── affinity.toml           # Color thresholds
│   ├── sprites/                    # Spritesheets
│   └── audio/                      # Placeholder audio
└── README.md
```

---

# 2. Data Schemas (TOML) [IMPLEMENTED]

## 2.1 Creature Schema

```toml
# assets/data/creatures.toml

[[creatures]]
id = "fire_imp"
name = "Fire Imp"
color = "red"                    # red|blue|green|white|black|colorless
tier = 1                         # 1-5 (evolution tier)
creature_type = "ranged"         # melee|ranged|support|assassin
base_damage = 15.0
attack_speed = 1.0               # attacks per second
base_hp = 50.0
movement_speed = 100.0
attack_range = 220.0
crit_t1 = 5.0                    # percentage (5.0 = 5%)
crit_t2 = 0.0
crit_t3 = 0.0
evolves_from = ""
evolves_into = "flame_fiend"
evolution_count = 3              # duplicates needed to evolve
kills_per_level = [10, 25, 50, 100, 150, 200, 300, 400, 500]
max_level = 10
abilities = ["fireball"]
respawn_time = 20.0              # seconds
description = "A small fire spirit that hurls fireballs at enemies."
projectile_count = 1
projectile_spread = 0.0          # radians
projectile_size = 8.0
projectile_speed = 500.0
projectile_penetration = 1       # enemies hit before despawn
projectile_type = "basic"        # basic|piercing|explosive|homing|chain
```

## 2.2 Weapon Schema

```toml
# assets/data/weapons.toml

[[weapons]]
id = "ember_staff"
name = "Ember Staff"
color = "red"
tier = 1
affinity_amount = 10.0           # contributes to color affinity
auto_damage = 8.0
auto_speed = 1.5                 # attacks per second
auto_range = 350.0
projectile_count = 3
projectile_pattern = "spread"    # single|spread|melee_sweep|melee_stab|homing|orbit
projectile_speed = 350.0
projectile_size = 14.0
projectile_penetration = 1
evolves_from = []
evolves_into = "blazing_staff"
evolution_recipe = ["ember_staff", "ember_staff"]
passive_effect = ""
description = "A staff that shoots three fireballs in a spread pattern."
```

## 2.3 Artifact Schema

```toml
# assets/data/artifacts.toml

[[artifacts]]
id = "molten_core"
name = "Molten Core"
tier = 2
target_scope = "color"           # global|color|type|creature
target_color = "red"
target_type = ""
target_creature = ""
damage_bonus = 20.0              # percentage
attack_speed_bonus = 0.0
hp_bonus = 0.0
crit_t1_bonus = 0.0
crit_t2_bonus = 0.0
crit_t3_bonus = 0.0
crit_damage_bonus = 0.0
special_effect = ""
description = "Increases damage of all fire creatures by 20%."
```

## 2.4 Enemy Schema

```toml
# assets/data/enemies.toml

[[enemies]]
id = "goblin"
name = "Goblin"
enemy_class = "fodder"           # fodder|elite|miniboss|boss
enemy_type = "melee"             # melee|ranged|fast|tank|healer|commander|splitter
color_resist = ""
color_weak = ""
base_hp = 30.0
base_damage = 5.0
attack_speed = 1.0
movement_speed = 80.0
attack_range = 40.0
ai_type = "chase"                # chase|kite|flank|support|guard
targets_creatures = false
min_wave = 1
spawn_weight = 100.0
group_size_min = 3
group_size_max = 8
xp_value = 1
drop_table = []
phases = 0
phase_abilities = []
description = "Basic melee fodder. Runs at you and attacks."
```

## 2.5 Affinity Thresholds Schema

```toml
# assets/data/affinity.toml

[[affinity_colors]]
color = "red"
overflow_bonus_per_point = 1.0   # +1% damage per point above 100

[[affinity_colors.thresholds]]
min = 0
damage_bonus = 0.0
attack_speed_bonus = 0.0
hp_bonus = 0.0
crit_t1_bonus = 0.0
crit_t2_unlock = false
crit_t3_unlock = false
special = ""

[[affinity_colors.thresholds]]
min = 11
damage_bonus = 10.0
attack_speed_bonus = 0.0
hp_bonus = 0.0
crit_t1_bonus = 0.0
crit_t2_unlock = false
crit_t3_unlock = false
special = ""

# ... more thresholds at 26, 51, 76
```

---

# 3. Red Deck Roster [IMPLEMENTED]

## 3.1 Red Creatures (14 total)

| ID | Name | Tier | Type | Evolves Into |
|----|------|------|------|--------------|
| fire_imp | Fire Imp | 1 | ranged | flame_fiend |
| ember_hound | Ember Hound | 1 | assassin | hellhound |
| fire_spirit | Fire Spirit | 1 | support | greater_fire_spirit |
| flame_fiend | Flame Fiend | 2 | ranged | inferno_demon |
| hellhound | Hellhound | 2 | assassin | hellhound_alpha |
| inferno_knight | Inferno Knight | 2 | melee | inferno_warlord |
| magma_elemental | Magma Elemental | 2 | melee | — |
| greater_fire_spirit | Greater Fire Spirit | 2 | support | — |
| inferno_demon | Inferno Demon | 3 | ranged | — |
| hellhound_alpha | Hellhound Alpha | 3 | assassin | — |
| inferno_warlord | Inferno Warlord | 3 | melee | inferno_titan |
| phoenix | Phoenix | 3 | ranged | eternal_phoenix |
| inferno_titan | Inferno Titan | 4 | melee | — |
| eternal_phoenix | Eternal Phoenix | 4 | ranged | — |

## 3.2 Red Weapons (13 total)

| ID | Name | Tier | Affinity | Pattern |
|----|------|------|----------|---------|
| ember_staff | Ember Staff | 1 | 10 | spread (3 proj) |
| flame_sword | Flame Sword | 1 | 15 | melee_sweep (3 proj) |
| fire_dagger | Fire Dagger | 1 | 8 | melee_stab (3 proj) |
| blazing_staff | Blazing Staff | 2 | 20 | single |
| inferno_blade | Inferno Blade | 2 | 25 | melee_sweep |
| twin_daggers | Twin Fire Daggers | 2 | 18 | melee_stab (2 proj) |
| flame_wand | Flame Wand | 2 | 22 | spread (2 proj) |
| volcanic_staff | Volcanic Staff | 3 | 35 | spread (3 proj) |
| inferno_greatsword | Inferno Greatsword | 3 | 40 | melee_slam |
| phoenix_wand | Phoenix Wand | 3 | 30 | homing (2 proj) |
| molten_hammer | Molten Hammer | 3 | 38 | melee_slam |
| sun_scepter | Sun Scepter | 4 | 50 | orbit (5 proj) |
| blade_of_the_inferno | Blade of the Inferno | 4 | 55 | melee_wave |

## 3.3 Enemies [IMPLEMENTED]

### Fodder (Wave 1+)
| ID | Name | Type | Min Wave |
|----|------|------|----------|
| goblin | Goblin | melee | 1 |
| goblin_archer | Goblin Archer | ranged | 6 |
| wolf | Wolf | fast | 11 |
| skeleton | Skeleton | melee | 31 |

### Elites/Bosses [PLANNED]
- orc_warrior (wave 16+)
- goblin_shaman (wave 21+)
- troll_chief (wave 10 boss)
- frost_giant (wave 20 boss)

---

# 4. Core Rust Types [IMPLEMENTED]

```rust
// src/components/creature.rs

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreatureType {
    #[default]
    Melee,
    Ranged,
    Support,
    Assassin,
}

#[derive(Component, Clone, Debug)]
pub struct CreatureStats {
    pub id: String,
    pub name: String,
    pub color: CreatureColor,
    pub tier: u8,
    pub creature_type: CreatureType,
    pub level: u32,
    pub kills: u32,
    pub base_damage: f64,
    pub attack_speed: f64,
    pub base_hp: f64,
    pub max_hp: f64,
    pub current_hp: f64,
    pub movement_speed: f64,
    pub attack_range: f64,
    pub crit_t1: f64,
    pub crit_t2: f64,
    pub crit_t3: f64,
    // ... projectile config fields
}

// src/resources/deck.rs

#[derive(Clone, Debug, PartialEq)]
pub enum CardType {
    Creature,
    Weapon,
    Artifact,
}

#[derive(Clone, Debug)]
pub struct DeckCard {
    pub card_type: CardType,
    pub id: String,
    pub weight: f64,
}

#[derive(Resource)]
pub struct PlayerDeck {
    pub cards: Vec<DeckCard>,
    pub total_weight: f64,
}

// src/math/crit.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CritTier {
    #[default]
    None,
    Normal,  // 2x damage
    Mega,    // damage²
    Super,   // damage⁴ (capped at 1e15)
}

pub struct CritResult {
    pub tier: CritTier,
    pub final_damage: f64,
    pub base_damage: f64,
}

pub const MAX_DAMAGE_CAP: f64 = 1e15;
```

---

# 5. Implementation Checklist

## Phase 1: Foundation [IMPLEMENTED]
- [x] Initialize Bevy project with Cargo.toml dependencies
- [x] Create folder structure
- [x] Create all TOML data files with Red deck content
- [x] Implement TOML loading into Rust structs
- [x] Basic window + camera setup (1920x1080)
- [x] Placeholder sprites (colored shapes)

## Phase 2: Core Entities [IMPLEMENTED]
- [x] Player entity with movement (WASD)
- [x] Creature spawning from data
- [x] Creature positioning (formation system)
- [x] Basic creature AI (follow formation, attack nearest)
- [x] Enemy spawning (wave-based)
- [x] Enemy AI (chase player)

## Phase 3: Combat [IMPLEMENTED]
- [x] Damage calculation system
- [x] HP tracking and death
- [x] Projectile system (ranged attacks)
- [x] Creature death → respawn timer
- [x] Enemy death → XP credit

## Phase 4: Progression [IMPLEMENTED]
- [x] Kill counter resource
- [x] Level-up trigger (15 kills base)
- [x] Probability deck roll on level
- [x] Card acquisition
- [x] Artifact application (permanent buffs)

## Phase 5: Scaling Systems [IMPLEMENTED]
- [x] Affinity calculation from weapons
- [x] Affinity threshold bonuses
- [x] Creature kill XP (individual leveling)
- [x] Evolution system (3x duplicate detection)
- [x] 3-tier crit system implementation
- [x] Crit overflow math

## Phase 6: Advanced [PARTIAL]
- [x] Director AI (adaptive spawning)
- [x] Pre-run deck builder UI
- [x] Object pooling (projectiles, damage numbers)
- [x] Spatial grid optimization
- [ ] Boss encounters (phase-based) [PLANNED]
- [ ] Player health system [PLANNED]

## Phase 7: Polish [PARTIAL]
- [x] UI (HUD, panels, tooltips)
- [x] Visual effects (crit indicators, evolution, level-up)
- [x] Tilemap with procedural chunks
- [ ] Audio (placeholder sounds) [PLANNED]
- [ ] Balance tuning [PLANNED]

---

# 6. Key Formulas [IMPLEMENTED]

## Damage Calculation

```rust
// Base damage with all bonuses
base_damage = creature.base_damage
affinity_bonus = get_affinity_bonus(creature.color)  // percentage
artifact_bonus = sum(applicable_artifact_bonuses)     // percentage
level_bonus = creature.level * 0.10                   // +10% per level

final_base = base_damage * (1 + affinity_bonus/100) * (1 + artifact_bonus/100) * (1 + level_bonus)

// Crit roll (independent per tier)
roll_t1 = random(0, 100)
roll_t2 = random(0, 100)
roll_t3 = random(0, 100)

// Highest tier wins
if roll_t3 < crit_t3:
    damage = min(final_base^4, 1e15)  // Super crit, capped
elif roll_t2 < crit_t2:
    damage = final_base^2              // Mega crit
elif roll_t1 < crit_t1:
    damage = final_base * 2 * (1 + overflow_bonus)  // Normal crit
else:
    damage = final_base
```

## Level-Up Threshold

```rust
// Kills needed for next level
kills_for_level(n) = 15 * (1.1)^(n-1)

// Level 1: 15 kills
// Level 5: ~22 kills
// Level 10: ~39 kills
```

## Wave-Based Enemy Spawning

```rust
// Director spawn rate (per spawn tick)
wave 1-10:   base 2-4, scales to 18-30
wave 11-15:  exponential increase
wave 16-20:  heavy pressure
wave 21+:    massive spawns (100-180 per tick)

// Elite chance
elite_chance = 0.02 + (wave * 0.006)  // 2% base, +0.6% per wave, caps at 20%
```

## Probability Roll

```rust
roll = random(0, deck.total_weight)
cumulative = 0
for card in deck.cards:
    cumulative += card.weight
    if roll < cumulative:
        return card
```

---

# 7. Dependencies [IMPLEMENTED]

```toml
[package]
name = "bloodtide"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.15"
bevy_ecs_tilemap = "0.15"
noise = "0.9"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
rand = "0.8"
```

---

# 8. Notes for Claude Code

1. **Data-driven design** — All game content in TOML. Change stats without recompiling.

2. **ECS architecture** — Entities (creatures, enemies, projectiles) + Components (stats, timers) + Systems (combat, spawning, AI).

3. **Performance patterns already in place:**
   - Spatial grid for O(1) enemy lookups (256px cells)
   - Object pooling for projectiles (5000) and damage numbers (500)
   - Enemy cap (2000 max)

4. **Key file locations:**
   - Combat logic: `src/systems/combat.rs` (47KB, largest file)
   - Crit math: `src/math/crit.rs`
   - Deck system: `src/resources/deck.rs`
   - Director AI: `src/resources/director.rs`
   - TOML structs: `src/data/mod.rs`

5. **Testing:** 191 tests covering data loading, deck probability, leveling, combat, crits.

6. **System execution order** (defined in main.rs):
   Director → Tilemap → Input → Movement → Combat → Death → Leveling → UI

---

*Technical Spec v2.0 — Updated to match actual implementation (December 2024)*
