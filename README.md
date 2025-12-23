# Bloodtide

A Vampire Survivors-style roguelike deck-building game built in Rust with Bevy. Summon creatures, collect weapons, and survive infinite waves of enemies with exponential scaling damage.

## Quick Start

```bash
cargo run          # Start the game
cargo test         # Run tests (191 tests)
```

**Controls:**
- WASD / Arrow Keys: Move
- ESC: Pause menu / Debug options
- Game starts in Deck Builder - select your starting weapon and configure your deck

**Current Status:** Phase 29 of ~50 (Core gameplay complete, deck builder functional)

---

## Game Overview

### Core Loop
1. **Deck Builder** (pre-run): Select starting weapon, configure creature/weapon/artifact probability weights
2. **Survival**: Creatures auto-fight enemies in a formation around the player
3. **Level Up**: Every 15 kills = level up = roll a card from your probability deck
4. **Scaling**: Acquire cards, evolve creatures (3x duplicate), gain affinity bonuses

### Card Types
| Type | Purpose | Dies? | Evolves? |
|------|---------|-------|----------|
| **Creatures** | Main damage dealers, follow player in formation | Yes (respawns) | Yes (3x same → evolved form) |
| **Weapons** | Auto-attack + provide color affinity | No | Yes (combine recipes) |
| **Artifacts** | Permanent stat buffs (damage, crit, HP, etc.) | N/A | No |

### Key Systems
- **Deck as Probability Table**: More copies of a card = higher roll chance. No "running out" of cards.
- **3-Tier Crit System**: T1 (2x), T2 (damage²), T3 (damage⁴ capped at 1e15)
- **Color Affinity**: Weapons grant affinity → threshold bonuses unlock (damage, attack speed, T2/T3 crits)
- **Director AI**: Adaptive enemy spawning based on player performance and wave number
- **Creature Evolution**: 3 identical creatures → choice to combine into evolved form (resets level, higher base stats)

---

## Architecture

### Tech Stack
- **Bevy 0.15** (ECS game engine)
- **bevy_ecs_tilemap 0.15** (procedural ground tiles)
- **Rust** (edition 2021)
- **TOML** (data-driven content)

### Project Structure
```
bloodtide/
├── src/
│   ├── main.rs                 # App init, system ordering
│   ├── components/             # Entity data
│   │   ├── creature.rs         # CreatureStats, CreatureColor, CreatureType
│   │   ├── enemy.rs            # EnemyStats, EnemyClass, EnemyType
│   │   ├── weapon.rs           # WeaponData, WeaponStats
│   │   └── player.rs           # Player marker
│   ├── systems/                # Game logic (~17 modules)
│   │   ├── combat.rs           # Damage, projectiles, attacks (47KB)
│   │   ├── spawning.rs         # Creature/enemy/weapon spawning
│   │   ├── leveling.rs         # Player level-ups, card rolling
│   │   ├── creature_xp.rs      # Per-creature kill tracking, evolution
│   │   ├── deck_builder_ui.rs  # Pre-run deck configuration
│   │   ├── debug_menu.rs       # Pause menu, debug controls
│   │   ├── ui_panels.rs        # HUD, creature panel, artifacts
│   │   ├── ai.rs               # Creature formation, enemy chase
│   │   ├── death.rs            # Death handling, respawn timers
│   │   ├── hp_bars.rs          # Health bar rendering
│   │   └── tilemap.rs          # Procedural map chunks
│   ├── resources/              # Global state (~13 modules)
│   │   ├── game_state.rs       # Kills, wave, level tracking
│   │   ├── director.rs         # Adaptive spawn AI
│   │   ├── deck.rs             # Probability deck, card rolling
│   │   ├── affinity.rs         # Color affinity + thresholds
│   │   ├── artifact_buffs.rs   # Equipment bonuses
│   │   ├── spatial.rs          # Spatial grid for O(1) lookups
│   │   ├── pools.rs            # Object pooling (projectiles, damage numbers)
│   │   └── game_data.rs        # Loaded TOML data
│   ├── math/
│   │   └── crit.rs             # 3-tier crit calculation
│   └── data/
│       └── mod.rs              # TOML struct definitions
├── assets/
│   ├── data/                   # Game content (TOML)
│   │   ├── creatures.toml      # 14 creatures (T1-T4)
│   │   ├── weapons.toml        # 13 weapons (T1-T4)
│   │   ├── artifacts.toml      # Stat buff items
│   │   ├── enemies.toml        # Enemy types
│   │   └── affinity.toml       # Color thresholds
│   └── sprites/                # Spritesheets (goblin, blood effects, tiles)
└── Cargo.toml
```

---

## Key Systems Deep Dive

### 3-Tier Crit System
The signature mechanic. Each tier rolls independently; highest successful tier wins.

| Tier | Multiplier | Example (100 base) |
|------|------------|-------------------|
| None | 1x | 100 |
| T1 Normal | 2x | 200 |
| T2 Mega | damage² | 10,000 |
| T3 Super | damage⁴ (capped 1e15) | 100,000,000 |

**Overflow**: If T1 crit chance exceeds 100%, overflow grants bonus crit damage.

```rust
// src/math/crit.rs
pub fn calculate_damage_with_crits(base_damage: f64, crit_t1: f64, crit_t2: f64, crit_t3: f64) -> CritResult
```

### Creature Evolution
```
Fire Imp (T1) → Flame Fiend (T2) → Inferno Demon (T3)
  ↓ 3 duplicates trigger evolution choice
```
- Evolved creature resets to L1 but has higher base stats
- Keep leveled creatures OR combine - strategic choice

### Affinity System
Weapons grant color affinity. Thresholds unlock bonuses:

| Red Affinity | Bonus |
|--------------|-------|
| 0-10 | Base |
| 11-25 | +10% damage |
| 26-50 | +25% damage, +10% attack speed |
| 51-75 | +50% damage, unlock T2 Mega Crit |
| 76-100 | +100% damage, burn DoT |
| 100+ | +1% damage per overflow point |

### Director AI
Tracks player DPS, creature count, HP%. Adjusts spawn rates:
- Wave 1: 2-4 enemies/spawn, target 15 total
- Wave 10: 18-30/spawn, target 285 total
- Wave 30: 100-180/spawn, target 2100+ total
- Elite spawn chance scales 2% → 20% across waves
- Enemy cap: 2000 max

---

## Data Files

All game content is defined in TOML files at `assets/data/`.

### Creature Schema (creatures.toml)
```toml
[[creatures]]
id = "fire_imp"
name = "Fire Imp"
color = "red"                    # red|blue|green|white|black|colorless
tier = 1                         # 1-4 (evolution tier)
creature_type = "ranged"         # melee|ranged|support|assassin
base_damage = 15.0
attack_speed = 1.0               # attacks per second
base_hp = 50.0
movement_speed = 100.0
attack_range = 220.0
crit_t1 = 5.0                    # percentage (5.0 = 5%)
crit_t2 = 0.0
crit_t3 = 0.0
evolves_into = "flame_fiend"
evolution_count = 3              # duplicates needed
respawn_time = 20.0              # seconds
projectile_count = 1
projectile_type = "basic"        # basic|piercing|explosive|homing|chain
```

### Weapon Schema (weapons.toml)
```toml
[[weapons]]
id = "ember_staff"
name = "Ember Staff"
color = "red"
tier = 1
affinity_amount = 10.0           # contributes to color affinity
auto_damage = 8.0
auto_speed = 1.5
auto_range = 350.0
projectile_count = 3
projectile_pattern = "spread"    # single|spread|melee_sweep|homing|orbit
evolves_into = "blazing_staff"
```

---

## Running & Testing

```bash
cargo run                # Start game (opens in deck builder)
cargo test               # Run all 191 tests
cargo build --release    # Optimized build
```

### Debug Controls (ESC menu)
- Pause/Resume gameplay
- Enemy spawn cap slider (100-5000)
- Toggle damage numbers
- Return to deck builder

---

## Development Status

### Current Phase: 29 - Pre-Run Deck Builder ✅
Full deck composition UI with starting weapon selection, probability bars, card management.

### Remaining Phases
- **Phase 25**: Debug spawn menu (Tab key)
- **Phase 26**: Bosses (phase system, telegraphed attacks)
- **Phase 27**: Player health + game over
- **Phase 28**: Balance pass + audio

### Future Features (Post-MVP)
- Siege Castle map with zones
- Multiple weapon slots
- Prophecy system (emergency abilities)
- Pity system for card rolls
- Save/load game
- Additional color decks (Blue, Green, White, Black)

---

## Phase History

<details>
<summary>Click to expand completed phases (1-29)</summary>

### Phase 1: Data Loading ✅
- TOML loading for creatures, weapons, artifacts, enemies, affinity
- Serde deserialization into Rust structs

### Phase 2: Basic Rendering ✅
- Bevy app with 1920x1080 window
- Player entity (white square), WASD movement, camera follow

### Phase 3: Creature Spawning ✅
- CreatureStats component with color-based rendering
- Spawn system from TOML data

### Phase 4: Creature AI ✅
- Formation system (creatures circle around player)
- Type-based positioning

### Phase 5: Enemy Spawning ✅
- Timer-based goblin spawning
- Spawn distance 600-800px from player

### Phase 6: Enemy AI ✅
- Chase behavior toward player
- Movement speed from TOML data

### Phase 7: Basic Combat ✅
- Auto-attack with projectiles
- Range checking, damage application

### Phase 8: Death System ✅
- Enemy death, entity removal
- Kill counter tracking

### Phase 9: Leveling ✅
- 15 kills = level up (1.1x scaling)
- Visual level-up effect

### Phase 10: Deck System ✅
- Probability-weighted card rolling
- Card type enum (Creature/Weapon/Artifact)

### Phase 11: Creature Stats ✅
- Full TOML stats integration
- Attack speed, movement speed from data

### Phase 12: Enemy Variety ✅
- Wave-based enemy unlocks
- Goblins, Archers, Wolves, Skeletons

### Phase 13: Basic UI ✅
- HUD (Level, Kills, Wave)
- Top-left positioning

### Phase 14: Creature Polish ✅
- HP bars (green/yellow/red)
- Enemy attacks on creatures
- Respawn timers (T1=20s, T2=30s, T3=45s)

### Phase 15: Crit System ✅
- 3-tier independent rolls
- Visual feedback (colors, screen shake)
- Floating damage numbers

### Phase 16: Artifacts Working ✅
- Target scopes (global, color, type, creature)
- Stat bonus stacking

### Phase 17: Weapons + Affinity ✅
- Weapon auto-attack
- Affinity threshold bonuses

### Phase 18: Director System ✅
- Adaptive spawn rates
- Performance monitoring

### Phase 19: Creature XP System ✅
- Per-creature kill tracking
- Individual leveling (+10%/level)

### Phase 20: UI Panels ✅
- Creature panel, artifact panel
- Deck probability display

### Phase 21: UI Polish ✅
- Card roll animations
- Level-up effects
- Kill rate tracking

### Phase 22: Creature Evolution ✅
- 3x duplicate detection
- Evolution choice UI
- Tier visual indicators

### Phase 23: Creature Visual Differentiation ✅
- Unique colors per creature ID
- Level indicators (L#)
- Tier borders (T2=green, T3=blue, T4=purple)

### Phase 24: Director Tuning ✅
- Enemy cap (2000)
- Spatial grid for O(1) lookups
- Projectile pooling (5000)
- Damage number pooling (500)

### Phase 29: Pre-Run Deck Builder ✅
- Full deck composition UI
- Starting weapon selection
- Probability bars and card management
- Tab system for card types

</details>

---

## Unit Tests

- Data loading tests (TOML parsing)
- Deck probability tests (weighted distribution)
- Leveling tests (threshold, overflow)
- Combat tests (range, damage)
- Wave tests (enemy variety)
- Crit tests (tier selection, overflow, cap)

---

## Notes

- Uses placeholder art (colored shapes)
- Spacebar spawning available until Phase 28 for testing
- Performance targets: 2000 enemies, 5000 projectiles at 60 FPS
