use bevy::prelude::*;

use crate::components::{
    Creature, CreatureStats, Enemy, EnemyStats, FlockingState, HerdRole, Player, Velocity,
    // Boss components
    GoblinKing, BossPhase, BossAttackState, BossAbilityTimers, BerserkerMode,
    BossChargeAttack, BossSlamAttack, ChargeTelegraph,
};
use crate::resources::{CreatureSpatialGrid, DebugSettings, GameData};

// === LEGACY CONSTANTS (kept for reference) ===
/// Distance creatures try to maintain from player
pub const CREATURE_FOLLOW_DISTANCE: f32 = 100.0;

/// Distance threshold - stop moving when within this range of target
pub const CREATURE_STOP_DISTANCE: f32 = 10.0;

/// Distance at which creatures move at boosted speed to catch up
pub const CREATURE_CATCHUP_DISTANCE: f32 = 200.0;

/// Speed multiplier when catching up
pub const CREATURE_CATCHUP_MULTIPLIER: f32 = 2.5;

/// Base speed multiplier for formation movement (creatures move faster than their base speed)
pub const CREATURE_FORMATION_SPEED_MULTIPLIER: f32 = 1.8;

// === HERD BEHAVIOR CONSTANTS ===

/// Preferred distance behind player for backline creatures
pub const BACKLINE_DISTANCE: f32 = 120.0;

/// Preferred distance in front of player for frontline creatures
pub const FRONTLINE_DISTANCE: f32 = 80.0;

/// Preferred distance to the side for flanker creatures
pub const FLANKER_DISTANCE: f32 = 100.0;

/// Angle spread for backline (radians, +/- from directly behind)
pub const BACKLINE_SPREAD: f32 = 0.8; // ~45 degrees

/// Angle spread for frontline
pub const FRONTLINE_SPREAD: f32 = 0.6; // ~35 degrees

// === FLOCKING BEHAVIOR ===

/// Separation: distance at which creatures start pushing apart
pub const SEPARATION_DISTANCE: f32 = 35.0;

/// Separation force strength
pub const SEPARATION_STRENGTH: f32 = 150.0;

/// Cohesion: distance to consider for group center
pub const COHESION_DISTANCE: f32 = 150.0;

/// Cohesion force strength (pull toward group center)
pub const COHESION_STRENGTH: f32 = 30.0;

/// Alignment: how strongly creatures match neighbors' velocities
pub const ALIGNMENT_STRENGTH: f32 = 0.3;

// === SPRING PHYSICS ===

/// Spring stiffness (higher = snappier movement)
pub const SPRING_STIFFNESS: f32 = 8.0;

/// Spring damping (higher = less oscillation)
pub const SPRING_DAMPING: f32 = 4.0;

/// Maximum spring velocity
pub const MAX_SPRING_VELOCITY: f32 = 400.0;

// === PLAYER DIRECTION SMOOTHING ===

/// How quickly to smooth player direction changes (0-1, higher = faster)
pub const DIRECTION_SMOOTHING: f32 = 3.0;

/// Minimum player velocity to update facing direction
pub const MIN_VELOCITY_FOR_DIRECTION: f32 = 10.0;

/// System that makes creatures follow the player
pub fn creature_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<Creature>)>,
    debug_settings: Res<DebugSettings>,
    mut creature_query: Query<(&Transform, &mut Velocity, &CreatureStats), With<Creature>>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        for (_, mut velocity, _) in creature_query.iter_mut() {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let creature_count = creature_query.iter().count();

    for (index, (creature_transform, mut velocity, stats)) in
        creature_query.iter_mut().enumerate()
    {
        let creature_pos = creature_transform.translation.truncate();

        // Calculate target position in a circle around player
        // Each creature gets a different angle based on their index
        let angle = if creature_count > 0 {
            (index as f32 / creature_count as f32) * std::f32::consts::TAU
        } else {
            0.0
        };

        let target_pos = player_pos
            + Vec2::new(
                angle.cos() * CREATURE_FOLLOW_DISTANCE,
                angle.sin() * CREATURE_FOLLOW_DISTANCE,
            );

        // Calculate direction and distance to target
        let to_target = target_pos - creature_pos;
        let distance = to_target.length();

        // Only move if we're far enough from target position
        if distance > CREATURE_STOP_DISTANCE {
            let direction = to_target.normalize();
            // Use movement speed from creature stats with formation multiplier and debug multiplier
            let base_speed = stats.movement_speed as f32 * CREATURE_FORMATION_SPEED_MULTIPLIER
                * debug_settings.creature_speed_multiplier;

            // Apply catch-up boost if far from target
            let speed = if distance > CREATURE_CATCHUP_DISTANCE {
                base_speed * CREATURE_CATCHUP_MULTIPLIER
            } else {
                // Smooth interpolation: faster when further, slower when closer
                let t = (distance / CREATURE_CATCHUP_DISTANCE).min(1.0);
                base_speed * (1.0 + t * (CREATURE_CATCHUP_MULTIPLIER - 1.0))
            };

            velocity.x = direction.x * speed;
            velocity.y = direction.y * speed;
        } else {
            // Stop when close to target
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}

/// System that makes enemies chase the player (excludes bosses - they have their own AI)
pub fn enemy_chase_system(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    debug_settings: Res<DebugSettings>,
    mut enemy_query: Query<(&Transform, &mut Velocity, &EnemyStats), (With<Enemy>, Without<GoblinKing>)>,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        for (_, mut velocity, _) in enemy_query.iter_mut() {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (enemy_transform, mut velocity, stats) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.truncate();

        // Calculate direction to player
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        // Move toward player if not already on top of them
        if distance > 5.0 {
            let direction = to_player.normalize();
            // Use movement speed from enemy stats with debug multiplier
            let speed = stats.movement_speed as f32 * debug_settings.enemy_speed_multiplier;
            velocity.x = direction.x * speed;
            velocity.y = direction.y * speed;
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}

/// System to update the creature spatial grid for flocking behavior
pub fn update_creature_spatial_grid_system(
    mut spatial_grid: ResMut<CreatureSpatialGrid>,
    creature_query: Query<(Entity, &Transform), With<Creature>>,
) {
    spatial_grid.clear();

    for (entity, transform) in creature_query.iter() {
        let pos = transform.translation.truncate();
        spatial_grid.insert(entity, pos);
    }
}

/// Rotate a Vec2 by angle (radians)
fn rotate_vec2(v: Vec2, angle: f32) -> Vec2 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a)
}

/// Calculate the target position for a creature based on its role
fn calculate_role_target(
    player_pos: Vec2,
    leader_dir: Vec2,
    role: HerdRole,
    index: usize,
    count: usize,
    base_distance: f32,
    spread: f32,
) -> Vec2 {
    match role {
        HerdRole::Backline => {
            // Position behind player
            let backward = -leader_dir;
            let angle_offset = if count > 1 {
                let t = index as f32 / (count - 1).max(1) as f32;
                spread * (t - 0.5) * 2.0 // -spread to +spread
            } else {
                0.0
            };

            let rotated = rotate_vec2(backward, angle_offset);
            player_pos + rotated * base_distance
        }
        HerdRole::Frontline => {
            // Position in front of player
            let forward = leader_dir;
            let angle_offset = if count > 1 {
                let t = index as f32 / (count - 1).max(1) as f32;
                spread * (t - 0.5) * 2.0
            } else {
                0.0
            };

            let rotated = rotate_vec2(forward, angle_offset);
            player_pos + rotated * base_distance
        }
        HerdRole::Flanker => {
            // Position to the sides
            let perpendicular = Vec2::new(-leader_dir.y, leader_dir.x);
            let side = if index % 2 == 0 { 1.0 } else { -1.0 };
            let offset = (index / 2) as f32 * 0.3; // Stagger back slightly

            player_pos + perpendicular * side * base_distance - leader_dir * offset * base_distance
        }
    }
}

/// System that makes creatures follow the player in a herd-like formation
pub fn creature_herd_system(
    time: Res<Time>,
    player_query: Query<(&Transform, &Velocity), (With<Player>, Without<Creature>)>,
    debug_settings: Res<DebugSettings>,
    mut creature_query: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            &CreatureStats,
            &mut FlockingState,
        ),
        With<Creature>,
    >,
) {
    // Don't process if game is paused
    if debug_settings.is_paused() {
        for (_, _, mut velocity, _, _) in creature_query.iter_mut() {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
        return;
    }

    let Ok((player_transform, player_velocity)) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let player_vel = Vec2::new(player_velocity.x, player_velocity.y);
    let dt = time.delta_secs();
    let player_moving = player_vel.length() > MIN_VELOCITY_FOR_DIRECTION;

    // Collect all creature data for neighbor calculations
    let creature_data: Vec<(Entity, Vec2, Vec2, HerdRole)> = creature_query
        .iter()
        .map(|(entity, transform, velocity, stats, _)| {
            let pos = transform.translation.truncate();
            let vel = Vec2::new(velocity.x, velocity.y);
            let role = HerdRole::from_creature_type(stats.creature_type);
            (entity, pos, vel, role)
        })
        .collect();

    // Count creatures by role for spread calculation
    let backline_count = creature_data
        .iter()
        .filter(|(_, _, _, r)| *r == HerdRole::Backline)
        .count();
    let frontline_count = creature_data
        .iter()
        .filter(|(_, _, _, r)| *r == HerdRole::Frontline)
        .count();
    let _flanker_count = creature_data
        .iter()
        .filter(|(_, _, _, r)| *r == HerdRole::Flanker)
        .count();

    // Track indices for angle assignment
    let mut backline_index = 0;
    let mut frontline_index = 0;
    let mut flanker_index = 0;

    for (entity, creature_transform, mut velocity, stats, mut flocking) in creature_query.iter_mut()
    {
        let creature_pos = creature_transform.translation.truncate();
        let role = HerdRole::from_creature_type(stats.creature_type);

        // === 1. Update smoothed player direction ===
        if player_moving {
            let target_direction = player_vel.normalize();
            flocking.smoothed_leader_direction = flocking
                .smoothed_leader_direction
                .lerp(target_direction, DIRECTION_SMOOTHING * dt)
                .normalize_or_zero();
        }
        // If player is stationary, keep last known direction

        let leader_dir = if flocking.smoothed_leader_direction.length() > 0.1 {
            flocking.smoothed_leader_direction
        } else {
            Vec2::new(1.0, 0.0) // Default to facing right
        };

        // === 2. Calculate target position based on role ===
        let (role_index, role_count, base_distance, spread) = match role {
            HerdRole::Backline => {
                let idx = backline_index;
                backline_index += 1;
                (idx, backline_count, BACKLINE_DISTANCE, BACKLINE_SPREAD)
            }
            HerdRole::Frontline => {
                let idx = frontline_index;
                frontline_index += 1;
                (idx, frontline_count, FRONTLINE_DISTANCE, FRONTLINE_SPREAD)
            }
            HerdRole::Flanker => {
                let idx = flanker_index;
                flanker_index += 1;
                (
                    idx,
                    _flanker_count,
                    FLANKER_DISTANCE,
                    std::f32::consts::FRAC_PI_2,
                )
            }
        };

        let target_pos = calculate_role_target(
            player_pos,
            leader_dir,
            role,
            role_index,
            role_count,
            base_distance,
            spread,
        );

        // === 3. Calculate flocking forces ===
        let mut separation_force = Vec2::ZERO;
        let mut cohesion_center = Vec2::ZERO;
        let mut alignment_velocity = Vec2::ZERO;
        let mut neighbor_count = 0;

        for (other_entity, other_pos, other_vel, _) in &creature_data {
            if *other_entity == entity {
                continue;
            }

            let distance = creature_pos.distance(*other_pos);

            // Separation: push away from close neighbors
            if distance < SEPARATION_DISTANCE && distance > 0.0 {
                let push_dir = (creature_pos - *other_pos).normalize();
                let force_magnitude = SEPARATION_STRENGTH * (1.0 - distance / SEPARATION_DISTANCE);
                separation_force += push_dir * force_magnitude;
            }

            // Cohesion and alignment: consider neighbors within range
            if distance < COHESION_DISTANCE {
                cohesion_center += *other_pos;
                alignment_velocity += *other_vel;
                neighbor_count += 1;
            }
        }

        // Finalize cohesion (pull toward group center)
        let cohesion_force = if neighbor_count > 0 {
            cohesion_center /= neighbor_count as f32;
            (cohesion_center - creature_pos).normalize_or_zero() * COHESION_STRENGTH
        } else {
            Vec2::ZERO
        };

        // Finalize alignment (match neighbor velocities)
        let alignment_force = if neighbor_count > 0 {
            alignment_velocity /= neighbor_count as f32;
            alignment_velocity * ALIGNMENT_STRENGTH
        } else {
            Vec2::ZERO
        };

        // === 4. Calculate target force (spring to target position) ===
        let to_target = target_pos - creature_pos;
        let distance_to_target = to_target.length();

        // Base target force using spring physics
        let spring_force = to_target * SPRING_STIFFNESS;
        let damping_force = -flocking.spring_velocity * SPRING_DAMPING;

        // === 5. Combine all forces ===
        let total_force =
            spring_force + damping_force + separation_force + cohesion_force + alignment_force;

        // Update spring velocity
        flocking.spring_velocity += total_force * dt;
        flocking.spring_velocity = flocking.spring_velocity.clamp_length_max(MAX_SPRING_VELOCITY);

        // === 6. Apply catch-up boost if far from target ===
        let speed_multiplier = if distance_to_target > CREATURE_CATCHUP_DISTANCE {
            CREATURE_CATCHUP_MULTIPLIER
        } else {
            1.0 + (distance_to_target / CREATURE_CATCHUP_DISTANCE)
                * (CREATURE_CATCHUP_MULTIPLIER - 1.0)
        };

        // === 7. Apply to velocity ===
        let base_speed = stats.movement_speed as f32
            * CREATURE_FORMATION_SPEED_MULTIPLIER
            * debug_settings.creature_speed_multiplier
            * speed_multiplier;

        let desired_velocity = flocking.spring_velocity.clamp_length_max(base_speed);

        // Stop if very close to target and player is stationary
        if distance_to_target < CREATURE_STOP_DISTANCE && !player_moving {
            velocity.x = 0.0;
            velocity.y = 0.0;
        } else {
            velocity.x = desired_velocity.x;
            velocity.y = desired_velocity.y;
        }
    }
}

// =============================================================================
// BOSS AI SYSTEMS
// =============================================================================

/// Distance at which boss stops approaching and attacks
pub const BOSS_ATTACK_DISTANCE: f32 = 100.0;

/// Distance for charge attack
pub const BOSS_CHARGE_DISTANCE: f32 = 300.0;

/// Charge attack speed (pixels per second)
pub const BOSS_CHARGE_SPEED: f32 = 800.0;

/// Phase 2 HP threshold (30%)
pub const BOSS_PHASE2_THRESHOLD: f64 = 0.3;

/// Goblin King AI system - handles movement, phase transitions, and ability cooldowns
pub fn goblin_king_ai_system(
    mut commands: Commands,
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    player_query: Query<&Transform, (With<Player>, Without<GoblinKing>)>,
    mut boss_query: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            &EnemyStats,
            &mut BossPhase,
            &mut BossAttackState,
            &mut BossAbilityTimers,
            Option<&BerserkerMode>,
        ),
        With<GoblinKing>,
    >,
) {
    // Don't process if paused
    if debug_settings.is_paused() {
        for (_, _, mut velocity, _, _, _, _, _) in boss_query.iter_mut() {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let dt = time.delta();

    for (entity, boss_transform, mut velocity, stats, mut phase, mut attack_state, mut ability_timers, berserker) in boss_query.iter_mut() {
        let boss_pos = boss_transform.translation.truncate();
        let to_player = player_pos - boss_pos;
        let distance = to_player.length();

        // Check for phase transition (100% -> 30% HP = Phase 2)
        let hp_percent = stats.current_hp / stats.base_hp;
        if *phase == BossPhase::Phase1 && hp_percent <= BOSS_PHASE2_THRESHOLD {
            *phase = BossPhase::Phase2;
            ability_timers.enter_berserker_mode();

            // Add berserker mode marker
            if berserker.is_none() {
                commands.entity(entity).insert(BerserkerMode::default());
            }

            info!("Goblin King enters BERSERKER MODE!");
        }

        // Tick ability cooldowns
        ability_timers.charge_cooldown.tick(dt);
        ability_timers.summon_cooldown.tick(dt);

        // Don't move if in the middle of an attack
        match *attack_state {
            BossAttackState::WindingUpSlam | BossAttackState::Slamming => {
                velocity.x = 0.0;
                velocity.y = 0.0;
                continue;
            }
            BossAttackState::ChargingUp | BossAttackState::Charging => {
                // Charge movement is handled by the charge system
                continue;
            }
            BossAttackState::Summoning => {
                velocity.x = 0.0;
                velocity.y = 0.0;
                continue;
            }
            BossAttackState::Idle => {}
        }

        // Check if we should start a charge attack
        if ability_timers.charge_cooldown.just_finished() && distance > BOSS_ATTACK_DISTANCE * 1.5 {
            *attack_state = BossAttackState::ChargingUp;

            // Add charge attack component
            let charge_damage = stats.base_damage * 1.5; // 75 damage (50 * 1.5)
            commands.entity(entity).insert(BossChargeAttack::new(
                boss_pos,
                player_pos,
                charge_damage,
            ));

            // Spawn telegraph visual
            let direction = to_player.normalize_or_zero();
            let telegraph_end = boss_pos + direction * BOSS_CHARGE_DISTANCE;
            commands.spawn((
                ChargeTelegraph {
                    boss_entity: entity,
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
                Sprite {
                    color: Color::srgba(1.0, 0.2, 0.2, 0.4), // Semi-transparent red
                    custom_size: Some(Vec2::new(BOSS_CHARGE_DISTANCE, 40.0)), // Wide line
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    (boss_pos.x + telegraph_end.x) / 2.0,
                    (boss_pos.y + telegraph_end.y) / 2.0,
                    0.35,
                ))
                .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ));

            continue;
        }

        // Check if we should summon goblins (Phase 1 only)
        if *phase == BossPhase::Phase1 && ability_timers.summon_cooldown.just_finished() {
            *attack_state = BossAttackState::Summoning;
            // Actual summon logic is handled by a separate system
            continue;
        }

        // Normal movement: chase player
        if distance > BOSS_ATTACK_DISTANCE {
            let direction = to_player.normalize();
            // In berserker mode, boss is 1.5x faster
            let speed_multiplier = if berserker.is_some() { 1.5 } else { 1.0 };
            let speed = stats.movement_speed as f32 * speed_multiplier * debug_settings.enemy_speed_multiplier;
            velocity.x = direction.x * speed;
            velocity.y = direction.y * speed;
        } else {
            // Close enough to attack - stop moving
            velocity.x = 0.0;
            velocity.y = 0.0;

            // Trigger slam attack
            if *attack_state == BossAttackState::Idle {
                *attack_state = BossAttackState::WindingUpSlam;
                commands.entity(entity).insert(BossSlamAttack::new(
                    stats.base_damage,
                    stats.attack_range,
                ));
            }
        }
    }
}

/// System to handle boss charge attack execution
pub fn boss_charge_system(
    mut commands: Commands,
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut boss_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut BossChargeAttack,
            &mut BossAttackState,
        ),
        With<GoblinKing>,
    >,
    mut telegraph_query: Query<(Entity, &mut ChargeTelegraph)>,
) {
    if debug_settings.is_paused() {
        return;
    }

    let dt = time.delta();

    for (entity, mut transform, mut velocity, mut charge, mut attack_state) in boss_query.iter_mut() {
        if charge.is_telegraphing {
            // Telegraph phase - boss stays still, shows warning
            charge.telegraph_timer.tick(dt);
            velocity.x = 0.0;
            velocity.y = 0.0;

            if charge.telegraph_timer.finished() {
                charge.is_telegraphing = false;
                *attack_state = BossAttackState::Charging;

                // Remove telegraph visual
                for (telegraph_entity, telegraph) in telegraph_query.iter_mut() {
                    if telegraph.boss_entity == entity {
                        commands.entity(telegraph_entity).despawn();
                    }
                }
            }
        } else {
            // Execution phase - dash toward target
            charge.charge_timer.tick(dt);

            let direction = (charge.target_pos - charge.start_pos).normalize_or_zero();
            let progress = charge.charge_timer.fraction();

            // Move along charge path
            let current_pos = charge.start_pos.lerp(charge.target_pos, progress);
            transform.translation.x = current_pos.x;
            transform.translation.y = current_pos.y;

            // Set velocity for collision detection
            velocity.x = direction.x * BOSS_CHARGE_SPEED;
            velocity.y = direction.y * BOSS_CHARGE_SPEED;

            if charge.charge_timer.finished() {
                // Charge complete
                commands.entity(entity).remove::<BossChargeAttack>();
                *attack_state = BossAttackState::Idle;
                velocity.x = 0.0;
                velocity.y = 0.0;
            }
        }
    }

    // Also tick telegraph timers for cleanup
    for (telegraph_entity, mut telegraph) in telegraph_query.iter_mut() {
        telegraph.timer.tick(dt);
        if telegraph.timer.finished() {
            commands.entity(telegraph_entity).despawn();
        }
    }
}

/// System to exclude boss from regular enemy chase (boss has its own AI)
pub fn exclude_boss_from_chase_system(
    mut enemy_query: Query<&mut Velocity, (With<Enemy>, With<GoblinKing>)>,
) {
    // This system runs after enemy_chase_system to override velocity for bosses
    // The goblin_king_ai_system handles boss movement separately
    for mut velocity in enemy_query.iter_mut() {
        // Boss velocity is set by goblin_king_ai_system, don't let enemy_chase override it
        // Actually, we need a different approach - we'll filter in enemy_chase_system
        let _ = velocity;
    }
}
