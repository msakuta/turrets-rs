use bevy::prelude::*;

use crate::{tower::spawn_towers, Level, Scoreboard, StageClear};

use super::{quit::HOVERED_BUTTON, StartEvent, SCOREBOARD_FONT_SIZE, TEXT_COLOR};

#[derive(Component)]
pub(super) struct DifficultyButtonFilter;

#[derive(Component)]
pub(super) struct DifficultyButton {
    color: UiColor,
    difficulty: usize,
}

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

pub(super) fn difficulty_event_system(
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

pub(super) fn difficulty_button_system(
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

pub(super) fn show_difficulty_buttons_system(
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
