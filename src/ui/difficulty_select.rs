use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    tower::{spawn_towers, Tower},
    Level, Scoreboard, StageClear, MAX_DIFFICULTY,
};

use super::{quit::HOVERED_BUTTON, StartEvent, TEXT_COLOR};

const DIFFICULTY_FONT_SIZE: f32 = 32.0;
const HIGHSCORE_FONT_SIZE: f32 = 16.0;

pub(super) struct DifficultySelectPlugin;

impl Plugin for DifficultySelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(difficulty_button_system);
        app.add_system(difficulty_event_system);
        app.add_system(show_difficulty_buttons_system);
    }
}

#[derive(Component)]
struct DifficultyButtonFilter;

#[derive(Component)]
struct DifficultyButton {
    color: UiColor,
    difficulty: usize,
}

#[derive(Component)]
struct DifficultyCleared(usize);

#[derive(Component)]
struct HighScoreText(usize);

pub(super) fn add_difficulty_buttons(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            color: Color::rgb(0.5, 0.5, 0.5).into(),
            ..default()
        })
        .insert(DifficultyButtonFilter)
        .with_children(|parent| {
            for difficulty in 0..MAX_DIFFICULTY {
                let color = Color::rgb(
                    0.15 + difficulty as f32 / MAX_DIFFICULTY as f32 * 0.5,
                    0.15,
                    0.15,
                )
                .into();

                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(300.0), Val::Px(65.0)),
                            margin: Rect::all(Val::Px(3.)),
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
                            .spawn_bundle(ImageBundle {
                                image: asset_server.load("checked.png").into(),
                                focus_policy: FocusPolicy::Pass,
                                ..default()
                            })
                            .insert(DifficultyButtonFilter)
                            .insert(DifficultyCleared(difficulty));

                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::FlexStart,
                                    flex_direction: FlexDirection::ColumnReverse,
                                    ..default()
                                },
                                focus_policy: FocusPolicy::Pass,
                                color: Color::NONE.into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            &format!("Start Level {}", difficulty),
                                            TextStyle {
                                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                                font_size: DIFFICULTY_FONT_SIZE,
                                                color: TEXT_COLOR,
                                            },
                                            Default::default(),
                                        ),
                                        ..default()
                                    })
                                    .insert(DifficultyButtonFilter);

                                parent
                                    .spawn_bundle(TextBundle {
                                        text: Text::with_section(
                                            &format!("High score: ?"),
                                            TextStyle {
                                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                                font_size: HIGHSCORE_FONT_SIZE,
                                                color: TEXT_COLOR,
                                            },
                                            Default::default(),
                                        ),
                                        ..default()
                                    })
                                    .insert(DifficultyButtonFilter)
                                    .insert(HighScoreText(difficulty));
                            });
                    });
            }
        });
}

fn difficulty_event_system(
    mut reader: EventReader<StartEvent>,
    mut commands: Commands,
    query: Query<Entity, With<StageClear>>,
    query_towers: Query<(), With<Tower>>,
    mut level: ResMut<Level>,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    // We only care about the last event if multiple StartEvents have issued
    if let Some(event) = reader.iter().last() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        *level = Level::start(event.0);
        scoreboard.score = 0.;

        let towers = query_towers.iter().count();
        if towers == 0 {
            spawn_towers(&mut commands, &asset_server);
        }
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

fn show_difficulty_buttons_system(
    mut button_query: Query<
        (&mut Visibility, Option<&DifficultyCleared>),
        With<DifficultyButtonFilter>,
    >,
    mut high_score_query: Query<(&mut Text, &HighScoreText)>,
    level: Res<Level>,
    scoreboard: Res<Scoreboard>,
) {
    for (mut button, cleared) in button_query.iter_mut() {
        button.is_visible = if let Level::Select = level.as_ref() {
            cleared
                .and_then(|cleared| scoreboard.stages.get(cleared.0))
                .map(|stage| stage.high_score.is_some())
                .unwrap_or(true)
        } else {
            false
        };
    }

    for (mut text, high_score_text) in high_score_query.iter_mut() {
        text.sections[0].value = if let Some(score) = scoreboard
            .stages
            .get(high_score_text.0)
            .and_then(|stage| stage.high_score)
        {
            format!("High score: {}", score)
        } else {
            "High score: None".to_string()
        };
    }
}
