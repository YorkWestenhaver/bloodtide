use bevy::prelude::*;
use rand::Rng;

use crate::components::{Creature, Player, WeaponData};
use crate::resources::{
    calculate_next_level_threshold, AffinityState, ArtifactBuffs, CardType, DebugSettings,
    GameData, GameState, PlayerDeck,
};
use crate::systems::{spawn_creature, spawn_weapon, try_weapon_evolution, CardRollState};

// =============================================================================
// CONSTANTS
// =============================================================================

const LEVEL_UP_EFFECT_DURATION: f32 = 0.6;
const LEVEL_UP_RING_START_SIZE: f32 = 80.0;
const LEVEL_UP_RING_END_SCALE: f32 = 5.0;
const MILESTONE_RING_END_SCALE: f32 = 7.0;

const SCREEN_FLASH_DURATION: f32 = 0.1;
const SCREEN_FLASH_OPACITY: f32 = 0.2;

const LEVEL_TEXT_DURATION: f32 = 1.2;
const LEVEL_TEXT_SCALE_UP_TIME: f32 = 0.15;
const LEVEL_TEXT_HOLD_TIME: f32 = 0.8;

const PARTICLE_COUNT: usize = 12;
const PARTICLE_SPEED: f32 = 200.0;
const PARTICLE_DURATION: f32 = 0.5;

// =============================================================================
// COMPONENTS
// =============================================================================

/// Marker component for level up ring visual effect
#[derive(Component)]
pub struct LevelUpEffect {
    pub timer: Timer,
    pub is_milestone: bool,
}

/// Marker component for level up screen flash
#[derive(Component)]
pub struct LevelUpScreenFlash {
    pub timer: Timer,
}

/// Level up text announcement
#[derive(Component)]
pub struct LevelUpText {
    pub timer: Timer,
    pub level: u32,
    pub is_milestone: bool,
}

/// Particle burst from level up
#[derive(Component)]
pub struct LevelUpParticle {
    pub timer: Timer,
    pub velocity: Vec2,
}

/// Resource to queue card rolls for multi-level ups
#[derive(Resource, Default)]
pub struct CardRollQueue {
    /// Queue of card rolls: (card_name, card_type_str, tier, is_milestone)
    pub pending: Vec<PendingCardRoll>,
    /// Timer between card popups
    pub popup_delay_timer: Option<Timer>,
}

#[derive(Clone)]
pub struct PendingCardRoll {
    pub card_name: String,
    pub card_type: String,
    pub tier: u8,
    pub is_milestone: bool,
}

// =============================================================================
// LEVELING SYSTEM
// =============================================================================

/// System that checks if player should level up based on kill count
/// Supports multi-level catchup when kill_count >= kills_for_next_level * 2
pub fn level_check_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut artifact_buffs: ResMut<ArtifactBuffs>,
    mut affinity_state: ResMut<AffinityState>,
    mut card_roll_state: ResMut<CardRollState>,
    mut card_roll_queue: ResMut<CardRollQueue>,
    debug_settings: Res<DebugSettings>,
    player_deck: Res<PlayerDeck>,
    game_data: Res<GameData>,
    player_query: Query<&Transform, With<Player>>,
    creature_query: Query<&Creature>,
    weapon_query: Query<(Entity, &WeaponData)>,
) {
    // Don't process leveling if paused
    if debug_settings.is_paused() {
        return;
    }

    // Count how many level ups we can do
    let mut levels_to_gain = 0u32;
    let mut temp_kills = game_state.kill_count;
    let mut temp_threshold = game_state.kills_for_next_level;
    let multiplier = debug_settings.level_scaling_multiplier;

    // Calculate how many levels we can gain (multi-level catchup)
    while temp_kills >= temp_threshold {
        levels_to_gain += 1;
        temp_kills -= temp_threshold;
        temp_threshold = calculate_next_level_threshold(temp_threshold, multiplier);

        // Safety cap to prevent infinite loops
        if levels_to_gain >= 10 {
            break;
        }
    }

    if levels_to_gain == 0 {
        return;
    }

    // Process all level ups
    for _ in 0..levels_to_gain {
        // Subtract kills used for this level (keep overflow)
        game_state.kill_count = game_state.kill_count.saturating_sub(game_state.kills_for_next_level);

        // Increment level
        game_state.current_level += 1;
        game_state.pending_level_ups += 1;

        // Increase kills needed for next level
        game_state.kills_for_next_level =
            calculate_next_level_threshold(game_state.kills_for_next_level, multiplier);

        // Check if this is a milestone level (every 10 levels)
        let is_milestone = game_state.current_level % 10 == 0;

        // Print SFX placeholder
        if is_milestone {
            println!("SFX_MILESTONE");
            println!(
                "MILESTONE LEVEL {}! Next level at {} kills.",
                game_state.current_level, game_state.kills_for_next_level
            );
        } else {
            println!("SFX_LEVEL_UP");
            println!(
                "LEVEL UP! Now level {}. Next level at {} kills.",
                game_state.current_level, game_state.kills_for_next_level
            );
        }

        // Roll a card from the deck
        // Milestones get guaranteed rare+ (tier 3+)
        if let Some(card) = player_deck.roll_card() {
            let tier = get_card_tier(&game_data, &card);
            let final_tier = if is_milestone && tier < 3 { 3 } else { tier };

            let card_name = get_card_name(&game_data, &card);
            let card_type_str = match card.card_type {
                CardType::Creature => "Creature",
                CardType::Weapon => "Weapon",
                CardType::Artifact => "Artifact",
            };

            println!("Rolled card: {}!", card.id);

            // Queue the card roll for display
            card_roll_queue.pending.push(PendingCardRoll {
                card_name: card_name.clone(),
                card_type: card_type_str.to_string(),
                tier: final_tier,
                is_milestone,
            });

            // Apply the card effect immediately
            match card.card_type {
                CardType::Creature => {
                    if let Ok(player_transform) = player_query.get_single() {
                        let creature_count = creature_query.iter().count();
                        let angle = creature_count as f32 * 0.8;
                        let offset_distance = 80.0;

                        let spawn_pos = Vec3::new(
                            player_transform.translation.x + angle.cos() * offset_distance,
                            player_transform.translation.y + angle.sin() * offset_distance,
                            0.5,
                        );

                        spawn_creature(&mut commands, &game_data, &artifact_buffs, &card.id, spawn_pos);
                    }
                }
                CardType::Weapon => {
                    spawn_weapon(&mut commands, &game_data, &mut affinity_state, &card.id);
                    try_weapon_evolution(&mut commands, &game_data, &mut affinity_state, &weapon_query);
                }
                CardType::Artifact => {
                    artifact_buffs.apply_artifact(&game_data, &card.id);
                }
            }
        }
    }

    // Flag that level up(s) occurred (for effects)
    if levels_to_gain > 0 {
        game_state.level_up_pending = true;
    }
}

/// System that processes the card roll queue with delays
pub fn card_roll_queue_system(
    time: Res<Time>,
    debug_settings: Res<DebugSettings>,
    mut card_roll_queue: ResMut<CardRollQueue>,
    mut card_roll_state: ResMut<CardRollState>,
) {
    if debug_settings.is_paused() {
        return;
    }

    // If we have a delay timer, tick it
    if let Some(ref mut timer) = card_roll_queue.popup_delay_timer {
        timer.tick(time.delta());
        if !timer.finished() {
            return;
        }
        card_roll_queue.popup_delay_timer = None;
    }

    // If there's no pending popup and we have queued cards, show the next one
    if card_roll_state.pending_popup.is_none() && !card_roll_queue.pending.is_empty() {
        let card = card_roll_queue.pending.remove(0);
        card_roll_state.pending_popup = Some((card.card_name, card.card_type, card.tier));

        // If there are more cards, set up a delay timer
        if !card_roll_queue.pending.is_empty() {
            card_roll_queue.popup_delay_timer = Some(Timer::from_seconds(0.5, TimerMode::Once));
        }
    }
}

// =============================================================================
// VISUAL EFFECTS
// =============================================================================

/// System that spawns and manages level up visual effects
pub fn level_up_effect_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    debug_settings: Res<DebugSettings>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut effect_query: Query<
        (Entity, &mut LevelUpEffect, &mut Sprite, &mut Transform),
        Without<Player>,
    >,
) {
    if debug_settings.is_paused() {
        return;
    }

    // Spawn effects when level ups are pending
    while game_state.pending_level_ups > 0 {
        game_state.pending_level_ups -= 1;
        let is_milestone = game_state.current_level % 10 == 0
            && game_state.pending_level_ups == 0; // Only last level up can be milestone

        if let Ok(player_transform) = player_query.get_single() {
            let player_pos = player_transform.translation;

            // Spawn expanding golden ring effect
            let ring_color = if is_milestone {
                Color::srgba(1.0, 0.85, 0.0, 0.9) // Brighter gold for milestone
            } else {
                Color::srgba(1.0, 0.9, 0.2, 0.8) // Yellow/gold
            };

            commands.spawn((
                LevelUpEffect {
                    timer: Timer::from_seconds(LEVEL_UP_EFFECT_DURATION, TimerMode::Once),
                    is_milestone,
                },
                Sprite {
                    color: ring_color,
                    custom_size: Some(Vec2::new(LEVEL_UP_RING_START_SIZE, LEVEL_UP_RING_START_SIZE)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(player_pos.x, player_pos.y, 0.8)),
            ));

            // Spawn screen flash
            commands.spawn((
                LevelUpScreenFlash {
                    timer: Timer::from_seconds(SCREEN_FLASH_DURATION, TimerMode::Once),
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, SCREEN_FLASH_OPACITY)),
                ZIndex(50),
            ));

            // Spawn level up text
            commands.spawn((
                LevelUpText {
                    timer: Timer::from_seconds(LEVEL_TEXT_DURATION, TimerMode::Once),
                    level: game_state.current_level,
                    is_milestone,
                },
                Text2d::new("LEVEL UP"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(if is_milestone {
                    Color::srgb(1.0, 0.85, 0.0) // Gold
                } else {
                    Color::srgb(1.0, 0.95, 0.4) // Yellow
                }),
                Transform::from_translation(Vec3::new(0.0, 100.0, 15.0)).with_scale(Vec3::ZERO),
            ));

            // Spawn particle burst
            let particle_count = if is_milestone { PARTICLE_COUNT * 2 } else { PARTICLE_COUNT };
            let mut rng = rand::thread_rng();
            for i in 0..particle_count {
                let angle = (i as f32 / particle_count as f32) * std::f32::consts::TAU;
                let speed_variance = 1.0 + rng.gen::<f32>() * 0.5;
                let velocity = Vec2::new(angle.cos(), angle.sin()) * PARTICLE_SPEED * speed_variance;

                let particle_color = if is_milestone {
                    Color::srgba(1.0, 0.85, 0.0, 1.0)
                } else {
                    Color::srgba(1.0, 0.95, 0.4, 1.0)
                };

                commands.spawn((
                    LevelUpParticle {
                        timer: Timer::from_seconds(PARTICLE_DURATION, TimerMode::Once),
                        velocity,
                    },
                    Sprite {
                        color: particle_color,
                        custom_size: Some(Vec2::new(6.0, 6.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(player_pos.x, player_pos.y, 0.85)),
                ));
            }
        }

        game_state.level_up_pending = false;
    }

    // Update existing ring effects
    for (entity, mut effect, mut sprite, mut transform) in effect_query.iter_mut() {
        effect.timer.tick(time.delta());

        let progress = effect.timer.fraction();
        let end_scale = if effect.is_milestone {
            MILESTONE_RING_END_SCALE
        } else {
            LEVEL_UP_RING_END_SCALE
        };
        let scale = 1.0 + progress * (end_scale - 1.0);
        transform.scale = Vec3::splat(scale);

        // Fade out
        let base_alpha = if effect.is_milestone { 0.9 } else { 0.8 };
        let alpha = (1.0 - progress) * base_alpha;
        let base_color = if effect.is_milestone {
            Color::srgba(1.0, 0.85, 0.0, alpha)
        } else {
            Color::srgba(1.0, 0.9, 0.2, alpha)
        };
        sprite.color = base_color;

        if effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// System that updates screen flash effects
pub fn screen_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LevelUpScreenFlash, &mut BackgroundColor)>,
) {
    for (entity, mut flash, mut bg_color) in query.iter_mut() {
        flash.timer.tick(time.delta());

        // Fade out
        let progress = flash.timer.fraction();
        let alpha = SCREEN_FLASH_OPACITY * (1.0 - progress);
        *bg_color = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, alpha));

        if flash.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// System that updates level up text
pub fn level_up_text_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LevelUpText, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut text_effect, mut transform, mut text_color) in query.iter_mut() {
        text_effect.timer.tick(time.delta());

        let elapsed = text_effect.timer.elapsed_secs();
        let total = text_effect.timer.duration().as_secs_f32();

        // Scale up animation
        if elapsed < LEVEL_TEXT_SCALE_UP_TIME {
            let scale_progress = elapsed / LEVEL_TEXT_SCALE_UP_TIME;
            // Ease out
            let scale = scale_progress * (2.0 - scale_progress);
            transform.scale = Vec3::splat(scale);
        } else if elapsed < LEVEL_TEXT_SCALE_UP_TIME + LEVEL_TEXT_HOLD_TIME {
            // Hold at full scale
            transform.scale = Vec3::ONE;
        } else {
            // Fade out
            let fade_start = LEVEL_TEXT_SCALE_UP_TIME + LEVEL_TEXT_HOLD_TIME;
            let fade_duration = total - fade_start;
            let fade_progress = (elapsed - fade_start) / fade_duration;
            let alpha = 1.0 - fade_progress;

            let base_color = if text_effect.is_milestone {
                Color::srgba(1.0, 0.85, 0.0, alpha)
            } else {
                Color::srgba(1.0, 0.95, 0.4, alpha)
            };
            *text_color = TextColor(base_color);
        }

        if text_effect.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// System that updates level up particles
pub fn level_up_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LevelUpParticle, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut particle, mut transform, mut sprite) in query.iter_mut() {
        particle.timer.tick(time.delta());

        // Move particle
        let dt = time.delta_secs();
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Slow down
        particle.velocity *= 0.95;

        // Fade and shrink
        let progress = particle.timer.fraction();
        let alpha = 1.0 - progress;
        let size = 6.0 * (1.0 - progress * 0.5);

        let Srgba { red, green, blue, .. } = sprite.color.to_srgba();
        sprite.color = Color::srgba(red, green, blue, alpha);
        sprite.custom_size = Some(Vec2::new(size, size));

        if particle.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn get_card_name(game_data: &GameData, card: &crate::resources::DeckCard) -> String {
    match card.card_type {
        CardType::Creature => game_data
            .creatures
            .iter()
            .find(|c| c.id == card.id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| card.id.clone()),
        CardType::Weapon => game_data
            .weapons
            .iter()
            .find(|w| w.id == card.id)
            .map(|w| w.name.clone())
            .unwrap_or_else(|| card.id.clone()),
        CardType::Artifact => game_data
            .artifacts
            .iter()
            .find(|a| a.id == card.id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| card.id.clone()),
    }
}

fn get_card_tier(game_data: &GameData, card: &crate::resources::DeckCard) -> u8 {
    match card.card_type {
        CardType::Creature => game_data
            .creatures
            .iter()
            .find(|c| c.id == card.id)
            .map(|c| c.tier)
            .unwrap_or(1),
        CardType::Weapon => game_data
            .weapons
            .iter()
            .find(|w| w.id == card.id)
            .map(|w| w.tier)
            .unwrap_or(1),
        CardType::Artifact => game_data
            .artifacts
            .iter()
            .find(|a| a.id == card.id)
            .map(|a| a.tier)
            .unwrap_or(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_up_effect_has_timer() {
        let effect = LevelUpEffect {
            timer: Timer::from_seconds(LEVEL_UP_EFFECT_DURATION, TimerMode::Once),
            is_milestone: false,
        };
        assert_eq!(effect.timer.duration().as_secs_f32(), LEVEL_UP_EFFECT_DURATION);
    }

    #[test]
    fn card_roll_queue_default() {
        let queue = CardRollQueue::default();
        assert!(queue.pending.is_empty());
        assert!(queue.popup_delay_timer.is_none());
    }

    #[test]
    fn pending_card_roll_clone() {
        let roll = PendingCardRoll {
            card_name: "Test".to_string(),
            card_type: "Creature".to_string(),
            tier: 1,
            is_milestone: false,
        };
        let cloned = roll.clone();
        assert_eq!(cloned.card_name, "Test");
        assert_eq!(cloned.tier, 1);
    }
}
