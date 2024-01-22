use bevy::prelude::*;

use crate::{Level, Scoreboard};

use super::{PADDING_PX, SCOREBOARD_FONT_SIZE, SCORE_COLOR, TEXT_COLOR};

#[derive(Component)]
pub(super) struct LevelText;

#[derive(Component)]
pub(super) struct ScoreText;

#[derive(Component)]
pub(super) struct CreditsText;

pub(super) fn add_scoreboard(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    // Scoreboard
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                position_type: PositionType::Absolute,
                top: PADDING_PX,
                left: PADDING_PX,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            background_color: Color::rgba(0., 0., 0., 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
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
                    ..default()
                })
                .insert(LevelText);

            parent
                .spawn(TextBundle {
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
                    ..default()
                })
                .insert(ScoreText);

            parent
                .spawn(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Credit: ".to_string(),
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
                    ..default()
                })
                .insert(CreditsText);
        });
}

pub(super) fn update_level(level: Res<Level>, mut query: Query<&mut Text, With<LevelText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[1].value = if let Level::Running { difficulty, .. } = level.as_ref() {
            format!("{}", difficulty)
        } else {
            "-".to_string()
        }
    }
}

pub(super) fn update_scoreboard(
    scoreboard: Res<Scoreboard>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[1].value = format!("{}", scoreboard.score);
    }
}

pub(super) fn update_credits(
    scoreboard: Res<Scoreboard>,
    mut query: Query<&mut Text, With<CreditsText>>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[1].value = format!("${}", scoreboard.credits);
    }
}
