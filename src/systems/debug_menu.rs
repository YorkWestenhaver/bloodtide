use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::resources::{DebugSettings, GameState, MenuState, SliderRange};

// =============================================================================
// CONSTANTS
// =============================================================================

const DEBUG_MENU_WIDTH: f32 = 320.0;
const DEBUG_MENU_PADDING: f32 = 15.0;
const SLIDER_HEIGHT: f32 = 24.0;
const SLIDER_BAR_HEIGHT: f32 = 8.0;
const SLIDER_LABEL_WIDTH: f32 = 160.0;
const SLIDER_VALUE_WIDTH: f32 = 50.0;
const SLIDER_BAR_WIDTH: f32 = 80.0;
const BUTTON_HEIGHT: f32 = 30.0;
const CHECKBOX_SIZE: f32 = 20.0;
const MENU_ANIMATION_SPEED: f32 = 5.0; // Speed of slide animation

const PAUSE_MENU_WIDTH: f32 = 300.0;
const PAUSE_MENU_HEIGHT: f32 = 280.0;

const PANEL_BACKGROUND: Color = Color::srgba(0.08, 0.08, 0.12, 0.95);
const SLIDER_BG: Color = Color::srgb(0.15, 0.15, 0.2);
const SLIDER_FILL: Color = Color::srgb(0.3, 0.6, 0.9);
const SLIDER_HANDLE: Color = Color::srgb(0.9, 0.9, 0.95);
const BUTTON_BG: Color = Color::srgb(0.2, 0.2, 0.3);
const BUTTON_HOVER: Color = Color::srgb(0.3, 0.3, 0.45);
const CHECKBOX_BG: Color = Color::srgb(0.2, 0.2, 0.25);
const CHECKBOX_CHECKED: Color = Color::srgb(0.3, 0.7, 0.4);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const OVERLAY_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);

// =============================================================================
// MARKER COMPONENTS
// =============================================================================

/// Marker for the debug menu container
#[derive(Component)]
pub struct DebugMenuPanel;

/// Marker for the pause menu container
#[derive(Component)]
pub struct PauseMenuPanel;

/// Marker for the pause menu overlay (dark background)
#[derive(Component)]
pub struct PauseMenuOverlay;

/// Debug slider component
#[derive(Component)]
pub struct DebugSlider {
    pub setting_id: SliderSettingId,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

/// The slider bar background (clickable area)
#[derive(Component)]
pub struct SliderBar {
    pub setting_id: SliderSettingId,
}

/// The slider fill indicator
#[derive(Component)]
pub struct SliderFill {
    pub setting_id: SliderSettingId,
}

/// The slider handle (draggable)
#[derive(Component)]
pub struct SliderHandle {
    pub setting_id: SliderSettingId,
}

/// The slider value text display
#[derive(Component)]
pub struct SliderValueText {
    pub setting_id: SliderSettingId,
}

/// Increment/decrement button for slider
#[derive(Component)]
pub struct SliderButton {
    pub setting_id: SliderSettingId,
    pub increment: f32, // Positive or negative for +/-
}

/// Debug checkbox component
#[derive(Component)]
pub struct DebugCheckbox {
    pub setting_id: CheckboxSettingId,
}

/// Checkbox indicator (the checkmark)
#[derive(Component)]
pub struct CheckboxIndicator {
    pub setting_id: CheckboxSettingId,
}

/// Reset to defaults button
#[derive(Component)]
pub struct ResetDefaultsButton;

/// Pause menu resume button
#[derive(Component)]
pub struct ResumeButton;

/// Pause menu restart button
#[derive(Component)]
pub struct RestartButton;

/// Pause menu quit button
#[derive(Component)]
pub struct QuitButton;

/// Toggle mode checkbox in pause menu
#[derive(Component)]
pub struct ToggleModeCheckbox;

// =============================================================================
// SETTING IDS
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SliderSettingId {
    PlayerSpeed,
    CreatureSpeed,
    CreatureDamage,
    EnemyDamage,
    EnemySpeed,
    SpawnRate,
    CritT1,
    CritT2,
    CritT3,
    ProjectileCount,
    ProjectileSize,
    ProjectileSpeed,
    AttackSpeed,
    PenetrationBonus,
    WaveOverride,
    LevelOverride,
}

impl SliderSettingId {
    fn label(&self) -> &'static str {
        match self {
            Self::PlayerSpeed => "Player Speed",
            Self::CreatureSpeed => "Creature Speed",
            Self::CreatureDamage => "Creature Damage",
            Self::EnemyDamage => "Enemy Damage",
            Self::EnemySpeed => "Enemy Speed",
            Self::SpawnRate => "Spawn Rate",
            Self::CritT1 => "Crit T1 Bonus",
            Self::CritT2 => "Crit T2 Bonus",
            Self::CritT3 => "Crit T3 Bonus",
            Self::ProjectileCount => "Projectile Count",
            Self::ProjectileSize => "Projectile Size",
            Self::ProjectileSpeed => "Projectile Speed",
            Self::AttackSpeed => "Attack Speed",
            Self::PenetrationBonus => "Penetration Bonus",
            Self::WaveOverride => "Wave Override",
            Self::LevelOverride => "Level Override",
        }
    }

    fn range(&self) -> SliderRange {
        match self {
            Self::PlayerSpeed | Self::CreatureSpeed | Self::EnemySpeed | Self::SpawnRate | Self::AttackSpeed => SliderRange::SPEED,
            Self::CreatureDamage | Self::EnemyDamage => SliderRange::DAMAGE,
            Self::CritT1 | Self::CritT2 | Self::CritT3 => SliderRange::CRIT,
            Self::ProjectileCount => SliderRange::PROJECTILE_COUNT,
            Self::ProjectileSize | Self::ProjectileSpeed => SliderRange::PROJECTILE_SIZE,
            Self::PenetrationBonus => SliderRange::PENETRATION,
            Self::WaveOverride | Self::LevelOverride => SliderRange::WAVE_LEVEL,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CheckboxSettingId {
    GodMode,
    ShowFps,
    ShowEnemyCount,
    ToggleMode,
}

impl CheckboxSettingId {
    fn label(&self) -> &'static str {
        match self {
            Self::GodMode => "God Mode",
            Self::ShowFps => "Show FPS",
            Self::ShowEnemyCount => "Show Enemy Count",
            Self::ToggleMode => "Toggle Mode (vs Hold)",
        }
    }
}

// =============================================================================
// SPAWN SYSTEMS
// =============================================================================

/// Spawns the debug menu (initially hidden off-screen)
pub fn spawn_debug_menu_system(mut commands: Commands) {
    commands.spawn((
        DebugMenuPanel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-DEBUG_MENU_WIDTH), // Start off-screen
            top: Val::Px(0.0),
            width: Val::Px(DEBUG_MENU_WIDTH),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(DEBUG_MENU_PADDING)),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        ZIndex(100),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Debug Settings"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
        ));

        // Speed multipliers section
        spawn_section_header(parent, "Speed Multipliers");
        spawn_slider(parent, SliderSettingId::PlayerSpeed);
        spawn_slider(parent, SliderSettingId::CreatureSpeed);
        spawn_slider(parent, SliderSettingId::EnemySpeed);

        // Damage multipliers section
        spawn_section_header(parent, "Damage Multipliers");
        spawn_slider(parent, SliderSettingId::CreatureDamage);
        spawn_slider(parent, SliderSettingId::EnemyDamage);

        // Spawn section
        spawn_section_header(parent, "Spawning");
        spawn_slider(parent, SliderSettingId::SpawnRate);

        // Crit section
        spawn_section_header(parent, "Crit Bonuses");
        spawn_slider(parent, SliderSettingId::CritT1);
        spawn_slider(parent, SliderSettingId::CritT2);
        spawn_slider(parent, SliderSettingId::CritT3);

        // Projectile section
        spawn_section_header(parent, "Projectiles");
        spawn_slider(parent, SliderSettingId::ProjectileCount);
        spawn_slider(parent, SliderSettingId::ProjectileSize);
        spawn_slider(parent, SliderSettingId::ProjectileSpeed);
        spawn_slider(parent, SliderSettingId::AttackSpeed);
        spawn_slider(parent, SliderSettingId::PenetrationBonus);

        // Override section
        spawn_section_header(parent, "Overrides");
        spawn_slider(parent, SliderSettingId::WaveOverride);
        spawn_slider(parent, SliderSettingId::LevelOverride);

        // Toggles section
        spawn_section_header(parent, "Toggles");
        spawn_checkbox(parent, CheckboxSettingId::GodMode);
        spawn_checkbox(parent, CheckboxSettingId::ShowFps);
        spawn_checkbox(parent, CheckboxSettingId::ShowEnemyCount);

        // Reset button
        parent.spawn((
            ResetDefaultsButton,
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(BUTTON_HEIGHT),
                margin: UiRect::top(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BUTTON_BG),
        )).with_children(|btn| {
            btn.spawn((
                Text::new("Reset to Defaults"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(TEXT_COLOR),
            ));
        });
    });
}

/// Spawns the pause menu (initially hidden)
pub fn spawn_pause_menu_system(mut commands: Commands) {
    // Overlay
    commands.spawn((
        PauseMenuOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(OVERLAY_COLOR),
        Visibility::Hidden,
        ZIndex(90),
    ));

    // Pause menu panel
    commands.spawn((
        PauseMenuPanel,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(PAUSE_MENU_WIDTH),
            margin: UiRect {
                left: Val::Px(-PAUSE_MENU_WIDTH / 2.0),
                top: Val::Px(-PAUSE_MENU_HEIGHT / 2.0),
                ..default()
            },
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(PANEL_BACKGROUND),
        Visibility::Hidden,
        ZIndex(91),
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("PAUSED"),
            TextFont { font_size: 32.0, ..default() },
            TextColor(TEXT_COLOR),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Resume button
        spawn_pause_button(parent, ResumeButton, "Resume");

        // Toggle mode checkbox
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(BUTTON_HEIGHT),
            margin: UiRect::vertical(Val::Px(10.0)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        }).with_children(|row| {
            row.spawn((
                ToggleModeCheckbox,
                Button,
                Node {
                    width: Val::Px(CHECKBOX_SIZE),
                    height: Val::Px(CHECKBOX_SIZE),
                    margin: UiRect::right(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(CHECKBOX_BG),
            )).with_children(|cb| {
                cb.spawn((
                    CheckboxIndicator { setting_id: CheckboxSettingId::ToggleMode },
                    Node {
                        width: Val::Px(CHECKBOX_SIZE - 6.0),
                        height: Val::Px(CHECKBOX_SIZE - 6.0),
                        ..default()
                    },
                    BackgroundColor(CHECKBOX_CHECKED),
                    Visibility::Inherited, // Will be updated based on setting
                ));
            });
            row.spawn((
                Text::new("Toggle Mode (vs Hold)"),
                TextFont { font_size: 14.0, ..default() },
                TextColor(TEXT_COLOR),
            ));
        });

        // Restart button
        spawn_pause_button(parent, RestartButton, "Restart Run");

        // Quit button
        spawn_pause_button(parent, QuitButton, "Quit Game");
    });
}

fn spawn_section_header(parent: &mut ChildBuilder, text: &str) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(0.6, 0.6, 0.7)),
        Node {
            margin: UiRect {
                top: Val::Px(12.0),
                bottom: Val::Px(6.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn spawn_slider(parent: &mut ChildBuilder, setting_id: SliderSettingId) {
    let range = setting_id.range();

    parent.spawn((
        DebugSlider {
            setting_id,
            min: range.min,
            max: range.max,
            step: range.step,
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(SLIDER_HEIGHT),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
    )).with_children(|row| {
        // Label
        row.spawn((
            Text::new(setting_id.label()),
            TextFont { font_size: 12.0, ..default() },
            TextColor(TEXT_COLOR),
            Node {
                width: Val::Px(SLIDER_LABEL_WIDTH),
                ..default()
            },
        ));

        // Value display
        row.spawn((
            SliderValueText { setting_id },
            Text::new("1.0"),
            TextFont { font_size: 12.0, ..default() },
            TextColor(Color::srgb(0.3, 0.8, 0.4)),
            Node {
                width: Val::Px(SLIDER_VALUE_WIDTH),
                ..default()
            },
        ));

        // Slider bar container
        row.spawn((
            SliderBar { setting_id },
            Button,
            Node {
                width: Val::Px(SLIDER_BAR_WIDTH),
                height: Val::Px(SLIDER_BAR_HEIGHT),
                ..default()
            },
            BackgroundColor(SLIDER_BG),
            RelativeCursorPosition::default(),
        )).with_children(|bar| {
            // Fill
            bar.spawn((
                SliderFill { setting_id },
                Node {
                    width: Val::Percent(50.0), // Will be updated
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(SLIDER_FILL),
            ));
        });
    });
}

fn spawn_checkbox(parent: &mut ChildBuilder, setting_id: CheckboxSettingId) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(SLIDER_HEIGHT),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        margin: UiRect::bottom(Val::Px(4.0)),
        ..default()
    }).with_children(|row| {
        // Checkbox
        row.spawn((
            DebugCheckbox { setting_id },
            Button,
            Node {
                width: Val::Px(CHECKBOX_SIZE),
                height: Val::Px(CHECKBOX_SIZE),
                margin: UiRect::right(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(CHECKBOX_BG),
        )).with_children(|cb| {
            cb.spawn((
                CheckboxIndicator { setting_id },
                Node {
                    width: Val::Px(CHECKBOX_SIZE - 6.0),
                    height: Val::Px(CHECKBOX_SIZE - 6.0),
                    ..default()
                },
                BackgroundColor(CHECKBOX_CHECKED),
                Visibility::Hidden, // Will be updated based on setting
            ));
        });

        // Label
        row.spawn((
            Text::new(setting_id.label()),
            TextFont { font_size: 12.0, ..default() },
            TextColor(TEXT_COLOR),
        ));
    });
}

fn spawn_pause_button<T: Component>(parent: &mut ChildBuilder, marker: T, text: &str) {
    parent.spawn((
        marker,
        Button,
        Node {
            width: Val::Percent(80.0),
            height: Val::Px(40.0),
            margin: UiRect::bottom(Val::Px(10.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BUTTON_BG),
    )).with_children(|btn| {
        btn.spawn((
            Text::new(text),
            TextFont { font_size: 18.0, ..default() },
            TextColor(TEXT_COLOR),
        ));
    });
}

// =============================================================================
// INPUT HANDLING
// =============================================================================

/// Handle debug menu and pause menu input
pub fn debug_menu_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<DebugSettings>,
) {
    // Escape key - toggle pause menu
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match debug_settings.menu_state {
            MenuState::Closed => {
                debug_settings.menu_state = MenuState::PauseMenuOpen;
            }
            MenuState::PauseMenuOpen => {
                debug_settings.menu_state = MenuState::Closed;
            }
            MenuState::DebugMenuOpen => {
                // Close debug menu and open pause menu
                debug_settings.menu_state = MenuState::PauseMenuOpen;
            }
        }
    }

    // Shift key - debug menu (toggle or hold based on setting)
    if debug_settings.menu_toggle_mode {
        // Toggle mode
        if keyboard_input.just_pressed(KeyCode::ShiftLeft) || keyboard_input.just_pressed(KeyCode::ShiftRight) {
            match debug_settings.menu_state {
                MenuState::Closed => {
                    debug_settings.menu_state = MenuState::DebugMenuOpen;
                }
                MenuState::DebugMenuOpen => {
                    debug_settings.menu_state = MenuState::Closed;
                }
                MenuState::PauseMenuOpen => {
                    // Don't toggle debug menu while pause menu is open
                }
            }
        }
    } else {
        // Hold mode
        let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
        if shift_pressed && debug_settings.menu_state == MenuState::Closed {
            debug_settings.menu_state = MenuState::DebugMenuOpen;
        } else if !shift_pressed && debug_settings.menu_state == MenuState::DebugMenuOpen {
            debug_settings.menu_state = MenuState::Closed;
        }
    }
}

// =============================================================================
// MENU ANIMATION
// =============================================================================

/// Animate the debug menu sliding in/out
pub fn debug_menu_animation_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut menu_query: Query<&mut Node, With<DebugMenuPanel>>,
) {
    let target = if debug_settings.menu_state == MenuState::DebugMenuOpen { 0.0 } else { -DEBUG_MENU_WIDTH };

    for mut node in menu_query.iter_mut() {
        if let Val::Px(current) = node.left {
            let new_pos = current + (target - current) * MENU_ANIMATION_SPEED * time.delta_secs();
            // Snap if close enough
            let final_pos = if (new_pos - target).abs() < 1.0 { target } else { new_pos };
            node.left = Val::Px(final_pos);
        }
    }
}

/// Show/hide pause menu
pub fn pause_menu_visibility_system(
    debug_settings: Res<DebugSettings>,
    mut overlay_query: Query<&mut Visibility, (With<PauseMenuOverlay>, Without<PauseMenuPanel>)>,
    mut panel_query: Query<&mut Visibility, (With<PauseMenuPanel>, Without<PauseMenuOverlay>)>,
) {
    let is_visible = debug_settings.menu_state == MenuState::PauseMenuOpen;

    for mut visibility in overlay_query.iter_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
    }
    for mut visibility in panel_query.iter_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
    }
}

// =============================================================================
// SLIDER INTERACTION
// =============================================================================

/// Handle slider bar clicks using RelativeCursorPosition for accurate click detection
pub fn slider_interaction_system(
    mut debug_settings: ResMut<DebugSettings>,
    slider_query: Query<&DebugSlider>,
    bar_query: Query<(&SliderBar, &Interaction, &RelativeCursorPosition)>,
) {
    for (slider_bar, interaction, relative_cursor) in bar_query.iter() {
        // Only process when pressed
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Get the normalized cursor position within the slider bar (0.0 to 1.0)
        let Some(normalized_pos) = relative_cursor.normalized else {
            continue;
        };

        // X position is what we care about (0.0 = left edge, 1.0 = right edge)
        let normalized_x = normalized_pos.x.clamp(0.0, 1.0);

        // Find the slider to get range
        for slider in slider_query.iter() {
            if slider.setting_id == slider_bar.setting_id {
                let range = slider.max - slider.min;
                let raw_value = slider.min + normalized_x * range;
                // Round to step
                let stepped_value = (raw_value / slider.step).round() * slider.step;
                let final_value = stepped_value.clamp(slider.min, slider.max);

                set_slider_value(&mut debug_settings, slider_bar.setting_id, final_value);
                break;
            }
        }
    }
}

/// Update slider visual fill based on current values
pub fn slider_fill_update_system(
    debug_settings: Res<DebugSettings>,
    slider_query: Query<&DebugSlider>,
    mut fill_query: Query<(&SliderFill, &mut Node)>,
) {
    for (fill, mut node) in fill_query.iter_mut() {
        // Find the slider with matching id
        for slider in slider_query.iter() {
            if slider.setting_id == fill.setting_id {
                let current = get_slider_value(&debug_settings, fill.setting_id);
                let percent = ((current - slider.min) / (slider.max - slider.min) * 100.0).clamp(0.0, 100.0);
                node.width = Val::Percent(percent);
                break;
            }
        }
    }
}

/// Update slider value text display
pub fn slider_value_text_system(
    debug_settings: Res<DebugSettings>,
    mut text_query: Query<(&SliderValueText, &mut Text)>,
) {
    for (value_text, mut text) in text_query.iter_mut() {
        let value = get_slider_value(&debug_settings, value_text.setting_id);

        // Format based on setting type
        let formatted = match value_text.setting_id {
            SliderSettingId::WaveOverride | SliderSettingId::LevelOverride => {
                if value <= 0.0 {
                    "Off".to_string()
                } else {
                    format!("{:.0}", value)
                }
            }
            SliderSettingId::CritT1 | SliderSettingId::CritT2 | SliderSettingId::CritT3 => {
                format!("{:.0}%", value)
            }
            SliderSettingId::ProjectileCount => {
                if value >= 0.0 {
                    format!("+{:.0}", value)
                } else {
                    format!("{:.0}", value)
                }
            }
            SliderSettingId::PenetrationBonus => {
                format!("+{:.0}", value)
            }
            _ => format!("{:.1}x", value),
        };

        **text = formatted;
    }
}

// =============================================================================
// CHECKBOX INTERACTION
// =============================================================================

/// Handle checkbox clicks
pub fn checkbox_interaction_system(
    mut debug_settings: ResMut<DebugSettings>,
    checkbox_query: Query<(&DebugCheckbox, &Interaction), Changed<Interaction>>,
) {
    for (checkbox, interaction) in checkbox_query.iter() {
        if *interaction == Interaction::Pressed {
            toggle_checkbox(&mut debug_settings, checkbox.setting_id);
        }
    }
}

/// Update checkbox visual indicator
pub fn checkbox_indicator_system(
    debug_settings: Res<DebugSettings>,
    mut indicator_query: Query<(&CheckboxIndicator, &mut Visibility)>,
) {
    for (indicator, mut visibility) in indicator_query.iter_mut() {
        let is_checked = get_checkbox_value(&debug_settings, indicator.setting_id);
        *visibility = if is_checked { Visibility::Visible } else { Visibility::Hidden };
    }
}

/// Handle toggle mode checkbox in pause menu
pub fn toggle_mode_checkbox_system(
    mut debug_settings: ResMut<DebugSettings>,
    toggle_query: Query<&Interaction, (With<ToggleModeCheckbox>, Changed<Interaction>)>,
) {
    for interaction in toggle_query.iter() {
        if *interaction == Interaction::Pressed {
            debug_settings.menu_toggle_mode = !debug_settings.menu_toggle_mode;
        }
    }
}

// =============================================================================
// BUTTON INTERACTIONS
// =============================================================================

/// Handle reset to defaults button
pub fn reset_button_system(
    mut debug_settings: ResMut<DebugSettings>,
    mut button_query: Query<(&Interaction, &mut BackgroundColor), (With<ResetDefaultsButton>, Changed<Interaction>)>,
) {
    for (interaction, mut bg) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                debug_settings.reset_to_defaults();
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_BG);
            }
        }
    }
}

/// Handle pause menu resume button
pub fn resume_button_system(
    mut debug_settings: ResMut<DebugSettings>,
    mut button_query: Query<(&Interaction, &mut BackgroundColor), (With<ResumeButton>, Changed<Interaction>)>,
) {
    for (interaction, mut bg) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                debug_settings.menu_state = MenuState::Closed;
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_BG);
            }
        }
    }
}

/// Handle pause menu restart button
pub fn restart_button_system(
    mut debug_settings: ResMut<DebugSettings>,
    mut game_state: ResMut<GameState>,
    mut button_query: Query<(&Interaction, &mut BackgroundColor), (With<RestartButton>, Changed<Interaction>)>,
) {
    for (interaction, mut bg) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Reset game state
                *game_state = GameState::default();
                debug_settings.menu_state = MenuState::Closed;
                debug_settings.reset_to_defaults();
                println!("Game restarted!");
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_BG);
            }
        }
    }
}

/// Handle pause menu quit button
pub fn quit_button_system(
    mut app_exit: EventWriter<AppExit>,
    mut button_query: Query<(&Interaction, &mut BackgroundColor), (With<QuitButton>, Changed<Interaction>)>,
) {
    for (interaction, mut bg) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                app_exit.send(AppExit::Success);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg = BackgroundColor(BUTTON_BG);
            }
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn get_slider_value(settings: &DebugSettings, id: SliderSettingId) -> f32 {
    match id {
        SliderSettingId::PlayerSpeed => settings.player_speed_multiplier,
        SliderSettingId::CreatureSpeed => settings.creature_speed_multiplier,
        SliderSettingId::CreatureDamage => settings.creature_damage_multiplier,
        SliderSettingId::EnemyDamage => settings.enemy_damage_multiplier,
        SliderSettingId::EnemySpeed => settings.enemy_speed_multiplier,
        SliderSettingId::SpawnRate => settings.enemy_spawn_rate_multiplier,
        SliderSettingId::CritT1 => settings.crit_t1_bonus,
        SliderSettingId::CritT2 => settings.crit_t2_bonus,
        SliderSettingId::CritT3 => settings.crit_t3_bonus,
        SliderSettingId::ProjectileCount => settings.projectile_count_bonus as f32,
        SliderSettingId::ProjectileSize => settings.projectile_size_multiplier,
        SliderSettingId::ProjectileSpeed => settings.projectile_speed_multiplier,
        SliderSettingId::AttackSpeed => settings.attack_speed_multiplier,
        SliderSettingId::PenetrationBonus => settings.global_penetration_bonus as f32,
        SliderSettingId::WaveOverride => settings.current_wave_override.map(|v| v as f32).unwrap_or(0.0),
        SliderSettingId::LevelOverride => settings.current_level_override.map(|v| v as f32).unwrap_or(0.0),
    }
}

fn set_slider_value(settings: &mut DebugSettings, id: SliderSettingId, value: f32) {
    match id {
        SliderSettingId::PlayerSpeed => settings.player_speed_multiplier = value,
        SliderSettingId::CreatureSpeed => settings.creature_speed_multiplier = value,
        SliderSettingId::CreatureDamage => settings.creature_damage_multiplier = value,
        SliderSettingId::EnemyDamage => settings.enemy_damage_multiplier = value,
        SliderSettingId::EnemySpeed => settings.enemy_speed_multiplier = value,
        SliderSettingId::SpawnRate => settings.enemy_spawn_rate_multiplier = value,
        SliderSettingId::CritT1 => settings.crit_t1_bonus = value,
        SliderSettingId::CritT2 => settings.crit_t2_bonus = value,
        SliderSettingId::CritT3 => settings.crit_t3_bonus = value,
        SliderSettingId::ProjectileCount => settings.projectile_count_bonus = value as i32,
        SliderSettingId::ProjectileSize => settings.projectile_size_multiplier = value,
        SliderSettingId::ProjectileSpeed => settings.projectile_speed_multiplier = value,
        SliderSettingId::AttackSpeed => settings.attack_speed_multiplier = value,
        SliderSettingId::PenetrationBonus => settings.global_penetration_bonus = value as u32,
        SliderSettingId::WaveOverride => {
            settings.current_wave_override = if value < 1.0 { None } else { Some(value as u32) };
        }
        SliderSettingId::LevelOverride => {
            settings.current_level_override = if value < 1.0 { None } else { Some(value as u32) };
        }
    }
}

fn get_checkbox_value(settings: &DebugSettings, id: CheckboxSettingId) -> bool {
    match id {
        CheckboxSettingId::GodMode => settings.god_mode,
        CheckboxSettingId::ShowFps => settings.show_fps,
        CheckboxSettingId::ShowEnemyCount => settings.show_enemy_count,
        CheckboxSettingId::ToggleMode => settings.menu_toggle_mode,
    }
}

fn toggle_checkbox(settings: &mut DebugSettings, id: CheckboxSettingId) {
    match id {
        CheckboxSettingId::GodMode => settings.god_mode = !settings.god_mode,
        CheckboxSettingId::ShowFps => settings.show_fps = !settings.show_fps,
        CheckboxSettingId::ShowEnemyCount => settings.show_enemy_count = !settings.show_enemy_count,
        CheckboxSettingId::ToggleMode => settings.menu_toggle_mode = !settings.menu_toggle_mode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_setting_ids_have_labels() {
        assert!(!SliderSettingId::PlayerSpeed.label().is_empty());
        assert!(!SliderSettingId::CreatureDamage.label().is_empty());
        assert!(!SliderSettingId::WaveOverride.label().is_empty());
    }

    #[test]
    fn checkbox_setting_ids_have_labels() {
        assert!(!CheckboxSettingId::GodMode.label().is_empty());
        assert!(!CheckboxSettingId::ShowFps.label().is_empty());
    }

    #[test]
    fn slider_ranges_are_valid() {
        let speed_range = SliderSettingId::PlayerSpeed.range();
        assert!(speed_range.min < speed_range.max);
        assert!(speed_range.step > 0.0);

        let damage_range = SliderSettingId::CreatureDamage.range();
        assert!(damage_range.min < damage_range.max);
    }

    #[test]
    fn get_set_slider_value_works() {
        let mut settings = DebugSettings::default();

        set_slider_value(&mut settings, SliderSettingId::PlayerSpeed, 2.5);
        assert_eq!(get_slider_value(&settings, SliderSettingId::PlayerSpeed), 2.5);

        set_slider_value(&mut settings, SliderSettingId::WaveOverride, 10.0);
        assert_eq!(settings.current_wave_override, Some(10));

        set_slider_value(&mut settings, SliderSettingId::WaveOverride, 0.0);
        assert_eq!(settings.current_wave_override, None);
    }

    #[test]
    fn checkbox_toggle_works() {
        let mut settings = DebugSettings::default();
        assert!(!settings.god_mode);

        toggle_checkbox(&mut settings, CheckboxSettingId::GodMode);
        assert!(settings.god_mode);

        toggle_checkbox(&mut settings, CheckboxSettingId::GodMode);
        assert!(!settings.god_mode);
    }
}
