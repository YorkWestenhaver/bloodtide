# Bloodtide Development Progress

## Current Phase: 24 - Director Tuning

---

## Known Issues / Backlog

### Critical (Blocking Progression)
- [x] ~~**Creature combination disabled**~~ - Fixed in Phase 22

### Visual/Polish (Not Blocking)
- [ ] **Level 5 crown idea** - Corners get colored, crown appears in center at max level (future enhancement)
- [ ] **Creature range too short** - Some creatures have very little range at start; consider baseline increase OR add a long-range creature early

### Scheduled (Now in Phases)
- Phase 23: Creature visual differentiation
- Phase 24: Director over-spawning fix
- Phase 25: Debug spawn menu

### Deferred
- [ ] **21F - Wave mechanics tuning** - Adjust wave mechanics, currently deprioritized
- [ ] **GPU Acceleration** - Implement GPU instancing, spatial partitioning, batch rendering if performance becomes an issue

---

## Phases

### Phase 1: Data Loading ✅

- [x] Create `src/data/mod.rs` with all struct definitions matching TOML schemas
- [x] Implement serde deserialization for Creature, Weapon, Artifact, Enemy, Affinity
- [x] Create `src/resources/game_data.rs` to hold loaded data as Bevy Resources
- [x] Load all 5 TOML files on startup
- [x] Test: Print count of loaded items to console ("Loaded 14 creatures, 13 weapons...")

### Phase 2: Basic Rendering ✅

- [x] Set up Bevy app with DefaultPlugins in `main.rs`
- [x] Create window (1920x1080)
- [x] Spawn player entity (white square, 48x48)
- [x] Implement player movement (WASD + arrow keys)
- [x] Camera follows player
- [x] Test: Player can move around screen

### Phase 3: Creature Spawning ✅

- [x] Create `src/components/creature.rs` with Creature, CreatureStats, CreatureColor, CreatureType
- [x] Create `src/systems/spawning.rs` with spawn_creature function
- [x] Spawn Fire Imp on spacebar press (for testing)
- [x] Creature rendered as colored square based on color (red for fire)
- [x] Test: Press space, Fire Imp appears

### Phase 4: Creature AI ✅

- [x] Create `src/systems/ai.rs`
- [x] Creature follows player (stays in formation)
- [x] Basic formation: creatures arrange in circle around player (100px radius)
- [x] Test: Fire Imps follow player around

### Phase 5: Enemy Spawning ✅

- [x] Create `src/components/enemy.rs` with Enemy, EnemyStats, EnemyClass, EnemyType
- [x] Spawn goblins on timer (every 1.5 seconds)
- [x] Goblins rendered as green squares (28x28)
- [x] Spawn 600-800 pixels away from player at random angle
- [x] Test: Goblins spawn periodically

### Phase 6: Enemy AI ✅

- [x] Goblins chase player (enemy_chase_system)
- [x] Move directly toward player position
- [x] Uses movement_speed from TOML data (80 px/sec for goblins)
- [x] Test: Goblins move toward player

### Phase 7: Basic Combat ✅

- [x] Create `src/systems/combat.rs`
- [x] Add AttackTimer and AttackRange components to creatures
- [x] Creatures auto-attack nearest enemy in range
- [x] Spawn projectiles (8x8 colored squares) that fly toward enemies at 500px/sec
- [x] Simple damage calculation (base_damage only, no crits yet)
- [x] Enemy takes damage, current_hp decreases
- [x] Test: Fire Imps shoot projectiles at nearby goblins

### Phase 8: Death System ✅

- [x] Enemy death when HP <= 0
- [x] Remove dead enemy entity
- [x] Increment kill counter (global resource)
- [x] Test: Kill goblins, see kill count increase

### Phase 9: Leveling ✅

- [x] Track kills in GameState resource
- [x] Level up at 25 kills (with scaling: 1.2x multiplier each level)
- [x] Print "Level Up!" to console on level up
- [x] Level up visual effect (expanding yellow ring)
- [x] Test: Kill 25 goblins, level up triggers

### Phase 10: Deck System ✅

- [x] Create `src/resources/deck.rs` with DeckCard, CardType, PlayerDeck
- [x] Implement probability-weighted card rolling (roll_card method)
- [x] Initialize starter deck in main.rs (fire_imp, ember_hound, ember_staff, molten_core)
- [x] Roll card on level up in level_check_system
- [x] Spawn creature if creature card rolled
- [x] Test: Level up, new creature spawns based on deck roll

### Phase 11: Creature Stats ✅

- [x] Update CreatureStats to include all TOML stats (crit_t1, crit_t2, crit_t3)
- [x] Update spawn_creature to populate ALL stats from GameData TOML
- [x] Update creature_follow_system to use actual movement_speed from stats
- [x] AttackTimer already uses attack_speed from TOML (1/attack_speed = timer duration)
- [x] Test: Different creatures have different speeds and attack rates

### Phase 12: Enemy Variety ✅

- [x] Add total_kills and kills_at_wave_start to GameState
- [x] Wave progression: every 50 kills advances to next wave
- [x] Enemy spawning based on wave:
  - Wave 1-5: Goblins only
  - Wave 6-10: +20% Goblin Archers
  - Wave 11-14: +15% Wolves, +20% Archers
  - Wave 15+: +Skeletons
- [x] Different colors for enemies (goblin=green, archer=dark green, wolf=gray)
- [x] spawn_enemy now prints HP and Speed from TOML
- [x] Test: Wave progression and enemy variety

### Phase 13: Basic UI ✅

- [x] Create `src/systems/ui.rs` with HudText marker component
- [x] spawn_ui_system: Spawns HUD in top-left with dark background
- [x] update_ui_system: Updates text with "Level: X | Kills: Y | Wave: Z"
- [x] Uses Bevy's Node/Text UI system with absolute positioning
- [x] Test: HUD displays and updates in real-time

### Phase 14: Creature Polish ✅

- [x] Updated creature colors (more vivid: red=fire, blue=ice, green=nature, white=holy, black=dark purple, colorless=gray)
- [x] Added max_hp to CreatureStats (initialized to base_hp)
- [x] Created enemy_attack_system: enemies deal damage to nearby creatures
- [x] Added EnemyAttackTimer component to enemies
- [x] Created creature_death_system: creatures despawn when HP <= 0
- [x] Created RespawnQueue resource and RespawnEntry struct
- [x] Created respawn_system: respawns creatures after tier-based delay (T1=20s, T2=30s, T3=45s)
- [x] Created HP bars system (`src/systems/hp_bars.rs`):
  - spawn_hp_bars_system: creates HP bars above creatures
  - update_hp_bars_system: updates position, width, and color based on HP%
  - HP bar colors: green (>60%), yellow (30-60%), red (<30%)
- [x] Test: Enemies attack creatures, creatures die and respawn, HP bars show damage

### Phase 15: Crit System ✅

- [x] Create src/math/mod.rs and src/math/crit.rs
- [x] CritTier enum: None, Normal, Mega, Super
- [x] CritResult struct: tier, final_damage, base_damage
- [x] calculate_damage_with_crits() function:
  - Takes base_damage and crit chances for all 3 tiers
  - Each tier rolls independently
  - Tier 1 Normal Crit: 2× damage (overflow adds bonus if >100%)
  - Tier 2 Mega Crit: damage² (squared)
  - Tier 3 Super Crit: damage^4 (capped at 1e15)
  - Highest successful tier wins
- [x] Update combat damage calculation to use calculate_damage_with_crits()
- [x] Pass creature's crit_t1, crit_t2, crit_t3 from CreatureStats
- [x] Visual feedback for crits:
  - Normal crit: yellow projectile
  - Mega crit: orange projectile + small screen shake
  - Super crit: red/purple projectile + bigger screen shake
- [x] Floating damage numbers:
  - Spawn text at hit location showing damage
  - Colors: white (normal), yellow (T1), orange (T2), red (T3)
  - Float upward and fade over 0.8 seconds
  - Large numbers use scientific notation (1.5e6)
- [x] Register math module in main.rs
- [x] Test: Fire Imps occasionally show yellow projectiles and damage numbers

### Phase 16: Artifacts Working ✅

- [x] Create src/resources/artifact_buffs.rs:
  - ArtifactBuffs resource to track all active artifact effects
  - StatBonuses struct with damage_bonus, attack_speed_bonus, hp_bonus, crit bonuses
  - HashMap storage for global, color, type, and creature-specific bonuses
  - acquired_artifacts Vec<String> for UI tracking
- [x] Create apply_artifact() method:
  - Takes artifact id, looks up in GameData
  - Based on target_scope, adds bonuses to appropriate bucket (global/color/type/creature)
  - Prints: "Artifact acquired: [name] - [description]"
- [x] Create get_total_bonuses() method:
  - Takes creature's color, type, and id
  - Returns combined bonuses from all applicable sources
- [x] Update level_check_system:
  - When artifact card rolled, calls artifact_buffs.apply_artifact()
- [x] Update creature_attack_system:
  - Apply damage bonus: base_damage \* (1 + total_damage_bonus / 100)
  - Apply crit bonuses to crit chances before rolling
- [x] Update spawn_creature:
  - Apply HP bonus when creature spawns: base_hp \* (1 + total_hp_bonus / 100)
  - Apply attack speed bonus to AttackTimer
- [x] Handle target_scope filtering (global, color, type, creature-specific)
- [x] Register ArtifactBuffs resource in main.rs
- [x] Test: 119 tests passing, build succeeds

### Phase 17: Weapons + Affinity ✅

- [x] Create src/components/weapon.rs:
  - Weapon marker component
  - WeaponData: id, name, color, tier, affinity_amount
  - WeaponStats: auto_damage, auto_speed, auto_range, projectile_count, pattern, speed
  - WeaponAttackTimer: timer for auto-attacks
- [x] Create src/resources/affinity.rs:
  - AffinityState resource: red, blue, green, white, black, colorless
  - add() and remove() methods for managing affinity
  - get() method to retrieve affinity by color
  - AffinityBonus struct with damage/attack_speed/hp/crit bonuses
- [x] Affinity threshold bonuses implemented
- [x] Weapon auto-attack system working
- [x] Test: Weapons contribute to affinity and auto-attack

### Phase 18: Director System ✅

- [x] Director AI tracks player state (DPS, HP%, creature count)
- [x] Adaptive spawn rate based on performance
- [x] Wave composition adjustments
- [x] Test: Director responds to player performance

### Phase 19: Creature XP System ✅

- [x] Individual kill tracking per creature
- [x] Creature leveling (kills_per_level thresholds)
- [x] Level-up stat bonuses (+10% per level)
- [x] Visual feedback for creature level-ups
- [x] Test: Creatures gain XP and level independently

### Phase 20: UI Panels ✅

- [x] Creature panel (right side)
- [x] Artifact panel
- [x] Deck probability display
- [x] Test: UI shows all relevant info

### Phase 21: UI Polish ✅

- [x] 21A: Card roll queue and animations
- [x] 21B: Level up effects (particles, flash, text)
- [x] 21C: Screen effects system
- [x] 21D: Kill rate tracking
- [x] 21E: HUD layout improvements (3 lines)
- [x] 21G: Weapon stats display with tooltips

### Phase 22: Creature Evolution & Combination ✅

- [x] Fix creature combination system:
  - 3x same creature → evolution trigger
  - Evolution choice UI (keep leveled creatures OR combine)
- [x] Visual feedback for evolution:
  - Evolution ready indicator
  - Combination animation
  - New creature spawn effect
- [x] Evolution data integration:
  - Use evolves_into from TOML
  - Reset to L1 with higher base stats
- [x] Test: Get 3 Fire Imps, evolve to Flame Fiend, verify stats

### Phase 23: Creature Visual Differentiation ✅

- [x] Different colored squares per creature ID (not just color affinity)
  - Tier 1: Fire Imp (bright orange-red), Ember Hound (orange), Fire Spirit (peach)
  - Tier 2: Flame Fiend (deep crimson), Hellhound (burnt orange), Inferno Knight (dark red), Magma Elemental (magma orange), Greater Fire Spirit (bright peach)
  - Tier 3: Inferno Demon (very dark red), Hellhound Alpha (intense orange-red), Inferno Warlord (maroon), Phoenix (brilliant orange)
  - Tier 4: Inferno Titan (nearly black-red), Eternal Phoenix (golden-orange)
- [x] Level indicator on creature:
  - Small "L#" text below each creature sprite
  - Updates automatically when creature levels up
- [x] Evolution tier indicator (T1/T2/T3 visible)
  - Tier 2+: Colored border behind creature sprite
  - T2: Green border, T3: Blue border, T4: Purple border, T5+: Gold border
- [x] Test: Build and tests pass, visual differentiation implemented

### Phase 24: Director Tuning & GPU Optimization ⬅️ CURRENT

- [x] Add enemy cap (MAX_ENEMIES = 2000) to prevent over-spawning
- [x] Add spatial grid for O(1) enemy lookups in collision detection
  - SpatialGrid resource with 256px cells
  - update_spatial_grid_system runs before combat
  - creature_attack_system uses grid instead of iterating all enemies
- [x] Implement projectile pooling (pre-allocate 5000, toggle visibility)
  - ProjectilePool resource in `src/resources/pools.rs`
  - init_pools_system pre-spawns entities at startup
  - Projectiles reused via Visibility::Hidden/Visible toggle
- [x] Implement damage number pooling (pre-allocate 500)
  - DamageNumberPool resource in `src/resources/pools.rs`
  - Same pattern as projectiles
- [x] Fix Bevy query conflicts in combat.rs
  - Added proper With<T>/Without<T> filters to all queries
  - Ensures disjoint entity sets for Transform access
- [x] Add toggle for damage numbers (pause menu)
  - `show_damage_numbers` setting in DebugSettings
  - Checkbox in pause menu under "Display Options"
  - Significantly improves performance when disabled
- [x] Add configurable enemy cap slider (debug menu)
  - `max_enemies` setting in DebugSettings (default: 1500)
  - Slider in debug menu under "Spawning" section
  - Range: 100-5000 in steps of 100
- [ ] Better pacing curve
- [ ] Smooth difficulty transitions
- [ ] Test: Standing still doesn't cause enemy flood

### Phase 25: Debug Spawn Menu

- [ ] Tab key toggles spawn menu
- [ ] Menu slides in from left side
- [ ] Pause game while menu open
- [ ] List all creatures from TOML
- [ ] Click to spawn creature at player position
- [ ] Useful for testing creature abilities and combinations

### Phase 26: Bosses

- [ ] Create src/components/boss.rs:
  - Boss marker component
  - BossData: id, phases, current_phase, phase_hp_thresholds
  - BossPhase: attack patterns, abilities
- [ ] Boss spawning:
  - Every 10 waves, spawn boss instead of normal enemies
  - Pause normal spawning during boss
  - Boss is much larger sprite (96x96 or bigger)
  - Boss has visible HP bar (larger, at top of screen)
- [ ] Boss AI:
  - Phase 1 (100-70% HP): Basic attacks
  - Phase 2 (70-40% HP): New attacks + summon adds
  - Phase 3 (40-0% HP): Enrage, faster attacks, danger zones
- [ ] Boss attacks:
  - Telegraphed attacks (show warning zone before damage)
  - Slam: AoE in front, kills creatures in zone
  - Summon: Spawn minions periodically
- [ ] Boss death:
  - Guaranteed rare+ card roll
  - Bonus artifact
  - Large XP burst
  - Victory fanfare
- [ ] First boss: Troll Chief (Wave 10)
- [ ] Test: Reach wave 10, fight boss, phases change, boss drops loot

### Phase 27: Formation by Type

- [ ] Update creature AI based on CreatureType:
  - Melee: Position in front of player (toward nearest enemy)
  - Ranged: Position behind player (away from enemies)
  - Support: Orbit close to player
  - Assassin: Ignore formation, hunt highest value target
- [ ] Formation visualization (debug mode):
  - Show front/back line indicators
- [ ] Melee creatures:
  - Body-block enemies (enemies must attack them first)
  - Shorter attack range but higher damage
- [ ] Test: Mixed army positions correctly, melee in front

### Phase 28: Player Health + Game Over

- [ ] Add player HP:
  - Player has health (starts at 100)
  - Enemies that reach player deal damage
  - Player HP bar at bottom of screen
- [ ] Creature protection:
  - Melee creatures intercept enemies
  - Enemies prefer attacking creatures first
  - Only attack player if no creatures nearby
- [ ] Game over:
  - Player HP reaches 0 = death
  - Show game over screen with stats
  - Option to restart
- [ ] Test: Enemies can kill player if all creatures dead

### Phase 29: Polish + Balance

- [ ] Remove spacebar test spawning (creatures only from deck)
- [ ] Balance pass:
  - Creature stats vs enemy stats
  - Spawn rates vs kill speed
  - Level up pacing
  - Crit chances
- [ ] Visual polish:
  - Better death effects
  - Hit flash on damaged entities
  - Projectile trails
- [ ] Audio (placeholder):
  - Hit sounds
  - Death sounds
  - Level up sound
  - Crit sound (satisfying)
- [ ] Performance check:
  - Test with 100+ creatures
  - Test with 500+ enemies
  - Optimize if needed

---

## Future Features (Post-MVP)

- [ ] Siege Castle map with zones
- [ ] Zone objectives
- [ ] Optional dungeon areas
- [ ] Multiple weapon slots
- [ ] Prophecy system
- [ ] Pity system for card rolls
- [ ] Trash card mechanic
- [ ] Settings menu
- [ ] Save/load game
- [ ] Multiple deck loadouts

---

## Unit Tests ✅

- [x] Data loading tests (creatures, weapons, artifacts, enemies, affinity)
- [x] Deck probability tests (weighted distribution)
- [x] Leveling tests (threshold, overflow)
- [x] Combat tests (range check, damage)
- [x] Wave tests (enemy variety by wave)

---

## Notes

- Use placeholder art (colored shapes) for everything
- Test each phase before moving to next
- Commit after each completed phase
- Keep spacebar spawning until Phase 28 for testing

## Current Blockers

(None)

## Last Updated

Phase 24 GPU Optimization complete. Added enemy cap (configurable, default 1500), spatial grid for O(1) collision lookups, projectile pooling (5000), and damage number pooling (500). Fixed Bevy query conflicts with proper With/Without filters. Added damage number toggle to pause menu and enemy cap slider to debug menu for runtime performance tuning.
