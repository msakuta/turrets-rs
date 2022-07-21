use crate::{BulletFilter, Health, Position, Rotation, StageClear, Velocity};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use std::collections::VecDeque;

const MAX_TIME_TO_LIVE: f32 = 10.;
const MISSILE_ROTATION_SPEED: f32 = std::f32::consts::PI * 0.5;
pub(super) const MISSILE_SPEED: f32 = 300.;

#[derive(Component)]
pub(crate) struct Missile {
    pub(super) time_to_live: f32,
    pub(super) target: Entity,
    pub(super) trail: Entity,
    pub(super) trail_nodes: VecDeque<Vec2>,
}

impl Missile {
    pub(super) fn new(target: Entity, trail: Entity, position: &Position) -> Self {
        let mut trail_nodes = VecDeque::new();
        trail_nodes.push_back(position.0);
        Self {
            time_to_live: MAX_TIME_TO_LIVE,
            target,
            trail,
            trail_nodes,
        }
    }
}

pub(super) fn missile_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Missile,
        &mut Rotation,
        &Position,
        &mut Velocity,
    )>,
    health_query: Query<&Health>,
    target_query: Query<(Entity, &Position, &BulletFilter)>,
    mut trail_query: Query<&mut Path>,
) {
    let delta_time = time.delta_seconds();
    for (entity, mut missile, mut rotation, position, mut velocity) in query.iter_mut() {
        // Delete expired missiles (and don't forget the trail)
        missile.time_to_live -= delta_time;
        if missile.time_to_live < 0. {
            commands.entity(entity).despawn();
            commands.entity(missile.trail).despawn();
            continue;
        }

        // Search for target if already have none
        if health_query
            .get_component::<Health>(missile.target)
            .map(|health| health.val <= 0.)
            .unwrap_or(true)
        {
            if let Some((_, nearest)) =
                target_query
                    .iter()
                    .fold(None, |acc: Option<(f32, Entity)>, cur| {
                        let cur_distance = cur.1 .0.distance(position.0);
                        if acc.map(|acc| cur_distance < acc.0).unwrap_or(true) {
                            Some((cur_distance, cur.0))
                        } else {
                            acc
                        }
                    })
            {
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

        const MAX_NODES: usize = 50;
        if MAX_NODES < missile.trail_nodes.len() {
            for _ in 0..(missile.trail_nodes.len() - MAX_NODES) {
                missile.trail_nodes.pop_front();
            }
        }

        if missile
            .trail_nodes
            .back()
            .map(|back| 10. < back.distance(position.0))
            .unwrap_or(false)
        {
            missile.trail_nodes.push_back(position.0);
        }

        if let Ok(mut trail) = trail_query.get_component_mut::<Path>(missile.trail) {
            let mut iter = missile.trail_nodes.iter();
            if let Some(first) = iter.next() {
                let mut trail_builder = PathBuilder::new();
                trail_builder.move_to(*first);
                for node in iter {
                    trail_builder.line_to(*node);
                }
                *trail = trail_builder.build();
            }
        }
    }
}

pub(super) fn gen_trail(commands: &mut Commands, position: &Position) -> Entity {
    // Build empty path, which we will replace later
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(position.0);
    let line = path_builder.build();

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &line,
            DrawMode::Stroke(StrokeMode::new(Color::rgba(0.8, 0.8, 0.7, 0.5), 3.0)),
            Transform::default(),
        ))
        .insert(StageClear)
        .id()
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
