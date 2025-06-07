use bevy::prelude::*;

use crate::{
    bullet::BulletShooter,
    mouse::SelectedTower,
    tower::{tower_max_exp, BeamTower, Healer, TowerLevel, TowerScore},
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
    app.add_systems(Startup, add_status_panel);
    app.add_systems(
        Update,
        (
            update_tower_scoreboard,
            update_tower_health,
            update_tower_level,
            update_tower_experience,
            update_tower_damage,
        ),
    );
}

fn add_status_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                top: Val::Px(PADDING * 2. + BUTTON_HEIGHT),
                right: Val::Px(PADDING * 2. + PALETTE_SIZE),
                padding: UiRect::all(Val::Px(2.)),
                ..default()
            },
            background_color: Color::rgba(0., 0., 0., 0.8).into(),
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
            .0
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
            .0
            .as_ref()
            .and_then(|tower| tower_health_query.get(tower.tower).ok())
        {
            text.sections[1].value = format!("{:.0}/{:.0}", health.val, health.max);
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
            .0
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
            .0
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
    tower_shooter_query: Query<(
        Option<&BulletShooter>,
        Option<&Healer>,
        &TowerLevel,
        Option<&BeamTower>,
    )>,
    mut text_query: Query<&mut Text, With<TowerShooterText>>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        match selected_tower
            .0
            .as_ref()
            .and_then(|tower| tower_shooter_query.get(tower.tower).ok())
        {
            Some((Some(tower_shooter), None, _, None)) => {
                text.sections[0].value = "Damage: ".to_string();
                text.sections[1].value = format!("{:.2}", tower_shooter.damage)
            }
            Some((None, Some(healer), _, None)) => {
                text.sections[0].value = "Heal: ".to_string();
                text.sections[1].value = format!("{:.2}", healer.heal_amt);
            }
            Some((None, None, level, Some(_))) => {
                text.sections[0].value = "DPS: ".to_string();
                text.sections[1].value =
                    format!("{:.2}", BeamTower::beam_dps_by_level(level.level));
            }
            _ => text.sections[1].value = "".to_string(),
        }
    }
}
