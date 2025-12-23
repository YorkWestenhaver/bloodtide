use bevy::prelude::*;

use crate::components::{Creature, CreatureStats};

/// Width of HP bars in pixels
pub const HP_BAR_WIDTH: f32 = 28.0;

/// Height of HP bars in pixels
pub const HP_BAR_HEIGHT: f32 = 4.0;

/// Offset above creature sprite
pub const HP_BAR_OFFSET_Y: f32 = 22.0;

/// Offset below creature sprite for level indicator
pub const LEVEL_LABEL_OFFSET_Y: f32 = -22.0;

/// Size of the tier border (creature size + border thickness)
pub const TIER_BORDER_SIZE: f32 = 38.0;

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

/// Marker component for level label text
#[derive(Component)]
pub struct CreatureLevelLabel {
    pub owner: Entity,
    pub last_level: u32,
}

/// Marker component for tier border/glow
#[derive(Component)]
pub struct CreatureTierBorder {
    pub owner: Entity,
}

/// Get the color for a creature's tier (for border/glow)
pub fn get_tier_border_color(tier: u8) -> Color {
    match tier {
        1 => Color::srgba(0.6, 0.6, 0.6, 0.5),    // Gray - common
        2 => Color::srgba(0.3, 0.8, 0.3, 0.6),    // Green - uncommon
        3 => Color::srgba(0.3, 0.5, 1.0, 0.7),    // Blue - rare
        4 => Color::srgba(0.7, 0.3, 0.9, 0.8),    // Purple - epic
        _ => Color::srgba(1.0, 0.75, 0.2, 0.9),   // Gold - legendary (tier 5+)
    }
}

/// System to spawn HP bars, level labels, and tier borders for creatures
pub fn spawn_hp_bars_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &CreatureStats), (With<Creature>, Without<HpBarBackground>)>,
    hp_bar_query: Query<&HpBarBackground>,
    level_label_query: Query<&CreatureLevelLabel>,
    tier_border_query: Query<&CreatureTierBorder>,
) {
    for (creature_entity, stats) in creature_query.iter() {
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

        // Check if this creature already has a level label
        let has_level_label = level_label_query
            .iter()
            .any(|label| label.owner == creature_entity);

        if !has_level_label {
            // Spawn level label below the creature
            commands.spawn((
                CreatureLevelLabel {
                    owner: creature_entity,
                    last_level: stats.level,
                },
                Text2d::new(format!("L{}", stats.level)),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, LEVEL_LABEL_OFFSET_Y, 0.85)),
            ));
        }

        // Check if this creature already has a tier border
        let has_tier_border = tier_border_query
            .iter()
            .any(|border| border.owner == creature_entity);

        if !has_tier_border && stats.tier >= 2 {
            // Only show tier border for tier 2+ (tier 1 is common, no border)
            let tier_color = get_tier_border_color(stats.tier);
            commands.spawn((
                CreatureTierBorder {
                    owner: creature_entity,
                },
                Sprite {
                    color: tier_color,
                    custom_size: Some(Vec2::new(TIER_BORDER_SIZE, TIER_BORDER_SIZE)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.45)), // Behind creature
            ));
        }
    }
}

/// System to update HP bar positions and widths
pub fn update_hp_bars_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Transform, &CreatureStats), With<Creature>>,
    mut bg_query: Query<
        (Entity, &HpBarBackground, &mut Transform),
        (Without<HpBarForeground>, Without<Creature>),
    >,
    mut fg_query: Query<
        (Entity, &HpBarForeground, &mut Transform, &mut Sprite),
        (Without<HpBarBackground>, Without<Creature>),
    >,
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

/// System to update level labels position and text
pub fn update_level_labels_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Transform, &CreatureStats), With<Creature>>,
    mut label_query: Query<
        (Entity, &mut CreatureLevelLabel, &mut Transform, &mut Text2d),
        Without<Creature>,
    >,
) {
    for (label_entity, mut label, mut label_transform, mut text) in label_query.iter_mut() {
        if let Ok((_, creature_transform, stats)) = creature_query.get(label.owner) {
            // Update position to follow creature
            label_transform.translation.x = creature_transform.translation.x;
            label_transform.translation.y = creature_transform.translation.y + LEVEL_LABEL_OFFSET_Y;

            // Update text if level changed
            if label.last_level != stats.level {
                label.last_level = stats.level;
                *text = Text2d::new(format!("L{}", stats.level));
            }
        } else {
            // Owner no longer exists, despawn the label
            commands.entity(label_entity).despawn();
        }
    }
}

/// System to update tier border positions
pub fn update_tier_borders_system(
    mut commands: Commands,
    creature_query: Query<(Entity, &Transform), With<Creature>>,
    mut border_query: Query<
        (Entity, &CreatureTierBorder, &mut Transform),
        Without<Creature>,
    >,
) {
    for (border_entity, border, mut border_transform) in border_query.iter_mut() {
        if let Ok((_, creature_transform)) = creature_query.get(border.owner) {
            // Update position to follow creature
            border_transform.translation.x = creature_transform.translation.x;
            border_transform.translation.y = creature_transform.translation.y;
        } else {
            // Owner no longer exists, despawn the border
            commands.entity(border_entity).despawn();
        }
    }
}
