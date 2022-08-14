use crate::{
    bullet::{BulletShooter, ENEMY_SIZE},
    mouse::tower_not_dragging,
    sprite_transform_single,
    tower::{apprach_angle, Tower},
    BulletFilter, Health, Level, Position, Rotation, StageClear, Target, Velocity,
};
use bevy::prelude::*;

pub(crate) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(tower_not_dragging)
                .with_system(spawn_enemies)
                .with_system(enemy_system)
                .with_system(agile_enemy_system),
        );
    }
}

#[derive(Component)]
pub(crate) struct Enemy;

#[derive(Component)]
struct AgileEnemy(bool);

const MAX_ENEMIES: usize = 100;

struct EnemySpec {
    waves: usize,
    image: &'static str,
    health: f32,
    size: f32,
    exp: usize,
    is_agile: bool,
    freq: fn(f: f32) -> f32,
}

impl EnemySpec {
    const fn default() -> Self {
        Self {
            waves: 0,
            image: "",
            health: 10.,
            size: ENEMY_SIZE,
            exp: 10,
            is_agile: false,
            freq: |f| {
                if f < 20. {
                    (f + 2.) / 20.
                } else {
                    1. / (f - 20. + 1.)
                }
            },
        }
    }
}

const ENEMY_SPECS: [EnemySpec; 3] = [
    EnemySpec {
        waves: 0,
        image: "enemy.png",
        health: 10.,
        size: ENEMY_SIZE,
        exp: 10,
        ..EnemySpec::default()
    },
    EnemySpec {
        waves: 5,
        image: "boss.png",
        health: 150.,
        size: ENEMY_SIZE * 2.,
        exp: 150,
        freq: |f| {
            if f < 40. {
                (f * 5000. + 10000.) / 10000000.
            } else {
                0.001 / (f - 40. + 1.)
            }
        },
        ..EnemySpec::default()
    },
    EnemySpec {
        waves: 10,
        image: "enemy3.png",
        health: 50.,
        size: ENEMY_SIZE * 1.5,
        exp: 50,
        is_agile: true,
        freq: |f| (f * 5000. + 10000.) / 10000000.,
    },
];

fn spawn_enemies(
    mut commands: Commands,
    query: Query<&Enemy>,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    time: Res<Time>,
    level: Res<Level>,
) {
    let enemy_count = query.iter().count();
    if MAX_ENEMIES <= enemy_count {
        return;
    }
    let difficulty = if let Level::Running { difficulty, .. } = level.as_ref() {
        difficulty
    } else {
        return;
    };

    let window = if let Some(window) = windows.iter().next() {
        window
    } else {
        return;
    };
    let (width, height) = (window.width(), window.height());

    for enemy_spec in ENEMY_SPECS.iter().take(*difficulty + 1) {
        // if (level.timer. / this.waveTime).floor() < enemy_spec.waves {
        //     continue;
        // }

        let num =
            poisson_random(time.delta_seconds() * (0.5 + (enemy_spec.freq)(*difficulty as f32)))
                .min(MAX_ENEMIES - enemy_count);
        for _ in 0..num {
            let axis = rand::random::<bool>();
            let side = rand::random::<bool>();
            let max = if axis { width } else { height };

            let mut x = (rand::random::<f32>() - 0.5) * width;
            let mut y = if side {
                -max / 2. + 10.
            } else {
                max / 2. - 10.
            };

            if axis {
                std::mem::swap(&mut x, &mut y);
            }

            let position = Position(Vec2::new(x, y));

            let mut transform = Transform::default();
            sprite_transform_single(&position, None, &mut transform, 0.05);

            let sprite = commands
                .spawn_bundle(SpriteBundle {
                    texture: asset_server.load(enemy_spec.image),
                    transform: Transform::from_scale(Vec3::ONE * 3.),
                    ..default()
                })
                .id();

            let mut builder = commands.spawn_bundle(TransformBundle {
                local: transform,
                ..default()
            });

            builder
                .insert(position)
                .insert(Velocity(
                    10. * Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5),
                ))
                .insert(Enemy)
                .insert(Health::new(enemy_spec.health))
                .insert(BulletShooter::new(true, 1.))
                .insert(BulletFilter {
                    filter: true,
                    radius: enemy_spec.size,
                    exp: enemy_spec.exp,
                })
                .insert(StageClear)
                .add_child(sprite);
            if enemy_spec.is_agile {
                builder.insert(Rotation(0.));
                builder.insert(Target(None));
                builder.insert(AgileEnemy(true));
            }
        }
    }
}

pub(crate) fn enemy_system(mut query: Query<&mut Velocity, With<Enemy>>, time: Res<Time>) {
    let delta_time = time.delta_seconds();
    for mut velocity in query.iter_mut() {
        velocity.x +=
            (-velocity.x * 0.005 + (rand::random::<f32>() - 0.5) * 15.) * 100. * delta_time;
        velocity.y +=
            (-velocity.y * 0.005 + (rand::random::<f32>() - 0.5) * 15.) * 100. * delta_time;
        velocity.x *= 1. - 0.2 * delta_time;
        velocity.y *= 1. - 0.2 * delta_time;
    }
}

fn agile_enemy_system(
    mut query: Query<
        (
            &mut Velocity,
            &Position,
            &mut Rotation,
            &mut Target,
            &mut BulletShooter,
            &mut AgileEnemy,
        ),
        With<Enemy>,
    >,
    query_towers: Query<(Entity, &Position), With<Tower>>,
) {
    for (mut velocity, position, mut rotation, mut target, mut bullet_shooter, mut agile_enemy) in
        query.iter_mut()
    {
        let new_target = if let Some((target, position)) = target.0.map(|target| {
            (
                target,
                query_towers.get_component::<Position>(target).unwrap(),
            )
        }) {
            Some((target, position))
        } else {
            let new_target = query_towers
                .iter()
                .fold(None, |acc, (tower_entity, enemy_position)| {
                    let this_dist = enemy_position.0.distance(position.0);
                    if let Some((prev_dist, _, _)) = acc {
                        if this_dist < prev_dist {
                            Some((this_dist, tower_entity, enemy_position))
                        } else {
                            acc
                        }
                    } else {
                        Some((this_dist, tower_entity, enemy_position))
                    }
                })
                .map(|res| (res.1, res.2));
            if let Some((new_target, _)) = new_target {
                target.0 = Some(new_target);
            }
            new_target
        };

        use std::f64::consts::PI;
        const ANGLE_SPEED: f64 = PI / 50.;
        const SPEED: f64 = 200.;
        const TOO_CLOSE: f32 = 200.;
        const TOO_FAR: f32 = 500.;

        if let Some((_new_target, tower_position)) = new_target {
            let mut delta = tower_position.0 - position.0;
            if delta.length_squared() < TOO_CLOSE.powf(2.) {
                agile_enemy.0 = true;
            } else if TOO_FAR.powf(2.) < delta.length_squared() {
                agile_enemy.0 = false;
            }

            if agile_enemy.0 {
                delta *= -1.;
            }
            let target_angle = delta.y.atan2(delta.x) as f64;

            let enabled;
            (rotation.0, enabled) = apprach_angle(rotation.0, target_angle, ANGLE_SPEED);
            bullet_shooter.enabled = enabled && !agile_enemy.0;
        }
        velocity.x = (rotation.0.cos() * SPEED) as f32;
        velocity.y = (rotation.0.sin() * SPEED) as f32;
    }
}

/// A pseudo-random number generator distributed in Poisson distribution.
/// It uses Knuth's algorithm, which is not optimal when lambda gets
/// so high.  We probably should use an approximation.
fn poisson_random(lambda: f32) -> usize {
    let l = (-lambda).exp();
    let mut k = 0;
    let mut p = 1.;
    loop {
        k += 1;
        p *= rand::random::<f32>();
        if p <= l {
            break;
        }
    }
    k - 1
}
