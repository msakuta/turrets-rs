mod missile;

use self::missile::{missile_system, Missile, MISSILE_SPEED};
use crate::{
    can_update, sprite_transform_single,
    tower::{MissileShooter, Shotgun, TempEnt, Tower},
    BulletFilter, Explosion, Health, Position, Rotation, Scoreboard, StageClear, Target, Textures,
    Velocity,
};
use bevy::{prelude::*, sprite::collide_aabb::collide, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

pub(crate) const ENEMY_SIZE: f32 = 20.;
const BULLET_SIZE: f32 = 20.;

pub(crate) const SHOOT_INTERVAL: f32 = 0.25;
const ENEMY_SHOOT_INTERVAL: f32 = 0.5;
const SHOTGUN_SHOOT_INTERVAL: f32 = 0.75;
const MISSILE_SHOOT_INTERVAL: f32 = 2.5;
const BULLET_SPEED: f32 = 500.;

#[derive(Event)]
pub(crate) struct GainExpEvent {
    pub entity: Entity,
    pub exp: usize,
    /// True if this event killed an enemy
    pub killed: bool,
}

pub(crate) struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin);
        app.add_event::<GainExpEvent>();
        app.add_systems(
            Update,
            (shoot_bullet, bullet_collision_system, missile_system).run_if(can_update),
        );
        app.add_systems(Update, cleanup);
    }
}

#[derive(Component)]
pub(crate) struct BulletShooter {
    pub enabled: bool,
    pub cooldown: f32,
    pub damage: f32,
}

impl BulletShooter {
    pub(crate) fn new(enabled: bool, damage: f32) -> Self {
        Self {
            enabled,
            cooldown: 0.,
            damage,
        }
    }
}

#[derive(Component)]
pub(crate) struct Bullet {
    filter: bool,
    owner: Entity,
    damage: f32,
}

pub(crate) fn shoot_bullet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    textures: Res<Textures>,
    mut query: Query<(
        Entity,
        &Position,
        &BulletFilter,
        Option<&Rotation>,
        &mut BulletShooter,
        Option<&Shotgun>,
        Option<&MissileShooter>,
        Option<&Target>,
    )>,
) {
    let delta = time.delta_seconds();
    for (
        entity,
        position,
        bullet_filter,
        rotation,
        mut bullet_shooter,
        shotgun,
        missile_shooter,
        target,
    ) in query.iter_mut()
    {
        if !bullet_shooter.enabled {
            continue;
        }
        if bullet_shooter.cooldown < delta {
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

                    let trail = missile_shooter
                        .map(|_| missile::gen_trail(&mut commands, &position, &textures));

                    sprite_transform_single(&position, Some(&bullet_rotation), &mut transform, 0.);
                    let sprite = commands
                        .spawn(SpriteBundle {
                            texture: asset_server.load(file),
                            transform: Transform::from_scale(Vec3::ONE * 3.),
                            ..default()
                        })
                        .id();

                    let mut builder = commands.spawn((
                        Bullet {
                            filter: !bullet_filter.filter,
                            owner: entity,
                            damage: bullet_shooter.damage,
                        },
                        TransformBundle {
                            local: transform,
                            ..default()
                        },
                        position,
                        bullet_rotation,
                        Velocity(speed * Vec2::new(angle.cos() as f32, angle.sin() as f32)),
                        StageClear,
                    ));
                    if let Some((target, trail)) = target.zip(trail) {
                        builder.insert(Missile::new(target, trail, &position));
                    }
                    builder.add_child(sprite);
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
                    bullet_shooter.cooldown += SHOTGUN_SHOOT_INTERVAL;
                } else if missile_shooter.is_some() {
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
                        bullet_shooter.cooldown += MISSILE_SHOOT_INTERVAL;
                    }
                } else {
                    shoot(
                        if bullet_filter.filter {
                            "agile-enemy-bullet.png"
                        } else {
                            "bullet.png"
                        },
                        rotation.0,
                        BULLET_SPEED,
                        0.,
                        None,
                    );
                    bullet_shooter.cooldown += if bullet_filter.filter {
                        ENEMY_SHOOT_INTERVAL
                    } else {
                        SHOOT_INTERVAL
                    };
                }
            } else {
                shoot(
                    "enemy-bullet.png",
                    rand::random::<f64>() * std::f64::consts::PI * 2.,
                    BULLET_SPEED,
                    0.,
                    None,
                );
                bullet_shooter.cooldown += SHOOT_INTERVAL * rand::random::<f32>() * 2.;
            }
        }
        bullet_shooter.cooldown -= delta;
    }
}

pub(crate) fn bullet_collision_system(
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
    mut event_writer: EventWriter<GainExpEvent>,
) {
    for (bullet_entity, bullet_transform, bullet, missile) in bullet_query.iter() {
        for (entity, transform, health, bullet_filter, tower) in target_query.iter_mut() {
            if bullet.filter == bullet_filter.filter {
                single_collision(
                    &mut commands,
                    bullet_entity,
                    bullet,
                    bullet_transform,
                    bullet_filter,
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

fn single_collision(
    commands: &mut Commands,
    bullet_entity: Entity,
    bullet: &Bullet,
    bullet_transform: &Transform,
    bullet_filter: &BulletFilter,
    missile: Option<&Missile>,
    entity: Entity,
    transform: &Transform,
    tower: Option<&Tower>,
    event_writer: &mut EventWriter<GainExpEvent>,
    mut health: Mut<Health>,
    textures: &Res<Textures>,
    scoreboard: &mut ResMut<Scoreboard>,
) {
    let collision = collide(
        bullet_transform.translation,
        Vec2::new(BULLET_SIZE, BULLET_SIZE),
        transform.translation,
        Vec2::new(bullet_filter.radius, bullet_filter.radius),
    );

    if collision.is_some() {
        commands.entity(bullet_entity).despawn_recursive();
        if let Some(missile) = missile {
            commands.entity(missile.trail).despawn_recursive();
        }
        if health.val < 1. {
            commands.entity(entity).despawn_recursive();
            if let Some(tower) = tower {
                commands.entity(tower.health_bar.0).despawn();
                commands.entity(tower.health_bar.1).despawn();
            }
            commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: textures.large_explosion.clone(),
                    transform: bullet_transform.clone().with_scale(Vec3::splat(4.0)),
                    ..default()
                })
                .insert(Explosion(Timer::from_seconds(0.15, TimerMode::Repeating)))
                .insert(StageClear)
                .insert(TempEnt);
            scoreboard.score += bullet_filter.exp as f64;
            scoreboard.credits += bullet_filter.exp as f64;

            event_writer.send(GainExpEvent {
                entity: bullet.owner,
                exp: bullet_filter.exp,
                killed: true,
            });
        } else {
            health.val -= bullet.damage;
        }

        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: textures.small_explosion.clone(),
                transform: bullet_transform.clone().with_scale(Vec3::splat(3.0)),
                ..default()
            })
            .insert(Explosion(Timer::from_seconds(0.06, TimerMode::Repeating)))
            .insert(StageClear)
            .insert(TempEnt);
    }
}

fn cleanup(
    mut commands: Commands,
    // windows: Res<Windows>,
    window: Query<&Window, With<PrimaryWindow>>,
    query: Query<(Entity, &Position, Option<&Missile>), (With<Bullet>, Without<Missile>)>,
) {
    // let window = if let Some(window) = windows.iter().next() {
    //     window
    // } else {
    //     return;
    // };
    let window = window.single();
    let (width, height) = (window.width(), window.height());
    for (entity, position, missile) in query.iter() {
        if position.0.x < -width / 2.
            || width / 2. < position.0.x
            || position.0.y < -height / 2.
            || height / 2. < position.0.y
        {
            commands.entity(entity).despawn_recursive();
            if let Some(missile) = missile {
                commands.entity(missile.trail).despawn_recursive();
            }
        }
    }
}
