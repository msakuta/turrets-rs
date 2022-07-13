use crate::{BulletFilter, BulletShooter, Health, Position, Rotation, Target, SHOOT_INTERVAL};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Tower {
    pub health_bar: (Entity, Entity),
}

#[derive(Bundle)]
pub(crate) struct TowerBundle {
    position: Position,
    rotation: Rotation,
    tower: Tower,
    health: Health,
    bullet_shooter: BulletShooter,
    target: Target,
    bullet_filter: BulletFilter,
}

impl TowerBundle {
    pub(crate) fn new(
        commands: &mut Commands,
        position: Position,
        rotation: Rotation,
        health: Health,
    ) -> Self {
        Self {
            position,
            rotation,
            tower: Tower {
                health_bar: health_bar(commands),
            },
            health,
            bullet_shooter: BulletShooter(false, rand::random::<f32>() * SHOOT_INTERVAL),
            target: Target(None),
            bullet_filter: BulletFilter(false),
        }
    }
}

const HEALTH_BAR_WIDTH: f32 = 80.;

fn health_bar(commands: &mut Commands) -> (Entity, Entity) {
    (
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 1., 0.25),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, 10.0)),
                    ..default()
                },
                ..default()
            })
            .id(),
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0., 0.),
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, 10.0)),
                    ..default()
                },
                ..default()
            })
            .id(),
    )
}

pub(crate) fn update_health_bar(
    query: Query<(&Position, &Tower, &Health)>,
    mut query_health_bar: Query<&mut Transform>,
) {
    for (position, tower, health) in query.iter() {
        if let Ok(mut bar) = query_health_bar.get_mut(tower.health_bar.0) {
            let factor = health.val / health.max;
            *bar = Transform::from_xyz(
                position.0.x - (1. - factor) * HEALTH_BAR_WIDTH / 2.,
                position.0.y + 50.,
                0.7,
            )
            .with_scale(Vec3::new(factor, 1., 1.));
        }
        if let Ok(mut bar) = query_health_bar.get_mut(tower.health_bar.1) {
            *bar = Transform::from_xyz(position.0.x, position.0.y + 50., 0.5);
        }
    }
}
