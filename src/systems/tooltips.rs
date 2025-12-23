use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Creature, CreatureStats, ProjectileConfig, ProjectileType};
use crate::resources::DebugSettings;

// =============================================================================
// CONSTANTS
// =============================================================================

const TOOLTIP_BACKGROUND: Color = Color::srgba(0.05, 0.05, 0.1, 0.95);
const TOOLTIP_BORDER: Color = Color::srgb(0.3, 0.3, 0.4);
const TOOLTIP_PADDING: f32 = 10.0;
const TOOLTIP_MAX_WIDTH: f32 = 300.0;
const TOOLTIP_OFFSET: Vec2 = Vec2::new(15.0, 10.0); // Offset from cursor
const TOOLTIP_Z_INDEX: i32 = 200;

// =============================================================================
// COMPONENTS
// =============================================================================

/// Marker for tooltip target UI elements
#[derive(Component)]
pub struct TooltipTarget {
    /// Content to display in tooltip
    pub content: TooltipContent,
}

/// Different types of tooltip content
#[derive(Clone)]
pub enum TooltipContent {
    /// Tooltip for a creature - stores the creature entity
    Creature(Entity),
    /// Custom text tooltip
    Text(String),
    /// Tooltip with title and description
    TitleAndDescription { title: String, description: String },
}

/// Marker for the active tooltip entity
#[derive(Component)]
pub struct Tooltip;

/// Tracks hover state for tooltip targets
#[derive(Resource)]
pub struct TooltipState {
    /// Currently hovered target entity
    pub hovered_target: Option<Entity>,
    /// Time hovering over current target (in seconds)
    pub hover_time: f32,
    /// Whether a tooltip is currently visible
    pub tooltip_visible: bool,
    /// Current tooltip entity (if any)
    pub tooltip_entity: Option<Entity>,
    /// Last known cursor position
    pub cursor_position: Vec2,
}

impl Default for TooltipState {
    fn default() -> Self {
        Self {
            hovered_target: None,
            hover_time: 0.0,
            tooltip_visible: false,
            tooltip_entity: None,
            cursor_position: Vec2::ZERO,
        }
    }
}

// =============================================================================
// SYSTEMS
// =============================================================================

/// System to track hover state on tooltip targets
pub fn tooltip_hover_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut tooltip_state: ResMut<TooltipState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    target_query: Query<(Entity, &Node, &GlobalTransform, &TooltipTarget)>,
    interaction_query: Query<(Entity, &Interaction), With<TooltipTarget>>,
) {
    // Update cursor position
    if let Ok(window) = window_query.get_single() {
        if let Some(cursor_pos) = window.cursor_position() {
            tooltip_state.cursor_position = cursor_pos;
        }
    }

    // Find currently hovered target using Interaction
    let mut new_hovered: Option<Entity> = None;
    for (entity, interaction) in interaction_query.iter() {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            new_hovered = Some(entity);
            break;
        }
    }

    // Update hover state
    if new_hovered != tooltip_state.hovered_target {
        // Target changed - reset hover time
        tooltip_state.hovered_target = new_hovered;
        tooltip_state.hover_time = 0.0;
    } else if new_hovered.is_some() {
        // Same target - accumulate hover time
        tooltip_state.hover_time += time.delta_secs();
    }

    // Check if we should show tooltip
    let delay_secs = debug_settings.tooltip_delay_ms as f32 / 1000.0;
    let should_show = tooltip_state.hovered_target.is_some()
        && tooltip_state.hover_time >= delay_secs
        && debug_settings.show_advanced_tooltips;

    // Update visibility flag
    if should_show != tooltip_state.tooltip_visible {
        tooltip_state.tooltip_visible = should_show;
    }
}

/// System to spawn and despawn tooltips based on hover state
pub fn tooltip_spawn_system(
    mut commands: Commands,
    tooltip_state: Res<TooltipState>,
    debug_settings: Res<DebugSettings>,
    target_query: Query<&TooltipTarget>,
    creature_query: Query<(&CreatureStats, &ProjectileConfig), With<Creature>>,
    existing_tooltip_query: Query<Entity, With<Tooltip>>,
) {
    // Despawn existing tooltip if we shouldn't show one
    if !tooltip_state.tooltip_visible || tooltip_state.hovered_target.is_none() {
        for entity in existing_tooltip_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        return;
    }

    // Don't spawn if already exists
    if !existing_tooltip_query.is_empty() {
        return;
    }

    // Get the target's content
    let Some(target_entity) = tooltip_state.hovered_target else {
        return;
    };
    let Ok(target) = target_query.get(target_entity) else {
        return;
    };

    // Build tooltip content based on type
    let (title, lines) = match &target.content {
        TooltipContent::Creature(creature_entity) => {
            if let Ok((stats, projectile_config)) = creature_query.get(*creature_entity) {
                build_creature_tooltip(stats, projectile_config)
            } else {
                ("Unknown".to_string(), vec!["No data available".to_string()])
            }
        }
        TooltipContent::Text(text) => {
            ("".to_string(), vec![text.clone()])
        }
        TooltipContent::TitleAndDescription { title, description } => {
            (title.clone(), vec![description.clone()])
        }
    };

    // Calculate tooltip position (near cursor)
    let pos = tooltip_state.cursor_position + TOOLTIP_OFFSET;

    // Spawn tooltip
    commands.spawn((
        Tooltip,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(pos.x),
            top: Val::Px(pos.y),
            max_width: Val::Px(TOOLTIP_MAX_WIDTH),
            padding: UiRect::all(Val::Px(TOOLTIP_PADDING)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(TOOLTIP_BACKGROUND),
        BorderColor(TOOLTIP_BORDER),
        Outline {
            width: Val::Px(1.0),
            color: TOOLTIP_BORDER,
            ..default()
        },
        ZIndex(TOOLTIP_Z_INDEX),
    )).with_children(|parent| {
        // Title (if any)
        if !title.is_empty() {
            parent.spawn((
                Text::new(&title),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));
        }

        // Content lines
        for line in lines {
            parent.spawn((
                Text::new(&line),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.85, 0.85, 0.85)),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ));
        }
    });
}

/// System to update tooltip position to follow cursor
pub fn tooltip_position_system(
    tooltip_state: Res<TooltipState>,
    mut tooltip_query: Query<&mut Node, With<Tooltip>>,
) {
    if !tooltip_state.tooltip_visible {
        return;
    }

    for mut node in tooltip_query.iter_mut() {
        let pos = tooltip_state.cursor_position + TOOLTIP_OFFSET;
        node.left = Val::Px(pos.x);
        node.top = Val::Px(pos.y);
    }
}

/// System to clear tooltips when settings change
pub fn tooltip_settings_change_system(
    mut commands: Commands,
    debug_settings: Res<DebugSettings>,
    mut tooltip_state: ResMut<TooltipState>,
    tooltip_query: Query<Entity, With<Tooltip>>,
) {
    // If tooltips are disabled, clear any existing tooltip
    if !debug_settings.show_advanced_tooltips && tooltip_state.tooltip_visible {
        for entity in tooltip_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        tooltip_state.tooltip_visible = false;
        tooltip_state.tooltip_entity = None;
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Build tooltip content for a creature
fn build_creature_tooltip(stats: &CreatureStats, projectile_config: &ProjectileConfig) -> (String, Vec<String>) {
    let title = format!("{} (Tier {})", stats.name, stats.tier);

    let mut lines = Vec::new();

    // Basic stats
    lines.push(format!("Level: {} | Kills: {}", stats.level, stats.kills));
    lines.push(format!("HP: {:.0}/{:.0}", stats.current_hp, stats.max_hp));
    lines.push(format!("Damage: {:.1} | Speed: {:.0}", stats.base_damage, stats.movement_speed));
    lines.push(format!("Attack Speed: {:.2}/s | Range: {:.0}", stats.attack_speed, stats.attack_range));

    // Crit chances
    if stats.crit_t1 > 0.0 || stats.crit_t2 > 0.0 || stats.crit_t3 > 0.0 {
        lines.push(format!(
            "Crit: T1 {:.0}% | T2 {:.0}% | T3 {:.0}%",
            stats.crit_t1, stats.crit_t2, stats.crit_t3
        ));
    }

    // Projectile info
    let projectile_type_str = match projectile_config.projectile_type {
        ProjectileType::Basic => "Basic",
        ProjectileType::Piercing => "Piercing",
        ProjectileType::Explosive => "Explosive",
        ProjectileType::Homing => "Homing",
        ProjectileType::Chain => "Chain",
    };

    lines.push(format!(
        "Projectiles: {}x {} (Pen: {})",
        projectile_config.count,
        projectile_type_str,
        projectile_config.penetration
    ));

    // Evolution info
    if !stats.evolves_into.is_empty() {
        lines.push(format!("Evolves into: {}", stats.evolves_into));
    }

    (title, lines)
}

/// Format a stat line for tooltip display
fn format_stat_line(label: &str, value: f64, suffix: &str) -> String {
    if value >= 1000.0 {
        format!("{}: {:.1}k{}", label, value / 1000.0, suffix)
    } else if value == value.floor() {
        format!("{}: {:.0}{}", label, value, suffix)
    } else {
        format!("{}: {:.1}{}", label, value, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_state_default() {
        let state = TooltipState::default();
        assert!(state.hovered_target.is_none());
        assert_eq!(state.hover_time, 0.0);
        assert!(!state.tooltip_visible);
        assert!(state.tooltip_entity.is_none());
    }

    #[test]
    fn format_stat_line_works() {
        assert_eq!(format_stat_line("Damage", 100.0, ""), "Damage: 100");
        assert_eq!(format_stat_line("HP", 1500.0, ""), "HP: 1.5k");
        assert_eq!(format_stat_line("Speed", 150.5, ""), "Speed: 150.5");
    }
}
