use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(animate)
        .add_system(linear_motion)
        .add_system(sprite_transform)
        .add_system(shoot_bullet)
        .run();
}

#[derive(Component, Clone, Copy, Debug)]
struct Position(Vec2);

#[derive(Component, Clone, Copy, Debug)]
struct Rotation(f64);

#[derive(Component, Clone, Copy, Debug)]
struct Velocity(Vec2);

#[derive(Component)]
struct Tower(f32);

#[derive(Component)]
struct Bullet;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    for i in 0..3 {
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("turret.png"),
                ..default()
            })
            .insert(Position(Vec2::new(i as f32 * 100.0 - 100., 0.0)))
            .insert(Rotation(i as f64 * std::f64::consts::PI / 3.))
            .insert(Tower(rand::random()));
    }
}

fn animate(time: Res<Time>, mut query: Query<(&mut Rotation, &Tower)>) {
    for (mut rotation, _) in query.iter_mut() {
        rotation.0 += time.delta_seconds_f64();
    }
}

fn linear_motion(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_seconds();
    }
}

fn sprite_transform(mut query: Query<(&Position, &Rotation, &mut Transform)>) {
    for (position, rotation, mut transform) in query.iter_mut() {
        *transform = Transform::from_xyz(position.0.x, position.0.y, 0.)
            .with_rotation(Quat::from_rotation_z(rotation.0 as f32))
            .with_scale(Vec3::new(3., 3., 3.));
    }
}

const SHOOT_INTERVAL: f32 = 1.;
const BULLET_SPEED: f32 = 300.;

fn shoot_bullet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut query: Query<(&Position, &Rotation, &mut Tower)>,
) {
    let delta = time.delta_seconds();
    for (position, rotation, mut tower) in query.iter_mut() {
        if tower.0 < delta {
            commands
                .spawn_bundle(SpriteBundle {
                    texture: asset_server.load("bullet.png"),
                    ..default()
                })
                .insert(*position)
                .insert(*rotation)
                .insert(Velocity(
                    BULLET_SPEED * Vec2::new(rotation.0.cos() as f32, rotation.0.sin() as f32),
                ))
                .insert(Bullet);
            tower.0 += SHOOT_INTERVAL;
        }
        tower.0 -= delta;
    }
}
