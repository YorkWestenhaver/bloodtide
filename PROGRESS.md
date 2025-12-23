# Bloodtide Development Progress

## Current Phase: 16 - Artifacts Working

---

## Phase 1: Data Loading ✅
- [x] Create `src/data/mod.rs` with all struct definitions matching TOML schemas
- [x] Implement serde deserialization for Creature, Weapon, Artifact, Enemy, Affinity
- [x] Create `src/resources/game_data.rs` to hold loaded data as Bevy Resources
- [x] Load all 5 TOML files on startup
- [x] Test: Print count of loaded items to console ("Loaded 14 creatures, 13 weapons...")

## Phase 2: Basic Rendering ✅
- [x] Set up Bevy app with DefaultPlugins in `main.rs`
- [x] Create window (1920x1080)
- [x] Spawn player entity (white square, 48x48)
- [x] Implement player movement (WASD + arrow keys)
- [x] Camera follows player
- [x] Test: Player can move around screen

## Phase 3: Creature Spawning ✅
- [x] Create `src/components/creature.rs` with Creature, CreatureStats, CreatureColor, CreatureType
- [x] Create `src/systems/spawning.rs` with spawn_creature function
- [x] Spawn Fire Imp on spacebar press (for testing)
- [x] Creature rendered as colored square based on color (red for fire)
- [x] Test: Press space, Fire Imp appears

## Phase 4: Creature AI ✅
- [x] Create `src/systems/ai.rs`
- [x] Creature follows player (stays in formation)
- [x] Basic formation: creatures arrange in circle around player (100px radius)
- [x] Test: Fire Imps follow player around

## Phase 5: Enemy Spawning ✅
- [x] Create `src/components/enemy.rs` with Enemy, EnemyStats, EnemyClass, EnemyType
- [x] Spawn goblins on timer (every 1.5 seconds)
- [x] Goblins rendered as green squares (28x28)
- [x] Spawn 600-800 pixels away from player at random angle
- [x] Test: Goblins spawn periodically

## Phase 6: Enemy AI ✅
- [x] Goblins chase player (enemy_chase_system)
- [x] Move directly toward player position
- [x] Uses movement_speed from TOML data (80 px/sec for goblins)
- [x] Test: Goblins move toward player

## Phase 7: Basic Combat ✅
- [x] Create `src/systems/combat.rs`
- [x] Add AttackTimer and AttackRange components to creatures
- [x] Creatures auto-attack nearest enemy in range
- [x] Spawn projectiles (8x8 colored squares) that fly toward enemies at 500px/sec
- [x] Simple damage calculation (base_damage only, no crits yet)
- [x] Enemy takes damage, current_hp decreases
- [x] Test: Fire Imps shoot projectiles at nearby goblins

## Phase 8: Death System ✅
- [x] Enemy death when HP <= 0
- [x] Remove dead enemy entity
- [x] Increment kill counter (global resource)
- [x] Test: Kill goblins, see kill count increase

## Phase 9: Leveling ✅
- [x] Track kills in GameState resource
- [x] Level up at 25 kills (with scaling: 1.2x multiplier each level)
- [x] Print "Level Up!" to console on level up
- [x] Level up visual effect (expanding yellow ring)
- [x] Test: Kill 25 goblins, level up triggers

## Phase 10: Deck System ✅
- [x] Create `src/resources/deck.rs` with DeckCard, CardType, PlayerDeck
- [x] Implement probability-weighted card rolling (roll_card method)
- [x] Initialize starter deck in main.rs (fire_imp, ember_hound, ember_staff, molten_core)
- [x] Roll card on level up in level_check_system
- [x] Spawn creature if creature card rolled
- [x] Test: Level up, new creature spawns based on deck roll

## Phase 11: Creature Stats ✅
- [x] Update CreatureStats to include all TOML stats (crit_t1, crit_t2, crit_t3)
- [x] Update spawn_creature to populate ALL stats from GameData TOML
- [x] Update creature_follow_system to use actual movement_speed from stats
- [x] AttackTimer already uses attack_speed from TOML (1/attack_speed = timer duration)
- [x] Test: Different creatures have different speeds and attack rates

## Phase 12: Enemy Variety ✅
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

## Phase 13: Basic UI ✅
- [x] Create `src/systems/ui.rs` with HudText marker component
- [x] spawn_ui_system: Spawns HUD in top-left with dark background
- [x] update_ui_system: Updates text with "Level: X | Kills: Y | Wave: Z"
- [x] Uses Bevy's Node/Text UI system with absolute positioning
- [x] Test: HUD displays and updates in real-time

## Phase 14: Creature Polish ✅
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

## Phase 15: Crit System ✅
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

## Phase 16: Artifacts Working ⬅️ CURRENT
- [ ] Create src/resources/artifact_buffs.rs:
  - ArtifactBuffs resource to track all active artifact effects
  - Store accumulated bonuses: damage_bonus, attack_speed_bonus, crit bonuses, etc.
- [ ] Update level_check_system:
  - When artifact card rolled, look up artifact in GameData
  - Apply artifact's bonuses to ArtifactBuffs resource
  - Print: "Artifact acquired: [name] - [effect description]"
- [ ] Update damage calculation:
  - Apply artifact_buffs.damage_bonus to base damage
  - Apply artifact crit bonuses to crit chances
- [ ] Apply attack speed bonuses to creature attack timers
- [ ] Apply HP bonuses when creatures spawn
- [ ] Handle target_scope filtering (global, color, type, creature-specific)
- [ ] Test: Roll artifacts, see damage/stats increase

## Phase 17: Weapons + Affinity
- [ ] Create src/components/weapon.rs:
  - Weapon marker component
  - WeaponData: id, color, affinity_amount, auto_damage, auto_speed, etc.
- [ ] Create src/resources/affinity.rs:
  - AffinityState resource: red_affinity, blue_affinity, etc.
- [ ] Weapon spawning:
  - When weapon card rolled, spawn weapon entity attached to player
  - Weapon auto-attacks (separate from creatures)
  - Weapon projectiles are smaller/different color
- [ ] Affinity calculation:
  - Sum affinity_amount from all equipped weapons per color
  - Look up thresholds from affinity.toml
  - Apply bonuses to matching color creatures
- [ ] Affinity threshold bonuses:
  - Damage bonus
  - Attack speed bonus
  - Crit tier unlocks
  - Special effects (burn_dot, etc.)
- [ ] Weapon evolution:
  - Track weapon counts
  - When recipe met (e.g., ember_staff x2), evolve to next tier
- [ ] Test: Roll weapons, see affinity increase, creatures deal more damage

## Phase 18: Creature XP + Evolution
- [ ] Add to CreatureStats: kills: u32, xp_level: u32
- [ ] Create creature_xp_system:
  - When creature's projectile kills enemy, increment that creature's kills
  - Track which creature dealt killing blow
- [ ] Creature leveling:
  - Check kills against kills_per_level array from TOML
  - On level up: +10% damage, +10% HP, slight attack speed boost
  - Print: "[Creature] leveled up to [X]!"
  - Visual effect on creature
- [ ] Evolution system:
  - Track count of each creature type in play
  - When 3x same creature exists, prompt/auto-evolve
  - Look up evolves_into from TOML
  - Despawn 3 creatures, spawn 1 evolved creature
  - Evolved creature starts at level 1 but higher base stats
- [ ] Test: Creatures gain kills, level up, 3x Fire Imps become Flame Fiend

## Phase 19: Better UI
- [ ] Creature Panel (right side of screen):
  - List all active creatures
  - Show: name, level, kills, HP bar
  - Show respawning creatures with timer
- [ ] Artifact Panel (left side or bottom):
  - List all acquired artifacts
  - Show: name, effect summary
  - Scrollable if many artifacts
- [ ] Affinity Display (top or corner):
  - Bar for each color showing current affinity
  - Show threshold markers
  - Highlight active bonuses
- [ ] Card Roll Popup:
  - When leveling up, show card being rolled
  - Animated reveal
  - Show card name, rarity, effect
  - Brief display then auto-dismiss (or click to dismiss)
- [ ] Damage number improvements:
  - Stack/combine rapid hits
  - Better positioning to avoid overlap
- [ ] Wave announcement:
  - "WAVE 5" text appears center screen briefly
- [ ] Test: All UI elements display and update correctly

## Phase 20: Director AI
- [ ] Create src/resources/director.rs:
  - Director resource tracking player state
  - Metrics: player_dps, creature_count, player_hp_percent, time_since_damage
- [ ] Create director_analysis_system:
  - Calculate player's current DPS (track damage dealt over time)
  - Calculate stress level based on metrics
- [ ] Adaptive spawning:
  - Stomping (high DPS, full HP): increase spawn rate, add elites
  - Comfortable: standard spawning
  - Struggling (low HP, creatures dying): reduce spawn rate, add health drops
  - Nearly Dead: pause spawning briefly
- [ ] Counter-color spawning:
  - Track player's primary color (most creatures of that color)
  - At wave 20+, occasionally spawn color-resist enemies
  - Water enemies vs Red deck, etc.
- [ ] Enemy HP scaling:
  - enemy_hp = player_dps × target_ttk × difficulty_modifier
  - Fodder TTK: 0.75 seconds
  - Elite TTK: 4 seconds
- [ ] Test: Play well → more enemies; play poorly → easier spawns

## Phase 21: Bosses
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

## Phase 22: Formation by Type
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

## Phase 23: Player Health + Game Over
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

## Phase 24: Polish + Balance
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
- Keep spacebar spawning until Phase 24 for testing

## Current Blockers
(None)

## Last Updated
Phase 15 completed - Crit system with visual feedback and floating damage numbers
