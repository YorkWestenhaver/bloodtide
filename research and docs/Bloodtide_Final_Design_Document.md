# Bloodtide: Final Design Document
## Version 1.0 — December 2024

---

# Part 1: Core Identity

## Game Concept
A roguelite wave-survival game that fuses Vampire Survivors' accessibility with Magic: The Gathering's deck-building depth and Balatro Cryptid's infinite scaling potential. Built in Rust (Bevy) for maximum performance.

## The Pitch
"Vampire Survivors meets Magic: The Gathering with infinite scaling crits."

## Primary Game Mode
**Siege Castle** — Zone-based progression through a castle with objectives per zone, culminating in a final boss.

---

# Part 2: Card System

## Deck Building (Pre-Run)

Your deck is a **probability table**, not a draw pile.

| Card in Deck | Weight |
|--------------|--------|
| Fire Imp (x5) | 25% |
| Ember Hound (x3) | 15% |
| Inferno Knight (x2) | 10% |
| Ember Staff (x3) | 15% |
| Molten Core (x4) | 20% |
| etc. | etc. |

- More copies of a card = higher probability to roll it
- No "running out" of cards — infinite rolls against your probability weights
- Deck composition is your strategy layer

## Card Types

### Weapons (= Lands)
| Attribute | Value |
|-----------|-------|
| Slot Limit | None |
| Can Die | No |
| Combines | Yes (Vampire Survivors style) |
| Purpose | Generate Color Affinity + Weak auto-attack + Spell effects built-in |

**Examples:**
- Ember Staff: +10 Red Affinity, shoots weak fireballs
- Flame Sword: +15 Red Affinity, melee sweep
- Inferno Blade (evolved): +35 Red Affinity, faster attacks, burn DoT

### Creatures (= Your Army)
| Attribute | Value |
|-----------|-------|
| Slot Limit | Soft cap per type (respawn mechanic) |
| Can Die | Yes (respawns on timer) |
| Combines | Yes (3x same → evolution) |
| Purpose | Main damage dealers |

**Examples:**
- Fire Imp → Flame Fiend → Inferno Demon
- Ember Hound → Hellhound Alpha → Cerberus
- Inferno Knight → Inferno Warlord → Inferno Titan

### Artifacts (= Consumables)
| Attribute | Value |
|-----------|-------|
| Slot Limit | N/A (consumed on acquire) |
| Can Die | N/A |
| Combines | No |
| Purpose | Permanent stat buffs for the run |

**Types:**
- Global: "+10% crit to all creatures"
- Color-Specific: "+30% damage to Fire creatures"
- Type-Specific: "+50% attack speed to Melee creatures"
- Creature-Specific: "Trolls gain +100% HP"

More specific = stronger buff, narrower use case.

### Prophecies (= Emergency Triggers)
| Attribute | Value |
|-----------|-------|
| Slot Limit | 1 active |
| Purpose | Defensive emergency ability |

**How They Work:**
- Unlock via conditions during run (not pre-selected)
- Triggers when conditions met (low HP, creatures dead, etc.)
- Examples: Shield burst, AoE knockback, mass heal, brief invincibility

---

# Part 3: Color Affinity System

## What Is Affinity?

Color Affinity = (Number of Color Weapons Equipped) × (Base Affinity per Weapon)

## Affinity Thresholds (Red Example)

| Red Affinity | Bonus |
|--------------|-------|
| 0-10 | Base fire damage (no bonus) |
| 11-25 | +10% fire creature damage |
| 26-50 | +25% fire creature damage, +10% attack speed |
| 51-75 | +50% fire creature damage, unlock Tier 2 Mega Crit |
| 76-100 | +100% fire creature damage, fire attacks gain burn DoT |
| 100+ | +1% damage per point above 100 (overflow) |

## Color Identities

| Color | Creature Style | Signature Mechanic |
|-------|---------------|-------------------|
| Red | High damage, aggressive | Burn DoT, attack speed |
| Blue | Control, mages | Slow, crowd control |
| Green | Tanks, nature | High HP, regeneration |
| White | Healers, support | Shields, healing auras |
| Black | Necromancy, death | Summons, life steal |
| Colorless (Eldrazi) | Game-breaking | Extreme cost, extreme power |

---

# Part 4: Creature Mechanics

## Dual Scaling Paths

### Path A: Kill-Based XP (Per-Creature)
Each creature gains XP from its own kills.

```
Troll Level Progression:
Level 1 → 2: 10 kills
Level 2 → 3: 25 kills
Level 3 → 4: 50 kills
...
Max Level: 10
```

Each level grants: +10% damage, +10% HP, slight attack speed increase.

### Path B: Card Evolution (Duplicates)
Drawing duplicate creatures triggers evolution.

```
3x Fire Imp → Flame Fiend (resets to L1, higher base stats)
3x Flame Fiend → Inferno Demon (resets to L1, even higher stats)
```

### Interaction
- Level 7 Troll + 2 more Troll cards = choice
- Keep L7 Troll + add two L1 Trolls? OR
- Combine into L1 Troll Chieftain (200% base stats)?
- Sometimes evolving is worse short-term but better long-term

## Creature Control

### Formation System
```
[Back Line] — Ranged, Supports
[Mid Line] — You (player)
[Front Line] — Melee, Tanks
[Free Roam] — Assassins
```

### Creature Types & Behavior

| Type | Position | Behavior |
|------|----------|----------|
| Melee | Front | Body-blocks enemies, charges nearest |
| Ranged | Back | Stays near you, auto-targets |
| Support | Mid | Orbits you, AoE heals/buffs |
| Assassin | Free | Ignores formation, hunts priority targets |

### Control Modes

**Auto Mode (Default):**
- Creatures follow type behavior automatically
- Good for casual play

**Command Mode (Right-Click):**
- Right-click ground → selected creatures move there
- Right-click enemy → focus fire that target

**Formation Lock (Toggle):**
- Creatures maintain relative position to you
- Good for kiting and defensive play

## Death & Respawn (Nafiri Mechanic)

- Creatures can die (killed by enemies)
- Dead creature → respawn timer starts
- Timer scales with creature tier (rarer = longer respawn)
- Respawns at Level 1
- Keeping creatures alive = better DPS (they keep their kill XP levels)

---

# Part 5: Leveling & Progression

## Kill-Based Leveling

**25 kills = 1 level = 1 card roll from your probability deck**

Visual: "Murder Meter" — circle fills with blood, pulses on level up.

## Card Roll Process

1. Kill threshold reached
2. System rolls against your probability table
3. Card appears in inventory
4. Auto-equip rules apply (or manual equip)
5. If duplicate creature → evolution prompt

## Auto-Equip Rules (Customizable)

- "Always equip Red creatures"
- "Prioritize highest rarity"
- "Never equip Green artifacts"
- "Auto-combine duplicates"

## Pity System

| Condition | Guarantee |
|-----------|-----------|
| Level 5 (if no creatures yet) | First creature card |
| Level 15 | Rare+ card |
| Level 30 | Epic+ card |
| Level 50 | Legendary card |
| 10 rolls without creature | Next roll = 100% creature |
| 10 rolls without artifact | Next roll = 100% artifact |

## Trash Mechanic

- Trash an unwanted card → +5% quality bonus on next roll
- Stacks up to +50%
- Lets you "sculpt" your probability mid-run

---

# Part 6: The 3-Tier Crit System

## Independent Probability Tiers

Each tier has its own separate roll chance.

| Tier | Chance (Example) | Multiplier |
|------|------------------|------------|
| Tier 1: Normal Crit | 35% | 2× damage |
| Tier 2: Mega Crit | 18% | damage² (squared) |
| Tier 3: Super Crit | 5% | damage^4 (capped power tower) |

## Crit Overflow

When crit chance exceeds 100%, overflow converts to bonus crit damage.

```
Example:
- 120% Tier 1 Crit Chance
- Result: Always crits + 20% bonus to crit multiplier
- Base T1 = 2× → With overflow = 2.2×
```

## Scaling Example

Inferno Warlord at Level 60:
- Base damage: 643
- Affinity bonus: +50%
- Artifact bonus: +20%
- Kill XP bonus: +70%
- Final base: ~1,200 damage

| Roll | Chance | Damage |
|------|--------|--------|
| Normal | 42% | 1,200 |
| T1 Crit | 35% | 2,400 |
| T2 Mega | 18% | 1,440,000 |
| T3 Super | 5% | 2+ trillion |

Expected DPS from Warlord alone at L60: **~9.5 million**

---

# Part 7: Enemy Scaling

## Philosophy
Bad: Enemies just get more HP (sponges).
Good: Enemies get new behaviors + HP scales to match your damage curve.

## Track 1: Stat Scaling

**Target Time-to-Kill (TTK):**
- Fodder: 0.5-1 second
- Elite: 3-5 seconds
- Mini-boss: 15-30 seconds
- Boss: 60-120 seconds

**Formula:**
```
Enemy HP = (Your Expected DPS) × (Target TTK) × (Difficulty Modifier)
```

**Scaling Table:**

| Level | Your DPS | Fodder HP | Elite HP | Boss HP |
|-------|----------|-----------|----------|---------|
| 10 | 90 | 68 | 360 | 8,100 |
| 30 | 850 | 637 | 3,400 | 76,500 |
| 50 | 45,000 | 33,750 | 180,000 | 4,050,000 |
| 70 | 850M | 637M | 3.4B | 76.5B |

**Enemy Damage:**
```
Enemy Damage = Base × (1.15)^(Wave Number)
```
Scales slower than HP — you don't get one-shot, but pressure increases.

## Track 2: Behavior Scaling

New enemy types every 5 waves:

| Waves | New Behavior |
|-------|--------------|
| 1-5 | Basic fodder (walk + melee) |
| 6-10 | Ranged enemies (archers, mages) |
| 11-15 | Fast enemies (wolves, flankers) |
| 16-20 | Shield enemies (block frontal) |
| 21-25 | Healers (restore other enemies) |
| 26-30 | Commanders (buff nearby enemies) |
| 31-35 | Splitters (spawn 2 on death) |
| 36-40 | Phasing (ignore frontline sometimes) |
| 41-45 | Reflectors (return % damage) |
| 46-50 | Elemental Resist (counter your color) |
| 51+ | Combinations of all above |

## Director System (Adaptive AI)

**Metrics Tracked:**
- Your DPS, HP%, creature count, time since damage taken

**Responses:**

| Your State | Director Action |
|------------|-----------------|
| Stomping | Spawn more elites, add counter-color enemies |
| Comfortable | Standard wave composition |
| Struggling | Reduce spawn rate, add health drops |
| Nearly Dead | Brief pause, spawn weaker enemies |

**Counter-Color Spawning:**
- If you're mono-Red, Director spawns Water enemies at Wave 20+
- Water enemies take 50% less fire damage
- Prevents "solved" autopilot builds

---

# Part 8: Boss Design

## Structure

Every 10 waves = Boss encounter.

**Phase System:**
- Phase 1 (100-70% HP): Basic attack pattern
- Phase 2 (70-40% HP): New attack + summons adds
- Phase 3 (40-0% HP): Enrage mode, faster, AoE danger zones

## Example: Frost Giant (Wave 10 Boss)

```
HP: 25,000 (scales with your DPS)
Damage: 150 per hit

Attacks:
- Slam: 1.5s telegraph, AoE front, kills creatures in zone
- Ice Breath: Cone, slows creatures 50%
- Summon: 4 Ice Wolves every 20 seconds

Phase 2 (70% HP):
- Ground Freeze: Random ice patches (damage + slow)

Phase 3 (40% HP):
- Enrage: 30% faster attacks
- Blizzard: Screen-wide slow, dodge ice shards
```

## Boss Rewards
- Guaranteed Rare+ card roll
- Bonus artifact
- Large XP/kill credit burst

---

# Part 9: Map Structure (Siege Castle)

## Zones

```
[Spawn Area] → [Outer Walls] → [Inner Courtyard] → [Castle Gate] → [Throne Room]
```

## Zone Details

| Zone | Difficulty | Special Mechanic |
|------|------------|------------------|
| Spawn Area | 1.0× | Tutorial fodder |
| Outer Walls | 1.5× | Archers on walls (ranged priority) |
| Inner Courtyard | 2.0× | Knight patrols, clear all |
| Castle Gate | 2.5× | Destroy gate while defending |
| Throne Room | 3.0× | Final boss + elite guards |

## Optional Areas
- **Dungeons/Prisons**: Side areas with rare loot, increased difficulty
- **Shrines**: Buffs for completing challenges
- Similar to Diablo dungeons — not required, but rewarding

## Time Pressure (Optional Toggle)
- Reinforcement waves from behind every 2 minutes
- Encourages forward momentum
- Not a death timer, just increasing pressure
- Can be disabled for casual play

---

# Part 10: Difficulty Settings

## Smart Difficulty (Not Just +HP)

| Setting | Effect |
|---------|--------|
| Easy | Slower spawns, weaker Director, no counter-color |
| Normal | Standard balancing |
| Hard | Smarter enemies, aggressive Director, more counters |
| Nightmare | Director actively counters your build, elites everywhere |

Difficulty affects enemy **behavior and intelligence**, not just stats.

---

# Part 11: Technical Notes

## Engine
- **Language**: Rust
- **Framework**: Bevy (ECS architecture)
- **Target**: 100,000+ entities at 60 FPS

## Math Handling
- Use `break_infinity` library for numbers beyond f64
- T3 Super Crits capped at damage^4 to prevent overflow
- Scientific notation display (1.00E+12)

## Performance Priorities
- GPU instancing for rendering
- Spatial partitioning for collision
- Sparse set storage for status effects
- Parallel systems for AI

---

# Part 12: What's Next (Implementation Checklist)

## Phase 1: Data Definition
- [ ] **Creature Roster**: 5-10 creatures per color with full stats
- [ ] **Weapon Roster**: Base weapons + combo evolution trees
- [ ] **Artifact Roster**: ~20 artifacts with buff values
- [ ] **Enemy Roster**: Fodder, elite, boss per zone
- [ ] **Affinity Thresholds**: Exact numbers for all 5 colors

## Phase 2: Core Systems
- [ ] Probability deck system
- [ ] Kill-based leveling
- [ ] Card roll + auto-equip
- [ ] Creature spawning + positioning
- [ ] Basic combat (auto-attack)
- [ ] Affinity calculation

## Phase 3: Creature Systems
- [ ] Kill XP tracking
- [ ] Evolution (3x duplicate)
- [ ] Death + respawn timers
- [ ] Formation/positioning
- [ ] Command mode (right-click)

## Phase 4: Scaling Systems
- [ ] 3-tier crit implementation
- [ ] Crit overflow math
- [ ] Enemy HP scaling formula
- [ ] Director AI (adaptive spawning)
- [ ] Behavior unlocks per wave

## Phase 5: Content
- [ ] Siege Castle map layout
- [ ] Zone objectives
- [ ] Boss encounters (1 per 10 waves)
- [ ] Optional dungeon areas

## Phase 6: Polish
- [ ] UI/UX (inventory, formation editor, probability display)
- [ ] Visual effects (crit tiers, evolution, etc.)
- [ ] Audio
- [ ] Balance tuning

## Prototype Scope (First Build)
**One color (Red) + One map (Siege Castle) + Core loop only**

---

# Appendix: Quick Reference Tables

## Card Type Summary

| Type | Limit | Dies | Evolves | Purpose |
|------|-------|------|---------|---------|
| Weapons | None | No | Yes | Affinity + weak damage |
| Creatures | Soft | Yes | Yes | Main damage |
| Artifacts | N/A | N/A | No | Permanent buffs |
| Prophecies | 1 | N/A | No | Emergency trigger |

## Scaling Summary

| System | How It Scales |
|--------|---------------|
| Creature Kill XP | +10% stats per level, max L10 |
| Card Evolution | 3× same → next tier |
| Affinity | Weapon count → threshold bonuses |
| Artifacts | Stack permanently all run |
| Crit Overflow | 100%+ → bonus crit damage |

## Crit Tiers

| Tier | Multiplier | When It Matters |
|------|------------|-----------------|
| T1 | 2× | Early-mid game |
| T2 | damage² | Mid-late game (the hockey stick) |
| T3 | damage^4 | Extreme late (god killer) |

---

*Document Version 1.0 — Ready for implementation phase*
