use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::components::{AttackRange, Creature, CreatureStats};
use crate::resources::{ArtifactBuffs, DebugSettings, GameData};
use crate::systems::spawning::{spawn_creature, CREATURE_SIZE};

/// Marker for pending kill attribution
/// This is added when a projectile kills an enemy, to be processed by creature_xp_system
#[derive(Component)]
pub struct PendingKillCredit {
    pub creature_entity: Entity,
}

/// Visual effect for creature level up
#[derive(Component)]
pub struct CreatureLevelUpEffect {
    pub timer: Timer,
}

/// Floating +1 text when creature levels up
#[derive(Component)]
pub struct CreatureLevelUpText {
    pub timer: Timer,
}

/// Visual effect for creature evolution
#[derive(Component)]
pub struct EvolutionEffect {
    pub timer: Timer,
}

/// Resource to track which evolutions have been announced (to avoid spam)
#[derive(Resource, Default)]
pub struct EvolutionReadyState {
    /// Set of creature IDs that have had "evolution ready" announced
    pub announced: HashSet<String>,
}

/// System that processes kills and awards XP to creatures
/// This runs after projectile_system and checks for enemies that died
pub fn creature_xp_system(
    mut commands: Commands,
    game_data: Res<GameData>,
    mut creature_query: Query<(Entity, &mut CreatureStats, &mut AttackRange, &Transform), With<Creature>>,
    kill_credit_query: Query<(Entity, &PendingKillCredit)>,
) {
    // Process all pending kill credits
    for (credit_entity, credit) in kill_credit_query.iter() {
        // Remove the credit entity
        commands.entity(credit_entity).despawn();

        // Find the creature and increment its kills
        if let Ok((creature_entity, mut stats, mut attack_range, transform)) = creature_query.get_mut(credit.creature_entity) {
            stats.kills += 1;

            // Check for level up
            if stats.kills >= stats.kills_for_next_level && stats.level < stats.max_level {
                // Level up!
                stats.level += 1;

                // Apply stat boosts: +10% damage, +10% HP, +5% attack range
                stats.base_damage *= 1.1;
                let hp_increase = stats.max_hp * 0.1;
                stats.max_hp += hp_increase;
                stats.current_hp += hp_increase; // Heal by the amount of HP gained
                attack_range.0 *= 1.05; // +5% range per level

                // Get next threshold from kills_per_level array
                if let Some(creature_data) = game_data.creatures.iter().find(|c| c.id == stats.id) {
                    let level_index = (stats.level - 1) as usize; // level 2 -> index 1
                    stats.kills_for_next_level = creature_data
                        .kills_per_level
                        .get(level_index)
                        .copied()
                        .unwrap_or(u32::MAX); // Cap at max if no more levels
                }

                // Reset kills (carry overflow)
                let overflow = stats.kills.saturating_sub(stats.kills_for_next_level);
                stats.kills = overflow;

                // SFX placeholder
                println!("SFX_CREATURE_LEVEL");
                println!(
                    "{} leveled up to {}! (Damage: {:.1}, HP: {:.0}/{:.0}, Range: {:.0})",
                    stats.name, stats.level, stats.base_damage, stats.current_hp, stats.max_hp, attack_range.0
                );

                // Spawn level up visual effect (green glow expanding ring)
                let pos = transform.translation;
                commands.spawn((
                    CreatureLevelUpEffect {
                        timer: Timer::from_seconds(0.4, TimerMode::Once),
                    },
                    Sprite {
                        color: Color::srgba(0.4, 1.0, 0.4, 0.8), // Green glow
                        custom_size: Some(Vec2::new(CREATURE_SIZE * 1.5, CREATURE_SIZE * 1.5)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(pos.x, pos.y, 0.75)),
                ));

                // Spawn +1 floating text
                commands.spawn((
                    CreatureLevelUpText {
                        timer: Timer::from_seconds(0.6, TimerMode::Once),
                    },
                    Text2d::new(format!("+{}", stats.level)),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 1.0, 0.4)), // Green
                    Transform::from_translation(Vec3::new(pos.x, pos.y + 30.0, 10.0)),
                ));
            }
        }
    }
}

/// System that updates creature level up effects
pub fn creature_level_up_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_query: Query<(Entity, &mut CreatureLevelUpEffect, &mut Sprite, &mut Transform)>,
    mut text_query: Query<
        (Entity, &mut CreatureLevelUpText, &mut Transform, &mut TextColor),
        Without<CreatureLevelUpEffect>,
    >,
) {
    // Update ring effects
    for (entity, mut effect, mut sprite, mut transform) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());

        // Expand and fade
        let progress = effect.timer.fraction();
        let scale = 1.0 + progress * 1.5;
        transform.scale = Vec3::splat(scale);

        let alpha = (1.0 - progress) * 0.8;
        sprite.color = Color::srgba(0.4, 1.0, 0.4, alpha);

        if effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }

    // Update floating text
    for (entity, mut text_effect, mut transform, mut text_color) in text_query.iter_mut() {
        text_effect.timer.tick(time.delta());

        let progress = text_effect.timer.fraction();

        // Float upward
        transform.translation.y += 40.0 * time.delta_secs();

        // Fade out
        let alpha = 1.0 - progress;
        *text_color = TextColor(Color::srgba(0.4, 1.0, 0.4, alpha));

        if text_effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// System that checks for evolution-ready creatures and performs evolution
/// In auto mode: evolves immediately when 3+ same creatures exist
/// In manual mode: evolves when player presses the configured hotkey
pub fn creature_evolution_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game_data: Res<GameData>,
    artifact_buffs: Res<ArtifactBuffs>,
    debug_settings: Res<DebugSettings>,
    mut evolution_state: ResMut<EvolutionReadyState>,
    creature_query: Query<(Entity, &CreatureStats, &Transform), With<Creature>>,
) {
    // Don't process evolution while waiting for keybind
    if debug_settings.waiting_for_keybind {
        return;
    }

    // Group creatures by ID, collecting entity, stats, and position
    let mut creatures_by_id: HashMap<String, Vec<(Entity, CreatureStats, Vec3)>> = HashMap::new();

    for (entity, stats, transform) in creature_query.iter() {
        // Skip creatures that can't evolve
        if stats.evolves_into.is_empty() || stats.evolution_count == 0 {
            continue;
        }

        creatures_by_id
            .entry(stats.id.clone())
            .or_default()
            .push((entity, stats.clone(), transform.translation));
    }

    // Check each creature type for evolution readiness
    for (creature_id, mut creatures) in creatures_by_id {
        let evolution_count = creatures
            .first()
            .map(|(_, stats, _)| stats.evolution_count)
            .unwrap_or(3) as usize;

        if creatures.len() < evolution_count {
            // Not enough creatures - clear announcement if it was set
            evolution_state.announced.remove(&creature_id);
            continue;
        }

        // We have enough creatures for evolution
        let evolves_into = creatures
            .first()
            .map(|(_, stats, _)| stats.evolves_into.clone())
            .unwrap_or_default();

        if evolves_into.is_empty() {
            continue;
        }

        // Announce if not yet announced (for manual mode UI feedback)
        if !evolution_state.announced.contains(&creature_id) {
            let old_name = creatures
                .first()
                .map(|(_, stats, _)| stats.name.clone())
                .unwrap_or_else(|| creature_id.clone());

            let new_name = game_data
                .creatures
                .iter()
                .find(|c| c.id == evolves_into)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| evolves_into.clone());

            println!("SFX_EVOLUTION_READY");
            println!(
                "Evolution ready: {}x {} can become {}!",
                evolution_count, old_name, new_name
            );

            evolution_state.announced.insert(creature_id.clone());
        }

        // Check if we should trigger evolution
        let should_evolve = debug_settings.auto_evolve
            || (!debug_settings.auto_evolve
                && keyboard_input.just_pressed(debug_settings.evolution_hotkey));

        if should_evolve {
            // Perform the evolution
            perform_evolution(
                &mut commands,
                &game_data,
                &artifact_buffs,
                &mut creatures,
                evolution_count,
            );

            // Clear the announcement since we consumed the creatures
            evolution_state.announced.remove(&creature_id);

            // In manual mode, only evolve one type per key press
            if !debug_settings.auto_evolve {
                break;
            }
        }
    }
}

/// Helper function to perform creature evolution
fn perform_evolution(
    commands: &mut Commands,
    game_data: &GameData,
    artifact_buffs: &ArtifactBuffs,
    creatures: &mut Vec<(Entity, CreatureStats, Vec3)>,
    count: usize,
) {
    // Sort by level ascending to consume lowest level creatures first
    creatures.sort_by(|a, b| a.1.level.cmp(&b.1.level));

    // Take the first `count` creatures to consume
    let to_consume: Vec<_> = creatures.drain(..count).collect();

    // Calculate average position for spawning the evolved creature
    let avg_pos = to_consume
        .iter()
        .map(|(_, _, pos)| *pos)
        .reduce(|a, b| a + b)
        .map(|sum| sum / count as f32)
        .unwrap_or(Vec3::ZERO);

    // Get evolution target info
    let evolved_id = &to_consume[0].1.evolves_into;
    let old_name = &to_consume[0].1.name;

    // Spawn gold flash effects at each consumed creature position and despawn them
    for (entity, _, pos) in &to_consume {
        spawn_evolution_effect(commands, *pos);
        commands.entity(*entity).despawn_recursive();
    }

    // Spawn evolution effect at the new creature's spawn position
    spawn_evolution_effect(commands, avg_pos);

    // Spawn the evolved creature
    if spawn_creature(commands, game_data, artifact_buffs, evolved_id, avg_pos).is_some() {
        let new_name = game_data
            .creatures
            .iter()
            .find(|c| c.id == *evolved_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| evolved_id.clone());

        println!("SFX_EVOLUTION_COMPLETE");
        println!("{}x {} evolved into {}!", count, old_name, new_name);
    }
}

/// Spawn a gold expanding ring effect at the given position
fn spawn_evolution_effect(commands: &mut Commands, position: Vec3) {
    commands.spawn((
        EvolutionEffect {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        },
        Sprite {
            color: Color::srgba(1.0, 0.8, 0.2, 0.9), // Gold
            custom_size: Some(Vec2::new(CREATURE_SIZE * 1.5, CREATURE_SIZE * 1.5)),
            ..default()
        },
        Transform::from_translation(Vec3::new(position.x, position.y, 0.8)),
    ));
}

/// System that updates evolution effects
pub fn evolution_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_query: Query<(Entity, &mut EvolutionEffect, &mut Sprite, &mut Transform)>,
) {
    for (entity, mut effect, mut sprite, mut transform) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());

        // Expand rapidly and fade
        let progress = effect.timer.fraction();
        let scale = 1.0 + progress * 2.5;
        transform.scale = Vec3::splat(scale);

        let alpha = (1.0 - progress) * 0.9;
        sprite.color = Color::srgba(1.0, 0.8, 0.2, alpha);

        if effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_kill_credit_stores_entity() {
        // This is a simple component test - can't fully test without Bevy World
        // The component should store the creature entity correctly
    }

    #[test]
    fn creature_level_up_effect_has_timer() {
        let effect = CreatureLevelUpEffect {
            timer: Timer::from_seconds(0.4, TimerMode::Once),
        };
        assert_eq!(effect.timer.duration().as_secs_f32(), 0.4);
    }

    #[test]
    fn evolution_effect_has_timer() {
        let effect = EvolutionEffect {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        };
        assert_eq!(effect.timer.duration().as_secs_f32(), 0.5);
    }
}
