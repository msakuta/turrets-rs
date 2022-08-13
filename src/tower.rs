mod healer;

use self::healer::{heal_target, healer_find_target};
use crate::{
    bullet::{BulletShooter, GainExpEvent},
    mouse::tower_not_dragging,
    BulletFilter, Enemy, Health, Position, Rotation, Target,
};
use ::serde::{Deserialize, Serialize};
use bevy::prelude::*;

pub(crate) use healer::Healer;

#[derive(Component, Serialize, Deserialize)]
pub(crate) struct Tower {
    pub health_bar: (Entity, Entity),
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
pub(crate) struct MissileTower;

#[derive(Component)]
pub(crate) struct Timeout(f32);

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
        bundle: TowerInitBundle,
    ) -> Self {
        Self {
            position,
            rotation,
            tower: Tower {
                health_bar: health_bar(commands),
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
                .with_system(timeout),
        );
        app.add_system(tower_killed_system);
    }
}

const TOWER_HEALTH: Health = Health::new(10.);
const SHOTGUN_HEALTH: Health = Health::new(20.);
const HEALER_HEALTH: Health = Health::new(20.);
const MISSILE_HEALTH: Health = Health::new(30.);

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

fn bullet_shooter_from_level(tower_level: &Option<TowerLevel>) -> BulletShooter {
    BulletShooter::new(
        false,
        bullet_damage_by_level(tower_level.as_ref().map(|l| l.level).unwrap_or(0)),
    )
}

pub(crate) fn spawn_turret(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
) -> Entity {
    let bullet_shooter = bullet_shooter_from_level(&bundle.tower_level);
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(TOWER_HEALTH)),
            ..bundle
        },
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("turret.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(bullet_shooter)
        .id()
}

pub(crate) fn spawn_shotgun(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
) -> Entity {
    let bullet_shooter = bullet_shooter_from_level(&bundle.tower_level);
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(SHOTGUN_HEALTH)),
            ..bundle
        },
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("shotgun.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(bullet_shooter)
        .insert(Shotgun)
        .id()
}

pub(crate) fn spawn_healer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
) -> Entity {
    let healer = Healer::new_with_heal_amt(heal_amt_by_level(
        bundle.tower_level.as_ref().map(|l| l.level).unwrap_or(0),
    ));
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(HEALER_HEALTH)),
            ..bundle
        },
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("healer.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(healer)
        .id()
}

pub(crate) fn spawn_missile_tower(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
    bundle: TowerInitBundle,
) -> Entity {
    let bullet_shooter = bullet_shooter_from_level(&bundle.tower_level);
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TowerInitBundle {
            health: Some(bundle.health.unwrap_or(MISSILE_HEALTH)),
            ..bundle
        },
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("missile-tower.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(bullet_shooter)
        .insert(MissileTower)
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
            bullet_shooter.enabled = if wrap_angle.abs() < ANGLE_SPEED {
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
    )>,
    mut reader: EventReader<GainExpEvent>,
) {
    for event in reader.iter() {
        if let Ok((mut tower, mut health, mut scoring_tower, mut bullet_shooter, mut healer)) =
            query.get_mut(event.entity)
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
                    bullet_shooter.damage = bullet_damage_by_level(tower.level);
                }
                if let Some(ref mut healer) = healer {
                    healer.heal_amt = heal_amt_by_level(tower.level);
                }
            }
        }
    }
}

fn bullet_damage_by_level(level: usize) -> f32 {
    (1.2f32).powf(level as f32)
}

fn heal_amt_by_level(level: usize) -> f32 {
    1. + 0.1 * level as f32
}
