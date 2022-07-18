use crate::{tower::Tower, Position};
use bevy::prelude::*;

pub(crate) struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedTower { tower: None });
        app.add_startup_system(setup);
        app.add_system(mouse_position);
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

pub(crate) struct SelectedTower {
    pub tower: Option<Entity>,
}

fn mouse_position(
    windows: Res<Windows>,
    mut query: Query<(&mut Transform, &mut Visibility), With<MouseCursor>>,
    query_towers: Query<(Entity, &Position), With<Tower>>,
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
        // println!("Mouse: {:?} -> {:?}", mouse_position, mouse_screen);
        for (entity, tower_position) in query_towers.iter() {
            if tower_position.0.distance(mouse_screen) < 30. {
                visibility.is_visible = true;
                *cursor_transform = Transform::from_xyz(tower_position.0.x, tower_position.0.y, 0.)
                    .with_scale(Vec3::new(2., 2., 1.));

                selected_tower.tower = Some(entity);

                if btn.just_pressed(MouseButton::Left) {
                    println!("Just_pressed! {:?} -> {:?}", mouse_position, mouse_screen);
                }
                return;
            }
        }
        visibility.is_visible = false;
    }
}
