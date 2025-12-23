# Bloodtide: Technical Specification
## For Claude Code Implementation

---

# 1. Project Structure

```
bloodtide/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── plugins/
│   │   ├── mod.rs
│   │   ├── game_state.rs
│   │   ├── combat.rs
│   │   ├── creatures.rs
│   │   ├── deck.rs
│   │   ├── enemies.rs
│   │   ├── leveling.rs
│   │   ├── affinity.rs
│   │   └── ui.rs
│   ├── components/
│   │   ├── mod.rs
│   │   ├── creature.rs
│   │   ├── weapon.rs
│   │   ├── enemy.rs
│   │   ├── stats.rs
│   │   └── combat.rs
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── movement.rs
│   │   ├── combat.rs
│   │   ├── spawning.rs
│   │   ├── ai.rs
│   │   └── scaling.rs
│   ├── resources/
│   │   ├── mod.rs
│   │   ├── deck.rs
│   │   ├── game_data.rs
│   │   └── director.rs
│   └── math/
│       ├── mod.rs
│       ├── big_number.rs      # break_infinity wrapper
│       └── crit.rs
├── assets/
│   ├── data/
│   │   ├── creatures.toml
│   │   ├── weapons.toml
│   │   ├── artifacts.toml
│   │   ├── enemies.toml
│   │   └── affinity.toml
│   ├── sprites/              # placeholder art
│   └── audio/                # placeholder audio
└── README.md
```

---

# 2. Data Schemas (TOML)

## 2.1 Creature Schema

```toml
# assets/data/creatures.toml

# Schema:
# id: string (unique identifier, snake_case)
# name: string (display name)
# color: "red" | "blue" | "green" | "white" | "black" | "colorless"
# tier: 1-5 (evolution tier, 1 = base, 5 = legendary)
# creature_type: "melee" | "ranged" | "support" | "assassin"
# 
# Base Stats:
# base_damage: f64
# attack_speed: f64 (attacks per second)
# base_hp: f64
# movement_speed: f64
# attack_range: f64 (pixels, melee ~50, ranged ~300)
#
# Crit Chances (percentage, 0-100):
# crit_t1: f64
# crit_t2: f64
# crit_t3: f64
#
# Evolution:
# evolves_from: string | null (creature id)
# evolves_into: string | null (creature id)
# evolution_count: i32 (how many duplicates needed, usually 3)
#
# Kill XP:
# kills_per_level: [i32] (array of kill thresholds per level)
# max_level: i32
#
# Special:
# abilities: [string] (list of ability ids, can be empty)
# respawn_time: f64 (seconds)
# description: string

[[creatures]]
id = "fire_imp"
name = "Fire Imp"
color = "red"
tier = 1
creature_type = "ranged"
base_damage = 15.0
attack_speed = 1.0
base_hp = 50.0
movement_speed = 100.0
attack_range = 200.0
crit_t1 = 5.0
crit_t2 = 0.0
crit_t3 = 0.0
evolves_from = null
evolves_into = "flame_fiend"
evolution_count = 3
kills_per_level = [10, 25, 50, 100, 150, 200, 300, 400, 500]
max_level = 10
abilities = ["fireball"]
respawn_time = 20.0
description = "A small fire spirit that hurls fireballs at enemies."
```

## 2.2 Weapon Schema

```toml
# assets/data/weapons.toml

# Schema:
# id: string (unique identifier)
# name: string (display name)
# color: "red" | "blue" | "green" | "white" | "black" | "colorless"
# tier: 1-5
#
# Affinity:
# affinity_amount: f64 (how much color affinity this provides)
#
# Auto-attack:
# auto_damage: f64
# auto_speed: f64 (attacks per second)
# auto_range: f64
# projectile_count: i32
# projectile_pattern: "single" | "spread" | "orbit" | "chain"
#
# Evolution:
# evolves_from: [string] (list of weapon ids needed to combine)
# evolves_into: string | null
#
# Special:
# passive_effect: string | null (effect id)
# description: string

[[weapons]]
id = "ember_staff"
name = "Ember Staff"
color = "red"
tier = 1
affinity_amount = 10.0
auto_damage = 8.0
auto_speed = 1.5
auto_range = 250.0
projectile_count = 1
projectile_pattern = "single"
evolves_from = []
evolves_into = "blazing_staff"
passive_effect = null
description = "A basic staff that shoots small fireballs."

[[weapons]]
id = "blazing_staff"
name = "Blazing Staff"
color = "red"
tier = 2
affinity_amount = 20.0
auto_damage = 15.0
auto_speed = 2.0
auto_range = 275.0
projectile_count = 1
projectile_pattern = "single"
evolves_from = ["ember_staff", "ember_staff"]
evolves_into = "inferno_staff"
passive_effect = "burn_dot"
description = "An upgraded staff with faster attacks and burn damage."
```

## 2.3 Artifact Schema

```toml
# assets/data/artifacts.toml

# Schema:
# id: string
# name: string
# tier: 1-5 (rarity)
#
# Target:
# target_scope: "global" | "color" | "type" | "creature"
# target_color: string | null (if scope is "color")
# target_type: string | null (if scope is "type": "melee", "ranged", etc.)
# target_creature: string | null (if scope is "creature": specific creature id)
#
# Stat Modifiers (all are percentage bonuses, can be negative):
# damage_bonus: f64
# attack_speed_bonus: f64
# hp_bonus: f64
# crit_t1_bonus: f64
# crit_t2_bonus: f64
# crit_t3_bonus: f64
# crit_damage_bonus: f64
#
# Special:
# special_effect: string | null
# description: string

[[artifacts]]
id = "molten_core"
name = "Molten Core"
tier = 2
target_scope = "color"
target_color = "red"
target_type = null
target_creature = null
damage_bonus = 20.0
attack_speed_bonus = 0.0
hp_bonus = 0.0
crit_t1_bonus = 0.0
crit_t2_bonus = 0.0
crit_t3_bonus = 0.0
crit_damage_bonus = 0.0
special_effect = null
description = "Increases damage of all fire creatures by 20%."

[[artifacts]]
id = "chaos_ember"
name = "Chaos Ember"
tier = 3
target_scope = "color"
target_color = "red"
target_type = null
target_creature = null
damage_bonus = 0.0
attack_speed_bonus = 0.0
hp_bonus = 0.0
crit_t1_bonus = 0.0
crit_t2_bonus = 5.0
crit_t3_bonus = 0.0
crit_damage_bonus = 0.0
special_effect = null
description = "Fire creatures gain 5% Tier 2 Mega Crit chance."
```

## 2.4 Enemy Schema

```toml
# assets/data/enemies.toml

# Schema:
# id: string
# name: string
# enemy_class: "fodder" | "elite" | "miniboss" | "boss"
# enemy_type: "melee" | "ranged" | "fast" | "tank" | "healer" | "commander" | "splitter"
# color_resist: string | null (takes 50% less from this color)
# color_weak: string | null (takes 50% more from this color)
#
# Base Stats (these get multiplied by wave scaling):
# base_hp: f64
# base_damage: f64
# attack_speed: f64
# movement_speed: f64
# attack_range: f64
#
# Behavior:
# ai_type: "chase" | "kite" | "flank" | "support" | "guard"
# targets_creatures: bool (if true, attacks player creatures; if false, targets player)
#
# Spawning:
# min_wave: i32 (first wave this can appear)
# spawn_weight: f64 (probability weight for Director)
# group_size_min: i32
# group_size_max: i32
#
# Drops:
# xp_value: i32 (kill credits)
# drop_table: [string] (item drop ids, can be empty)
#
# Boss-specific:
# phases: i32 | null (number of phases, bosses only)
# phase_abilities: [[string]] | null (abilities per phase)
#
# description: string

[[enemies]]
id = "goblin"
name = "Goblin"
enemy_class = "fodder"
enemy_type = "melee"
color_resist = null
color_weak = null
base_hp = 30.0
base_damage = 5.0
attack_speed = 1.0
movement_speed = 80.0
attack_range = 40.0
ai_type = "chase"
targets_creatures = false
min_wave = 1
spawn_weight = 100.0
group_size_min = 3
group_size_max = 8
xp_value = 1
drop_table = []
phases = null
phase_abilities = null
description = "Basic melee fodder. Runs at you and attacks."

[[enemies]]
id = "goblin_archer"
name = "Goblin Archer"
enemy_class = "fodder"
enemy_type = "ranged"
color_resist = null
color_weak = null
base_hp = 20.0
base_damage = 8.0
attack_speed = 0.8
movement_speed = 60.0
attack_range = 250.0
ai_type = "kite"
targets_creatures = false
min_wave = 6
spawn_weight = 50.0
group_size_min = 2
group_size_max = 5
xp_value = 1
drop_table = []
phases = null
phase_abilities = null
description = "Ranged fodder. Keeps distance and shoots arrows."
```

## 2.5 Affinity Thresholds Schema

```toml
# assets/data/affinity.toml

# Schema:
# Each color has threshold breakpoints with cumulative bonuses

[[affinity_colors]]
color = "red"
thresholds = [
    { min = 0, damage_bonus = 0.0, attack_speed_bonus = 0.0, crit_t2_unlock = false, special = null },
    { min = 11, damage_bonus = 10.0, attack_speed_bonus = 0.0, crit_t2_unlock = false, special = null },
    { min = 26, damage_bonus = 25.0, attack_speed_bonus = 10.0, crit_t2_unlock = false, special = null },
    { min = 51, damage_bonus = 50.0, attack_speed_bonus = 10.0, crit_t2_unlock = true, special = null },
    { min = 76, damage_bonus = 100.0, attack_speed_bonus = 10.0, crit_t2_unlock = true, special = "burn_dot" },
]
overflow_bonus_per_point = 1.0  # +1% damage per point above 100

[[affinity_colors]]
color = "blue"
thresholds = [
    { min = 0, damage_bonus = 0.0, attack_speed_bonus = 0.0, crit_t2_unlock = false, special = null },
    { min = 11, damage_bonus = 10.0, attack_speed_bonus = 0.0, crit_t2_unlock = false, special = "slow_5" },
    { min = 26, damage_bonus = 20.0, attack_speed_bonus = 0.0, crit_t2_unlock = false, special = "slow_10" },
    { min = 51, damage_bonus = 30.0, attack_speed_bonus = 0.0, crit_t2_unlock = true, special = "slow_15" },
    { min = 76, damage_bonus = 50.0, attack_speed_bonus = 0.0, crit_t2_unlock = true, special = "freeze_chance" },
]
overflow_bonus_per_point = 0.5  # Blue gets less damage, more control
```

---

# 3. Red Deck Roster (For Claude Code to Generate)

## 3.1 Red Creatures

| ID | Name | Tier | Type | Description | Evolves Into |
|----|------|------|------|-------------|--------------|
| fire_imp | Fire Imp | 1 | ranged | Small fire spirit, shoots fireballs | flame_fiend |
| flame_fiend | Flame Fiend | 2 | ranged | Larger fire demon, faster fireballs, burn DoT | inferno_demon |
| inferno_demon | Inferno Demon | 3 | ranged | Massive demon, multi-shot fireballs, AoE burn | — |
| ember_hound | Ember Hound | 1 | assassin | Fast fire dog, runs around freely attacking | hellhound |
| hellhound | Hellhound | 2 | assassin | Larger, faster, leaves fire trails | hellhound_alpha |
| hellhound_alpha | Hellhound Alpha | 3 | assassin | Pack leader, buffs other hellhounds nearby | — |
| inferno_knight | Inferno Knight | 2 | melee | Armored fire warrior, frontline tank | inferno_warlord |
| inferno_warlord | Inferno Warlord | 3 | melee | Heavy armor, AoE slam attack, very tanky | inferno_titan |
| inferno_titan | Inferno Titan | 4 | melee | Massive, slow, devastating damage | — |
| phoenix | Phoenix | 3 | ranged | On death: AoE explosion, respawns faster | eternal_phoenix |
| eternal_phoenix | Eternal Phoenix | 4 | ranged | Instant respawn, bigger explosion, burn aura | — |
| magma_elemental | Magma Elemental | 2 | melee | Slow but leaves damaging ground fire | — |
| fire_spirit | Fire Spirit | 1 | support | Orbits player, periodically heals fire creatures | greater_fire_spirit |
| greater_fire_spirit | Greater Fire Spirit | 2 | support | Stronger heals, damage buff aura | — |

## 3.2 Red Weapons

| ID | Name | Tier | Affinity | Evolves From | Description |
|----|------|------|----------|--------------|-------------|
| ember_staff | Ember Staff | 1 | 10 | — | Basic fireball shooter |
| flame_sword | Flame Sword | 1 | 15 | — | Melee sweep, short range |
| blazing_staff | Blazing Staff | 2 | 20 | ember_staff ×2 | Faster fireballs, burn DoT |
| inferno_blade | Inferno Blade | 2 | 25 | flame_sword + ember_staff | Melee + ranged hybrid |
| volcanic_staff | Volcanic Staff | 3 | 35 | blazing_staff ×2 | Multi-shot fireballs |
| inferno_greatsword | Inferno Greatsword | 3 | 40 | inferno_blade + flame_sword | Huge melee sweep, ground fire |
| phoenix_wand | Phoenix Wand | 3 | 30 | — | Projectiles home on targets |
| sun_scepter | Sun Scepter | 4 | 50 | volcanic_staff + phoenix_wand | Orbital fireballs, massive burn |

## 3.3 Red Artifacts

| ID | Name | Tier | Scope | Effect |
|----|------|------|-------|--------|
| molten_core | Molten Core | 2 | color:red | +20% damage |
| chaos_ember | Chaos Ember | 3 | color:red | +5% T2 Mega Crit |
| heart_of_flame | Heart of Flame | 3 | color:red | +15% attack speed |
| burning_blood | Burning Blood | 2 | global | +10% damage, -10% HP |
| pyromaniac | Pyromaniac | 3 | color:red | +3% T3 Super Crit |
| volcanic_heart | Volcanic Heart | 4 | color:red | +50% damage, attacks cause ground fire |
| ember_soul | Ember Soul | 2 | type:ranged | +25% damage to ranged creatures |
| inferno_plate | Inferno Plate | 2 | type:melee | +30% HP to melee creatures |
| hound_collar | Hound Collar | 3 | creature:ember_hound | +100% damage to Ember Hound line |
| phoenix_feather | Phoenix Feather | 4 | creature:phoenix | Phoenix respawns instantly |

## 3.4 Enemies (All Colors/Generic)

### Fodder (Wave 1+)
| ID | Name | Type | Description |
|----|------|------|-------------|
| goblin | Goblin | melee | Basic runner |
| goblin_archer | Goblin Archer | ranged | Basic shooter (wave 6+) |
| wolf | Wolf | fast | Flanker, fast movement (wave 11+) |
| skeleton | Skeleton | melee | Basic, splits into 2 bone piles on death (wave 31+) |

### Elites (Wave 10+)
| ID | Name | Type | Description |
|----|------|------|-------------|
| orc_warrior | Orc Warrior | tank | High HP, shield blocks frontal (wave 16+) |
| goblin_shaman | Goblin Shaman | healer | Heals nearby enemies (wave 21+) |
| orc_warchief | Orc Warchief | commander | Buffs nearby enemy damage (wave 26+) |
| shadow_assassin | Shadow Assassin | fast | Phases through your frontline (wave 36+) |
| fire_resistant_golem | Fire Golem | tank | 50% fire resist, counter-spawns vs red (wave 46+) |

### Bosses (Every 10 Waves)
| ID | Name | Wave | Phases | Description |
|----|------|------|--------|-------------|
| troll_chief | Troll Chief | 10 | 2 | Slam attacks, summons goblin adds |
| frost_giant | Frost Giant | 20 | 3 | Ice breath cone, ground freeze, blizzard |
| dragon_knight | Dragon Knight | 30 | 3 | Fire breath (ironic counter to red), charge, summons drakes |
| lich_king | Lich King | 40 | 3 | Raises dead enemies as skeletons, death beam, soul drain |
| castle_lord | Castle Lord | 50 | 4 | Final boss, all attack types, enrage below 25% |

---

# 4. Core Rust Types (Reference)

```rust
// src/components/stats.rs

use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Stats {
    pub base_damage: f64,
    pub attack_speed: f64,
    pub base_hp: f64,
    pub current_hp: f64,
    pub movement_speed: f64,
    pub attack_range: f64,
    pub crit_t1: f64,  // percentage 0-100+
    pub crit_t2: f64,
    pub crit_t3: f64,
}

#[derive(Component, Clone, Debug)]
pub struct CreatureData {
    pub id: String,
    pub color: Color,
    pub tier: u8,
    pub creature_type: CreatureType,
    pub level: u32,
    pub kills: u32,
    pub kills_for_next_level: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CreatureType {
    Melee,
    Ranged,
    Support,
    Assassin,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    Red,
    Blue,
    Green,
    White,
    Black,
    Colorless,
}

// src/resources/deck.rs

#[derive(Resource)]
pub struct PlayerDeck {
    pub cards: Vec<DeckCard>,
    pub total_weight: f64,
}

#[derive(Clone, Debug)]
pub struct DeckCard {
    pub card_type: CardType,
    pub id: String,
    pub weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CardType {
    Creature,
    Weapon,
    Artifact,
}

// src/math/big_number.rs
// Wrapper around break_infinity or custom implementation

pub struct BigNumber {
    pub mantissa: f64,
    pub exponent: i64,
}

impl BigNumber {
    pub fn multiply(&self, other: &BigNumber) -> BigNumber { ... }
    pub fn power(&self, exp: f64) -> BigNumber { ... }
    pub fn to_display_string(&self) -> String { ... }  // "1.23e15"
}
```

---

# 5. Implementation Checklist

## Phase 1: Foundation (Week 1)
- [ ] Initialize Bevy project with Cargo.toml dependencies
- [ ] Create folder structure as specified above
- [ ] Create all TOML data files with Red deck content
- [ ] Implement TOML loading into Rust structs
- [ ] Basic window + camera setup
- [ ] Placeholder sprites (colored shapes)

## Phase 2: Core Entities (Week 1-2)
- [ ] Player entity with movement (WASD)
- [ ] Creature spawning from data
- [ ] Creature positioning (formation system)
- [ ] Basic creature AI (follow formation, attack nearest)
- [ ] Enemy spawning (wave-based)
- [ ] Enemy AI (chase player)

## Phase 3: Combat (Week 2)
- [ ] Damage calculation system
- [ ] HP tracking and death
- [ ] Projectile system (ranged attacks)
- [ ] Melee attack hitboxes
- [ ] Creature death → respawn timer
- [ ] Enemy death → XP credit

## Phase 4: Progression (Week 2-3)
- [ ] Kill counter resource
- [ ] Level-up trigger (25 kills)
- [ ] Probability deck roll on level
- [ ] Card acquisition UI (minimal)
- [ ] Creature equipment system
- [ ] Artifact application (permanent buffs)

## Phase 5: Scaling Systems (Week 3)
- [ ] Affinity calculation from weapons
- [ ] Affinity threshold bonuses
- [ ] Creature kill XP (individual leveling)
- [ ] Evolution system (3x duplicate detection)
- [ ] 3-tier crit system implementation
- [ ] Crit overflow math

## Phase 6: Advanced (Week 3-4)
- [ ] Director AI (adaptive spawning)
- [ ] Enemy behavior types (tank, healer, etc.)
- [ ] Boss encounters (phase-based)
- [ ] Zone/map transitions
- [ ] Objectives system

## Phase 7: Polish (Week 4+)
- [ ] UI (inventory, stats display, minimap)
- [ ] Visual effects (crit indicators, evolution)
- [ ] Audio (placeholder sounds)
- [ ] Balance tuning
- [ ] Performance optimization

---

# 6. Key Formulas (Reference)

## Damage Calculation
```
base_damage = creature.base_damage
affinity_bonus = get_affinity_bonus(creature.color)
artifact_bonus = sum(applicable_artifact_bonuses)
level_bonus = creature.level * 0.10  // +10% per level

final_base = base_damage * (1 + affinity_bonus) * (1 + artifact_bonus) * (1 + level_bonus)

// Crit roll (independent per tier)
roll_t1 = random(0, 100)
roll_t2 = random(0, 100)
roll_t3 = random(0, 100)

if roll_t3 < crit_t3:
    damage = final_base ^ 4  // or tetration if uncapped
elif roll_t2 < crit_t2:
    damage = final_base ^ 2
elif roll_t1 < crit_t1:
    damage = final_base * 2 * (1 + crit_overflow_bonus)
else:
    damage = final_base
```

## Enemy HP Scaling
```
target_ttk = match enemy.class {
    Fodder => 0.75,
    Elite => 4.0,
    Miniboss => 20.0,
    Boss => 90.0,
}

enemy_hp = player_expected_dps * target_ttk * difficulty_modifier
```

## Enemy Damage Scaling
```
enemy_damage = base_damage * (1.15 ^ wave_number)
```

## Probability Roll
```
roll = random(0, deck.total_weight)
cumulative = 0
for card in deck.cards:
    cumulative += card.weight
    if roll < cumulative:
        return card
```

---

# 7. Dependencies (Cargo.toml)

```toml
[package]
name = "bloodtide"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.12"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
rand = "0.8"
# break_infinity or custom big number implementation
```

---

# 8. Notes for Claude Code

1. **Start with data loading** — Get TOML files parsing into Rust structs first. Everything depends on this.

2. **Use placeholder art** — Just colored rectangles/circles. Don't waste time on sprites.

3. **Test each system in isolation** — Spawn one creature, make it attack one enemy, verify damage numbers.

4. **Keep big number math separate** — The crit scaling will get wild. Isolate it in the math module.

5. **Director AI can wait** — Basic wave spawning first, smart AI later.

6. **The game feel matters** — Even with placeholder art, make hits feel impactful (screen shake, flash, etc.)

---

*Technical Spec v1.0 — Ready for Claude Code implementation*
