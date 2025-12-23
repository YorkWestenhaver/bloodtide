# Bloodtide Development Progress

## Current Phase: 15 - (Next Phase TBD)

---

## Phase 1: Data Loading
- [x] Create `src/data/mod.rs` with all struct definitions matching TOML schemas
- [x] Implement serde deserialization for Creature, Weapon, Artifact, Enemy, Affinity
- [x] Create `src/resources/game_data.rs` to hold loaded data as Bevy Resources
- [x] Load all 5 TOML files on startup
- [x] Test: Print count of loaded items to console ("Loaded 14 creatures, 13 weapons...")

## Phase 2: Basic Rendering
- [x] Set up Bevy app with DefaultPlugins in `main.rs`
- [x] Create window (1920x1080)
- [x] Spawn player entity (white square, 48x48)
- [x] Implement player movement (WASD + arrow keys)
- [x] Camera follows player
- [x] Test: Player can move around screen

## Phase 3: Creature Spawning
- [x] Create `src/components/creature.rs` with Creature, CreatureStats, CreatureColor, CreatureType
- [x] Create `src/systems/spawning.rs` with spawn_creature function
- [x] Spawn Fire Imp on spacebar press (for testing)
- [x] Creature rendered as colored square based on color (red for fire)
- [x] Test: Press space, Fire Imp appears

## Phase 4: Creature AI
- [x] Create `src/systems/ai.rs`
- [x] Creature follows player (stays in formation)
- [x] Basic formation: creatures arrange in circle around player (100px radius)
- [x] Test: Fire Imps follow player around

## Phase 5: Enemy Spawning
- [x] Create `src/components/enemy.rs` with Enemy, EnemyStats, EnemyClass, EnemyType
- [x] Spawn goblins on timer (every 1.5 seconds)
- [x] Goblins rendered as green squares (28x28)
- [x] Spawn 600-800 pixels away from player at random angle
- [x] Test: Goblins spawn periodically

## Phase 6: Enemy AI
- [x] Goblins chase player (enemy_chase_system)
- [x] Move directly toward player position
- [x] Uses movement_speed from TOML data (80 px/sec for goblins)
- [x] Test: Goblins move toward player

## Phase 7: Basic Combat
- [x] Create `src/systems/combat.rs`
- [x] Add AttackTimer and AttackRange components to creatures
- [x] Creatures auto-attack nearest enemy in range
- [x] Spawn projectiles (8x8 colored squares) that fly toward enemies at 500px/sec
- [x] Simple damage calculation (base_damage only, no crits yet)
- [x] Enemy takes damage, current_hp decreases
- [x] Test: Fire Imps shoot projectiles at nearby goblins

## Phase 8: Death System
- [x] Enemy death when HP <= 0
- [x] Remove dead enemy entity
- [x] Increment kill counter (global resource)
- [x] Test: Kill goblins, see kill count increase

## Phase 9: Leveling
- [x] Track kills in GameState resource
- [x] Level up at 25 kills (with scaling: 1.2x multiplier each level)
- [x] Print "Level Up!" to console on level up
- [x] Level up visual effect (expanding yellow ring)
- [x] Test: Kill 25 goblins, level up triggers

## Phase 10: Deck System
- [x] Create `src/resources/deck.rs` with DeckCard, CardType, PlayerDeck
- [x] Implement probability-weighted card rolling (roll_card method)
- [x] Initialize starter deck in main.rs (fire_imp, ember_hound, ember_staff, molten_core)
- [x] Roll card on level up in level_check_system
- [x] Spawn creature if creature card rolled
- [x] Test: Level up, new creature spawns based on deck roll

## Phase 11: Creature Stats
- [x] Update CreatureStats to include all TOML stats (crit_t1, crit_t2, crit_t3)
- [x] Update spawn_creature to populate ALL stats from GameData TOML
- [x] Update creature_follow_system to use actual movement_speed from stats
- [x] AttackTimer already uses attack_speed from TOML (1/attack_speed = timer duration)
- [x] Test: Different creatures have different speeds and attack rates

## Phase 12: Enemy Variety
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

## Phase 13: Basic UI
- [x] Create `src/systems/ui.rs` with HudText marker component
- [x] spawn_ui_system: Spawns HUD in top-left with dark background
- [x] update_ui_system: Updates text with "Level: X | Kills: Y | Wave: Z"
- [x] Uses Bevy's Node/Text UI system with absolute positioning
- [x] Test: HUD displays and updates in real-time

## Phase 14: Creature Polish
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

---

## Future Phases (Not Yet)
- [ ] Crit system (3 tiers)
- [ ] Affinity calculation
- [ ] Artifacts
- [ ] Weapon system
- [ ] Evolution system
- [ ] Creature XP/leveling
- [ ] Director AI
- [ ] Bosses

---

## Notes
- Use placeholder art (colored shapes) for everything
- Test each phase before moving to next
- Commit after each completed phase

## Current Blockers
(None yet)

## Last Updated
Phase 14 completed - Creature polish: enemies attack creatures, creatures die and respawn, HP bars above creatures
