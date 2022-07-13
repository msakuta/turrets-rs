mod bullet;
mod enemy;
mod tower;

use crate::{
    bullet::bullet_collision,
    enemy::{enemy_system, spawn_enemies, Enemy},
    tower::{update_health_bar, Healer, Shotgun, Timeout, TowerBundle, TowerPlugin},
};
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .add_plugins(DefaultPlugins)
        .add_plugin(TowerPlugin)
        .add_startup_system(setup)
        .add_system(spawn_enemies)
        .add_system(enemy_system)
        .add_system(linear_motion)
        .add_system(sprite_transform)
        .add_system(shoot_bullet)
        .add_system(bullet_collision)
        .add_system(animate_sprite)
        .add_system(update_scoreboard)
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
    fn new(val: f32) -> Self {
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

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(UiCameraBundle::default());

    // Scoreboard
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                    },
                },
            ],
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        },
        ..default()
    });

    for i in 0..3 {
        let tower = TowerBundle::new(
            &mut commands,
            Position(Vec2::new(i as f32 * 100.0 - 100., 0.0)),
            Rotation(i as f64 * std::f64::consts::PI / 3.),
            Health::new(100.),
        );
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("turret.png"),
                ..default()
            })
            .insert_bundle(tower)
            .insert(BulletShooter::new());
    }

    let tower = TowerBundle::new(
        &mut commands,
        Position(Vec2::new(0.0, -100.0)),
        Rotation(0.),
        Health::new(200.),
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("shotgun.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(BulletShooter::new())
        .insert(Shotgun);

    let tower = TowerBundle::new(
        &mut commands,
        Position(Vec2::new(0.0, 100.0)),
        Rotation(0.),
        Health::new(200.),
    );
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("healer.png"),
            ..default()
        })
        .insert_bundle(tower)
        .insert(Healer::new());
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

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", scoreboard.score);
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
