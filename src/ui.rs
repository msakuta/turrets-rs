use bevy::prelude::*;

pub(crate) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(build_ui);
    }
}

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

fn build_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Scoreboard
    commands.spawn_bundle(TextBundle {
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
    });
}
