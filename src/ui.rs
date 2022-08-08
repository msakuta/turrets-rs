use bevy::prelude::*;

use crate::{
    mouse::{MouseCursor, SelectedTower},
    tower::{spawn_towers, spawn_turret, Tower, TowerScore},
    ClearEvent, Health, Level, Position, Scoreboard, StageClear,
};

pub(crate) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartEvent>();
        app.add_event::<QuitEvent>();
        app.add_startup_system(build_ui);
        app.add_system(update_progress_bar);
        app.add_system(update_level);
        app.add_system(update_scoreboard);
        app.add_system(update_tower_scoreboard);
        app.add_system(update_tower_health);
        app.add_system(palette_mouse_system);
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
const PADDING: f32 = 5.;
const PADDING_PX: Val = Val::Px(PADDING);
const TEXT_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const STATUS_FONT_SIZE: f32 = 20.0;

const BUTTON_HEIGHT: f32 = 65.0;
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

const PALETTE_SIZE: f32 = 64.;

fn build_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Scoreboard
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                position_type: PositionType::Absolute,
                position: Rect {
                    top: PADDING_PX,
                    left: PADDING_PX,
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
                    //         top: PADDING_PX,
                    //         left: PADDING_PX,
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
                    //         top: PADDING_PX,
                    //         left: PADDING_PX,
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

    add_quit_button(&mut commands, &asset_server);
    add_palette_buttons(&mut commands, &asset_server);
    add_difficulty_buttons(&mut commands, &asset_server);
    add_status_panel(&mut commands, &asset_server);
}

fn add_quit_button(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(BUTTON_HEIGHT)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: PADDING_PX,
                    right: PADDING_PX,
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
}

fn add_palette_buttons(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(PALETTE_SIZE), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(PADDING * 2. + BUTTON_HEIGHT),
                    right: PADDING_PX,
                    ..default()
                },
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            add_tower_icon(parent, &asset_server, "turret.png");
            add_tower_icon(parent, &asset_server, "shotgun.png");
            add_tower_icon(parent, &asset_server, "healer.png");
            add_tower_icon(parent, &asset_server, "missile-tower.png");
        });
}

#[derive(Component)]
struct TowerPalette;

fn add_tower_icon(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>, file: &str) {
    parent
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(PALETTE_SIZE), Val::Px(PALETTE_SIZE)),
                border: Rect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(PALETTE_SIZE), Val::Px(PALETTE_SIZE)),
                        ..default()
                    },
                    image: UiImage::from(asset_server.load(file)).into(),
                    ..default()
                })
                .insert(Interaction::default())
                .insert(TowerPalette);
        });
}

fn palette_mouse_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    level: Res<Level>,
    mut query: Query<(&mut Transform, &mut Visibility), With<MouseCursor>>,
    query_towers: Query<(&Interaction, &Parent), (Changed<Interaction>, With<TowerPalette>)>,
    mut query_ui_color: Query<&mut UiColor>,
    btn: Res<Input<MouseButton>>,
    mut selected_tower: ResMut<SelectedTower>,
) {
    if selected_tower.dragging || !level._is_running() {
        return;
    }

    let window = if let Some(window) = windows.iter().next() {
        window
    } else {
        return;
    };
    let mouse = window.cursor_position();

    if let Some(((mut cursor_transform, mut visibility), mouse_position)) =
        query.get_single_mut().ok().zip(mouse)
    {
        let (width, height) = (window.width(), window.height());
        let mouse_screen = Vec2::new(
            mouse_position.x - width / 2.,
            mouse_position.y - height / 2.,
        );
        // println!("Mouse: {:?} -> {:?}", mouse_position, mouse_screen);
        for (interaction, parent) in query_towers.iter() {
            if let Ok(mut ui_color) = query_ui_color.get_component_mut::<UiColor>(**parent) {
                // println!("Has ui_color: {ui_color:?}");
                match *interaction {
                    Interaction::Clicked => {
                        println!("Clicked tower palette at {mouse_screen:?}");
                        *ui_color = Color::rgba(1., 0., 1., 0.75).into();

                        visibility.is_visible = true;
                        *cursor_transform =
                            Transform::from_xyz(mouse_screen.x, mouse_screen.y, 0.2)
                                .with_scale(Vec3::new(2., 2., 1.));

                        let tower = spawn_turret(&mut commands, &asset_server, mouse_screen, 0.);
                        selected_tower.tower = Some(tower);

                        if btn.just_pressed(MouseButton::Left) {
                            selected_tower.dragging = true;
                        }
                        return;
                    }
                    Interaction::Hovered => {
                        *ui_color = Color::rgba(0.5, 0., 0., 0.5).into();
                    }
                    Interaction::None => {
                        *ui_color = Color::rgba(0.0, 0., 0., 0.5).into();
                    }
                }
            }
        }
    }
    if btn.just_released(MouseButton::Left) {
        selected_tower.dragging = false;
    }
}

fn add_difficulty_buttons(commands: &mut Commands, asset_server: &Res<AssetServer>) {
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

struct StartEvent(usize);
struct QuitEvent;

fn quit_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>, With<QuitButtonFilter>),
    >,
    mut writer: EventWriter<QuitEvent>,
    level: Res<Level>,
) {
    if let Level::Select = level.as_ref() {
        return;
    }
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                writer.send(QuitEvent);
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
    mut commands: Commands,
    query: Query<Entity, With<StageClear>>,
    mut level: ResMut<Level>,
    mut reader: EventReader<QuitEvent>,
    mut writer: EventWriter<ClearEvent>,
) {
    if reader.iter().last().is_some() {
        println!("Received QuitEvent");
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        *level = Level::Select;
        writer.send(ClearEvent);
    }
}

fn difficulty_event_system(
    mut reader: EventReader<StartEvent>,
    mut commands: Commands,
    query: Query<Entity, With<StageClear>>,
    mut level: ResMut<Level>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    // We only care about the last event if multiple StartEvents have issued
    if let Some(event) = reader.iter().last() {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
        *level = Level::start(event.0);
        scoreboard.score = 0.;
        spawn_towers(&mut commands, &asset_server);
    }
}

fn difficulty_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &DifficultyButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut writer: EventWriter<StartEvent>,
    level: Res<Level>,
) {
    if let Level::Select = level.as_ref() {
        for (interaction, mut color, difficulty) in interaction_query.iter_mut() {
            match *interaction {
                Interaction::Clicked => {
                    writer.send(StartEvent(difficulty.difficulty));
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
