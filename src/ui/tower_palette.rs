use bevy::prelude::*;

use crate::{
    mouse::{MouseCursor, SelectedTower},
    tower::{spawn_healer, spawn_missile_tower, spawn_shotgun, spawn_turret},
    Level, Scoreboard,
};

use super::{BUTTON_HEIGHT, PADDING, PADDING_PX, PALETTE_SIZE};

#[derive(Component)]
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
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                position_type: PositionType::Absolute,
                position: UiRect {
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
    comp: TowerPalette,
) {
    parent
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(PALETTE_SIZE), Val::Px(PALETTE_SIZE)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(PALETTE_SIZE), Val::Px(PALETTE_SIZE)),
                        ..default()
                    },
                    image: UiImage::from(asset_server.load(file)).into(),
                    ..default()
                })
                .insert(Interaction::default())
                .insert(comp);
        });
}

pub(super) fn palette_mouse_system(
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
