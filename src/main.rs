use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(animate)
        .add_system(sprite_transform)
        .run();
}

#[derive(Component)]
struct Position(f64, f64);

#[derive(Component)]
struct Rotation(f64);

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
            .insert(Position(i as f64 * 100.0 - 100., 0.0))
            .insert(Rotation(i as f64 * std::f64::consts::PI / 3.));
    }
}

fn animate(time: Res<Time>, mut query: Query<&mut Rotation>) {
    for mut rotation in query.iter_mut() {
        rotation.0 += time.delta_seconds_f64();
    }
}

fn sprite_transform(mut query: Query<(&Position, &Rotation, &mut Transform)>) {
    for (position, rotation, mut transform) in query.iter_mut() {
        *transform = Transform::from_xyz(position.0 as f32, position.1 as f32, 0.)
            .with_rotation(Quat::from_rotation_z(rotation.0 as f32))
            .with_scale(Vec3::new(3., 3., 3.));
    }
}
