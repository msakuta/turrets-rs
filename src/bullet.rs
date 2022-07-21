use std::collections::VecDeque;

use crate::{
    sprite_transform_single,
    tower::{MissileTower, Shotgun, Tower, TowerScore},
    BulletFilter, BulletShooter, Explosion, Health, Position, Rotation, Scoreboard, StageClear,
    Target, Textures, Velocity,
};
use bevy::{prelude::*, sprite::collide_aabb::collide};
use bevy_prototype_lyon::prelude::*;

const ENEMY_SIZE: f32 = 20.;
const BULLET_SIZE: f32 = 20.;

pub(crate) const SHOOT_INTERVAL: f32 = 0.5;
const SHOTGUN_SHOOT_INTERVAL: f32 = 1.5;
const BULLET_SPEED: f32 = 500.;
const MISSILE_SPEED: f32 = 300.;

pub(crate) struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin);
        app.add_system(shoot_bullet);
        app.add_system(bullet_collision);
        app.add_system(missile_system);
        app.add_system(cleanup::<Bullet>);
    }
}

#[derive(Component)]
pub(crate) struct Bullet {
    filter: bool,
    owner: Entity,
}

#[derive(Component)]
pub(crate) struct Missile {
    target: Entity,
    trail: Entity,
    trail_nodes: VecDeque<Vec2>,
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
    target_query: Query<&Position>,
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

                    let trail = target
                        .and_then(|target| target_query.get_component::<Position>(target).ok())
                        .map(|target_position| {
                            // Build empty path, which we will replace later
                            let mut path_builder = PathBuilder::new();
                            path_builder.move_to(position.0);
                            let line = path_builder.build();

                            commands
                                .spawn_bundle(GeometryBuilder::build_as(
                                    &line,
                                    DrawMode::Stroke(StrokeMode::new(
                                        Color::rgba(0.8, 0.8, 0.7, 0.5),
                                        3.0,
                                    )),
                                    Transform::default(),
                                ))
                                .insert(StageClear)
                                .id()
                        });

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
                    let mut trail_nodes = VecDeque::new();
                    trail_nodes.push_back(position.0);
                    if let Some((target, trail)) = target.zip(trail) {
                        builder.insert(Missile {
                            target,
                            trail,
                            trail_nodes,
                        });
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
                        for i in 0..2 {
                            shoot(
                                "missile.png",
                                rotation.0,
                                MISSILE_SPEED,
                                i as f32 * 20. - 10.,
                                Some(target),
                            );
                        }
                        bullet_shooter.1 += SHOTGUN_SHOOT_INTERVAL;
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
    mut scoring_tower: Query<&mut TowerScore>,
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
    missile: Option<&Missile>,
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
            .insert(Explosion(Timer::from_seconds(0.06, true)))
            .insert(StageClear);
    }
}

const MISSILE_ROTATION_SPEED: f32 = std::f32::consts::PI * 0.5;

fn missile_system(
    time: Res<Time>,
    mut query: Query<(&mut Missile, &mut Rotation, &Position, &mut Velocity)>,
    health_query: Query<&Health>,
    target_query: Query<(Entity, &Position, &BulletFilter)>,
    mut trail_query: Query<&mut Path>,
) {
    for (mut missile, mut rotation, position, mut velocity) in query.iter_mut() {
        // Search for target if already have none
        if health_query
            .get_component::<Health>(missile.target)
            .map(|health| health.val <= 0.)
            .unwrap_or(true)
        {
            if let Some((_, nearest)) =
                target_query
                    .iter()
                    .fold(None, |acc: Option<(f32, Entity)>, cur| {
                        let cur_distance = cur.1 .0.distance(position.0);
                        if acc.map(|acc| cur_distance < acc.0).unwrap_or(true) {
                            Some((cur_distance, cur.0))
                        } else {
                            acc
                        }
                    })
            {
                missile.target = nearest;
            }
        }

        // Guide toward target
        if health_query
            .get_component::<Health>(missile.target)
            .map(|health| 0. < health.val)
            .unwrap_or(false)
        {
            // (this.target !== null && 0 < this.target.health){
            let target_position =
                if let Ok(position) = target_query.get_component::<Position>(missile.target) {
                    position
                } else {
                    continue;
                };
            let delta = target_position.0 - position.0;
            let angle = rapproach(
                rotation.0 as f32,
                delta.y.atan2(delta.x),
                MISSILE_ROTATION_SPEED * time.delta_seconds(),
            );
            rotation.0 = angle as f64;
            velocity.0.x = MISSILE_SPEED * angle.cos();
            velocity.0.y = MISSILE_SPEED * angle.sin();
        }

        const MAX_NODES: usize = 50;
        if MAX_NODES < missile.trail_nodes.len() {
            for _ in 0..(missile.trail_nodes.len() - MAX_NODES) {
                missile.trail_nodes.pop_front();
            }
        }

        if missile
            .trail_nodes
            .back()
            .map(|back| 10. < back.distance(position.0))
            .unwrap_or(false)
        {
            missile.trail_nodes.push_back(position.0);
        }

        if let Ok(mut trail) = trail_query.get_component_mut::<Path>(missile.trail) {
            let mut iter = missile.trail_nodes.iter();
            if let Some(first) = iter.next() {
                let mut trail_builder = PathBuilder::new();
                trail_builder.move_to(*first);
                for node in iter {
                    trail_builder.line_to(*node);
                }
                *trail = trail_builder.build();
            }
        }
    }
}

/// Rotation approach
fn rapproach(src: f32, dst: f32, delta: f32) -> f32 {
    return approach(
        src + std::f32::consts::PI,
        dst + std::f32::consts::PI,
        delta,
        std::f32::consts::PI * 2.,
    ) - std::f32::consts::PI;
}

/// Approach src to dst by delta, optionally wrapping around wrap
fn approach(src: f32, dst: f32, delta: f32, wrap: f32) -> f32 {
    if src < dst {
        if dst - src < delta {
            return dst;
        } else if wrap != 0. && wrap / 2. < dst - src {
            let ret = src - delta - ((src - delta) / wrap).floor() * wrap/*fmod(src - delta + wrap, wrap)*/;
            return if src < ret && ret < dst { dst } else { ret };
        }
        return src + delta;
    } else {
        if src - dst < delta {
            return dst;
        } else if wrap != 0. && wrap / 2. < src - dst {
            let ret = src + delta - ((src + delta) / wrap).floor() * wrap/*fmod(src + delta, wrap)*/;
            return if ret < src && dst < ret { dst } else { ret };
        }
        return src - delta;
    }
}

fn cleanup<T: Component>(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Position, Option<&Missile>), With<T>>,
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
