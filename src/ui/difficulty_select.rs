use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    tower::{spawn_towers, Tower},
    Level, Scoreboard, StageClear, Textures, MAX_DIFFICULTY,
};

use super::{quit::HOVERED_BUTTON, StartEvent, TEXT_COLOR};

const DIFFICULTY_FONT_SIZE: f32 = 32.0;
const HIGHSCORE_FONT_SIZE: f32 = 16.0;

pub(super) struct DifficultySelectPlugin;

impl Plugin for DifficultySelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                difficulty_button_system,
                difficulty_event_system,
                show_difficulty_buttons_system,
                cleared_icon_system,
                high_score_text_system,
            ),
        );
    }
}

#[derive(Component)]
struct DifficultyButtonFilter;

#[derive(Component)]
struct DifficultyButton {
    color: BackgroundColor,
    difficulty: usize,
}

#[derive(Component)]
struct DifficultyCleared(usize);

#[derive(Component)]
struct HighScoreText(usize);

pub(super) fn add_difficulty_buttons(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::rgb(0.5, 0.5, 0.5).into(),
            ..default()
        })
        .insert(DifficultyButtonFilter)
        .with_children(|parent| {
            for difficulty in 0..MAX_DIFFICULTY {
                let background_color = Color::rgb(
                    0.15 + difficulty as f32 / MAX_DIFFICULTY as f32 * 0.5,
                    0.15,
                    0.15,
                )
                .into();

                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(300.0),
                            height: Val::Px(65.0),
                            margin: UiRect::all(Val::Px(3.)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color,
                        ..default()
                    })
                    .insert(DifficultyButton {
                        color: background_color,
                        difficulty,
                    })
                    .insert(DifficultyButtonFilter)
                    .with_children(|parent| {
                        parent
                            .spawn(ImageBundle {
                                image: asset_server.load("checked.png").into(),
                                focus_policy: FocusPolicy::Pass,
                                ..default()
                            })
                            .insert(DifficultyCleared(difficulty));

                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::FlexStart,
                                    flex_direction: FlexDirection::ColumnReverse,
                                    ..default()
                                },
                                focus_policy: FocusPolicy::Pass,
                                background_color: Color::NONE.into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn(TextBundle {
                                        text: Text::from_section(
                                            &format!("Start Level {}", difficulty),
                                            TextStyle {
                                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                                font_size: DIFFICULTY_FONT_SIZE,
                                                color: TEXT_COLOR,
                                            },
                                        ),
                                        ..default()
                                    })
                                    .insert(DifficultyButtonFilter);

                                parent
                                    .spawn(TextBundle {
                                        text: Text::from_section(
                                            &format!("High score: ?"),
                                            TextStyle {
                                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                                font_size: HIGHSCORE_FONT_SIZE,
                                                color: TEXT_COLOR,
                                            },
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
    textures: Res<Textures>,
) {
    // We only care about the last event if multiple StartEvents have issued
    if let Some(event) = reader.read().last() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        *level = Level::start(event.0);
        scoreboard.score = 0.;

        let towers = query_towers.iter().count();
        if towers == 0 {
            spawn_towers(&mut commands, &asset_server, &textures);
        }
    }
}

fn difficulty_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &DifficultyButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut writer: EventWriter<StartEvent>,
    level: Res<Level>,
    scoreboard: Res<Scoreboard>,
) {
    if let Level::Select = level.as_ref() {
        for (interaction, mut color, difficulty) in interaction_query.iter_mut() {
            match *interaction {
                Interaction::Pressed => {
                    if scoreboard.stages[difficulty.difficulty].unlocked {
                        writer.send(StartEvent(difficulty.difficulty));
                    } else {
                        println!("Difficulty {} is locked!", difficulty.difficulty);
                    }
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
    mut button_query: Query<&mut Visibility, With<DifficultyButtonFilter>>,
    level: Res<Level>,
) {
    for mut visibility in button_query.iter_mut() {
        *visibility = if matches!(level.as_ref(), Level::Select) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn cleared_icon_system(
    mut icon_query: Query<(&mut Visibility, &mut UiImage, &DifficultyCleared)>,
    level: Res<Level>,
    scoreboard: Res<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    for (mut visibility, mut image, cleared) in icon_query.iter_mut() {
        *visibility = if let Level::Select = level.as_ref() {
            if let Some(score) = scoreboard.stages.get(cleared.0) {
                if score.high_score.is_some() {
                    *image = asset_server.load("checked.png").into();
                    Visibility::Inherited
                } else {
                    *image = asset_server.load("locked.png").into();
                    if score.unlocked {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    }
                }
            } else {
                Visibility::Hidden
            }
        } else {
            Visibility::Hidden
        };
    }
}

fn high_score_text_system(
    mut high_score_query: Query<(&mut Text, &HighScoreText)>,
    scoreboard: Res<Scoreboard>,
) {
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
