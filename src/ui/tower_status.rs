use bevy::prelude::*;

use crate::{
    bullet::BulletShooter,
    mouse::SelectedTower,
    tower::{tower_max_exp, TowerLevel, TowerScore},
    Health,
};

use super::{spawn_text, BUTTON_HEIGHT, PADDING, PALETTE_SIZE};

#[derive(Component)]
pub(super) struct TowerHealthText;

#[derive(Component)]
pub(super) struct TowerScoreText;

#[derive(Component)]
struct TowerLevelText;

#[derive(Component)]
struct TowerExpText;

#[derive(Component)]
struct TowerShooterText;

pub(super) fn build_tower_status(app: &mut App) {
    app.add_startup_system(add_status_panel);
    app.add_system(update_tower_scoreboard);
    app.add_system(update_tower_health);
    app.add_system(update_tower_level);
    app.add_system(update_tower_experience);
    app.add_system(update_tower_damage);
}

fn add_status_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(PADDING * 2. + BUTTON_HEIGHT),
                    right: Val::Px(PADDING * 2. + PALETTE_SIZE),
                    ..default()
                },
                padding: Rect::all(Val::Px(2.)),
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.8).into(),
            ..default()
        })
        .with_children(|parent| {
            spawn_text(&asset_server, parent, &["Health: ", ""], |mut parent| {
                parent.insert(TowerHealthText);
            });

            spawn_text(&asset_server, parent, &["Kills: ", ""], |mut parent| {
                parent.insert(TowerScoreText);
            });

            spawn_text(&asset_server, parent, &["Level: ", ""], |mut parent| {
                parent.insert(TowerLevelText);
            });

            spawn_text(&asset_server, parent, &["Exp: ", ""], |mut parent| {
                parent.insert(TowerExpText);
            });

            spawn_text(&asset_server, parent, &["Damage: ", ""], |mut parent| {
                parent.insert(TowerShooterText);
            });
        });
}

fn update_tower_scoreboard(
    selected_tower: Res<SelectedTower>,
    tower_score_query: Query<&TowerScore>,
    mut text_query: Query<&mut Text, With<TowerScoreText>>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        if let Some(selected_tower) = selected_tower
            .as_ref()
            .as_ref()
            .and_then(|tower| tower_score_query.get(tower.tower).ok())
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
            .as_ref()
            .as_ref()
            .and_then(|tower| tower_health_query.get(tower.tower).ok())
        {
            text.sections[1].value = format!("{}/{}", health.val, health.max);
        } else {
            text.sections[1].value = "".to_string();
        }
    }
}

fn update_tower_level(
    selected_tower: Res<SelectedTower>,
    tower_level_query: Query<&TowerLevel>,
    mut text_query: Query<&mut Text, With<TowerLevelText>>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        if let Some(selected_tower) = selected_tower
            .as_ref()
            .as_ref()
            .and_then(|tower| tower_level_query.get(tower.tower).ok())
        {
            text.sections[1].value = format!("{:?}", selected_tower.level);
        } else {
            text.sections[1].value = "".to_string();
        }
    }
}

fn update_tower_experience(
    selected_tower: Res<SelectedTower>,
    tower_level_query: Query<&TowerLevel>,
    mut text_query: Query<&mut Text, With<TowerExpText>>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        if let Some(tower_level) = selected_tower
            .as_ref()
            .as_ref()
            .and_then(|tower| tower_level_query.get(tower.tower).ok())
        {
            text.sections[1].value =
                format!("{}/{}", tower_level.exp, tower_max_exp(tower_level.level));
        } else {
            text.sections[1].value = "".to_string();
        }
    }
}

fn update_tower_damage(
    selected_tower: Res<SelectedTower>,
    tower_shooter_query: Query<&BulletShooter>,
    mut text_query: Query<&mut Text, With<TowerShooterText>>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        if let Some(tower_shooter) = selected_tower
            .as_ref()
            .as_ref()
            .and_then(|tower| tower_shooter_query.get(tower.tower).ok())
        {
            text.sections[1].value = format!("{}", tower_shooter.damage);
        } else {
            text.sections[1].value = "".to_string();
        }
    }
}
