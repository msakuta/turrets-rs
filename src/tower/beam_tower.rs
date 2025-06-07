use super::{
    apprach_angle, tower_circle, tower_sprite_bundle, tower_transform_bundle, Tower, TowerBundle,
    TowerInitBundle, TowerLevel, BEAM_TOWER_HEALTH,
};
use crate::{
    bullet::GainExpEvent, enemy::Enemy, BulletFilter, Explosion, Health, Position, Rotation,
    StageClear, Target, Textures,
};
use ::serde::{Deserialize, Serialize};
use bevy::prelude::*;

const BEAM_TOWER_SIZE: f32 = 48.;
const BEAM_RANGE: f32 = 1000.;
const BEAM_SPRITE_SIZE: f32 = 32.;
const SHOOT_DURATION: f32 = 2.;
const SHOOT_INTERVAL: f32 = 5.;

#[derive(Component, Serialize, Deserialize)]
pub(crate) struct BeamTower {
    pub shoot_phase: f32,
    pub cooldown: f32,
    pub filter: bool,
    #[serde(skip)]
    pub beam: Option<Entity>,
}

impl BeamTower {
    pub(crate) fn new(beam: Entity) -> Self {
        Self {
            cooldown: 0.,
            shoot_phase: 0.,
            filter: true,
            beam: Some(beam),
        }
    }

    pub(crate) fn beam_dps_by_level(level: usize) -> f32 {
        50. * (1.2f32).powf(level as f32)
    }
}

pub(crate) fn spawn_beam_tower(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
    textures: &Textures,
) -> Entity {
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        BEAM_TOWER_SIZE,
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(BEAM_TOWER_HEALTH)),
            ..bundle
        },
    );
    let sprite = commands
        .spawn(tower_sprite_bundle("beam-tower.png", asset_server, 3.))
        .id();
    let beam = commands
        .spawn(SpriteBundle {
            texture: asset_server.load("beam.png"),
            transform: Transform::from_translation(Vec3::new(BEAM_RANGE / 2., 0., 0.025))
                .with_scale(Vec3::new(BEAM_RANGE / BEAM_SPRITE_SIZE, 1., 1.)),
            visibility: Visibility::Hidden,
            ..default()
        })
        .id();
    let shape = commands.spawn(tower_circle(BEAM_TOWER_SIZE, textures)).id();
    commands
        .spawn(tower)
        .insert(BeamTower::new(beam))
        .insert(tower_transform_bundle(position))
        .add_child(sprite)
        .add_child(shape)
        .add_child(beam)
        .id()
}

pub(crate) fn beam_tower_find_target(
    mut query: Query<
        (
            Entity,
            &mut Rotation,
            &Position,
            &mut BeamTower,
            &mut Target,
        ),
        With<Tower>,
    >,
    mut enemy_query: Query<(Entity, &Position), With<Enemy>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    for (entity, mut rotation, position, mut beamer, mut target) in query.iter_mut() {
        let new_target =
            enemy_query
                .iter_mut()
                .fold(None, |acc, (target_entity, target_position)| {
                    if entity == target_entity {
                        return acc;
                    }
                    let this_dist = target_position.0.distance(position.0);
                    if let Some((prev_dist, _, _)) = acc {
                        if this_dist < BEAM_RANGE && this_dist < prev_dist {
                            Some((this_dist, target_entity, target_position))
                        } else {
                            acc
                        }
                    } else {
                        Some((this_dist, target_entity, target_position))
                    }
                });

        use std::f64::consts::PI;
        const ANGLE_SPEED: f64 = PI / 2.;

        if let Some((_, new_target, enemy_position)) = new_target {
            target.0 = Some(new_target);

            let delta = enemy_position.0 - position.0;
            let target_angle = delta.y.atan2(delta.x) as f64;
            let (new_rotation, enabled) =
                apprach_angle(rotation.0, target_angle, ANGLE_SPEED * delta_time as f64);
            rotation.0 = new_rotation;
            if enabled && beamer.cooldown == 0. {
                beamer.shoot_phase = SHOOT_DURATION;
                beamer.cooldown = SHOOT_INTERVAL;
            }
        }
    }
}

pub(crate) fn shoot_beam(
    mut commands: Commands,
    time: Res<Time>,
    textures: Res<Textures>,
    mut query: Query<(Entity, &mut BeamTower, &TowerLevel, &Position, &Rotation)>,
    mut target_query: Query<(&Position, &mut Health, &BulletFilter)>,
    mut beam_query: Query<&mut Visibility>,
    mut exp_event: EventWriter<GainExpEvent>,
) {
    let delta = time.delta_seconds();
    for (entity, mut beamer, level, position, rotation) in query.iter_mut() {
        beamer.cooldown = (beamer.cooldown - delta).max(0.);
        if delta < beamer.shoot_phase {
            beamer.shoot_phase -= delta;
        } else {
            beamer.shoot_phase = 0.;
            if let Some(mut beam) = beamer.beam.and_then(|beam| beam_query.get_mut(beam).ok()) {
                *beam = Visibility::Hidden;
            }
            continue;
        }

        if let Some(mut beam) = beamer.beam.and_then(|beam| beam_query.get_mut(beam).ok()) {
            *beam = Visibility::Inherited;
        }

        for (target_position, mut target, bullet_filter) in target_query.iter_mut() {
            if target.val <= 0. || bullet_filter.filter != beamer.filter {
                continue;
            }

            let (dist_to_beam, dist_along_beam) = {
                let delta_vec = target_position.0 - position.0;
                let beam_direction = Vec2::new(rotation.0.cos() as f32, rotation.0.sin() as f32);
                let dot = delta_vec.dot(beam_direction);
                let perpendicular = delta_vec - dot * beam_direction;
                (perpendicular.length(), dot)
            };

            if bullet_filter.radius < dist_to_beam
                || dist_along_beam < 0.
                || BEAM_RANGE < dist_along_beam
            {
                continue;
            }

            target.val = (target.val - delta * BeamTower::beam_dps_by_level(level.level)).max(0.);
            if target.val == 0. {
                exp_event.send(GainExpEvent {
                    entity,
                    exp: bullet_filter.exp,
                    killed: true,
                });
            }

            commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: textures.small_explosion_blue.clone(),
                    transform: Transform::from_translation(Vec3::new(
                        target_position.0.x,
                        target_position.0.y,
                        0.2,
                    ))
                    .with_scale(Vec3::splat(3.0)),
                    ..default()
                })
                .insert(Explosion(Timer::from_seconds(0.06, TimerMode::Repeating)))
                .insert(StageClear);
        }
    }
}
