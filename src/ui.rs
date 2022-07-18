use bevy::prelude::*;

use crate::{
    mouse::SelectedTower,
    tower::{spawn_towers, Tower, TowerHealthBar, TowerScore},
    Bullet, Enemy, Level, Scoreboard, Timeout,
};

pub(crate) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StartEvent(false));
        app.add_startup_system(build_ui);
        app.add_system(update_progress_bar);
        app.add_system(update_scoreboard);
        app.add_system(update_tower_scoreboard);
        app.add_system(button_system);
        app.add_system(start_event_system);
    }
}

#[derive(Component)]
struct ScoreText;

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
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn build_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Scoreboard
    commands
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
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: SCOREBOARD_TEXT_PADDING,
                    left: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(ScoreText);

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

    // Start button
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
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Start",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                    Default::default(),
                ),
                ..default()
            });
        });

    commands
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
            style: Style {
                // justify_content: JustifyContent::Center,
                // align_items: AlignItems::Center,
                // flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(80.0),
                    right: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(TowerScoreText);
}

fn update_progress_bar(level: Res<Level>, mut query: Query<&mut Style, With<ProgressBar>>) {
    // println!("dur: {}", level.timer.elapsed_secs());
    let bar = query.get_single_mut();
    // println!("bar: {bar:?}");
    if let Ok(mut bar) = bar {
        if let Level::Running { timer } = level.as_ref() {
            bar.size.width = Val::Percent(timer.percent() * 100.);
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
    mut text_query: Query<(&mut Text, &mut Visibility), With<TowerScoreText>>,
) {
    if let Ok((mut text, mut visibility)) = text_query.get_single_mut() {
        if let Some(selected_tower) = selected_tower
            .tower
            .and_then(|tower| tower_score_query.get_component::<TowerScore>(tower).ok())
        {
            visibility.is_visible = true;
            text.sections[1].value = format!("{:?}", selected_tower.kills);
        } else {
            visibility.is_visible = false;
        }
    }
}

struct StartEvent(bool);

fn start_event_system(
    mut event: ResMut<StartEvent>,
    mut commands: Commands,
    query: Query<(
        Entity,
        Option<&Tower>,
        Option<&Enemy>,
        Option<&Bullet>,
        Option<&Timeout>,
        Option<&TowerHealthBar>,
    )>,
    mut level: ResMut<Level>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    if event.0 {
        println!("Clearing");
        event.0 = false;
        for query_res in query.iter() {
            match query_res {
                (entity, Some(_), _, _, _, _)
                | (entity, _, Some(_), _, _, _)
                | (entity, _, _, Some(_), _, _)
                | (entity, _, _, _, Some(_), _)
                | (entity, _, _, _, _, Some(_)) => commands.entity(entity).despawn(),
                _ => (),
            }
        }
        *level = Level::start();
        scoreboard.score = 0.;
        spawn_towers(&mut commands, &asset_server);
    }
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut event: ResMut<StartEvent>,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                event.0 = true;
                *color = PRESSED_BUTTON.into();
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
