use super::{
    heal_amt_by_level, tower_circle, tower_sprite_bundle, tower_transform_bundle, TempEnt, Timeout,
    Tower, TowerBundle, TowerInitBundle, HEALER_HEALTH, TOWER_SIZE,
};
use crate::{
    bullet::GainExpEvent, tower::apprach_angle, Health, Position, Rotation, Target, Textures,
    Velocity,
};
use bevy::prelude::*;

const HEALER_RANGE: f32 = 300.;
const HEALER_INTERVAL: f32 = 2.;

#[derive(Component)]
pub(crate) struct Healer {
    pub enabled: bool,
    pub cooldown: f32,
    pub heal_amt: f32,
}

impl Healer {
    pub(crate) fn new_with_heal_amt(heal_amt: f32) -> Self {
        Self {
            enabled: false,
            cooldown: 2.,
            heal_amt,
        }
    }
}

pub(crate) fn spawn_healer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
    textures: &Textures,
) -> Entity {
    let healer = Healer::new_with_heal_amt(heal_amt_by_level(
        bundle.tower_level.as_ref().map(|l| l.level).unwrap_or(0),
    ));
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TOWER_SIZE,
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(HEALER_HEALTH)),
            ..bundle
        },
    );
    let sprite = commands
        .spawn(tower_sprite_bundle("healer.png", asset_server, 3.))
        .id();
    let shape = commands.spawn(tower_circle(TOWER_SIZE, textures)).id();
    commands
        .spawn(tower)
        .insert(tower_transform_bundle(position))
        .insert(healer)
        .add_child(sprite)
        .add_child(shape)
        .id()
}

pub(crate) fn healer_find_target(
    mut query: Query<(Entity, &mut Rotation, &Position, &mut Healer, &mut Target), With<Tower>>,
    mut friend_query: Query<(Entity, &Position, &Health), With<Tower>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
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
        const ANGLE_SPEED: f64 = PI;

        if let Some((_, new_target, enemy_position)) = new_target {
            target.0 = Some(new_target);

            let delta = enemy_position.0 - position.0;
            let target_angle = delta.y.atan2(delta.x) as f64;
            (rotation.0, healer.enabled) =
                apprach_angle(rotation.0, target_angle, ANGLE_SPEED * delta_time as f64);
        } else {
            healer.enabled = false;
        }
    }
}

pub(crate) fn heal_target(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut Healer, &Target, &Position)>,
    mut target_query: Query<(&Position, &mut Health)>,
    mut exp_event: EventWriter<GainExpEvent>,
) {
    let delta = time.delta_seconds();
    for (entity, mut healer, target, position) in query.iter_mut() {
        if !healer.enabled {
            continue;
        }
        if delta < healer.cooldown {
            healer.cooldown -= delta;
            continue;
        }

        if let Some(target) = target.0 {
            if let Ok((target_position, mut target)) = target_query.get_mut(target) {
                if target.val < target.max {
                    target.val += healer.heal_amt;
                    healer.cooldown += HEALER_INTERVAL;
                    exp_event.send(GainExpEvent {
                        entity,
                        exp: (3. * healer.heal_amt).ceil() as usize,
                        killed: false,
                    });
                    commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load("heal-effect.png"),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(20.0, 20.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(Position(target_position.0))
                        .insert(Velocity(Vec2::new(0., 5.)))
                        .insert(Timeout(HEALER_INTERVAL))
                        .insert(TempEnt);

                    let delta = position.0 - target_position.0;
                    let centroid = (position.0 + target_position.0) / 2.;

                    commands
                        .spawn(SpriteBundle {
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
                        .insert(Timeout(HEALER_INTERVAL / 2.))
                        .insert(TempEnt);
                    continue;
                }
            }
        }
        healer.cooldown = 0.;
    }
}
