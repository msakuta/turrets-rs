use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    mouse::{MouseCursor, SelectedTower, SelectedTowerProps},
    tower::{
        spawn_beam_tower, spawn_healer, spawn_missile_tower, spawn_shotgun, spawn_turret, Tower,
    },
    Level, Scoreboard, Textures,
};

use super::{
    spawn_text, BUTTON_HEIGHT, PADDING, PADDING_PX, PALETTE_ICON_SIZE, PALETTE_SIZE,
    STATUS_FONT_SIZE,
};

use std::iter::Iterator;

pub(super) fn build_tower_palette(app: &mut App) {
    app.add_systems(Startup, add_palette_tooltip_panel);
    app.add_systems(Startup, add_trashcan_hint_panel);
    app.add_systems(
        Update,
        (
            palette_mouse_system,
            update_palette_system,
            palette_tooltip_system,
            trashcan_tooltip_system,
        ),
    );
}

#[derive(Component, Debug)]
enum TowerPalette {
    Turret,
    Shotgun,
    Healer,
    BeamTower,
    MissileTower,
}

impl TowerPalette {
    fn spawn(
        &self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        position: Vec2,
        textures: &Textures,
    ) -> Entity {
        match self {
            Self::Turret => spawn_turret(commands, asset_server, position, 0., default(), textures),
            Self::Shotgun => {
                spawn_shotgun(commands, asset_server, position, 0., default(), textures)
            }
            Self::Healer => spawn_healer(commands, asset_server, position, 0., default(), textures),
            Self::BeamTower => {
                spawn_beam_tower(commands, asset_server, position, 0., default(), textures)
            }
            Self::MissileTower => {
                spawn_missile_tower(commands, asset_server, position, 0., default(), textures)
            }
        }
    }

    fn cost(&self, tower_count: usize) -> f64 {
        match self {
            Self::Turret => ((1.5f64).powf(tower_count as f64) * 100.).ceil(),
            Self::Shotgun => ((1.5f64).powf(tower_count as f64) * 150.).ceil(),
            Self::Healer => ((1.5f64).powf(tower_count as f64) * 200.).ceil(),
            Self::BeamTower => ((1.5f64).powf(tower_count as f64) * 350.).ceil(),
            Self::MissileTower => ((1.5f64).powf(tower_count as f64) * 200.).ceil(),
        }
    }
}

#[derive(Component, Debug)]
struct TowerTrashcan;

pub(super) fn add_palette_buttons(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(PALETTE_SIZE),
                // height: Val::Percent(100.0),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                top: Val::Px(PADDING * 2. + BUTTON_HEIGHT),
                right: PADDING_PX,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            add_tower_icon(parent, &asset_server, "turret.png", TowerPalette::Turret);
            add_tower_icon(parent, &asset_server, "shotgun.png", TowerPalette::Shotgun);
            add_tower_icon(parent, &asset_server, "healer.png", TowerPalette::Healer);
            add_tower_icon(
                parent,
                &asset_server,
                "beam-tower.png",
                TowerPalette::BeamTower,
            );
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
    file: &'static str,
    tower_palette: impl Component,
) {
    let asset = asset_server.load(file);
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(PALETTE_SIZE),
                height: Val::Px(PALETTE_SIZE),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(PALETTE_ICON_SIZE),
                        height: Val::Px(PALETTE_ICON_SIZE),
                        ..default()
                    },
                    image: UiImage::from(asset).into(),
                    ..default()
                })
                .insert(Interaction::default())
                .insert(tower_palette);
        });
}

fn palette_mouse_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // windows: Res<Windows>,
    window: Query<&Window, With<PrimaryWindow>>,
    level: Res<Level>,
    mut scoreboard: ResMut<Scoreboard>,
    mut query_mouse: Query<(&mut Transform, &mut Visibility), With<MouseCursor>>,
    query_towers: Query<&Tower>,
    query_palette: Query<(&Interaction, &Parent, &TowerPalette), Changed<Interaction>>,
    mut query_ui_color: Query<&mut BackgroundColor>,
    mut selected_tower: ResMut<SelectedTower>,
    textures: Res<Textures>,
) {
    if selected_tower.is_dragging() || !level._is_running() {
        return;
    }

    let window = window.single();
    let mouse = window.cursor_position();

    if let Some(((mut cursor_transform, mut visibility), mouse_position)) =
        query_mouse.get_single_mut().ok().zip(mouse)
    {
        let (width, height) = (window.width(), window.height());
        let mouse_screen = Vec2::new(
            mouse_position.x - width / 2.,
            mouse_position.y - height / 2.,
        );

        let tower_count = query_towers.iter().count();

        // println!("Mouse: {:?} -> {:?}", mouse_position, mouse_screen);
        for (interaction, parent, palette) in query_palette.iter() {
            if let Ok(mut ui_color) = query_ui_color.get_component_mut::<BackgroundColor>(**parent)
            {
                // println!("Has ui_color: {ui_color:?}");
                match *interaction {
                    Interaction::Pressed => {
                        println!("Clicked tower palette at {mouse_screen:?}");

                        let cost = palette.cost(tower_count);
                        if scoreboard.credits < cost {
                            return;
                        }

                        *ui_color = Color::rgba(1., 0., 1., 0.75).into();

                        *visibility = Visibility::Inherited;
                        *cursor_transform =
                            Transform::from_xyz(mouse_screen.x, mouse_screen.y, 0.2)
                                .with_scale(Vec3::new(2., 2., 1.));

                        let tower =
                            palette.spawn(&mut commands, &asset_server, mouse_screen, &textures);
                        selected_tower.0 = Some(SelectedTowerProps {
                            tower,
                            dragging: true,
                            hovering_trashcan: false,
                        });

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
    query_towers: Query<&Tower>,
    mut query: Query<(&mut BackgroundColor, &TowerPalette)>,
    scoreboard: Res<Scoreboard>,
) {
    let tower_count = query_towers.iter().count();
    for (mut color, palette) in query.iter_mut() {
        *color = if scoreboard.credits < palette.cost(tower_count) {
            Color::rgba(0.5, 0.5, 0.5, 0.5).into()
        } else {
            Color::WHITE.into()
        };
    }
}

#[derive(Component)]
struct PaletteTooltipText;

#[derive(Component)]
struct PaletteTooltipTowerType;

fn add_palette_tooltip_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                top: Val::Px(PADDING * 4. + BUTTON_HEIGHT + STATUS_FONT_SIZE * 6.),
                right: Val::Px(PADDING * 2. + PALETTE_SIZE),
                ..default()
            },
            visibility: Visibility::Hidden,
            background_color: Color::rgba(0., 0., 0., 0.8).into(),
            ..default()
        })
        .insert(PaletteTooltipText)
        .with_children(|parent| {
            spawn_text(&asset_server, parent, &["Tower type"], |mut parent| {
                parent
                    .insert(PaletteTooltipText)
                    .insert(PaletteTooltipTowerType);
            });

            spawn_text(&asset_server, parent, &["Cost: ", ""], |mut parent| {
                parent.insert(PaletteTooltipText);
            });
        });
}

fn palette_tooltip_system(
    query_towers: Query<&Tower>,
    query_palette: Query<(&Interaction, &TowerPalette), Changed<Interaction>>,
    mut query_tooltip_visible: Query<&mut Visibility, With<PaletteTooltipText>>,
    mut query_tooltip_tower_type: Query<
        &mut Text,
        (With<PaletteTooltipText>, With<PaletteTooltipTowerType>),
    >,
    mut query_tooltip_cost: Query<
        &mut Text,
        (With<PaletteTooltipText>, Without<PaletteTooltipTowerType>),
    >,
) {
    let tower_count = query_towers.iter().count();
    for (interaction, palette) in query_palette.iter() {
        match *interaction {
            Interaction::Hovered => {
                for mut visibility in query_tooltip_visible.iter_mut() {
                    *visibility = Visibility::Inherited;
                }
                if let Ok(mut text) = query_tooltip_tower_type.get_single_mut() {
                    text.sections[0].value = format!("{:?}", palette);
                }
                if let Ok(mut text) = query_tooltip_cost.get_single_mut() {
                    text.sections[1].value = format!("${}", palette.cost(tower_count));
                }
            }
            Interaction::None => {
                for mut visibility in query_tooltip_visible.iter_mut() {
                    *visibility = Visibility::Inherited;
                }
            }
            _ => (),
        }
    }
}

#[derive(Component)]
struct TrashcanTooltipText;

fn add_trashcan_hint_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(PALETTE_SIZE),
                height: Val::Auto,
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                bottom: Val::Px(PADDING),
                right: PADDING_PX,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            add_tower_icon(parent, &asset_server, "trashcan.png", TowerTrashcan)
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                // justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                bottom: Val::Px(PADDING + PALETTE_SIZE - STATUS_FONT_SIZE),
                right: Val::Px(PADDING * 2. + PALETTE_SIZE),
                ..default()
            },
            visibility: Visibility::Hidden,
            background_color: Color::rgba(0., 0., 0., 0.8).into(),
            ..default()
        })
        .insert(TrashcanTooltipText)
        .with_children(|parent| {
            spawn_text(
                &asset_server,
                parent,
                &["Trashcan: drag a tower here to delete"],
                |mut parent| {
                    parent.insert(TrashcanTooltipText);
                },
            );
        });
}

fn trashcan_tooltip_system(
    query_trashcan: Query<(&Interaction, &Parent), (With<TowerTrashcan>, Changed<Interaction>)>,
    mut query_tooltip_visible: Query<&mut Visibility, With<TrashcanTooltipText>>,
    mut query_ui_color: Query<&mut BackgroundColor>,
    mut selected_tower: ResMut<SelectedTower>,
) {
    for (interaction, parent) in query_trashcan.iter() {
        match *interaction {
            Interaction::Hovered => {
                for mut visibility in query_tooltip_visible.iter_mut() {
                    *visibility = Visibility::Inherited;
                }
                if let Ok(mut ui_color) = query_ui_color.get_mut(**parent) {
                    *ui_color = Color::rgba(0.5, 0., 0., 0.5).into();
                }
                if let Some(selected_tower) = selected_tower.0.as_mut() {
                    selected_tower.hovering_trashcan = true;
                }
            }
            Interaction::None => {
                for mut visibility in query_tooltip_visible.iter_mut() {
                    *visibility = Visibility::Hidden;
                }
                if let Ok(mut ui_color) = query_ui_color.get_mut(**parent) {
                    *ui_color = Color::rgba(0.0, 0., 0., 0.5).into();
                }
                if let Some(selected_tower) = selected_tower.0.as_mut() {
                    selected_tower.hovering_trashcan = false;
                }
            }
            _ => (),
        }
    }
}
