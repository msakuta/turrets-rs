use crate::{
    BulletFilter, BulletShooter, Health, Position, Rotation, Target, Tower, SHOOT_INTERVAL,
};
use bevy::prelude::*;

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
    pub(crate) fn new(position: Position, rotation: Rotation, health: Health) -> Self {
        Self {
            position,
            rotation,
            tower: Tower,
            health,
            bullet_shooter: BulletShooter(false, rand::random::<f32>() * SHOOT_INTERVAL),
            target: Target(None),
            bullet_filter: BulletFilter(false),
        }
    }
}
