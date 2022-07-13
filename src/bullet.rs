use crate::{tower::Tower, Bullet, BulletFilter, Explosion, Health, Scoreboard, Textures};
use bevy::{prelude::*, sprite::collide_aabb::collide};

const ENEMY_SIZE: f32 = 20.;
const BULLET_SIZE: f32 = 20.;

pub(crate) fn bullet_collision(
    mut commands: Commands,
    mut target_query: Query<(
        Entity,
        &Transform,
        &mut Health,
        &BulletFilter,
        Option<&Tower>,
    )>,
    bullet_query: Query<(Entity, &Transform, &Bullet)>,
    textures: Res<Textures>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (entity, transform, health, bullet_filter, tower) in target_query.iter_mut() {
            if bullet.0 == bullet_filter.0 {
                entity_collision(
                    &mut commands,
                    bullet_entity,
                    bullet_transform,
                    entity,
                    transform,
                    tower,
                    health,
                    &textures,
                    &mut scoreboard,
                );
            }
        }
    }
}

fn entity_collision(
    commands: &mut Commands,
    bullet_entity: Entity,
    bullet_transform: &Transform,
    entity: Entity,
    transform: &Transform,
    tower: Option<&Tower>,
    mut health: Mut<Health>,
    textures: &Res<Textures>,
    scoreboard: &mut ResMut<Scoreboard>,
) {
    let collision = collide(
        bullet_transform.translation,
        Vec2::new(BULLET_SIZE, BULLET_SIZE),
        transform.translation,
        Vec2::new(ENEMY_SIZE, ENEMY_SIZE),
    );

    if collision.is_some() {
        commands.entity(bullet_entity).despawn();
        if health.val < 1. {
            commands.entity(entity).despawn();
            if let Some(tower) = tower {
                commands.entity(tower.health_bar.0).despawn();
                commands.entity(tower.health_bar.1).despawn();
            }
            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: textures.large_explosion.clone(),
                    transform: bullet_transform.clone().with_scale(Vec3::splat(4.0)),
                    ..default()
                })
                .insert(Explosion(Timer::from_seconds(0.15, true)));
            scoreboard.score += 10.;
        } else {
            health.val -= 1.;
        }

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures.small_explosion.clone(),
                transform: bullet_transform.clone().with_scale(Vec3::splat(3.0)),
                ..default()
            })
            .insert(Explosion(Timer::from_seconds(0.06, true)));
    }
}
