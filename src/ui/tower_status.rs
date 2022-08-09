use bevy::prelude::*;

use crate::{mouse::SelectedTower, tower::TowerScore, Health};

use super::{BUTTON_HEIGHT, PADDING, PALETTE_SIZE, SCORE_COLOR, STATUS_FONT_SIZE, TEXT_COLOR};

#[derive(Component)]
pub(super) struct TowerHealthText;

#[derive(Component)]
pub(super) struct TowerScoreText;

pub(super) fn add_status_panel(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                position: UiRect {
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

pub(super) fn update_tower_scoreboard(
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

pub(super) fn update_tower_health(
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
