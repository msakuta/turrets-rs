mod difficulty_select;
mod quit;
mod scoreboard;
mod tower_palette;

use bevy::prelude::*;

use self::{
    difficulty_select::{
        add_difficulty_buttons, difficulty_button_system, difficulty_event_system,
        show_difficulty_buttons_system,
    },
    quit::{add_quit_button, quit_button_system, quit_event_system, show_quit_button_system},
    scoreboard::{add_scoreboard, update_credits, update_level, update_scoreboard},
    tower_palette::{add_palette_buttons, palette_mouse_system},
};
use crate::{mouse::SelectedTower, tower::TowerScore, Health, Level};

pub(crate) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartEvent>();
        app.add_event::<QuitEvent>();
        app.add_startup_system(build_ui);
        app.add_system(update_progress_bar);
        app.add_system(update_level);
        app.add_system(update_scoreboard);
        app.add_system(update_credits);
        app.add_system(update_tower_scoreboard);
        app.add_system(update_tower_health);
        app.add_system(palette_mouse_system);
        app.add_system(quit_event_system);
        app.add_system(quit_button_system);
        app.add_system(show_quit_button_system);
        app.add_system(difficulty_button_system);
        app.add_system(show_difficulty_buttons_system);
        app.add_system(difficulty_event_system);
    }
}

struct StartEvent(usize);
struct QuitEvent;

#[derive(Component)]
struct TowerHealthText;

#[derive(Component)]
struct TowerScoreText;

#[derive(Component)]
struct ProgressBar;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const PADDING: f32 = 5.;
const PADDING_PX: Val = Val::Px(PADDING);
const TEXT_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const STATUS_FONT_SIZE: f32 = 20.0;

const BUTTON_HEIGHT: f32 = 65.0;

const PALETTE_SIZE: f32 = 64.;

fn build_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    add_scoreboard(&mut commands, &asset_server);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(80.0), Val::Px(20.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    ..default()
                },
                border: Rect::all(Val::Px(2.0)),
                ..default()
            },
            color: Color::rgb(0.4, 0.4, 1.0).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    color: Color::rgb(0.8, 0.8, 1.0).into(),
                    ..default()
                })
                .insert(ProgressBar);
        });

    add_quit_button(&mut commands, &asset_server);
    add_palette_buttons(&mut commands, &asset_server);
    add_difficulty_buttons(&mut commands, &asset_server);
    add_status_panel(&mut commands, &asset_server);
}

fn add_status_panel(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(PADDING * 2. + BUTTON_HEIGHT),
                    right: Val::Px(PADDING * 2. + PALETTE_SIZE),
                    ..default()
                },
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Health: ".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: STATUS_FONT_SIZE,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: STATUS_FONT_SIZE,
                                    color: SCORE_COLOR,
                                },
                            },
                        ],
                        ..default()
                    },
                    ..default()
                })
                .insert(TowerHealthText);

            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Kills: ".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: STATUS_FONT_SIZE,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: STATUS_FONT_SIZE,
                                    color: SCORE_COLOR,
                                },
                            },
                        ],
                        ..default()
                    },
                    ..default()
                })
                .insert(TowerScoreText);
        });
}

fn update_progress_bar(level: Res<Level>, mut query: Query<&mut Style, With<ProgressBar>>) {
    // println!("dur: {}", level.timer.elapsed_secs());
    let bar = query.get_single_mut();
    // println!("bar: {bar:?}");
    if let Ok(mut bar) = bar {
        if let Level::Running { timer, .. } = level.as_ref() {
            bar.size.width = Val::Percent(timer.percent() * 100.);
        }
    }
}

fn update_tower_scoreboard(
    selected_tower: Res<SelectedTower>,
    tower_score_query: Query<&TowerScore>,
    mut text_query: Query<&mut Text, With<TowerScoreText>>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        if let Some(selected_tower) = selected_tower
            .tower
            .and_then(|tower| tower_score_query.get_component::<TowerScore>(tower).ok())
        {
            text.sections[1].value = format!("{:?}", selected_tower.kills);
        } else {
            text.sections[1].value = "".to_string();
        }
    }
}

fn update_tower_health(
    selected_tower: Res<SelectedTower>,
    tower_health_query: Query<&Health>,
    mut text_query: Query<&mut Text, With<TowerHealthText>>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        if let Some(health) = selected_tower
            .tower
            .and_then(|tower| tower_health_query.get_component::<Health>(tower).ok())
        {
            text.sections[1].value = format!("{}/{}", health.val, health.max);
        } else {
            text.sections[1].value = "".to_string();
        }
    }
}
