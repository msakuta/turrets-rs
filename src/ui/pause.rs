use bevy::prelude::*;

use crate::Level;

use super::{
    PauseEvent, PauseState, BUTTON_HEIGHT, PADDING, PADDING_PX, SCOREBOARD_FONT_SIZE, TEXT_COLOR,
};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const ACTIVE_BUTTON: Color = Color::rgb(0.40, 0.40, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_ACTIVE_BUTTON: Color = Color::rgb(0.50, 0.50, 0.25);

#[derive(Component)]
pub(super) struct PauseButtonFilter;

pub(super) fn add_pause_button(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(BUTTON_HEIGHT),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                top: PADDING_PX,
                right: Val::Px(PADDING + super::quit::BUTTON_WIDTH),
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(PauseButtonFilter)
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "Pause",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: TEXT_COLOR,
                        },
                    ),
                    ..default()
                })
                .insert(PauseButtonFilter);
        });
}

pub(super) fn pause_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<PauseButtonFilter>),
    >,
    mut writer: EventWriter<PauseEvent>,
    level: Res<Level>,
    pause_state: Res<PauseState>,
) {
    if let Level::Select = level.as_ref() {
        return;
    }
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                writer.send(PauseEvent);
            }
            Interaction::Hovered => {
                *color = if pause_state.0 {
                    HOVERED_ACTIVE_BUTTON.into()
                } else {
                    HOVERED_BUTTON.into()
                };
            }
            Interaction::None => {
                *color = if pause_state.0 {
                    ACTIVE_BUTTON.into()
                } else {
                    NORMAL_BUTTON.into()
                }
            }
        }
    }
}

pub(super) fn pause_event_system(
    mut reader: EventReader<PauseEvent>,
    mut pause_state: ResMut<PauseState>,
) {
    if reader.read().last().is_some() {
        println!("Received PauseEvent: {}", pause_state.0);
        pause_state.0 = !pause_state.0;
    }
}

pub(crate) fn not_paused(pause_state: Res<PauseState>) -> bool {
    !pause_state.0
}

pub(super) fn show_pause_button_system(
    mut button_query: Query<&mut Visibility, With<PauseButtonFilter>>,
    level: Res<Level>,
) {
    for mut button in button_query.iter_mut() {
        *button = if let Level::Select = level.as_ref() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
