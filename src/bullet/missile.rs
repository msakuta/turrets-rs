use super::Bullet;
use crate::{BulletFilter, Health, Position, Rotation, Velocity};
use bevy::prelude::*;
use bevy_polyline::prelude::Polyline;

use super::MISSILE_SPEED;
const MISSILE_ROTATION_SPEED: f32 = std::f32::consts::PI * 0.5;

#[derive(Component)]
pub(crate) struct Missile {
    pub target: Entity,
    pub target_line: Entity,
}

pub(super) fn missile_system(
    time: Res<Time>,
    mut query: Query<(
        &mut Missile,
        &Bullet,
        &mut Rotation,
        &Position,
        &mut Velocity,
    )>,
    health_query: Query<&Health>,
    target_query: Query<(Entity, &Position, &BulletFilter)>,
) {
    for (mut missile, bullet, mut rotation, position, mut velocity) in query.iter_mut() {
        // Search for target if already have none
        if health_query
            .get_component::<Health>(missile.target)
            .map(|health| health.val <= 0.)
            .unwrap_or(true)
        {
            if let Some((_dist, nearest)) = target_query
                .iter()
                .filter(|(_, _, bullet_filter)| bullet_filter.0 == bullet.filter)
                .fold(None, |acc: Option<(f32, Entity)>, cur| {
                    let cur_distance = cur.1 .0.distance(position.0);
                    if acc.map(|acc| cur_distance < acc.0).unwrap_or(true) {
                        Some((cur_distance, cur.0))
                    } else {
                        acc
                    }
                })
            {
                // dbg!(_dist, nearest);
                missile.target = nearest;
            }
        }

        // Guide toward target
        if health_query
            .get_component::<Health>(missile.target)
            .map(|health| 0. < health.val)
            .unwrap_or(false)
        {
            // (this.target !== null && 0 < this.target.health){
            let target_position =
                if let Ok(position) = target_query.get_component::<Position>(missile.target) {
                    position
                } else {
                    continue;
                };
            let delta = target_position.0 - position.0;
            let angle = rapproach(
                rotation.0 as f32,
                delta.y.atan2(delta.x),
                MISSILE_ROTATION_SPEED * time.delta_seconds(),
            );
            rotation.0 = angle as f64;
            velocity.0.x = MISSILE_SPEED * angle.cos();
            velocity.0.y = MISSILE_SPEED * angle.sin();
        }
    }
}

pub(super) fn missile_trail_system(
    missile_query: Query<(&Missile, &Position)>,
    polyline_query: Query<&Handle<Polyline>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    for (missile, position) in missile_query.iter() {
        if let Ok(polyline) = polyline_query.get_component::<Handle<Polyline>>(missile.target_line)
        {
            dbg!(&polyline.id);
            if let Some(polyline) = polylines.get_mut(polyline.id) {
                dbg!(polyline.vertices.len());
                polyline
                    .vertices
                    .push(Vec3::new(position.0.x, position.0.y, 0.));
            }
        }
    }
}

/// Rotation approach
fn rapproach(src: f32, dst: f32, delta: f32) -> f32 {
    return approach(
        src + std::f32::consts::PI,
        dst + std::f32::consts::PI,
        delta,
        std::f32::consts::PI * 2.,
    ) - std::f32::consts::PI;
}

/// Approach src to dst by delta, optionally wrapping around wrap
fn approach(src: f32, dst: f32, delta: f32, wrap: f32) -> f32 {
    if src < dst {
        if dst - src < delta {
            return dst;
        } else if wrap != 0. && wrap / 2. < dst - src {
            let ret = src - delta - ((src - delta) / wrap).floor() * wrap/*fmod(src - delta + wrap, wrap)*/;
            return if src < ret && ret < dst { dst } else { ret };
        }
        return src + delta;
    } else {
        if src - dst < delta {
            return dst;
        } else if wrap != 0. && wrap / 2. < src - dst {
            let ret = src + delta - ((src + delta) / wrap).floor() * wrap/*fmod(src + delta, wrap)*/;
            return if ret < src && dst < ret { dst } else { ret };
        }
        return src - delta;
    }
}
