use bevy::{prelude::*, sprite::collide_aabb::collide};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(spawn_enemies)
        .add_system(tower_find_target)
        .add_system(linear_motion)
        .add_system(sprite_transform)
        .add_system(shoot_bullet)
        .add_system(bullet_collision)
        .add_system(animate_sprite)
        .add_system(update_scoreboard)
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
struct Tower;

#[derive(Component)]
struct BulletShooter(bool, f32);

#[derive(Component)]
struct Shotgun;

#[derive(Component)]
struct Target(Option<Entity>);

#[derive(Component)]
struct Bullet(bool);

#[derive(Component)]
struct BulletFilter(bool);

#[derive(Component, Deref, DerefMut)]
struct Health(f32);

#[derive(Component)]
struct Enemy;

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
        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("turret.png"),
                ..default()
            })
            .insert(Position(Vec2::new(i as f32 * 100.0 - 100., 0.0)))
            .insert(Rotation(i as f64 * std::f64::consts::PI / 3.))
            .insert(Tower)
            .insert(Health(100.))
            .insert(BulletShooter(false, rand::random::<f32>() * SHOOT_INTERVAL))
            .insert(Target(None))
            .insert(BulletFilter(false));
    }

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("shotgun.png"),
            ..default()
        })
        .insert(Position(Vec2::new(0.0, -100.0)))
        .insert(Rotation(0.))
        .insert(Tower)
        .insert(Health(200.))
        .insert(BulletShooter(
            false,
            rand::random::<f32>() * SHOTGUN_SHOOT_INTERVAL,
        ))
        .insert(Shotgun)
        .insert(Target(None))
        .insert(BulletFilter(false));
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
            bullet_shooter.0 = if wrap_angle.abs() < ANGLE_SPEED {
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
            bullet_shooter.0 = false;
        }
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
    let (_width, height) = (window.width(), window.height());

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
        .insert(Enemy)
        .insert(Health(3.))
        .insert(BulletShooter(true, SHOOT_INTERVAL))
        .insert(BulletFilter(true));
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
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load(file),
                        ..default()
                    })
                    .insert(*position)
                    .insert(Rotation(angle))
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

const ENEMY_SIZE: f32 = 20.;
const BULLET_SIZE: f32 = 20.;

fn bullet_collision(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut Health, &BulletFilter)>,
    bullet_query: Query<(Entity, &Transform, &Bullet)>,
    textures: Res<Textures>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (entity, transform, mut health, bullet_filter) in enemy_query.iter_mut() {
            if bullet.0 == bullet_filter.0 {
                entity_collision(
                    &mut commands,
                    bullet_entity,
                    bullet_transform,
                    entity,
                    transform,
                    health,
                    &textures,
                    &mut scoreboard,
                );
            }
        }
    }
}

fn entity_collision(
    commands: &mut Commands,
    bullet_entity: Entity,
    bullet_transform: &Transform,
    entity: Entity,
    transform: &Transform,
    mut health: Mut<Health>,
    textures: &Res<Textures>,
    scoreboard: &mut ResMut<Scoreboard>,
) {
    let collision = collide(
        bullet_transform.translation,
        Vec2::new(BULLET_SIZE, BULLET_SIZE),
        transform.translation,
        Vec2::new(ENEMY_SIZE, ENEMY_SIZE),
    );

    if collision.is_some() {
        commands.entity(bullet_entity).despawn();
        if **health < 1. {
            commands.entity(entity).despawn();
            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: textures.large_explosion.clone(),
                    transform: bullet_transform.clone().with_scale(Vec3::splat(4.0)),
                    ..default()
                })
                .insert(Explosion(Timer::from_seconds(0.15, true)));
            scoreboard.score += 10.;
        } else {
            **health -= 1.;
        }

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: textures.small_explosion.clone(),
                transform: bullet_transform.clone().with_scale(Vec3::splat(3.0)),
                ..default()
            })
            .insert(Explosion(Timer::from_seconds(0.06, true)));
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
            println!("Despawned {entity:?} ({})", std::any::type_name::<T>());
        }
    }
}
