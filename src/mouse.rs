use crate::{tower::Tower, Position};
use bevy::{
    ecs::{schedule::ShouldRun, system::QueryComponentError},
    prelude::*,
};

pub(crate) struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedTower::None);
        app.add_startup_system(setup);
        app.add_system(mouse_system);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("select-marker.png"),
            visibility: Visibility {
                is_visible: false,
                ..default()
            },
            ..default()
        })
        .insert(MouseCursor);
}

#[derive(Component)]
pub(crate) struct MouseCursor;

pub(crate) struct SelectedTowerProps {
    pub tower: Entity,
    pub dragging: bool,
    pub hovering_trashcan: bool,
}

pub(crate) type SelectedTower = Option<SelectedTowerProps>;

pub(crate) fn tower_not_dragging(selected_tower: Res<SelectedTower>) -> ShouldRun {
    if selected_tower
        .as_ref()
        .as_ref()
        .map(|f| f.dragging)
        .unwrap_or(false)
    {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

fn mouse_system(
    mut commands: Commands,
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &mut Visibility), With<MouseCursor>>,
    mut query_towers: Query<(Entity, &mut Position, &Tower)>,
    query_tower_health: Query<&Tower>,
    btn: Res<Input<MouseButton>>,
    mut selected_tower: ResMut<SelectedTower>,
) {
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

        let mut dragging = false;

        if let Some(selected_tower) = selected_tower.as_ref() {
            if selected_tower.dragging {
                match (|| -> Result<(), QueryComponentError> {
                    let my_size = query_towers
                        .get_component::<Tower>(selected_tower.tower)?
                        .size;

                    let hit_others = query_towers.iter().any(|(entity, position, tower)| {
                        selected_tower.tower != entity
                            && mouse_screen.distance_squared(position.0)
                                < (tower.size + my_size).powf(2.)
                    });

                    if !hit_others {
                        let mut tower =
                            query_towers.get_component_mut::<Position>(selected_tower.tower)?;
                        tower.0 = mouse_screen;
                        *cursor_transform =
                            Transform::from_xyz(mouse_screen.x, mouse_screen.y, 0.2)
                                .with_scale(Vec3::new(2., 2., 1.));
                    }
                    Ok(())
                })() {
                    Ok(()) => (),
                    Err(e) => println!("Query component error! the logic seems wrong! {e:?}"),
                };
                dragging = true;
            }
        }

        if !dragging {
            for (entity, tower_position, _) in query_towers.iter() {
                if tower_position.0.distance(mouse_screen) < 30. {
                    visibility.is_visible = true;
                    *cursor_transform =
                        Transform::from_xyz(tower_position.0.x, tower_position.0.y, 0.2)
                            .with_scale(Vec3::new(2., 2., 1.));

                    if let Some(selected_tower) = selected_tower.as_mut() {
                        selected_tower.tower = entity;
                        if btn.just_pressed(MouseButton::Left) {
                            selected_tower.dragging = true;
                        }
                    } else {
                        *selected_tower = Some(SelectedTowerProps {
                            tower: entity,
                            dragging: btn.just_pressed(MouseButton::Left),
                            hovering_trashcan: false,
                        });
                    }

                    return;
                }
            }

            visibility.is_visible = false;
        }
    }
    if btn.just_released(MouseButton::Left) {
        if let Some(selected_tower) = selected_tower.as_ref() {
            if selected_tower.dragging && selected_tower.hovering_trashcan {
                if let Ok(tower) = query_tower_health.get(selected_tower.tower) {
                    commands.entity(tower.health_bar.0).despawn();
                    commands.entity(tower.health_bar.1).despawn();
                }
                commands.entity(selected_tower.tower).despawn_recursive();
            }
        }
        *selected_tower = None;
    }
}
