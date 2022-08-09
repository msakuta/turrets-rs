use bevy::prelude::*;

use crate::{
    mouse::{MouseCursor, SelectedTower},
    tower::{spawn_healer, spawn_missile_tower, spawn_shotgun, spawn_turret},
    Level, Scoreboard,
};

use super::{
    BUTTON_HEIGHT, PADDING, PADDING_PX, PALETTE_ICON_SIZE, PALETTE_SIZE, SCORE_COLOR,
    STATUS_FONT_SIZE, TEXT_COLOR,
};

pub(super) fn build_tower_palette(app: &mut App) {
    app.add_startup_system(add_palette_hint_panel);
    app.add_system(palette_mouse_system);
    app.add_system(update_palette_system);
    app.add_system(palette_tooltip_system);
}

#[derive(Component, Debug)]
pub(super) enum TowerPalette {
    Turret,
    Shotgun,
    Healer,
    MissileTower,
}

impl TowerPalette {
    fn spawn(
        &self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        position: Vec2,
    ) -> Entity {
        match self {
            Self::Turret => spawn_turret(commands, asset_server, position, 0.),
            Self::Shotgun => spawn_shotgun(commands, asset_server, position, 0.),
            Self::Healer => spawn_healer(commands, asset_server, position, 0.),
            Self::MissileTower => spawn_missile_tower(commands, asset_server, position, 0.),
        }
    }

    fn cost(&self) -> f64 {
        // TODO: Scale with the number of existing towers
        match self {
            Self::Turret => 100.,
            Self::Shotgun => 150.,
            Self::Healer => 200.,
            Self::MissileTower => 350.,
        }
    }
}

pub(super) fn add_palette_buttons(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(PALETTE_SIZE), Val::Percent(100.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(PADDING * 2. + BUTTON_HEIGHT),
                    right: PADDING_PX,
                    ..default()
                },
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            add_tower_icon(parent, &asset_server, "turret.png", TowerPalette::Turret);
            add_tower_icon(parent, &asset_server, "shotgun.png", TowerPalette::Shotgun);
            add_tower_icon(parent, &asset_server, "healer.png", TowerPalette::Healer);
            add_tower_icon(
                parent,
                &asset_server,
                "missile-tower.png",
                TowerPalette::MissileTower,
            );
        });
}

fn add_tower_icon(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    file: &str,
    tower_palette: TowerPalette,
) {
    parent
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(PALETTE_SIZE), Val::Px(PALETTE_SIZE)),
                border: Rect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(PALETTE_ICON_SIZE), Val::Px(PALETTE_ICON_SIZE)),
                        ..default()
                    },
                    image: UiImage::from(asset_server.load(file)).into(),
                    ..default()
                })
                .insert(Interaction::default())
                .insert(tower_palette);
        });
}

fn palette_mouse_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    level: Res<Level>,
    mut scoreboard: ResMut<Scoreboard>,
    mut query: Query<(&mut Transform, &mut Visibility), With<MouseCursor>>,
    query_towers: Query<(&Interaction, &Parent, &TowerPalette), Changed<Interaction>>,
    mut query_ui_color: Query<&mut UiColor>,
    mut selected_tower: ResMut<SelectedTower>,
) {
    if selected_tower.dragging || !level._is_running() {
        return;
    }

    let window = if let Some(window) = windows.iter().next() {
        window
    } else {
        return;
    };
    let mouse = window.cursor_position();

    if let Some(((mut cursor_transform, mut visibility), mouse_position)) =
        query.get_single_mut().ok().zip(mouse)
    {
        let (width, height) = (window.width(), window.height());
        let mouse_screen = Vec2::new(
            mouse_position.x - width / 2.,
            mouse_position.y - height / 2.,
        );
        // println!("Mouse: {:?} -> {:?}", mouse_position, mouse_screen);
        for (interaction, parent, palette) in query_towers.iter() {
            if let Ok(mut ui_color) = query_ui_color.get_component_mut::<UiColor>(**parent) {
                // println!("Has ui_color: {ui_color:?}");
                match *interaction {
                    Interaction::Clicked => {
                        println!("Clicked tower palette at {mouse_screen:?}");

                        let cost = palette.cost();
                        if scoreboard.credits < cost {
                            return;
                        }

                        *ui_color = Color::rgba(1., 0., 1., 0.75).into();

                        visibility.is_visible = true;
                        *cursor_transform =
                            Transform::from_xyz(mouse_screen.x, mouse_screen.y, 0.2)
                                .with_scale(Vec3::new(2., 2., 1.));

                        let tower = palette.spawn(&mut commands, &asset_server, mouse_screen);
                        selected_tower.tower = Some(tower);
                        selected_tower.dragging = true;

                        scoreboard.credits -= cost;

                        return;
                    }
                    Interaction::Hovered => {
                        *ui_color = Color::rgba(0.5, 0., 0., 0.5).into();
                    }
                    Interaction::None => {
                        *ui_color = Color::rgba(0.0, 0., 0., 0.5).into();
                    }
                }
            }
        }
    }
}

fn update_palette_system(
    mut query: Query<(&mut UiColor, &TowerPalette)>,
    scoreboard: Res<Scoreboard>,
) {
    for (mut color, palette) in query.iter_mut() {
        *color = if scoreboard.credits < palette.cost() {
            Color::rgba(0.5, 0.5, 0.5, 0.5).into()
        } else {
            Color::WHITE.into()
        };
    }
}

#[derive(Component)]
struct PaletteHintText;

#[derive(Component)]
struct PaletteHintTowerType;

fn add_palette_hint_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(PADDING * 4. + BUTTON_HEIGHT + STATUS_FONT_SIZE * 2.),
                    right: Val::Px(PADDING * 2. + PALETTE_SIZE),
                    ..default()
                },
                ..default()
            },
            visibility: Visibility { is_visible: false },
            color: Color::rgba(0., 0., 0., 0.8).into(),
            ..default()
        })
        .insert(PaletteHintText)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "Tower type".to_string(),
                            style: TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: STATUS_FONT_SIZE,
                                color: TEXT_COLOR,
                            },
                        }],
                        ..default()
                    },
                    visibility: Visibility { is_visible: false },
                    ..default()
                })
                .insert(PaletteHintText)
                .insert(PaletteHintTowerType);

            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: "Cost: ".to_string(),
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
                    visibility: Visibility { is_visible: false },
                    ..default()
                })
                .insert(PaletteHintText);
        });
}

fn palette_tooltip_system(
    query_towers: Query<(&Interaction, &TowerPalette), Changed<Interaction>>,
    mut query_tooltip_visible: Query<&mut Visibility, With<PaletteHintText>>,
    mut query_tooltip_tower_type: Query<
        &mut Text,
        (With<PaletteHintText>, With<PaletteHintTowerType>),
    >,
    mut query_tooltip_cost: Query<
        &mut Text,
        (With<PaletteHintText>, Without<PaletteHintTowerType>),
    >,
) {
    for (interaction, palette) in query_towers.iter() {
        match *interaction {
            Interaction::Hovered => {
                for mut visibility in query_tooltip_visible.iter_mut() {
                    visibility.is_visible = true;
                }
                if let Ok(mut text) = query_tooltip_tower_type.get_single_mut() {
                    text.sections[0].value = format!("{:?}", palette);
                }
                if let Ok(mut text) = query_tooltip_cost.get_single_mut() {
                    text.sections[1].value = format!("${}", palette.cost());
                }
            }
            Interaction::None => {
                for mut visibility in query_tooltip_visible.iter_mut() {
                    visibility.is_visible = false;
                }
            }
            _ => (),
        }
    }
}
