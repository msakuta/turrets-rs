use crate::{
    bullet::{BulletShooter, ENEMY_SIZE},
    mouse::tower_not_dragging,
    sprite_transform_single,
    tower::{apprach_angle, MissileShooter, Tower},
    BulletFilter, Health, Level, Position, Rotation, StageClear, Target, Velocity,
};
use bevy::{ecs::system::EntityCommands, prelude::*};

pub(crate) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(tower_not_dragging)
                .with_system(spawn_enemies)
                .with_system(enemy_system)
                .with_system(agile_enemy_system)
                .with_system(sturdy_enemy_system)
                .with_system(missile_enemy_system),
        );
    }
}

#[derive(Component)]
pub(crate) struct Enemy;

#[derive(Component)]
struct AgileEnemy(bool);

#[derive(Component)]
struct SturdyEnemy;

const MAX_ENEMIES: usize = 100;

struct EnemySpec {
    #[allow(dead_code)]
    waves: usize,
    image: &'static str,
    health: f32,
    size: f32,
    sprite_scale: f32,
    exp: usize,
    bullet_damage: f32,
    more_components: fn(&mut EntityCommands),
    freq: fn(f32) -> f32,
}

// I wanted to use `const fn default` to define the default set of parameters,
//  but for some reason I cannot put a member field with a type of function pointer
// `more_components: fn(&mut EntityCommands)` in a constant expression, so I couldn't make it a callback
// that is customizable for each EnemySpec.
// If I try, the compiler throws an error `mutable references are not allowed in constant functions`,
// which doesn't make sense at all. I'm passing a function pointer that takes a mutable reference as
// a parameter, but the function pointer itself is by no means mutable. It is not even a closure.
// If I change the signature of the function pointer to `fn(&EntityCommands)`, it compiles, which
// makes even less sense. In any case, I need a mutable EntityCommands to add components. Thank you.
//
// impl EnemySpec {
//     const fn default() -> Self {
//         Self {
//             waves: 0,
//             image: "",
//             health: 10.,
//             size: ENEMY_SIZE,
//             exp: 10,
//             more_components: |_| (),
//             freq: |f| {
//                 if f < 20. {
//                     (f + 2.) / 20.
//                 } else {
//                     1. / (f - 20. + 1.)
//                 }
//             },
//         }
//     }
// }

const ENEMY_SPECS: [EnemySpec; 5] = [
    EnemySpec {
        waves: 0,
        image: "enemy.png",
        health: 10.,
        size: ENEMY_SIZE,
        sprite_scale: 3.,
        exp: 10,
        bullet_damage: 1.,
        more_components: |_| (),
        freq: |f| {
            if f < 20. {
                (f + 2.) / 20.
            } else {
                1. / (f - 20. + 1.)
            }
        },
        // ..EnemySpec::default()
    },
    EnemySpec {
        waves: 5,
        image: "boss.png",
        health: 150.,
        size: ENEMY_SIZE * 2.,
        sprite_scale: 3.,
        exp: 150,
        bullet_damage: 1.,
        more_components: |_| (),
        freq: |f| {
            if f < 40. {
                (f * 5000. + 10000.) / 10000000.
            } else {
                0.001 / (f - 40. + 1.)
            }
        },
        // ..EnemySpec::default()
    },
    EnemySpec {
        waves: 10,
        image: "enemy3.png",
        health: 50.,
        size: ENEMY_SIZE * 1.2,
        sprite_scale: 3.,
        exp: 50,
        bullet_damage: 1.,
        more_components: |builder| {
            builder.insert(Rotation(0.));
            builder.insert(Target(None));
            builder.insert(AgileEnemy(true));
        },
        freq: |f| (f * 5000. + 10000.) / 10000000.,
    },
    EnemySpec {
        waves: 20,
        image: "enemy4.png",
        health: 500.,
        size: ENEMY_SIZE * 1.5,
        sprite_scale: 3.,
        exp: 500,
        bullet_damage: 1.,
        more_components: |builder| {
            builder.insert(Rotation(0.));
            builder.insert(Target(None));
            builder.insert(SturdyEnemy);
        },
        freq: |f| (f * 5000. + 10000.) / 10000000.,
    },
    EnemySpec {
        waves: 40,
        image: "missile-enemy.png",
        health: 3500.,
        size: ENEMY_SIZE * 1.5,
        sprite_scale: 2.,
        exp: 3500,
        bullet_damage: 3.,
        more_components: |builder| {
            builder.insert(Rotation(0.));
            builder.insert(Target(None));
            builder.insert(MissileShooter);
        },
        freq: |f| (f * 5000. + 10000.) / 20000000.,
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
                    transform: Transform::from_scale(Vec3::splat(enemy_spec.sprite_scale)),
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
                .insert(BulletShooter::new(true, enemy_spec.bullet_damage))
                .insert(BulletFilter {
                    filter: true,
                    radius: enemy_spec.size,
                    exp: enemy_spec.exp,
                })
                .insert(StageClear)
                .add_child(sprite);

            (enemy_spec.more_components)(&mut builder);
        }
    }
}

fn enemy_system(mut query: Query<&mut Velocity, With<Enemy>>, time: Res<Time>) {
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

/// Try to find a closest tower and set its Entity to Target component.
///
/// This enemy will keep targetting the tower until the tower dies.
/// We skip searching the towers until we lose current tower which hopefully
/// helps performance.
fn try_find_tower<'q, 'a>(
    position: &Position,
    target: &mut Target,
    query_towers: &'q Query<(Entity, &'a Position), With<Tower>>,
) -> Option<(Entity, &'q Position)> {
    if let Some((target, position)) = target
        .0
        .and_then(|target| Some((target, query_towers.get_component::<Position>(target).ok()?)))
    {
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
        let new_target = try_find_tower(position, target.as_mut(), &query_towers);

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

fn sturdy_enemy_system(
    mut query: Query<
        (
            &mut Velocity,
            &Position,
            &mut Rotation,
            &mut Target,
            &mut BulletShooter,
        ),
        (With<Enemy>, With<SturdyEnemy>),
    >,
    query_towers: Query<(Entity, &Position), With<Tower>>,
) {
    for (mut velocity, position, mut rotation, mut target, mut bullet_shooter) in query.iter_mut() {
        let new_target = try_find_tower(position, target.as_mut(), &query_towers);

        use std::f64::consts::PI;
        const ANGLE_SPEED: f64 = PI / 150.;
        const SPEED: f64 = 50.;
        const TOO_CLOSE: f32 = 250.;

        if let Some((_new_target, tower_position)) = new_target {
            let delta = tower_position.0 - position.0;
            let target_angle = delta.y.atan2(delta.x) as f64;

            (rotation.0, bullet_shooter.enabled) =
                apprach_angle(rotation.0, target_angle, ANGLE_SPEED);
            if TOO_CLOSE.powf(2.) < delta.length_squared() {
                velocity.x = (rotation.0.cos() * SPEED) as f32;
                velocity.y = (rotation.0.sin() * SPEED) as f32;
            } else {
                **velocity = Vec2::ZERO;
            }
        } else {
            **velocity = Vec2::ZERO;
        }
    }
}

fn missile_enemy_system(
    mut query: Query<
        (
            &mut Velocity,
            &Position,
            &mut Rotation,
            &mut Target,
            &mut BulletShooter,
        ),
        (With<Enemy>, With<MissileShooter>),
    >,
    query_towers: Query<(Entity, &Position), With<Tower>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    for (mut velocity, position, mut rotation, mut target, mut bullet_shooter) in query.iter_mut() {
        let new_target = try_find_tower(position, target.as_mut(), &query_towers);

        use std::f64::consts::PI;
        const ANGLE_SPEED: f64 = PI / 0.;
        const SPEED: f64 = 50.;
        const TOO_CLOSE: f32 = 250.;

        if let Some((_new_target, tower_position)) = new_target {
            let delta = tower_position.0 - position.0;
            let target_angle = delta.y.atan2(delta.x) as f64;

            (rotation.0, bullet_shooter.enabled) =
                apprach_angle(rotation.0, target_angle, ANGLE_SPEED * delta_time as f64);
            if TOO_CLOSE.powf(2.) < delta.length_squared() {
                velocity.x = (rotation.0.cos() * SPEED) as f32;
                velocity.y = (rotation.0.sin() * SPEED) as f32;
            } else {
                **velocity = Vec2::ZERO;
            }
        } else {
            **velocity = Vec2::ZERO;
        }
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
