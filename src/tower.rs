mod healer;

use self::healer::{heal_target, healer_find_target};
use crate::{
    bullet::{BulletShooter, GainExpEvent},
    mouse::tower_not_dragging,
    BulletFilter, Enemy, Health, Level, Position, Rotation, Scoreboard, Target,
};
use bevy::prelude::*;

pub(crate) use healer::Healer;

#[derive(Component)]
pub(crate) struct Tower {
    pub health_bar: (Entity, Entity),
}

#[derive(Component)]
pub(crate) struct TowerLevel {
    pub level: usize,
    pub exp: usize,
    pub max_health: &'static (dyn Fn(usize) -> f32 + Send + Sync),
}

#[derive(Component)]
pub(crate) struct TowerScore {
    pub kills: usize,
}

#[derive(Component)]
pub(crate) struct Shotgun;

#[derive(Component)]
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
        health: Health,
    ) -> Self {
        Self {
            position,
            rotation,
            tower: Tower {
                health_bar: health_bar(commands),
            },
            tower_level: TowerLevel {
                level: 0,
                exp: 0,
                max_health: &|level| ((1.2f32).powf(level as f32) * 10.).ceil(),
            },
            tower_score: TowerScore { kills: 0 },
            health,
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
                .with_system(timeout)
                .with_system(spawn_towers_new_game),
        );
        app.add_system(tower_killed_system);
    }
}

fn spawn_towers_new_game(
    mut commands: Commands,
    level: Res<Level>,
    query: Query<&Tower>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    if level.timer_finished() {
        println!("Round finished!");
        if query.iter().next().is_none() {
            spawn_towers(&mut commands, &asset_server);
            scoreboard.score = 0.;
        }
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
        );
    }
}

pub(crate) fn spawn_turret(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
) -> Entity {
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        TOWER_HEALTH,
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("turret.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(BulletShooter::new(false, 1.))
        .id()
}

pub(crate) fn spawn_shotgun(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
) -> Entity {
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        SHOTGUN_HEALTH,
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("shotgun.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(BulletShooter::new(false, 1.))
        .insert(Shotgun)
        .id()
}

pub(crate) fn spawn_healer(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
) -> Entity {
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        HEALER_HEALTH,
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("healer.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(Healer::new())
        .id()
}

pub(crate) fn spawn_missile_tower(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    rotation: f64,
) -> Entity {
    let tower = TowerBundle::new(
        commands,
        Position(position),
        Rotation(rotation),
        MISSILE_HEALTH,
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("missile-tower.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(BulletShooter::new(false, 1.))
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
                health.max = (*tower.max_health)(tower.level);
                health.val = health.max;
                if let Some(ref mut bullet_shooter) = bullet_shooter {
                    bullet_shooter.damage = (1.2f32).powf(tower.level as f32);
                }
                if let Some(ref mut healer) = healer {
                    healer.heal_amt = 1. + 0.1 * tower.level as f32;
                }
            }
        }
    }
}
