mod bullet;
mod enemy;
mod mouse;
mod save;
mod tower;
mod ui;

use crate::{
    bullet::BulletPlugin,
    enemy::{enemy_system, spawn_enemies, Enemy},
    mouse::{tower_not_dragging, MousePlugin},
    save::SaveGameEvent,
    tower::{spawn_towers, update_health_bar, Timeout, Tower, TowerPlugin},
    ui::UIPlugin,
};
use bevy::prelude::*;
use save::{load_game, save_game};
use serde::{Deserialize, Serialize};

const MAX_DIFFICULTY: usize = 5;

fn main() {
    App::new()
        .add_event::<ClearEvent>()
        .add_event::<SaveGameEvent>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .add_plugins(DefaultPlugins)
        .add_plugin(UIPlugin)
        .add_plugin(TowerPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(MousePlugin)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(tower_not_dragging)
                .with_system(time_level)
                .with_system(reset_game)
                .with_system(spawn_enemies)
                .with_system(enemy_system)
                .with_system(linear_motion)
                .with_system(animate_sprite),
        )
        .add_system(sprite_transform)
        .add_system(update_health_bar)
        .add_system(save_game)
        .run();
}

/// Marker component for objects that should be cleared on starting game
#[derive(Component)]
struct StageClear;

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
struct Position(Vec2);

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
struct Rotation(f64);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Target(Option<Entity>);

#[derive(Component)]
struct BulletFilter {
    filter: bool,
    radius: f32,
    exp: usize,
}

#[derive(Component, Serialize, Deserialize)]
struct Health {
    val: f32,
    max: f32,
}

impl Health {
    const fn new(val: f32) -> Self {
        Self { val, max: val }
    }
}

#[derive(Component, Deref, DerefMut)]
struct Explosion(Timer);

// #[derive(Component)]
struct Textures {
    small_explosion: Handle<TextureAtlas>,
    large_explosion: Handle<TextureAtlas>,
}

#[derive(Serialize, Deserialize)]
struct Scoreboard {
    score: f64,
    credits: f64,
    #[serde(default)]
    stages: Vec<StageScore>,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            score: 0.,
            credits: 0.,
            stages: Self::stage_scores(),
        }
    }
}

impl Scoreboard {
    fn stage_scores() -> Vec<StageScore> {
        (0..MAX_DIFFICULTY)
            .map(|difficulty| StageScore {
                unlocked: difficulty <= 0,
                high_score: None,
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
struct StageScore {
    unlocked: bool,
    high_score: Option<f64>,
}

enum Level {
    Select,
    Running { difficulty: usize, timer: Timer },
}

impl Level {
    fn start(difficulty: usize) -> Self {
        Self::Running {
            difficulty,
            timer: Timer::from_seconds(60., true),
        }
    }

    fn timer_finished(&self) -> bool {
        match self {
            Self::Select => false,
            Self::Running { timer, .. } => timer.just_finished(),
        }
    }

    fn _is_running(&self) -> bool {
        if let Self::Running { .. } = self {
            true
        } else {
            false
        }
    }

    fn difficulty(&self) -> usize {
        if let Self::Running { difficulty, .. } = self {
            *difficulty
        } else {
            0
        }
    }
}

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

    let mut scoreboard = Scoreboard::default();
    load_game(&mut commands, &asset_server, &mut scoreboard);

    commands.insert_resource(scoreboard);
    commands.insert_resource(Level::Select);
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("cliff-crop.png"),
        transform: Transform::from_scale(Vec3::ONE * 2.),
        ..default()
    });

    // spawn_towers(&mut commands, &asset_server);
}

fn time_level(mut level: ResMut<Level>, time: Res<Time>) {
    if let Level::Running { timer, .. } = level.as_mut() {
        timer.tick(time.delta());
    }
}

struct ClearEvent;

fn reset_game(
    mut commands: Commands,
    mut level: ResMut<Level>,
    query: Query<Entity, With<StageClear>>,
    mut query_towers: Query<&mut Health, With<Tower>>,
    mut writer: EventWriter<SaveGameEvent>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    if level.timer_finished() {
        println!("Round finished!");
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Restore full health on stage clear
        let mut any_tower = false;
        for mut tower_health in query_towers.iter_mut() {
            tower_health.val = tower_health.max;
            any_tower = true;
        }

        if !any_tower {
            spawn_towers(&mut commands, &asset_server);
            scoreboard.score = 0.;
        } else if let Level::Running { difficulty, .. } = level.as_ref() {
            let score = scoreboard.score;
            let high_score = &mut scoreboard.stages[*difficulty].high_score;
            if high_score
                .map(|high_score| high_score < score)
                .unwrap_or(true)
            {
                *high_score = Some(score);
            }
            if let Some(next_difficulty) = scoreboard.stages.get_mut(*difficulty + 1) {
                next_difficulty.unlocked = true;
            }
        }

        *level = Level::Select;

        writer.send(SaveGameEvent);
    }
}

fn linear_motion(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in query.iter_mut() {
        position.0 += velocity.0 * time.delta_seconds();
    }
}

fn sprite_transform(
    mut query: Query<(
        &Position,
        Option<&Rotation>,
        &mut Transform,
        Option<&Timeout>,
    )>,
) {
    for (position, rotation, mut transform, timeout) in query.iter_mut() {
        sprite_transform_single(
            position,
            rotation,
            transform.as_mut(),
            if timeout.is_some() { 0.1 } else { 0.05 },
        );
    }
}

fn sprite_transform_single(
    position: &Position,
    rotation: Option<&Rotation>,
    transform: &mut Transform,
    z: f32,
) {
    let mut trans = Transform::from_xyz(position.0.x, position.0.y, z);
    if let Some(rotation) = rotation {
        trans = trans.with_rotation(Quat::from_rotation_z(rotation.0 as f32));
    }
    *transform = trans;
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
