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
struct Rotation(f64);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("turret.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        })
        .insert(Rotation(0.));
}

fn animate(time: Res<Time>, mut query: Query<&mut Rotation>) {
    for mut rotation in query.iter_mut() {
        rotation.0 += time.delta_seconds_f64();
    }
}

fn sprite_transform(mut query: Query<(&Rotation, &mut Transform)>) {
    for (rotation, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(rotation.0 as f32);
    }
}
