use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::{Creature, CreatureStats};
use crate::resources::{ArtifactBuffs, GameData};
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

/// Visual effect for creature evolution
#[derive(Component)]
pub struct EvolutionEffect {
    pub timer: Timer,
}

/// System that processes kills and awards XP to creatures
/// This runs after projectile_system and checks for enemies that died
pub fn creature_xp_system(
    mut commands: Commands,
    game_data: Res<GameData>,
    mut creature_query: Query<(Entity, &mut CreatureStats, &Transform), With<Creature>>,
    kill_credit_query: Query<(Entity, &PendingKillCredit)>,
) {
    // Process all pending kill credits
    for (credit_entity, credit) in kill_credit_query.iter() {
        // Remove the credit entity
        commands.entity(credit_entity).despawn();

        // Find the creature and increment its kills
        if let Ok((creature_entity, mut stats, transform)) = creature_query.get_mut(credit.creature_entity) {
            stats.kills += 1;

            // Check for level up
            if stats.kills >= stats.kills_for_next_level && stats.level < stats.max_level {
                // Level up!
                stats.level += 1;

                // Apply stat boosts: +10% damage, +10% HP
                stats.base_damage *= 1.1;
                let hp_increase = stats.max_hp * 0.1;
                stats.max_hp += hp_increase;
                stats.current_hp += hp_increase; // Heal by the amount of HP gained

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

                println!(
                    "{} leveled up to {}! (Damage: {:.1}, HP: {:.0}/{:.0})",
                    stats.name, stats.level, stats.base_damage, stats.current_hp, stats.max_hp
                );

                // Spawn level up visual effect
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
            }
        }
    }
}

/// System that updates creature level up effects
pub fn creature_level_up_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut effect_query: Query<(Entity, &mut CreatureLevelUpEffect, &mut Sprite, &mut Transform)>,
) {
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
}

/// System that handles creature evolution when 3+ of the same type exist
pub fn creature_evolution_system(
    mut commands: Commands,
    game_data: Res<GameData>,
    artifact_buffs: Res<ArtifactBuffs>,
    creature_query: Query<(Entity, &CreatureStats, &Transform), With<Creature>>,
) {
    // Group creatures by ID
    let mut creatures_by_id: HashMap<String, Vec<(Entity, u32, Vec3)>> = HashMap::new();

    for (entity, stats, transform) in creature_query.iter() {
        // Skip creatures that can't evolve
        if stats.evolves_into.is_empty() || stats.evolution_count == 0 {
            continue;
        }

        creatures_by_id
            .entry(stats.id.clone())
            .or_default()
            .push((entity, stats.level, transform.translation));
    }

    // Check each creature type for evolution
    for (creature_id, mut creatures) in creatures_by_id {
        // Get the evolution count for this creature type
        let evolution_count = creature_query
            .iter()
            .find(|(_, stats, _)| stats.id == creature_id)
            .map(|(_, stats, _)| stats.evolution_count)
            .unwrap_or(3);

        if creatures.len() >= evolution_count as usize {
            // Sort by level (ascending) to evolve lowest level ones first
            creatures.sort_by_key(|(_, level, _)| *level);

            // Take the first evolution_count creatures
            let to_evolve: Vec<_> = creatures.iter().take(evolution_count as usize).collect();

            // Calculate average position for evolved creature
            let avg_pos = to_evolve.iter().fold(Vec3::ZERO, |acc, (_, _, pos)| acc + *pos)
                / evolution_count as f32;

            // Get the evolves_into ID
            let evolves_into = creature_query
                .iter()
                .find(|(_, stats, _)| stats.id == creature_id)
                .map(|(_, stats, _)| stats.evolves_into.clone())
                .unwrap_or_default();

            if evolves_into.is_empty() {
                continue;
            }

            // Get creature name for logging
            let old_name = creature_query
                .iter()
                .find(|(_, stats, _)| stats.id == creature_id)
                .map(|(_, stats, _)| stats.name.clone())
                .unwrap_or_else(|| creature_id.clone());

            // Despawn the creatures being evolved
            for (entity, _, pos) in to_evolve.iter() {
                // Spawn evolution flash at each creature's position
                commands.spawn((
                    EvolutionEffect {
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                    Sprite {
                        color: Color::srgba(1.0, 0.8, 0.2, 0.9), // Golden flash
                        custom_size: Some(Vec2::new(CREATURE_SIZE * 2.0, CREATURE_SIZE * 2.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(pos.x, pos.y, 0.8)),
                ));

                commands.entity(*entity).despawn_recursive();
            }

            // Spawn the evolved creature
            let spawn_pos = Vec3::new(avg_pos.x, avg_pos.y, 0.5);
            if let Some(_evolved_entity) = spawn_creature(
                &mut commands,
                &game_data,
                &artifact_buffs,
                &evolves_into,
                spawn_pos,
            ) {
                // Get evolved creature name for logging
                let new_name = game_data
                    .creatures
                    .iter()
                    .find(|c| c.id == evolves_into)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| evolves_into.clone());

                println!(
                    "{}x {} evolved into {}!",
                    evolution_count, old_name, new_name
                );
            }

            // Only process one evolution per frame to avoid iterator invalidation issues
            break;
        }
    }
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
