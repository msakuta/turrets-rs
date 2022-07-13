use crate::{
    BulletFilter, BulletShooter, Enemy, Health, Position, Rotation, Target, Velocity,
    SHOOT_INTERVAL,
};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Tower {
    pub health_bar: (Entity, Entity),
}

#[derive(Component)]
pub(crate) struct Shotgun;

#[derive(Component)]
pub(crate) struct Healer(bool, f32);

impl Healer {
    pub(crate) fn new() -> Self {
        Self(false, 2.)
    }
}

#[derive(Component)]
pub(crate) struct Timeout(f32);

#[derive(Bundle)]
pub(crate) struct TowerBundle {
    position: Position,
    rotation: Rotation,
    tower: Tower,
    health: Health,
    target: Target,
    bullet_filter: BulletFilter,
}

impl TowerBundle {
    pub(crate) fn new(
        commands: &mut Commands,
        position: Position,
        rotation: Rotation,
        health: Health,
    ) -> Self {
        Self {
            position,
            rotation,
            tower: Tower {
                health_bar: health_bar(commands),
            },
            health,
            target: Target(None),
            bullet_filter: BulletFilter(false),
        }
    }
}

impl BulletShooter {
    pub(crate) fn new() -> Self {
        BulletShooter(false, rand::random::<f32>() * SHOOT_INTERVAL)
    }
}

pub(crate) struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_health_bar)
            .add_system(tower_find_target)
            .add_system(healer_find_target)
            .add_system(heal_target)
            .add_system(timeout);
    }
}

const HEALTH_BAR_WIDTH: f32 = 80.;

fn health_bar(commands: &mut Commands) -> (Entity, Entity) {
    (
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 1., 0.25),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, 10.0)),
                    ..default()
                },
                ..default()
            })
            .id(),
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0., 0.),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, 10.0)),
                    ..default()
                },
                ..default()
            })
            .id(),
    )
}

pub(crate) fn update_health_bar(
    query: Query<(&Position, &Tower, &Health)>,
    mut query_health_bar: Query<&mut Transform>,
) {
    for (position, tower, health) in query.iter() {
        if let Ok(mut bar) = query_health_bar.get_mut(tower.health_bar.0) {
            let factor = health.val / health.max;
            *bar = Transform::from_xyz(
                position.0.x - (1. - factor) * HEALTH_BAR_WIDTH / 2.,
                position.0.y + 50.,
                0.7,
            )
            .with_scale(Vec3::new(factor, 1., 1.));
        }
        if let Ok(mut bar) = query_health_bar.get_mut(tower.health_bar.1) {
            *bar = Transform::from_xyz(position.0.x, position.0.y + 50., 0.5);
        }
    }
}

fn tower_find_target(
    mut query: Query<(&mut Rotation, &Position, &mut BulletShooter, &mut Target), With<Tower>>,
    enemy_query: Query<(Entity, &Position), With<Enemy>>,
) {
    for (mut rotation, position, mut bullet_shooter, mut target) in query.iter_mut() {
        let new_target = enemy_query
            .iter()
            .fold(None, |acc, (enemy_entity, enemy_position)| {
                let this_dist = enemy_position.0.distance(position.0);
                if let Some((prev_dist, _, _)) = acc {
                    if this_dist < prev_dist {
                        Some((this_dist, enemy_entity, enemy_position))
                    } else {
                        acc
                    }
                } else {
                    Some((this_dist, enemy_entity, enemy_position))
                }
            });

        use std::f64::consts::PI;
        const TWOPI: f64 = PI * 2.;
        const ANGLE_SPEED: f64 = PI / 50.;

        if let Some((_, new_target, enemy_position)) = new_target {
            target.0 = Some(new_target);

            let delta = enemy_position.0 - position.0;
            let target_angle = delta.y.atan2(delta.x) as f64;
            let delta_angle = target_angle - rotation.0;
            let wrap_angle =
                ((delta_angle + PI) - ((delta_angle + PI) / TWOPI).floor() * TWOPI) - PI;
            bullet_shooter.0 = if wrap_angle.abs() < ANGLE_SPEED {
                rotation.0 = target_angle;
                true
            } else if wrap_angle < 0. {
                rotation.0 = (rotation.0 - ANGLE_SPEED) % TWOPI;
                wrap_angle.abs() < PI / 4.
            } else {
                rotation.0 = (rotation.0 + ANGLE_SPEED) % TWOPI;
                wrap_angle.abs() < PI / 4.
            };
        } else {
            bullet_shooter.0 = false;
        }
    }
}

const HEALER_RANGE: f32 = 300.;

fn healer_find_target(
    mut query: Query<(Entity, &mut Rotation, &Position, &mut Healer, &mut Target), With<Tower>>,
    mut friend_query: Query<(Entity, &Position, &Health), With<Tower>>,
) {
    for (entity, mut rotation, position, mut healer, mut target) in query.iter_mut() {
        let new_target =
            friend_query
                .iter_mut()
                .fold(None, |acc, (target_entity, target_position, health)| {
                    if entity == target_entity || health.val == health.max {
                        return acc;
                    }
                    let this_dist = target_position.0.distance(position.0);
                    let rel_health = health.val / health.max;
                    if let Some((prev_health, _, _)) = acc {
                        if this_dist < HEALER_RANGE && rel_health < prev_health {
                            Some((rel_health, target_entity, target_position))
                        } else {
                            acc
                        }
                    } else {
                        Some((rel_health, target_entity, target_position))
                    }
                });

        use std::f64::consts::PI;
        const TWOPI: f64 = PI * 2.;
        const ANGLE_SPEED: f64 = PI / 50.;

        if let Some((_, new_target, enemy_position)) = new_target {
            target.0 = Some(new_target);

            let delta = enemy_position.0 - position.0;
            let target_angle = delta.y.atan2(delta.x) as f64;
            let delta_angle = target_angle - rotation.0;
            let wrap_angle =
                ((delta_angle + PI) - ((delta_angle + PI) / TWOPI).floor() * TWOPI) - PI;
            healer.0 = if wrap_angle.abs() < ANGLE_SPEED {
                rotation.0 = target_angle;
                true
            } else if wrap_angle < 0. {
                rotation.0 = (rotation.0 - ANGLE_SPEED) % TWOPI;
                wrap_angle.abs() < PI / 4.
            } else {
                rotation.0 = (rotation.0 + ANGLE_SPEED) % TWOPI;
                wrap_angle.abs() < PI / 4.
            };
        } else {
            healer.0 = false;
        }
    }
}

const HEALER_AMOUNT: f32 = 3.;
const HEALER_INTERVAL: f32 = 2.;

fn heal_target(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut Healer, &Target)>,
    mut target_query: Query<(&Position, &mut Health)>,
) {
    let delta = time.delta_seconds();
    for (mut healer, target) in query.iter_mut() {
        if !healer.0 {
            continue;
        }
        if delta < healer.1 {
            healer.1 -= delta;
            continue;
        }

        if let Some(target) = target.0 {
            if let Ok((position, mut target)) = target_query.get_mut(target) {
                if target.val < target.max {
                    target.val += HEALER_AMOUNT;
                    healer.1 += HEALER_INTERVAL;
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: asset_server.load("heal-effect.png"),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(20.0, 20.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(Position(position.0))
                        .insert(Velocity(Vec2::new(0., 5.)))
                        .insert(Timeout(HEALER_INTERVAL));
                    continue;
                }
            }
        }
        healer.1 = 0.;
    }
}

fn timeout(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut Timeout)>,
) {
    let delta = time.delta_seconds();
    for (entity, mut sprite, mut timeout) in query.iter_mut() {
        if timeout.0 < delta {
            commands.entity(entity).despawn();
            continue;
        }
        timeout.0 -= delta;
        if timeout.0 < 1. {
            sprite.color = Color::rgba(1., 1., 1., timeout.0);
        }
    }
}
