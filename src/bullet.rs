mod missile;

use self::missile::{missile_system, Missile, MISSILE_SPEED};
use crate::{
    mouse::tower_not_dragging,
    sprite_transform_single,
    tower::{MissileTower, Shotgun, Tower},
    BulletFilter, BulletShooter, Explosion, Health, Position, Rotation, Scoreboard, StageClear,
    Target, Textures, Velocity,
};
use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_prototype_lyon::prelude::*;

const ENEMY_SIZE: f32 = 20.;
const BULLET_SIZE: f32 = 20.;

pub(crate) const SHOOT_INTERVAL: f32 = 0.5;
const SHOTGUN_SHOOT_INTERVAL: f32 = 1.5;
const MISSILE_SHOOT_INTERVAL: f32 = 2.5;
const BULLET_SPEED: f32 = 500.;

pub(crate) struct KilledEvent {
    pub entity: Entity,
    pub exp: usize,
}

pub(crate) struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin);
        app.add_event::<KilledEvent>();
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(tower_not_dragging)
                .with_system(shoot_bullet)
                .with_system(bullet_collision)
                .with_system(missile_system),
        );
        app.add_system(cleanup);
    }
}

#[derive(Component)]
pub(crate) struct Bullet {
    filter: bool,
    owner: Entity,
}

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
        Option<&MissileTower>,
        Option<&Target>,
    )>,
) {
    let delta = time.delta_seconds();
    for (entity, position, rotation, mut bullet_shooter, shotgun, missile_tower, target) in
        query.iter_mut()
    {
        if !bullet_shooter.0 {
            continue;
        }
        if bullet_shooter.1 < delta {
            let mut shoot =
                |file, angle: f64, speed: f32, horz_offset: f32, target: Option<Entity>| {
                    let bullet_rotation = Rotation(angle);
                    let mut transform = default();
                    let position = Position(
                        position.0
                            + Vec2::new(
                                angle.sin() as f32 * horz_offset,
                                -angle.cos() as f32 * horz_offset,
                            ),
                    );

                    let trail = missile_tower.map(|_| missile::gen_trail(&mut commands, &position));

                    sprite_transform_single(&position, Some(&bullet_rotation), &mut transform, 0.);
                    let mut builder = commands.spawn_bundle(SpriteBundle {
                        texture: asset_server.load(file),
                        transform,
                        ..default()
                    });
                    builder.insert(position);
                    builder.insert(bullet_rotation);
                    builder.insert(Velocity(
                        speed * Vec2::new(angle.cos() as f32, angle.sin() as f32),
                    ));
                    builder.insert(Bullet {
                        filter: rotation.is_some(),
                        owner: entity,
                    });
                    builder.insert(StageClear);
                    if let Some((target, trail)) = target.zip(trail) {
                        builder.insert(Missile::new(target, trail, &position));
                    }
                };

            if let Some(rotation) = rotation {
                if shotgun.is_some() {
                    for i in -3..=3 {
                        shoot(
                            "shotgun-bullet.png",
                            rotation.0 + i as f64 * std::f64::consts::PI / 20.,
                            BULLET_SPEED,
                            0.,
                            None,
                        );
                    }
                    bullet_shooter.1 += SHOTGUN_SHOOT_INTERVAL;
                } else if missile_tower.is_some() {
                    if let Some(target) = target.and_then(|target| target.0) {
                        for i in -2..=2 {
                            if i == 0 {
                                continue;
                            }
                            shoot(
                                "missile.png",
                                rotation.0 - i as f64 * std::f64::consts::PI * 0.05,
                                MISSILE_SPEED,
                                i as f32 * 20.,
                                Some(target),
                            );
                        }
                        bullet_shooter.1 += MISSILE_SHOOT_INTERVAL;
                    }
                } else {
                    shoot("bullet.png", rotation.0, BULLET_SPEED, 0., None);
                    bullet_shooter.1 += SHOOT_INTERVAL;
                }
            } else {
                shoot(
                    "enemy-bullet.png",
                    rand::random::<f64>() * std::f64::consts::PI * 2.,
                    BULLET_SPEED,
                    0.,
                    None,
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
    bullet_query: Query<(Entity, &Transform, &Bullet, Option<&Missile>)>,
    textures: Res<Textures>,
    mut scoreboard: ResMut<Scoreboard>,
    mut event_writer: EventWriter<KilledEvent>,
    // mut scoring_tower: Query<(&mut TowerLevel, &mut Health, &mut TowerScore)>,
) {
    for (bullet_entity, bullet_transform, bullet, missile) in bullet_query.iter() {
        for (entity, transform, health, bullet_filter, tower) in target_query.iter_mut() {
            if bullet.filter == bullet_filter.0 {
                entity_collision(
                    &mut commands,
                    bullet_entity,
                    bullet,
                    bullet_transform,
                    missile,
                    entity,
                    transform,
                    tower,
                    &mut event_writer,
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
    missile: Option<&Missile>,
    entity: Entity,
    transform: &Transform,
    tower: Option<&Tower>,
    event_writer: &mut EventWriter<KilledEvent>,
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
        if let Some(missile) = missile {
            commands.entity(missile.trail).despawn();
        }
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
            scoreboard.credits += 10.;

            event_writer.send(KilledEvent {
                entity: bullet.owner,
                exp: 10,
            });
        } else {
            health.val -= 1.;
        }

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures.small_explosion.clone(),
                transform: bullet_transform.clone().with_scale(Vec3::splat(3.0)),
                ..default()
            })
            .insert(Explosion(Timer::from_seconds(0.06, true)))
            .insert(StageClear);
    }
}

fn cleanup(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Position, Option<&Missile>), (With<Bullet>, Without<Missile>)>,
) {
    let window = if let Some(window) = windows.iter().next() {
        window
    } else {
        return;
    };
    let (width, height) = (window.width(), window.height());
    for (entity, position, missile) in query.iter() {
        if position.0.x < -width / 2.
            || width / 2. < position.0.x
            || position.0.y < -height / 2.
            || height / 2. < position.0.y
        {
            commands.entity(entity).despawn();
            if let Some(missile) = missile {
                commands.entity(missile.trail).despawn();
            }
            // println!("Despawned {entity:?} ({})", std::any::type_name::<T>());
        }
    }
}
