use bevy::prelude::*;

use crate::components::Creature;
use crate::resources::{ArtifactBuffs, Director, GameState};

/// Marker component for the main HUD text
#[derive(Component)]
pub struct HudText;

/// System that spawns the UI on startup
pub fn spawn_ui_system(mut commands: Commands) {
    // Spawn HUD container
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                HudText,
                Text::new("Level: 1 | Kills: 0 | Wave: 1 | Creatures: 0 | Enemies: 0 | FPS: 60"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    println!("UI spawned");
}

/// System that updates the HUD text with current game state
pub fn update_ui_system(
    game_state: Res<GameState>,
    artifact_buffs: Res<ArtifactBuffs>,
    director: Res<Director>,
    creature_query: Query<&Creature>,
    mut query: Query<&mut Text, With<HudText>>,
) {
    let creature_count = creature_query.iter().count();
    let artifact_count = artifact_buffs.acquired_artifacts.len();

    // Color FPS based on performance
    let fps_text = if director.current_fps < 30.0 {
        format!("FPS: {:.0}!", director.current_fps)  // Add ! for warning
    } else {
        format!("FPS: {:.0}", director.current_fps)
    };

    for mut text in query.iter_mut() {
        **text = format!(
            "Lv:{} | K:{} | W:{} | C:{} | E:{} | {}",
            game_state.current_level,
            game_state.total_kills,
            game_state.current_wave,
            creature_count,
            director.enemies_alive,
            fps_text
        );
    }
}
