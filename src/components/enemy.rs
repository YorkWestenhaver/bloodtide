use bevy::prelude::*;

/// Marker component for enemy entities
#[derive(Component)]
pub struct Enemy;

/// Animation state for sprite-based enemies
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
    Dying,
}

/// Sprite animation controller
#[derive(Component)]
pub struct SpriteAnimation {
    /// Current animation state
    pub state: AnimationState,
    /// Timer for advancing animation frames
    pub frame_timer: Timer,
    /// Current frame index in the spritesheet
    pub current_frame: usize,
}

impl SpriteAnimation {
    /// Create a new animation in idle state (frame 0)
    pub fn new() -> Self {
        Self {
            state: AnimationState::Idle,
            frame_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
            current_frame: 0,
        }
    }

    /// Transition to walking animation (frames 1-2)
    pub fn start_walking(&mut self) {
        if self.state != AnimationState::Dying {
            self.state = AnimationState::Walking;
            self.current_frame = 1;
            self.frame_timer = Timer::from_seconds(0.15, TimerMode::Repeating);
        }
    }

    /// Transition to idle animation (frame 0)
    pub fn go_idle(&mut self) {
        if self.state != AnimationState::Dying {
            self.state = AnimationState::Idle;
            self.current_frame = 0;
        }
    }

    /// Transition to dying animation (frames 3-4-5)
    pub fn start_dying(&mut self) {
        self.state = AnimationState::Dying;
        self.current_frame = 3;
        self.frame_timer = Timer::from_seconds(0.12, TimerMode::Repeating);
    }
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self::new()
    }
}

/// Enemy class/tier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnemyClass {
    #[default]
    Fodder,
    Elite,
    Miniboss,
    Boss,
}

impl EnemyClass {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fodder" => EnemyClass::Fodder,
            "elite" => EnemyClass::Elite,
            "miniboss" => EnemyClass::Miniboss,
            "boss" => EnemyClass::Boss,
            _ => EnemyClass::Fodder,
        }
    }
}

/// Enemy behavior type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnemyType {
    #[default]
    Melee,
    Ranged,
    Fast,
    Tank,
    Healer,
    Commander,
}

impl EnemyType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "melee" => EnemyType::Melee,
            "ranged" => EnemyType::Ranged,
            "fast" => EnemyType::Fast,
            "tank" => EnemyType::Tank,
            "healer" => EnemyType::Healer,
            "commander" => EnemyType::Commander,
            _ => EnemyType::Melee,
        }
    }
}

/// Runtime data for an enemy entity
#[derive(Component, Clone, Debug)]
pub struct EnemyStats {
    pub id: String,
    pub name: String,
    pub enemy_class: EnemyClass,
    pub enemy_type: EnemyType,
    pub base_hp: f64,
    pub current_hp: f64,
    pub base_damage: f64,
    pub attack_speed: f64,
    pub movement_speed: f64,
    pub attack_range: f64,
}

impl EnemyStats {
    pub fn new(
        id: String,
        name: String,
        enemy_class: EnemyClass,
        enemy_type: EnemyType,
        base_hp: f64,
        base_damage: f64,
        attack_speed: f64,
        movement_speed: f64,
        attack_range: f64,
    ) -> Self {
        Self {
            id,
            name,
            enemy_class,
            enemy_type,
            base_hp,
            current_hp: base_hp,
            base_damage,
            attack_speed,
            movement_speed,
            attack_range,
        }
    }
}

/// Attack cooldown timer for enemies
#[derive(Component)]
pub struct EnemyAttackTimer {
    pub timer: Timer,
}

impl EnemyAttackTimer {
    pub fn new(attack_speed: f64) -> Self {
        let duration = if attack_speed > 0.0 {
            1.0 / attack_speed
        } else {
            1.0
        };
        Self {
            timer: Timer::from_seconds(duration as f32, TimerMode::Repeating),
        }
    }
}

// =============================================================================
// BOSS COMPONENTS
// =============================================================================

/// Marker component for the Goblin King boss
#[derive(Component)]
pub struct GoblinKing;

/// Tracks which phase the boss is currently in
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossPhase {
    /// Normal phase (100% - 30% HP)
    Phase1,
    /// Berserker mode (below 30% HP)
    Phase2,
}

impl Default for BossPhase {
    fn default() -> Self {
        BossPhase::Phase1
    }
}

/// Current attack state for boss wind-up animations
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BossAttackState {
    #[default]
    Idle,
    /// Winding up melee slam attack
    WindingUpSlam,
    /// Executing melee slam
    Slamming,
    /// Preparing charge attack (showing telegraph)
    ChargingUp,
    /// Executing charge dash
    Charging,
    /// Summoning goblin adds
    Summoning,
}

/// Component for boss charge attack
#[derive(Component)]
pub struct BossChargeAttack {
    /// Timer for telegraph phase
    pub telegraph_timer: Timer,
    /// Timer for charge execution
    pub charge_timer: Timer,
    /// Target position for charge
    pub target_pos: Vec2,
    /// Starting position of charge
    pub start_pos: Vec2,
    /// Whether we're in telegraph phase or execution phase
    pub is_telegraphing: bool,
    /// Charge damage
    pub damage: f64,
}

impl BossChargeAttack {
    pub fn new(start_pos: Vec2, target_pos: Vec2, damage: f64) -> Self {
        Self {
            telegraph_timer: Timer::from_seconds(1.0, TimerMode::Once),
            charge_timer: Timer::from_seconds(0.3, TimerMode::Once),
            target_pos,
            start_pos,
            is_telegraphing: true,
            damage,
        }
    }
}

/// Component for boss melee slam attack
#[derive(Component)]
pub struct BossSlamAttack {
    /// Timer for wind-up phase
    pub windup_timer: Timer,
    /// Whether we're in wind-up or execution
    pub is_winding_up: bool,
    /// Slam damage
    pub damage: f64,
    /// Slam range
    pub range: f64,
}

impl BossSlamAttack {
    pub fn new(damage: f64, range: f64) -> Self {
        Self {
            windup_timer: Timer::from_seconds(0.6, TimerMode::Once),
            is_winding_up: true,
            damage,
            range,
        }
    }
}

/// Timer for boss special abilities
#[derive(Component)]
pub struct BossAbilityTimers {
    /// Timer for charge attack cooldown
    pub charge_cooldown: Timer,
    /// Timer for summon ability cooldown
    pub summon_cooldown: Timer,
}

impl BossAbilityTimers {
    pub fn new() -> Self {
        Self {
            charge_cooldown: Timer::from_seconds(8.0, TimerMode::Repeating),
            summon_cooldown: Timer::from_seconds(12.0, TimerMode::Repeating),
        }
    }

    /// Called when entering Phase 2 (berserker mode)
    pub fn enter_berserker_mode(&mut self) {
        // Reduce charge cooldown in berserker mode
        self.charge_cooldown = Timer::from_seconds(5.0, TimerMode::Repeating);
        // No more summoning in berserker mode
    }
}

impl Default for BossAbilityTimers {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker for berserker mode (Phase 2)
#[derive(Component)]
pub struct BerserkerMode {
    /// Visual pulse timer for red glow effect
    pub pulse_timer: Timer,
}

impl Default for BerserkerMode {
    fn default() -> Self {
        Self {
            pulse_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

/// Visual telegraph for charge attack
#[derive(Component)]
pub struct ChargeTelegraph {
    /// Entity of the boss performing the charge
    pub boss_entity: Entity,
    /// Timer synced with boss telegraph timer
    pub timer: Timer,
}

/// Animation state for Goblin King boss
///
/// Frame layout (12 frames total at 128x192 each):
/// - Frame 0: idle
/// - Frames 1-2: walk cycle
/// - Frames 3-4: charge attack (windup, dash)
/// - Frames 5-6: sword swipe (windup, strike)
/// - Frames 7-8: ground pound (windup, impact)
/// - Frames 9-11: death animation
#[derive(Component)]
pub struct GoblinKingAnimation {
    /// Current frame index
    pub current_frame: usize,
    /// Animation timer for frame transitions
    pub frame_timer: Timer,
    /// Current animation state
    pub state: GoblinKingAnimState,
}

/// Animation states for the Goblin King
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GoblinKingAnimState {
    #[default]
    Idle,
    Walking,
    ChargeWindup,
    ChargeDash,
    SwipeWindup,
    SwipeStrike,
    PoundWindup,
    PoundImpact,
    Dying,
    Dead,
}

impl GoblinKingAnimation {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            frame_timer: Timer::from_seconds(0.18, TimerMode::Repeating),
            state: GoblinKingAnimState::Idle,
        }
    }

    /// Start walking animation
    pub fn start_walking(&mut self) {
        self.state = GoblinKingAnimState::Walking;
        self.current_frame = 1;
        self.frame_timer = Timer::from_seconds(0.18, TimerMode::Repeating);
    }

    /// Return to idle
    pub fn go_idle(&mut self) {
        self.state = GoblinKingAnimState::Idle;
        self.current_frame = 0;
    }

    /// Start charge windup animation
    pub fn start_charge_windup(&mut self) {
        self.state = GoblinKingAnimState::ChargeWindup;
        self.current_frame = 3;
        self.frame_timer = Timer::from_seconds(0.5, TimerMode::Once);
    }

    /// Start charge dash animation
    pub fn start_charge_dash(&mut self) {
        self.state = GoblinKingAnimState::ChargeDash;
        self.current_frame = 4;
        self.frame_timer = Timer::from_seconds(0.1, TimerMode::Repeating);
    }

    /// Start sword swipe windup
    pub fn start_swipe_windup(&mut self) {
        self.state = GoblinKingAnimState::SwipeWindup;
        self.current_frame = 5;
        self.frame_timer = Timer::from_seconds(0.3, TimerMode::Once);
    }

    /// Execute sword swipe
    pub fn start_swipe_strike(&mut self) {
        self.state = GoblinKingAnimState::SwipeStrike;
        self.current_frame = 6;
        self.frame_timer = Timer::from_seconds(0.08, TimerMode::Once);
    }

    /// Start ground pound windup
    pub fn start_pound_windup(&mut self) {
        self.state = GoblinKingAnimState::PoundWindup;
        self.current_frame = 7;
        self.frame_timer = Timer::from_seconds(0.6, TimerMode::Once);
    }

    /// Execute ground pound impact
    pub fn start_pound_impact(&mut self) {
        self.state = GoblinKingAnimState::PoundImpact;
        self.current_frame = 8;
        self.frame_timer = Timer::from_seconds(0.15, TimerMode::Once);
    }

    /// Start death animation
    pub fn start_dying(&mut self) {
        self.state = GoblinKingAnimState::Dying;
        self.current_frame = 9;
        self.frame_timer = Timer::from_seconds(0.2, TimerMode::Repeating);
    }

    /// Advance walk animation between frames 1-2
    pub fn advance_walk_frame(&mut self) {
        self.current_frame = if self.current_frame == 1 { 2 } else { 1 };
    }

    /// Advance death animation through frames 9-11
    pub fn advance_death_frame(&mut self) -> bool {
        if self.current_frame < 11 {
            self.current_frame += 1;
            false // Not finished
        } else {
            self.state = GoblinKingAnimState::Dead;
            true // Finished
        }
    }
}

impl Default for GoblinKingAnimation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // EnemyClass Tests
    // =========================================================================

    #[test]
    fn enemy_class_from_str_parses_all_classes() {
        assert_eq!(EnemyClass::from_str("fodder"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("elite"), EnemyClass::Elite);
        assert_eq!(EnemyClass::from_str("miniboss"), EnemyClass::Miniboss);
        assert_eq!(EnemyClass::from_str("boss"), EnemyClass::Boss);
    }

    #[test]
    fn enemy_class_from_str_is_case_insensitive() {
        assert_eq!(EnemyClass::from_str("FODDER"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("Fodder"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("ELITE"), EnemyClass::Elite);
        assert_eq!(EnemyClass::from_str("Elite"), EnemyClass::Elite);
        assert_eq!(EnemyClass::from_str("BOSS"), EnemyClass::Boss);
    }

    #[test]
    fn enemy_class_from_str_defaults_to_fodder_for_unknown() {
        assert_eq!(EnemyClass::from_str("unknown"), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str(""), EnemyClass::Fodder);
        assert_eq!(EnemyClass::from_str("legendary"), EnemyClass::Fodder);
    }

    #[test]
    fn enemy_class_default_is_fodder() {
        assert_eq!(EnemyClass::default(), EnemyClass::Fodder);
    }

    // =========================================================================
    // EnemyType Tests
    // =========================================================================

    #[test]
    fn enemy_type_from_str_parses_all_types() {
        assert_eq!(EnemyType::from_str("melee"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("ranged"), EnemyType::Ranged);
        assert_eq!(EnemyType::from_str("fast"), EnemyType::Fast);
        assert_eq!(EnemyType::from_str("tank"), EnemyType::Tank);
        assert_eq!(EnemyType::from_str("healer"), EnemyType::Healer);
        assert_eq!(EnemyType::from_str("commander"), EnemyType::Commander);
    }

    #[test]
    fn enemy_type_from_str_is_case_insensitive() {
        assert_eq!(EnemyType::from_str("MELEE"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("Melee"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("RANGED"), EnemyType::Ranged);
        assert_eq!(EnemyType::from_str("TANK"), EnemyType::Tank);
    }

    #[test]
    fn enemy_type_from_str_defaults_to_melee_for_unknown() {
        assert_eq!(EnemyType::from_str("unknown"), EnemyType::Melee);
        assert_eq!(EnemyType::from_str(""), EnemyType::Melee);
        assert_eq!(EnemyType::from_str("assassin"), EnemyType::Melee);
    }

    #[test]
    fn enemy_type_default_is_melee() {
        assert_eq!(EnemyType::default(), EnemyType::Melee);
    }

    // =========================================================================
    // EnemyStats Tests
    // =========================================================================

    #[test]
    fn enemy_stats_new_sets_current_hp_equal_to_base_hp() {
        let stats = EnemyStats::new(
            "goblin".to_string(),
            "Goblin".to_string(),
            EnemyClass::Fodder,
            EnemyType::Melee,
            30.0,  // base_hp
            5.0,   // base_damage
            1.0,   // attack_speed
            80.0,  // movement_speed
            40.0,  // attack_range
        );
        assert_eq!(stats.current_hp, 30.0);
        assert_eq!(stats.current_hp, stats.base_hp);
    }

    #[test]
    fn enemy_stats_new_preserves_all_input_values() {
        let stats = EnemyStats::new(
            "orc_warrior".to_string(),
            "Orc Warrior".to_string(),
            EnemyClass::Elite,
            EnemyType::Tank,
            200.0, // base_hp
            15.0,  // base_damage
            0.8,   // attack_speed
            60.0,  // movement_speed
            50.0,  // attack_range
        );
        assert_eq!(stats.id, "orc_warrior");
        assert_eq!(stats.name, "Orc Warrior");
        assert_eq!(stats.enemy_class, EnemyClass::Elite);
        assert_eq!(stats.enemy_type, EnemyType::Tank);
        assert_eq!(stats.base_hp, 200.0);
        assert_eq!(stats.base_damage, 15.0);
        assert_eq!(stats.attack_speed, 0.8);
        assert_eq!(stats.movement_speed, 60.0);
        assert_eq!(stats.attack_range, 50.0);
    }

    // =========================================================================
    // EnemyAttackTimer Tests
    // =========================================================================

    #[test]
    fn enemy_attack_timer_calculates_duration_from_attack_speed() {
        // attack_speed = 1.0 means 1 attack per second, so duration = 1.0
        let timer = EnemyAttackTimer::new(1.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);

        // attack_speed = 2.0 means 2 attacks per second, so duration = 0.5
        let timer = EnemyAttackTimer::new(2.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 0.5);

        // attack_speed = 0.5 means 0.5 attacks per second, so duration = 2.0
        let timer = EnemyAttackTimer::new(0.5);
        assert_eq!(timer.timer.duration().as_secs_f32(), 2.0);
    }

    #[test]
    fn enemy_attack_timer_defaults_to_1_second_for_zero_attack_speed() {
        let timer = EnemyAttackTimer::new(0.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);
    }

    #[test]
    fn enemy_attack_timer_defaults_to_1_second_for_negative_attack_speed() {
        let timer = EnemyAttackTimer::new(-1.0);
        assert_eq!(timer.timer.duration().as_secs_f32(), 1.0);
    }

    #[test]
    fn enemy_attack_timer_is_repeating() {
        let timer = EnemyAttackTimer::new(1.0);
        assert_eq!(timer.timer.mode(), TimerMode::Repeating);
    }
}
