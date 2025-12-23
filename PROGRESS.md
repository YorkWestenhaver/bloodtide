# Bloodtide Development Progress

## Current Phase: 21 - Bosses

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

## Phase 16: Artifacts Working ✅

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

## Phase 17: Weapons + Affinity ✅

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
  - get_affinity_bonuses() function: looks up thresholds from GameData
- [x] Weapon spawning:
  - spawn_weapon() function: creates weapon entity, adds affinity to AffinityState
  - Weapons auto-attack via weapon_attack_system
  - Weapon projectiles are white/silver colored, 6x6 size
- [x] Affinity calculation:
  - Sum affinity_amount from all equipped weapons per color
  - Look up thresholds from affinity.toml via GameData
  - Apply bonuses to matching color creatures
- [x] Affinity threshold bonuses:
  - Damage bonus applied to creatures
  - Attack speed bonus applied to creatures
  - HP bonus applied to creatures
  - Crit T2/T3 tier unlocks (require affinity threshold to activate)
- [x] Weapon evolution:
  - try_weapon_evolution() checks evolution_recipe for each weapon
  - When recipe met (e.g., ember_staff x2), despawn components, spawn evolved
  - Properly manages affinity removal/addition during evolution
- [x] Update level_check_system to handle weapon cards
- [x] Update creature_attack_system to apply affinity bonuses
- [x] Register AffinityState resource and weapon_attack_system in main.rs
- [x] Test: 130 tests passing, build succeeds

## Phase 18: Creature XP + Evolution ✅

- [x] Update CreatureStats with new fields:
  - kills_for_next_level: u32 (from kills_per_level array index)
  - max_level: u32 (from TOML)
  - evolves_into: String (from TOML)
  - evolution_count: u32 (from TOML)
- [x] Track kill attribution:
  - Added source_creature: Option<Entity> to Projectile component
  - creature_attack_system stores creature Entity in projectiles
  - PendingKillCredit component spawned when projectile kills enemy
- [x] Create src/systems/creature_xp.rs:
  - creature_xp_system: processes PendingKillCredit, increments creature kills
  - On level up: +10% base_damage, +10% max_hp (heals by amount gained)
  - Gets next threshold from kills_per_level array
  - Prints: "[Creature name] leveled up to [X]!"
  - CreatureLevelUpEffect: green expanding ring visual
  - creature_level_up_effect_system: animates the effect
- [x] Evolution system:
  - creature_evolution_system: groups creatures by id
  - When count >= evolution_count (usually 3), auto-evolves
  - Despawns component creatures, spawns evolved creature at average position
  - Prefers lowest level creatures for evolution
  - EvolutionEffect: golden flash visual at each consumed creature
  - evolution_effect_system: animates the effect
  - Prints: "3x [old creature] evolved into [new creature]!"
- [x] Update spawn_creature to set kills_for_next_level from TOML kills_per_level[0]
- [x] Register creature_xp_system, creature_level_up_effect_system, creature_evolution_system, evolution_effect_system in main.rs
- [x] Test: 133 tests passing, build succeeds

## Phase 19: Better UI ✅

- [x] Creature Panel (right side of screen):
  - Created CreaturePanel and CreaturePanelContent marker components
  - spawn_creature_panel_system: positions panel absolute right, top
  - update_creature_panel_system: lists all creatures with name, level (Lv.X), kills (K:X)
  - Shows HP bar per creature with color coding (green/yellow/red)
  - Respawning creatures shown with grayed name and countdown timer
- [x] Artifact Panel (bottom-left):
  - Created ArtifactPanel and ArtifactPanelContent marker components
  - spawn_artifact_panel_system: positions panel absolute left, bottom
  - update_artifact_panel_system: lists acquired artifacts with name and effect summary
  - Tier-colored artifact names (gray/green/blue/purple/gold)
  - Shows "None yet" when no artifacts acquired
  - Max height with overflow scrolling
- [x] Affinity Display (top-right below creature panel):
  - Created AffinityDisplay and AffinityDisplayContent marker components
  - spawn_affinity_display_system: positions panel below creature panel
  - update_affinity_display_system: shows bars for each color with affinity > 0
  - Visual bar with fill based on affinity value (0-100)
  - Threshold markers at 11, 26, 51, 76, 100
  - Markers turn yellow when reached
- [x] Card Roll Popup:
  - Created CardRollPopup component with timer, card_name, card_type, tier
  - Created CardRollState resource with pending_popup field
  - level_check_system sets pending_popup when card rolled
  - show_card_roll_popup_system: spawns centered popup with tier-colored border
  - Shows "New [Type]!", card name, and tier name (Common/Uncommon/Rare/Epic/Legendary)
  - card_roll_popup_update_system: fades out, auto-dismisses after 2.5s or click
- [x] Damage number improvements:
  - Created DamageNumberOffsets resource with recent_positions tracking
  - calculate_damage_number_offset function for avoiding overlap
  - Random horizontal offset + vertical stacking for nearby hits
- [x] Wave announcement:
  - Created WaveAnnouncement component with timer and wave_number
  - Created WaveAnnouncementState resource tracking last_announced_wave
  - show_wave_announcement_system: spawns "WAVE X" text at center when wave changes
  - wave_announcement_update_system: scale-up animation then fade out
  - Milestone waves (10, 20, etc.) shown in gold
- [x] HUD Updated:
  - Shows "Level: X | Kills: Y | Wave: Z | Creatures: N | Artifacts: M"
  - Creature and artifact counts update in real-time
- [x] All systems registered in main.rs with proper ordering
- [x] Test: 136 tests passing, build succeeds

## Phase 20: Director AI ✅

- [x] Created src/resources/director.rs:
  - Director resource tracking: player_dps, creature_count, total_creature_hp_percent, stress_level, enemies_alive
  - spawn_rate_modifier, damage_dealt_window, current_fps, low_fps_duration, performance_throttle
- [x] MASSIVE HORDE spawning (Vampire Survivors-style):
  - Wave 1-5: 25-50 enemies per spawn, target 350 on screen
  - Wave 6-10: 50-100 enemies per spawn, target 750 on screen
  - Wave 11-15: 100-175 enemies per spawn, target 1500 on screen
  - Wave 16-20: 175-250 enemies per spawn, target 2750 on screen
  - Wave 21-30: 250-400 enemies per spawn, target 5000 on screen
  - Wave 31+: 400-600 enemies per spawn, target 6000+ on screen
- [x] Spawn from 2-4 cluster points per spawn event (360 degree surround)
- [x] Soft cap system with dynamic spawn intervals:
  - Below 50% target: 0.1s interval (FAST)
  - Below target: 0.2s interval
  - At target: 0.3s interval
  - Above target: 0.5s interval
- [x] Stress-based adaptive spawning:
  - Stomping (stress < 0.3): 2x spawn rate
  - Comfortable: normal spawn rate
  - Struggling (stress > 0.7): 0.6x spawn rate (minimum floor applies)
  - Minimum 15 enemies per second always
- [x] Elite spawning by wave:
  - Wave 1-5: 2% elites
  - Wave 6-10: 5% elites
  - Wave 11-15: 10% elites
  - Wave 16-20: 15% elites
  - Wave 21+: 20% elites
  - Elites: 3x HP, 1.5x damage, slightly larger, brighter color
- [x] Enemy HP scaling: 1.0 + (wave - 1) \* 0.08
- [x] Performance safeguards:
  - FPS tracking per frame
  - FPS < 30 for 3+ seconds: reduce spawn rate by 25%
  - FPS < 20: reduce spawn rate by 50%, print warning
  - FPS > 45: restore normal spawn rate
- [x] Enemy cleanup: despawn enemies >2500 pixels from player
- [x] HUD updated: "Lv:X | K:Y | W:Z | C:N | E:M | FPS:X"
- [x] director_update_system and enemy_cleanup_system registered in main.rs
- [x] Test: 142 tests passing, build succeeds

## Phase 20B: Debug Settings Menu ✅

- [x] Created src/resources/debug_settings.rs:
  - DebugSettings resource with all tunable values
  - Speed multipliers: player, creature, enemy (range 0.1 to 5.0)
  - Damage multipliers: creature, enemy (range 0.1 to 10.0)
  - enemy_spawn_rate_multiplier (range 0.1 to 5.0)
  - Crit bonuses: crit_t1_bonus, crit_t2_bonus, crit_t3_bonus (range 0 to 100)
  - Wave/level overrides: current_wave_override, current_level_override (None = use normal)
  - god_mode: bool (creatures can't die, healed to max HP instead)
  - show_fps: bool, show_enemy_count: bool
  - MenuState enum: Closed, DebugMenuOpen, PauseMenuOpen
  - menu_toggle_mode: bool (toggle vs hold for Shift key)
- [x] Created src/systems/debug_menu.rs:
  - Debug menu slides in from left side of screen
  - Press Shift: opens/closes debug menu (or hold based on toggle_mode setting)
  - Menu shows sliders for ALL tunable values with label, current value, slider bar
  - Checkboxes for bool values (god_mode, show_fps, etc.)
  - Reset to Defaults button at bottom
  - Game CONTINUES running while debug menu is open (no pause)
  - Slider interaction: click on bar to set value
- [x] Pause Menu (center screen overlay):
  - Press Escape: pauses game and opens pause menu
  - Dark overlay behind pause menu
  - "Resume" button (or press Escape again)
  - "Toggle Mode" checkbox: controls whether Shift is hold vs toggle
  - "Restart Run" button (resets game to wave 1, level 1)
  - "Quit" button (closes application)
  - Game is PAUSED while pause menu is open
- [x] Applied debug settings to gameplay:
  - Player movement: speed \* player_speed_multiplier
  - Creature movement: speed \* creature_speed_multiplier
  - Creature damage: damage \* creature_damage_multiplier + crit bonuses
  - Enemy damage: damage \* enemy_damage_multiplier
  - Enemy movement: speed \* enemy_speed_multiplier
  - Spawn system: spawn_rate \* enemy_spawn_rate_multiplier (both interval and count)
  - Crit calculation: add crit_tX_bonus to creature's base crit chances
  - Wave/level overrides force GameState values when set
  - God mode heals creatures to max HP instead of dying
- [x] HUD updates based on debug settings:
  - If show_fps: display FPS in corner
  - If show_enemy_count: display enemy count in HUD
  - Shows "GOD" indicator when god_mode enabled
  - Shows "PAUSED" indicator when paused
- [x] All gameplay systems check is_paused() and skip processing when true
- [x] All menu systems registered in main.rs with proper ordering
- [x] Test: 155 tests passing, build succeeds

## Phase 21A: Projectile Stats Foundation ✅

- [x] Added ProjectileConfig component to creatures:
  - count: u32 (number of projectiles per attack)
  - spread: f32 (total spread angle in radians)
  - size: f32 (projectile size in pixels)
  - speed: f32 (projectile speed in pixels/sec)
- [x] Updated Creature data struct in data/mod.rs:
  - Added projectile_count, projectile_spread, projectile_size, projectile_speed fields
  - Serde defaults: count=1, spread=0.0, size=8.0, speed=500.0
- [x] Updated creatures.toml with projectile config for all creatures:
  - Tier 1: single projectile, base stats
  - Tier 2: some get 2 projectiles with spread (e.g., flame_fiend)
  - Tier 3: 2-3 projectiles, larger spread (e.g., inferno_demon gets 3)
  - Tier 4: 3-5 projectiles (e.g., eternal_phoenix gets 5 with 0.8 spread)
- [x] Updated spawn_creature to add ProjectileConfig component from TOML data
- [x] Updated Projectile component with size and speed fields
- [x] Updated creature_attack_system for multi-shot spread:
  - Spawns multiple projectiles based on count
  - Spreads projectiles evenly across spread angle
  - Each projectile uses custom size/speed from config
- [x] Added projectile debug settings to DebugSettings:
  - projectile_count_bonus: i32 (added to base count, can be negative)
  - projectile_size_multiplier: f32 (multiplied by base size)
  - projectile_speed_multiplier: f32 (multiplied by base speed)
  - attack_speed_multiplier: f32 (scales attack timer tick rate)
- [x] Updated debug menu UI with projectile sliders:
  - New "Projectiles" section with count/size/speed/attack speed
  - Proper formatting: count shows +/- bonus, others show multiplier
- [x] Test: 158 tests passing, build succeeds

## Phase 21B: Penetration System ✅

- [x] Added projectile_penetration field to ProjectileConfig:
  - penetration: u32 (how many enemies the projectile can hit)
- [x] Updated Creature data struct in data/mod.rs:
  - Added projectile_penetration field with serde default of 1
- [x] Updated Projectile component with penetration tracking:
  - penetration_remaining: u32 (decrements on each hit)
  - enemies_hit: Vec<Entity> (tracks which enemies already hit, prevents double damage)
- [x] Updated creatures.toml with penetration values for all creatures:
  - Tier 1: penetration 1 (single hit)
  - Tier 2: penetration 2-3
  - Tier 3: penetration 3-5
  - Tier 4: penetration 8-10
- [x] Updated projectile spawning:
  - Initialize penetration_remaining from creature's projectile_penetration + debug bonus
  - Initialize enemies_hit as empty Vec
  - Use longer lifetime (3s) for penetrating projectiles (vs 1s for non-penetrating)
- [x] Rewrote projectile_system hit detection for penetration:
  - Check all enemies for collision (not just original target)
  - Skip enemies already in enemies_hit list
  - On hit: add enemy to list, deal damage, decrement penetration_remaining
  - If penetration_remaining == 0: despawn projectile
  - If penetration_remaining > 0: projectile continues flying
  - Only hit one enemy per frame to prevent multi-hit in same position
- [x] Visual feedback for penetration:
  - On hit without despawn: projectile flashes brighter (color * 1.5)
  - Projectile shrinks 10% per hit (visual wear)
  - Projectile slows 10% per hit
- [x] Added global_penetration_bonus to DebugSettings:
  - Range 0-20, step 1, default 0
  - Added to base penetration for all creature projectiles
- [x] Updated debug menu UI:
  - Added "Penetration Bonus" slider in Projectiles section
  - Shows +X format for display
- [x] Added projectile cleanup for performance:
  - Maximum lifetime of 3 seconds for penetrating projectiles
  - Despawn projectiles >1200 pixels from player
- [x] Test: 158 tests passing, build succeeds

## Phase 21C: Projectile Types ✅

- [x] Added ProjectileType enum to creature.rs:
  - Basic: Standard square projectile (default)
  - Piercing: Thin rectangle that rotates toward travel direction
  - Explosive: On final hit, deals AoE damage to nearby enemies
  - Homing: Curves toward nearest enemy
  - Chain: On hit, redirects toward nearby enemy (chain count = penetration)
- [x] Updated ProjectileConfig with projectile_type field
- [x] Updated Projectile component with projectile_type tracking
- [x] Updated Creature data struct in data/mod.rs:
  - Added projectile_type field with serde default of "basic"
- [x] Updated creatures.toml with projectile_type for all creatures:
  - fire_imp: basic
  - ember_hound/hellhound: piercing (fast assassins)
  - fire_spirit/greater_fire_spirit: homing (support)
  - magma_elemental/inferno_demon/phoenix/inferno_titan/eternal_phoenix: explosive
  - hellhound_alpha: chain (pack synergy)
  - flame_fiend/inferno_knight/inferno_warlord: basic
- [x] Implemented visual differentiation:
  - Basic: Standard square, base color
  - Piercing: Elongated rectangle (2x width, 0.5x height), rotates to face travel direction
  - Explosive: Slightly larger (1.2x), orange tinted
  - Homing: Smaller (0.8x), cyan tinted
  - Chain: Base size, electric blue tinted
- [x] Implemented Piercing behavior:
  - piercing_rotation_system: rotates projectile Transform to face velocity direction
- [x] Implemented Explosive behavior:
  - On final hit (penetration_remaining == 0), creates AoE explosion
  - AoE radius: 100 pixels
  - AoE damage: 50% of projectile damage with distance falloff
  - Spawns visual explosion effect (expanding orange circle)
  - AoE can grant kill credit to source creature
- [x] Implemented Homing behavior:
  - homing_projectile_system: adjusts velocity to curve toward nearest enemy
  - Turn rate: 3.0 radians/second
  - Blends current direction with desired direction over time
- [x] Implemented Chain behavior:
  - On hit (when penetration_remaining > 0), finds nearest unhit enemy within 150px
  - Redirects projectile velocity toward new target
  - Spawns visual chain lightning effect (blue line between positions)
- [x] Added visual effect systems:
  - ExplosionEffect component: expanding circle that fades out over 0.3s
  - ChainEffect component: blue line that fades out over 0.15s
  - explosion_effect_system and chain_effect_system update and despawn effects
- [x] Updated spawn_creature to pass projectile_type to ProjectileConfig::new()
- [x] Registered new systems in main.rs:
  - homing_projectile_system (before projectile_system)
  - piercing_rotation_system (after projectile_system)
  - explosion_effect_system
  - chain_effect_system
- [x] Test: 162 tests passing, build succeeds

## Phase 22: Bosses ⬅️ CURRENT

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

## Phase 23: Formation by Type

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

## Phase 24: Player Health + Game Over

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

## Phase 25: Polish + Balance

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

Phase 21C completed - Projectile Types with 5 behaviors (Basic, Piercing, Explosive, Homing, Chain), visual differentiation (shape/color tinting), explosion AoE damage, homing curves, chain redirects, and visual effects (expanding explosions, chain lightning lines)
