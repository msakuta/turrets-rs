mod bullet;
mod enemy;
mod tower;
mod ui;

use crate::{
    bullet::bullet_collision,
    enemy::{enemy_system, spawn_enemies, Enemy},
    tower::{update_health_bar, Shotgun, Timeout, TowerPlugin},
    ui::UIPlugin,
};
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .add_plugins(DefaultPlugins)
        .add_plugin(UIPlugin)
        .add_plugin(TowerPlugin)
        .add_startup_system(setup)
        .add_system(time_level)
        .add_system(erase_entities_new_game::<Enemy>)
        .add_system(erase_entities_new_game::<Bullet>)
        .add_system(spawn_enemies)
        .add_system(enemy_system)
        .add_system(linear_motion)
        .add_system(sprite_transform)
        .add_system(shoot_bullet)
        .add_system(bullet_collision)
        .add_system(animate_sprite)
        .add_system(update_health_bar)
        .add_system(cleanup::<Bullet>)
        .run();
}

#[derive(Component, Clone, Copy, Debug)]
struct Position(Vec2);

#[derive(Component, Clone, Copy, Debug)]
struct Rotation(f64);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct BulletShooter(bool, f32);

#[derive(Component)]
struct Target(Option<Entity>);

#[derive(Component)]
struct Bullet(bool);

#[derive(Component)]
struct BulletFilter(bool);

#[derive(Component)]
struct Health {
    val: f32,
    max: f32,
}

impl Health {
    const fn new(val: f32) -> Self {
        Self { val, max: val }
    }
}

#[derive(Component, Deref, DerefMut)]
struct Explosion(Timer);

// #[derive(Component)]
struct Textures {
    small_explosion: Handle<TextureAtlas>,
    large_explosion: Handle<TextureAtlas>,
}

struct Scoreboard {
    score: f64,
}

enum Level {
    Select,
    Running { timer: Timer },
}

impl Level {
    fn start() -> Self {
        Self::Running {
            timer: Timer::from_seconds(120., true),
        }
    }

    fn timer_finished(&self) -> bool {
        match self {
            Self::Select => false,
            Self::Running { timer } => timer.just_finished(),
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut gen_texture_handle = |file, size, columns| {
        let texture_handle = asset_server.load(file);
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(size, size), columns, 1);
        texture_atlases.add(texture_atlas)
    };
    commands.insert_resource(Textures {
        small_explosion: gen_texture_handle("explode.png", 16., 8),
        large_explosion: gen_texture_handle("explode2.png", 32., 6),
    });
    commands.insert_resource(Scoreboard { score: 0. });
    commands.insert_resource(Level::Select);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(UiCameraBundle::default());

    // spawn_towers(&mut commands, &asset_server);
}

fn time_level(mut level: ResMut<Level>, time: Res<Time>) {
    if let Level::Running { timer } = level.as_mut() {
        timer.tick(time.delta());
    }
}

fn erase_entities_new_game<T: Component>(
    mut commands: Commands,
    level: Res<Level>,
    query: Query<Entity, With<T>>,
) {
    if level.timer_finished() {
        println!("Round finished!");
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn linear_motion(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_seconds();
    }
}

fn sprite_transform(
    mut query: Query<(
        &Position,
        Option<&Rotation>,
        &mut Transform,
        Option<&Timeout>,
    )>,
) {
    for (position, rotation, mut transform, timeout) in query.iter_mut() {
        sprite_transform_single(
            position,
            rotation,
            transform.as_mut(),
            if timeout.is_some() { 0.1 } else { 0. },
        );
    }
}

fn sprite_transform_single(
    position: &Position,
    rotation: Option<&Rotation>,
    transform: &mut Transform,
    z: f32,
) {
    let mut trans = Transform::from_xyz(position.0.x, position.0.y, z);
    if let Some(rotation) = rotation {
        trans = trans.with_rotation(Quat::from_rotation_z(rotation.0 as f32));
    }
    trans = trans.with_scale(Vec3::new(3., 3., 3.));
    *transform = trans;
}

const SHOOT_INTERVAL: f32 = 0.5;
const SHOTGUN_SHOOT_INTERVAL: f32 = 1.5;
const BULLET_SPEED: f32 = 500.;

fn shoot_bullet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut query: Query<(
        &Position,
        Option<&Rotation>,
        &mut BulletShooter,
        Option<&Shotgun>,
    )>,
) {
    let delta = time.delta_seconds();
    for (position, rotation, mut bullet_shooter, shotgun) in query.iter_mut() {
        if !bullet_shooter.0 {
            continue;
        }
        if bullet_shooter.1 < delta {
            let mut shoot = |file, angle: f64| {
                let bullet_rotation = Rotation(angle);
                let mut transform = default();
                sprite_transform_single(position, Some(&bullet_rotation), &mut transform, 0.);
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load(file),
                        transform,
                        ..default()
                    })
                    .insert(*position)
                    .insert(bullet_rotation)
                    .insert(Velocity(
                        BULLET_SPEED * Vec2::new(angle.cos() as f32, angle.sin() as f32),
                    ))
                    .insert(Bullet(rotation.is_some()));
            };

            if let Some(rotation) = rotation {
                if shotgun.is_some() {
                    for i in -3..=3 {
                        shoot(
                            "shotgun-bullet.png",
                            rotation.0 + i as f64 * std::f64::consts::PI / 20.,
                        );
                    }
                    bullet_shooter.1 += SHOTGUN_SHOOT_INTERVAL;
                } else {
                    shoot("bullet.png", rotation.0);
                    bullet_shooter.1 += SHOOT_INTERVAL;
                }
            } else {
                shoot(
                    "enemy-bullet.png",
                    rand::random::<f64>() * std::f64::consts::PI * 2.,
                );
                bullet_shooter.1 += SHOOT_INTERVAL * rand::random::<f32>();
            }
        }
        bullet_shooter.1 -= delta;
    }
}

fn animate_sprite(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &mut Explosion,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            if sprite.index + 1 == texture_atlas.textures.len() {
                commands.entity(entity).despawn();
            } else {
                sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
            }
        }
    }
}

fn cleanup<T: Component>(
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Position, &T)>,
) {
    let window = if let Some(window) = windows.iter().next() {
        window
    } else {
        return;
    };
    let (width, height) = (window.width(), window.height());
    for (entity, position, _) in query.iter() {
        if position.0.x < -width / 2.
            || width / 2. < position.0.x
            || position.0.y < -height / 2.
            || height / 2. < position.0.y
        {
            commands.entity(entity).despawn();
            // println!("Despawned {entity:?} ({})", std::any::type_name::<T>());
        }
    }
}
