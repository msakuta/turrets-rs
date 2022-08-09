use crate::{
    bullet::SHOOT_INTERVAL, mouse::SelectedTower, BulletFilter, BulletShooter, Health, Level,
    Position, StageClear, Velocity,
};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Enemy;

const MAX_ENEMIES: usize = 100;

pub(crate) fn spawn_enemies(
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
    let num = poisson_random(time.delta_seconds() * (0.5 + *difficulty as f32))
        .min(MAX_ENEMIES - enemy_count);
    if num == 0 {
        return;
    }

    let window = if let Some(window) = windows.iter().next() {
        window
    } else {
        return;
    };
    let (width, height) = (window.width(), window.height());

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

        commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("enemy.png"),
                ..default()
            })
            .insert(Position(Vec2::new(x, y)))
            .insert(Velocity(
                10. * Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5),
            ))
            .insert(Enemy)
            .insert(Health::new(3.))
            .insert(BulletShooter(true, SHOOT_INTERVAL))
            .insert(BulletFilter(true))
            .insert(StageClear);
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
