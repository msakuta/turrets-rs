use super::{Timeout, Tower};
use crate::{mouse::SelectedTower, Health, Position, Rotation, Target, Velocity};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Healer(bool, f32);

impl Healer {
    pub(crate) fn new() -> Self {
        Self(false, 2.)
    }
}

const HEALER_RANGE: f32 = 300.;
const HEALER_AMOUNT: f32 = 1.;
const HEALER_INTERVAL: f32 = 2.;

pub(crate) fn healer_find_target(
    mut query: Query<(Entity, &mut Rotation, &Position, &mut Healer, &mut Target), With<Tower>>,
    mut friend_query: Query<(Entity, &Position, &Health), With<Tower>>,
    selected_tower: Res<SelectedTower>,
) {
    if selected_tower.dragging {
        return;
    }

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

pub(crate) fn heal_target(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut Healer, &Target, &Position)>,
    mut target_query: Query<(&Position, &mut Health)>,
    selected_tower: Res<SelectedTower>,
) {
    if selected_tower.dragging {
        return;
    }
    let delta = time.delta_seconds();
    for (mut healer, target, position) in query.iter_mut() {
        if !healer.0 {
            continue;
        }
        if delta < healer.1 {
            healer.1 -= delta;
            continue;
        }

        if let Some(target) = target.0 {
            if let Ok((target_position, mut target)) = target_query.get_mut(target) {
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
                        .insert(Position(target_position.0))
                        .insert(Velocity(Vec2::new(0., 5.)))
                        .insert(Timeout(HEALER_INTERVAL));

                    let delta = position.0 - target_position.0;
                    let centroid = (position.0 + target_position.0) / 2.;

                    commands
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(0.25, 1., 0.25),
                                custom_size: Some(Vec2::new(delta.length(), 2.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(
                                centroid.x, centroid.y, 0.1,
                            ))
                            .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
                            ..default()
                        })
                        .insert(Timeout(HEALER_INTERVAL / 2.));
                    continue;
                }
            }
        }
        healer.1 = 0.;
    }
}
