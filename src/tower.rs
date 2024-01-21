mod beam_tower;
mod healer;

use self::{
    beam_tower::{beam_tower_find_target, shoot_beam},
    healer::{heal_target, healer_find_target},
};
use crate::{
    bullet::{BulletShooter, GainExpEvent},
    mouse::tower_not_dragging,
    BulletFilter, Enemy, Health, Position, Rotation, Target,
};
use ::serde::{Deserialize, Serialize};
use bevy::prelude::*;
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*, shapes::Circle};

pub(crate) use self::{
    beam_tower::{spawn_beam_tower, BeamTower},
    healer::{spawn_healer, Healer},
};

const TOWER_SIZE: f32 = 32.;
const MISSILE_TOWER_SIZE: f32 = 48.;

#[derive(Component, Serialize, Deserialize)]
pub(crate) struct Tower {
    pub health_bar: (Entity, Entity),
    pub size: f32,
}

#[derive(Component, Serialize, Deserialize)]
pub(crate) struct TowerLevel {
    pub level: usize,
    pub exp: usize,
    pub max_health_base: f32,
    pub max_health_exponent: f32,
}

#[derive(Component, Serialize, Deserialize)]
pub(crate) struct TowerScore {
    pub kills: usize,
}

#[derive(Component, Serialize, Deserialize)]
pub(crate) struct Shotgun;

#[derive(Component, Serialize, Deserialize)]
pub(crate) struct MissileShooter;

#[derive(Component)]
pub(crate) struct Timeout(f32);

/// Indicates temporary entities
#[derive(Component)]
pub(crate) struct TempEnt;

#[derive(Bundle)]
pub(crate) struct TowerBundle {
    position: Position,
    rotation: Rotation,
    tower: Tower,
    tower_level: TowerLevel,
    tower_score: TowerScore,
    health: Health,
    target: Target,
    bullet_filter: BulletFilter,
}

impl TowerBundle {
    pub(crate) fn new(
        commands: &mut Commands,
        position: Position,
        rotation: Rotation,
        size: f32,
        bundle: TowerInitBundle,
    ) -> Self {
        Self {
            position,
            rotation,
            tower: Tower {
                health_bar: health_bar(commands),
                size,
            },
            tower_level: bundle.tower_level.unwrap_or(TowerLevel {
                level: 0,
                exp: 0,
                max_health_base: 10.,
                max_health_exponent: 1.2,
            }),
            tower_score: bundle.tower_score.unwrap_or(TowerScore { kills: 0 }),
            health: bundle.health.unwrap(),
            target: Target(None),
            bullet_filter: BulletFilter {
                filter: false,
                radius: 10.,
                exp: 10,
            },
        }
    }
}

fn shape_from_size(size: f32) -> ShapeBundle {
    let line = Circle {
        radius: size,
        center: Vec2::ZERO,
    };

    GeometryBuilder::build_as(
        &line,
        DrawMode::Stroke(StrokeMode::new(Color::rgba(0.0, 0.8, 0.8, 1.), 1.0)),
        Transform::from_xyz(0., 0., 0.05),
    )
}

#[derive(Component)]
pub(crate) struct TowerHealthBar;

pub(crate) struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_health_bar).add_system_set(
            SystemSet::new()
                .with_run_criteria(tower_not_dragging)
                .with_system(tower_find_target)
                .with_system(healer_find_target)
                .with_system(heal_target)
                .with_system(beam_tower_find_target)
                .with_system(shoot_beam)
                .with_system(timeout),
        );
        app.add_system(tower_killed_system);
    }
}

const TOWER_HEALTH: Health = Health::new(10.);
const SHOTGUN_HEALTH: Health = Health::new(20.);
const HEALER_HEALTH: Health = Health::new(20.);
const MISSILE_HEALTH: Health = Health::new(30.);
const BEAM_TOWER_HEALTH: Health = Health::new(30.);

pub(crate) fn spawn_towers(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    for i in 0..2 {
        spawn_turret(
            commands,
            asset_server,
            Vec2::new(i as f32 * 200.0 - 100., 0.0),
            i as f64 * std::f64::consts::PI * 2. / 3.,
            default(),
        );
    }
}

#[derive(Default)]
pub(crate) struct TowerInitBundle {
    pub tower_level: Option<TowerLevel>,
    pub tower_score: Option<TowerScore>,
    pub health: Option<Health>,
}

fn bullet_shooter_from_level(tower_level: &Option<TowerLevel>, missile: bool) -> BulletShooter {
    BulletShooter::new(
        false,
        bullet_damage_by_level(tower_level.as_ref().map(|l| l.level).unwrap_or(0), missile),
    )
}

fn tower_sprite_bundle(texture_name: &str, asset_server: &AssetServer, scale: f32) -> SpriteBundle {
    SpriteBundle {
        texture: asset_server.load(texture_name),
        transform: Transform::from_translation(Vec3::new(0., 0., 0.1))
            .with_scale(Vec3::new(scale, scale, scale)),
        ..default()
    }
}

fn tower_transform_bundle(position: Vec2) -> TransformBundle {
    TransformBundle {
        local: Transform::from_translation(Vec3::new(position.x, position.y, 0.1)),
        ..default()
    }
}

pub(crate) fn spawn_turret(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
) -> Entity {
    let bullet_shooter = bullet_shooter_from_level(&bundle.tower_level, false);
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TOWER_SIZE,
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(TOWER_HEALTH)),
            ..bundle
        },
    );
    let sprite = commands
        .spawn_bundle(tower_sprite_bundle("turret.png", asset_server, 3.))
        .id();
    let shape = commands.spawn_bundle(shape_from_size(TOWER_SIZE)).id();
    commands
        .spawn_bundle(tower)
        .insert_bundle(tower_transform_bundle(position))
        .insert(bullet_shooter)
        .add_child(sprite)
        .add_child(shape)
        .id()
}

pub(crate) fn spawn_shotgun(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
) -> Entity {
    let bullet_shooter = bullet_shooter_from_level(&bundle.tower_level, false);
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TOWER_SIZE,
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(SHOTGUN_HEALTH)),
            ..bundle
        },
    );
    let sprite = commands
        .spawn_bundle(tower_sprite_bundle("shotgun.png", asset_server, 3.))
        .id();
    let shape = commands.spawn_bundle(shape_from_size(TOWER_SIZE)).id();
    commands
        .spawn_bundle(tower)
        .insert_bundle(tower_transform_bundle(position))
        .insert(bullet_shooter)
        .insert(Shotgun)
        .add_child(sprite)
        .add_child(shape)
        .id()
}

pub(crate) fn spawn_missile_tower(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
) -> Entity {
    let bullet_shooter = bullet_shooter_from_level(&bundle.tower_level, true);
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        MISSILE_TOWER_SIZE,
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(MISSILE_HEALTH)),
            ..bundle
        },
    );
    let sprite = commands
        .spawn_bundle(tower_sprite_bundle("missile-tower.png", asset_server, 3.))
        .id();
    let shape = commands
        .spawn_bundle(shape_from_size(MISSILE_TOWER_SIZE))
        .id();
    commands
        .spawn_bundle(tower)
        .insert_bundle(tower_transform_bundle(position))
        .insert(bullet_shooter)
        .insert(MissileShooter)
        .add_child(sprite)
        .add_child(shape)
        .id()
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
            .insert(TowerHealthBar)
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
            .insert(TowerHealthBar)
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

/// Try to approach the target angle from current angle.
/// Returns a pair (angle, in_range) where angle is the result angle and in_range is whether
/// the target is close to the target (thus allowed to shoot).
pub(super) fn apprach_angle(
    current_angle: f64,
    target_angle: f64,
    angle_speed: f64,
) -> (f64, bool) {
    use std::f64::consts::PI;
    const TWOPI: f64 = PI * 2.;

    let delta_angle = target_angle - current_angle;
    let wrap_angle = ((delta_angle + PI) - ((delta_angle + PI) / TWOPI).floor() * TWOPI) - PI;

    if wrap_angle.abs() < angle_speed {
        (target_angle, true)
    } else if wrap_angle < 0. {
        (
            (current_angle - angle_speed) % TWOPI,
            wrap_angle.abs() < PI / 4.,
        )
    } else {
        (
            (current_angle + angle_speed) % TWOPI,
            wrap_angle.abs() < PI / 4.,
        )
    }
}

fn tower_find_target(
    mut query: Query<(&mut Rotation, &Position, &mut BulletShooter, &mut Target), With<Tower>>,
    enemy_query: Query<(Entity, &Position), With<Enemy>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
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

        const ANGLE_SPEED: f64 = PI;

        if let Some((_, new_target, enemy_position)) = new_target {
            target.0 = Some(new_target);

            let delta = enemy_position.0 - position.0;
            let target_angle = delta.y.atan2(delta.x) as f64;
            (rotation.0, bullet_shooter.enabled) =
                apprach_angle(rotation.0, target_angle, ANGLE_SPEED * delta_time as f64);
        } else {
            bullet_shooter.enabled = false;
        }
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

pub(crate) fn tower_max_exp(level: usize) -> usize {
    ((1.5f64).powf(level as f64) * 100.).ceil() as usize
}

fn tower_killed_system(
    mut query: Query<(
        &mut TowerLevel,
        &mut Health,
        &mut TowerScore,
        Option<&mut BulletShooter>,
        Option<&mut Healer>,
        Option<&MissileShooter>,
    )>,
    mut reader: EventReader<GainExpEvent>,
) {
    for event in reader.iter() {
        if let Ok((
            mut tower,
            mut health,
            mut scoring_tower,
            mut bullet_shooter,
            mut healer,
            missile_tower,
        )) = query.get_mut(event.entity)
        {
            if event.killed {
                scoring_tower.kills += 1;
            }

            tower.exp += event.exp;
            while tower_max_exp(tower.level) <= tower.exp {
                tower.level += 1;
                health.max = (tower.max_health_exponent.powf(tower.level as f32)
                    * tower.max_health_base)
                    .ceil();
                health.val = health.max;
                if let Some(ref mut bullet_shooter) = bullet_shooter {
                    bullet_shooter.damage =
                        bullet_damage_by_level(tower.level, missile_tower.is_some());
                }
                if let Some(ref mut healer) = healer {
                    healer.heal_amt = heal_amt_by_level(tower.level);
                }
            }
        }
    }
}

fn bullet_damage_by_level(level: usize, missile: bool) -> f32 {
    let base = if missile { 30. } else { 1. };
    base * (1.2f32).powf(level as f32)
}

fn heal_amt_by_level(level: usize) -> f32 {
    1. + 0.1 * level as f32
}
