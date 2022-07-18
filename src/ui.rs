use bevy::prelude::*;

use crate::{
    mouse::SelectedTower,
    tower::{spawn_towers, TowerScore},
    Health, Level, Scoreboard, StageClear,
};

pub(crate) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StartEvent(false, 0));
        app.insert_resource(QuitEvent(false));
        app.add_startup_system(build_ui);
        app.add_system(update_progress_bar);
        app.add_system(update_level);
        app.add_system(update_scoreboard);
        app.add_system(update_tower_scoreboard);
        app.add_system(update_tower_health);
        app.add_system(quit_event_system);
        app.add_system(quit_button_system);
        app.add_system(difficulty_button_system);
        app.add_system(show_quit_button_system);
        app.add_system(show_difficulty_buttons_system);
        app.add_system(difficulty_event_system);
    }
}

#[derive(Component)]
struct QuitButtonFilter;

#[derive(Component)]
struct DifficultyButton {
    color: UiColor,
    difficulty: usize,
}

#[derive(Component)]
struct DifficultyButtonFilter;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TowerHealthText;

#[derive(Component)]
struct TowerScoreText;

#[derive(Component)]
struct ProgressBar;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const STATUS_FONT_SIZE: f32 = 20.0;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

fn build_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Scoreboard
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                position_type: PositionType::Absolute,
                position: Rect {
                    top: SCOREBOARD_TEXT_PADDING,
                    left: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
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
                                value: "Level: ".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: SCOREBOARD_FONT_SIZE,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: SCOREBOARD_FONT_SIZE,
                                    color: SCORE_COLOR,
                                },
                            },
                        ],
                        ..default()
                    },
                    // style: Style {
                    //     position_type: PositionType::Absolute,
                    //     position: Rect {
                    //         top: SCOREBOARD_TEXT_PADDING,
                    //         left: SCOREBOARD_TEXT_PADDING,
                    //         ..default()
                    //     },
                    //     ..default()
                    // },
                    ..default()
                })
                .insert(LevelText);

            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Score: ".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: SCOREBOARD_FONT_SIZE,
                                    color: TEXT_COLOR,
                                },
                            },
                            TextSection {
                                value: "".to_string(),
                                style: TextStyle {
                                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                    font_size: SCOREBOARD_FONT_SIZE,
                                    color: SCORE_COLOR,
                                },
                            },
                        ],
                        ..default()
                    },
                    // style: Style {
                    //     position_type: PositionType::Absolute,
                    //     position: Rect {
                    //         top: SCOREBOARD_TEXT_PADDING,
                    //         left: SCOREBOARD_TEXT_PADDING,
                    //         ..default()
                    //     },
                    //     ..default()
                    // },
                    ..default()
                })
                .insert(ScoreText);
        });

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

    // Quit button
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: SCOREBOARD_TEXT_PADDING,
                    right: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(QuitButtonFilter)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Quit",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: TEXT_COLOR,
                        },
                        Default::default(),
                    ),
                    ..default()
                })
                .insert(QuitButtonFilter);
        });

    // Difficulty buttons
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            ..default()
        })
        .insert(DifficultyButtonFilter)
        .with_children(|parent| {
            for difficulty in 0..3 {
                let color = Color::rgb(0.15 + difficulty as f32 / 3. * 0.85, 0.15, 0.15).into();
                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(300.0), Val::Px(65.0)),
                            margin: Rect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        color,
                        ..default()
                    })
                    .insert(DifficultyButton { color, difficulty })
                    .insert(DifficultyButtonFilter)
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    &format!("Start Level {}", difficulty),
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: SCOREBOARD_FONT_SIZE,
                                        color: TEXT_COLOR,
                                    },
                                    Default::default(),
                                ),
                                ..default()
                            })
                            .insert(DifficultyButtonFilter);
                    });
            }
        });

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(80.0),
                    right: SCOREBOARD_TEXT_PADDING,
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

fn update_level(level: Res<Level>, mut query: Query<&mut Text, With<LevelText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[1].value = if let Level::Running { difficulty, .. } = level.as_ref() {
            format!("{}", difficulty)
        } else {
            "-".to_string()
        }
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[1].value = format!("{}", scoreboard.score);
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

struct StartEvent(bool, usize);
struct QuitEvent(bool);

fn quit_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>, With<QuitButtonFilter>),
    >,
    mut event: ResMut<QuitEvent>,
    level: Res<Level>,
) {
    if let Level::Select = level.as_ref() {
        return;
    }
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                event.0 = true;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn quit_event_system(
    mut event: ResMut<QuitEvent>,
    mut commands: Commands,
    query: Query<Entity, With<StageClear>>,
    mut level: ResMut<Level>,
) {
    if event.0 {
        event.0 = false;
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        *level = Level::Select;
    }
}

fn difficulty_event_system(
    mut event: ResMut<StartEvent>,
    mut commands: Commands,
    query: Query<Entity, With<StageClear>>,
    mut level: ResMut<Level>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    if event.0 {
        event.0 = false;
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        *level = Level::start(event.1);
        scoreboard.score = 0.;
        spawn_towers(&mut commands, &asset_server);
    }
}

fn difficulty_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &DifficultyButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut event: ResMut<StartEvent>,
    level: Res<Level>,
) {
    if let Level::Select = level.as_ref() {
        for (interaction, mut color, difficulty) in interaction_query.iter_mut() {
            match *interaction {
                Interaction::Clicked => {
                    event.0 = true;
                    event.1 = difficulty.difficulty;
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                }
                Interaction::None => {
                    *color = difficulty.color;
                }
            }
        }
    }
}

fn show_quit_button_system(
    mut button_query: Query<&mut Visibility, With<QuitButtonFilter>>,
    level: Res<Level>,
) {
    for mut button in button_query.iter_mut() {
        button.is_visible = if let Level::Select = level.as_ref() {
            false
        } else {
            true
        };
    }
}

fn show_difficulty_buttons_system(
    mut button_query: Query<&mut Visibility, With<DifficultyButtonFilter>>,
    level: Res<Level>,
) {
    for mut button in button_query.iter_mut() {
        button.is_visible = if let Level::Select = level.as_ref() {
            true
        } else {
            false
        };
    }
}
