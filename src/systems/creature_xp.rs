use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::components::{Creature, CreatureStats};
use crate::resources::GameData;
use crate::systems::spawning::CREATURE_SIZE;

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

                // SFX placeholder
                println!("SFX_CREATURE_LEVEL");
                println!(
                    "{} leveled up to {}! (Damage: {:.1}, HP: {:.0}/{:.0})",
                    stats.name, stats.level, stats.base_damage, stats.current_hp, stats.max_hp
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

/// System that checks for evolution-ready creatures and announces (does NOT auto-evolve)
/// Future phases will add UI button to trigger evolution manually
pub fn creature_evolution_system(
    game_data: Res<GameData>,
    mut evolution_state: ResMut<EvolutionReadyState>,
    creature_query: Query<&CreatureStats, With<Creature>>,
) {
    // Group creatures by ID
    let mut creatures_by_id: HashMap<String, usize> = HashMap::new();

    for stats in creature_query.iter() {
        // Skip creatures that can't evolve
        if stats.evolves_into.is_empty() || stats.evolution_count == 0 {
            continue;
        }

        *creatures_by_id.entry(stats.id.clone()).or_default() += 1;
    }

    // Check each creature type for evolution readiness
    for (creature_id, count) in creatures_by_id {
        // Get the evolution count for this creature type
        let evolution_count = creature_query
            .iter()
            .find(|stats| stats.id == creature_id)
            .map(|stats| stats.evolution_count)
            .unwrap_or(3);

        if count >= evolution_count as usize {
            // Only announce once per creature type
            if evolution_state.announced.contains(&creature_id) {
                continue;
            }

            // Get the evolves_into ID
            let evolves_into = creature_query
                .iter()
                .find(|stats| stats.id == creature_id)
                .map(|stats| stats.evolves_into.clone())
                .unwrap_or_default();

            if evolves_into.is_empty() {
                continue;
            }

            // Get creature name for logging
            let old_name = creature_query
                .iter()
                .find(|stats| stats.id == creature_id)
                .map(|stats| stats.name.clone())
                .unwrap_or_else(|| creature_id.clone());

            // Get evolved creature name
            let new_name = game_data
                .creatures
                .iter()
                .find(|c| c.id == evolves_into)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| evolves_into.clone());

            println!("SFX_EVOLUTION");
            println!(
                "Evolution ready: {}x {} can become {}!",
                evolution_count, old_name, new_name
            );

            // Mark as announced
            evolution_state.announced.insert(creature_id);
        }
    }

    // Clear announcements for creatures that are no longer evolution-ready
    // (e.g., if one died and count dropped below threshold)
    evolution_state.announced.retain(|creature_id| {
        let count = creature_query
            .iter()
            .filter(|stats| stats.id == *creature_id)
            .count();
        let evolution_count = creature_query
            .iter()
            .find(|stats| stats.id == *creature_id)
            .map(|stats| stats.evolution_count)
            .unwrap_or(3);
        count >= evolution_count as usize
    });
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
