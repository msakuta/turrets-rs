use bevy::prelude::*;

use crate::{ClearEvent, Level, StageClear};

use super::{QuitEvent, BUTTON_HEIGHT, PADDING_PX, SCOREBOARD_FONT_SIZE, TEXT_COLOR};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub(super) const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

#[derive(Component)]
pub(super) struct QuitButtonFilter;

pub(super) fn add_quit_button(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(BUTTON_HEIGHT)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
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
                    text: Text::from_section(
                        "Quit",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: TEXT_COLOR,
                        },
                    ),
                    ..default()
                })
                .insert(QuitButtonFilter);
        });
}

pub(super) fn quit_button_system(
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

pub(super) fn quit_event_system(
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

pub(super) fn show_quit_button_system(
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
