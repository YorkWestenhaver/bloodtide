use bevy::prelude::*;
use bevy::sprite::TextureAtlas;
use rand::Rng;

use crate::components::{BloodSplatter, DeathAnimation, Player};
use crate::resources::DeathSprites;

/// System that updates death animations, advancing frames and spawning blood on completion
/// Death animation plays frames 3→4→5 at 120ms each
pub fn death_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    death_sprites: Res<DeathSprites>,
    mut query: Query<(Entity, &mut DeathAnimation, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in query.iter_mut() {
        anim.timer.tick(time.delta());
        anim.frame_timer.tick(time.delta());

        // Advance frame when frame timer fires (frames 3→4→5)
        if anim.frame_timer.just_finished() && anim.current_frame < 5 {
            anim.current_frame += 1;
            // Update the texture atlas index embedded in the sprite
            if let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = anim.current_frame;
            }
        }

        // Fade out slightly in last frame
        let remaining = anim.timer.fraction_remaining();
        if remaining < 0.25 {
            sprite.color = sprite.color.with_alpha(remaining * 4.0);
        }

        // Animation complete - spawn blood splatters and despawn animation entity
        if anim.timer.finished() {
            let mut rng = rand::thread_rng();

            // Spawn 3-5 blood splatters with random offsets
            let splatter_count = rng.gen_range(3..=5);
            for _ in 0..splatter_count {
                let variant = rng.gen_range(0..4);
                // Random offset ±30 pixels
                let offset_x = rng.gen_range(-30.0..=30.0);
                let offset_y = rng.gen_range(-30.0..=30.0);

                commands.spawn((
                    BloodSplatter::new(variant),
                    Sprite::from_atlas_image(
                        death_sprites.blood_splatters.clone(),
                        TextureAtlas {
                            layout: death_sprites.blood_atlas.clone(),
                            index: variant,
                        },
                    ),
                    Transform::from_translation(Vec3::new(
                        anim.death_position.x + offset_x,
                        anim.death_position.y + offset_y,
                        -1.0, // Z=-1: Behind everything including background grid
                    )),
                ));
            }

            commands.entity(entity).despawn();
        }
    }
}

/// System that cleans up blood splatters over time and by distance
/// Blood has 30 second lifetime and starts fading at 50% remaining (15 seconds)
pub fn blood_cleanup_system(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut blood_query: Query<(Entity, &mut BloodSplatter, &mut Sprite, &Transform)>,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (entity, mut blood, mut sprite, transform) in blood_query.iter_mut() {
        blood.lifetime.tick(time.delta());

        // Distance-based cleanup (same as enemy despawn distance)
        let distance = player_pos.distance(transform.translation.truncate());
        if distance > 2500.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Fade out in last 15 seconds (50% of 30 second lifetime)
        let remaining = blood.lifetime.fraction_remaining();
        if remaining < 0.5 {
            let alpha = remaining / 0.5; // Fade from 1.0 to 0.0 over 15 seconds
            sprite.color = sprite.color.with_alpha(alpha);
        }

        if blood.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
