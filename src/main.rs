use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(spawn_enemies)
        .add_system(animate)
        .add_system(linear_motion)
        .add_system(sprite_transform)
        .add_system(shoot_bullet)
        .add_system(cleanup::<Bullet>)
        .add_system(cleanup::<Enemy>)
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

#[derive(Component)]
struct Enemy;

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

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    if time.delta_seconds() < rand::random() {
        return;
    }

    let window = if let Some(window) = windows.iter().next() {
        window
    } else {
        return;
    };
    let (width, height) = (window.width(), window.height());

    let down = rand::random::<bool>();

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("enemy.png"),
            ..default()
        })
        .insert(Position(Vec2::new(
            rand::random(),
            if down {
                -height / 2. + 10.
            } else {
                height / 2. - 10.
            },
        )))
        .insert(Velocity(
            BULLET_SPEED
                * Vec2::new(
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() * (if down { 1. } else { -1. }),
                ),
        ))
        .insert(Enemy);
}

fn linear_motion(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_seconds();
    }
}

fn sprite_transform(mut query: Query<(&Position, Option<&Rotation>, &mut Transform)>) {
    for (position, rotation, mut transform) in query.iter_mut() {
        let mut trans = Transform::from_xyz(position.0.x, position.0.y, 0.);
        if let Some(rotation) = rotation {
            trans = trans.with_rotation(Quat::from_rotation_z(rotation.0 as f32));
        }
        trans = trans.with_scale(Vec3::new(3., 3., 3.));
        *transform = trans;
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
            println!("Despawned {entity:?} ({})", std::any::type_name::<T>());
        }
    }
}
