use bevy::prelude::*;

use crate::components::{Creature, CreatureStats};

/// Width of HP bars in pixels
pub const HP_BAR_WIDTH: f32 = 28.0;

/// Height of HP bars in pixels
pub const HP_BAR_HEIGHT: f32 = 4.0;

/// Offset above creature sprite
pub const HP_BAR_OFFSET_Y: f32 = 22.0;

/// Marker component for HP bar backgrounds
#[derive(Component)]
pub struct HpBarBackground {
    pub owner: Entity,
}

/// Marker component for HP bar foregrounds (the actual HP indicator)
#[derive(Component)]
pub struct HpBarForeground {
    pub owner: Entity,
}

/// System to spawn HP bars for creatures that don't have them
pub fn spawn_hp_bars_system(
    mut commands: Commands,
    creature_query: Query<Entity, (With<Creature>, Without<HpBarBackground>)>,
    hp_bar_query: Query<&HpBarBackground>,
) {
    for creature_entity in creature_query.iter() {
        // Check if this creature already has an HP bar
        let has_hp_bar = hp_bar_query
            .iter()
            .any(|bg| bg.owner == creature_entity);

        if !has_hp_bar {
            // Spawn background (dark bar)
            commands.spawn((
                HpBarBackground {
                    owner: creature_entity,
                },
                Sprite {
                    color: Color::srgba(0.2, 0.2, 0.2, 0.8),
                    custom_size: Some(Vec2::new(HP_BAR_WIDTH, HP_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, HP_BAR_OFFSET_Y, 0.8)),
            ));

            // Spawn foreground (green bar)
            commands.spawn((
                HpBarForeground {
                    owner: creature_entity,
                },
                Sprite {
                    color: Color::srgb(0.2, 0.9, 0.3),
                    custom_size: Some(Vec2::new(HP_BAR_WIDTH, HP_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, HP_BAR_OFFSET_Y, 0.81)),
            ));
        }
    }
}

/// System to update HP bar positions and widths
pub fn update_hp_bars_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Transform, &CreatureStats), With<Creature>>,
    mut bg_query: Query<(Entity, &HpBarBackground, &mut Transform), Without<HpBarForeground>>,
    mut fg_query: Query<(Entity, &HpBarForeground, &mut Transform, &mut Sprite), Without<HpBarBackground>>,
) {
    // Update background bars
    for (bar_entity, hp_bar, mut bar_transform) in bg_query.iter_mut() {
        if let Ok((_, creature_transform, _)) = creature_query.get(hp_bar.owner) {
            // Update position to follow creature
            bar_transform.translation.x = creature_transform.translation.x;
            bar_transform.translation.y = creature_transform.translation.y + HP_BAR_OFFSET_Y;
        } else {
            // Owner no longer exists, despawn the bar
            commands.entity(bar_entity).despawn();
        }
    }

    // Update foreground bars (HP indicator)
    for (bar_entity, hp_bar, mut bar_transform, mut sprite) in fg_query.iter_mut() {
        if let Ok((_, creature_transform, stats)) = creature_query.get(hp_bar.owner) {
            // Calculate HP percentage
            let hp_percent = (stats.current_hp / stats.max_hp).clamp(0.0, 1.0);

            // Update bar width based on HP
            let bar_width = HP_BAR_WIDTH * hp_percent as f32;
            sprite.custom_size = Some(Vec2::new(bar_width, HP_BAR_HEIGHT));

            // Update position (left-aligned)
            let offset_x = (HP_BAR_WIDTH - bar_width) / 2.0;
            bar_transform.translation.x = creature_transform.translation.x - offset_x;
            bar_transform.translation.y = creature_transform.translation.y + HP_BAR_OFFSET_Y;

            // Change color based on HP percentage
            sprite.color = if hp_percent > 0.6 {
                Color::srgb(0.2, 0.9, 0.3) // Green
            } else if hp_percent > 0.3 {
                Color::srgb(0.9, 0.9, 0.2) // Yellow
            } else {
                Color::srgb(0.9, 0.2, 0.2) // Red
            };
        } else {
            // Owner no longer exists, despawn the bar
            commands.entity(bar_entity).despawn();
        }
    }
}
