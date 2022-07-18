use crate::{
    sprite_transform_single,
    tower::{Shotgun, Tower, TowerScore},
    Bullet, BulletFilter, BulletShooter, Explosion, Health, Position, Rotation, Scoreboard,
    StageClear, Textures, Velocity,
};
use bevy::{prelude::*, sprite::collide_aabb::collide};

const ENEMY_SIZE: f32 = 20.;
const BULLET_SIZE: f32 = 20.;

pub(crate) const SHOOT_INTERVAL: f32 = 0.5;
const SHOTGUN_SHOOT_INTERVAL: f32 = 1.5;
const BULLET_SPEED: f32 = 500.;

pub(crate) fn shoot_bullet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &Position,
        Option<&Rotation>,
        &mut BulletShooter,
        Option<&Shotgun>,
    )>,
) {
    let delta = time.delta_seconds();
    for (entity, position, rotation, mut bullet_shooter, shotgun) in query.iter_mut() {
        if !bullet_shooter.0 {
            continue;
        }
        if bullet_shooter.1 < delta {
            let mut shoot = |file, angle: f64| {
                let bullet_rotation = Rotation(angle);
                let mut transform = default();
                sprite_transform_single(position, Some(&bullet_rotation), &mut transform, 0.);
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load(file),
                        transform,
                        ..default()
                    })
                    .insert(*position)
                    .insert(bullet_rotation)
                    .insert(Velocity(
                        BULLET_SPEED * Vec2::new(angle.cos() as f32, angle.sin() as f32),
                    ))
                    .insert(Bullet {
                        filter: rotation.is_some(),
                        owner: entity,
                    })
                    .insert(StageClear);
            };

            if let Some(rotation) = rotation {
                if shotgun.is_some() {
                    for i in -3..=3 {
                        shoot(
                            "shotgun-bullet.png",
                            rotation.0 + i as f64 * std::f64::consts::PI / 20.,
                        );
                    }
                    bullet_shooter.1 += SHOTGUN_SHOOT_INTERVAL;
                } else {
                    shoot("bullet.png", rotation.0);
                    bullet_shooter.1 += SHOOT_INTERVAL;
                }
            } else {
                shoot(
                    "enemy-bullet.png",
                    rand::random::<f64>() * std::f64::consts::PI * 2.,
                );
                bullet_shooter.1 += SHOOT_INTERVAL * rand::random::<f32>();
            }
        }
        bullet_shooter.1 -= delta;
    }
}

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
    mut scoring_tower: Query<&mut TowerScore>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (entity, transform, health, bullet_filter, tower) in target_query.iter_mut() {
            if bullet.filter == bullet_filter.0 {
                entity_collision(
                    &mut commands,
                    bullet_entity,
                    bullet,
                    bullet_transform,
                    entity,
                    transform,
                    tower,
                    &mut scoring_tower,
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
    bullet: &Bullet,
    bullet_transform: &Transform,
    entity: Entity,
    transform: &Transform,
    tower: Option<&Tower>,
    scoring_tower: &mut Query<&mut TowerScore>,
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
                .insert(Explosion(Timer::from_seconds(0.15, true)))
                .insert(StageClear);
            scoreboard.score += 10.;

            if let Ok(mut scoring_tower) =
                scoring_tower.get_component_mut::<TowerScore>(bullet.owner)
            {
                scoring_tower.kills += 1;
            }
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
