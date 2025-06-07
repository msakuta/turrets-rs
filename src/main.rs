mod bullet;
mod enemy;
mod mouse;
mod save;
mod tower;
mod ui;

use crate::{
    bullet::BulletPlugin,
    enemy::{Enemy, EnemyPlugin},
    mouse::{tower_not_dragging, MousePlugin},
    save::{load_game, save_game, SaveGameEvent},
    tower::{spawn_towers, update_health_bar, Tower, TowerPlugin},
    ui::UIPlugin,
};
use bevy::prelude::*;
use mouse::SelectedTower;
use serde::{Deserialize, Serialize};
use tower::TempEnt;
use ui::{not_paused, PauseState};

const MAX_DIFFICULTY: usize = 5;

fn main() {
    App::new()
        .add_event::<ClearEvent>()
        .add_event::<SaveGameEvent>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.2)))
        .add_plugins((
            DefaultPlugins,
            UIPlugin,
            TowerPlugin,
            BulletPlugin,
            MousePlugin,
            EnemyPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (time_level, timeout_level, linear_motion, animate_sprite).run_if(can_update),
        )
        .add_systems(
            Update,
            (reset_game, sprite_transform, update_health_bar, save_game),
        )
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

#[derive(Resource)]
struct Textures {
    small_explosion: Handle<TextureAtlas>,
    large_explosion: Handle<TextureAtlas>,
    small_explosion_blue: Handle<TextureAtlas>,
    tower_circle_material: Handle<ColorMaterial>,
    trail_material: Handle<ColorMaterial>,
}

#[derive(Serialize, Deserialize, Resource)]
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

#[derive(Resource)]
enum Level {
    Select,
    Running { difficulty: usize, timer: Timer },
}

impl Level {
    fn start(difficulty: usize) -> Self {
        Self::Running {
            difficulty,
            timer: Timer::from_seconds(60., TimerMode::Once),
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

    fn _difficulty(&self) -> usize {
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
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(size, size),
            columns,
            1,
            None,
            None,
        );
        texture_atlases.add(texture_atlas)
    };
    let textures = Textures {
        small_explosion: gen_texture_handle("explode.png", 16., 8),
        large_explosion: gen_texture_handle("explode2.png", 32., 6),
        small_explosion_blue: gen_texture_handle("explode-blue.png", 16., 8),
        tower_circle_material: asset_server.add(ColorMaterial {
            color: Color::rgba(0.0, 0.8, 0.8, 1.),
            ..default()
        }),
        trail_material: asset_server.add(ColorMaterial {
            color: Color::rgba(0.8, 0.8, 0.7, 0.5),
            ..default()
        }),
    };

    let mut scoreboard = Scoreboard::default();
    load_game(&mut commands, &asset_server, &mut scoreboard, &textures);

    commands.insert_resource(textures);
    commands.insert_resource(scoreboard);
    commands.insert_resource(Level::Select);
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
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

#[derive(Event)]
struct ClearEvent;

fn timeout_level(level: ResMut<Level>, mut writer: EventWriter<ClearEvent>) {
    if level.timer_finished() {
        writer.send(ClearEvent);
    }
}

fn reset_game(
    mut commands: Commands,
    mut level: ResMut<Level>,
    query: Query<Entity, With<StageClear>>,
    mut query_towers: Query<&mut Health, With<Tower>>,
    mut reader: EventReader<ClearEvent>,
    mut writer: EventWriter<SaveGameEvent>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
    textures: Res<Textures>,
) {
    if reader.read().next().is_some() {
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
            spawn_towers(&mut commands, &asset_server, textures.as_ref());
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
        Option<&TempEnt>,
    )>,
) {
    for (position, rotation, mut transform, temp_ent) in query.iter_mut() {
        sprite_transform_single(
            position,
            rotation,
            transform.as_mut(),
            if temp_ent.is_some() { 0.75 } else { 0.05 },
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

fn can_update(selected_tower: Res<SelectedTower>, pause_state: Res<PauseState>) -> bool {
    tower_not_dragging(selected_tower) && not_paused(pause_state)
}
