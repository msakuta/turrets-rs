mod difficulty_select;
mod quit;
mod scoreboard;
mod tower_palette;
mod tower_status;

use bevy::{ecs::system::EntityCommands, prelude::*};

use self::{
    difficulty_select::{add_difficulty_buttons, DifficultySelectPlugin},
    quit::{add_quit_button, quit_button_system, quit_event_system, show_quit_button_system},
    scoreboard::{add_scoreboard, update_credits, update_level, update_scoreboard},
    tower_palette::{add_palette_buttons, build_tower_palette},
    tower_status::build_tower_status,
};
use crate::Level;

pub(crate) struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DifficultySelectPlugin);
        app.add_event::<StartEvent>();
        app.add_event::<QuitEvent>();
        app.add_startup_system(build_ui);
        app.add_system(update_progress_bar);
        app.add_system(update_level);
        app.add_system(update_scoreboard);
        app.add_system(update_credits);
        build_tower_status(app);
        build_tower_palette(app);
        app.add_system(quit_event_system);
        app.add_system(quit_button_system);
        app.add_system(show_quit_button_system);
    }
}

struct StartEvent(usize);
struct QuitEvent;

#[derive(Component)]
struct ProgressBar;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const PADDING: f32 = 5.;
const PADDING_PX: Val = Val::Px(PADDING);
const TEXT_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const STATUS_FONT_SIZE: f32 = 20.0;

const BUTTON_HEIGHT: f32 = 65.0;

const PALETTE_SIZE: f32 = 64.;
const PALETTE_ICON_SIZE: f32 = PALETTE_SIZE * 0.75;

fn build_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    add_scoreboard(&mut commands, &asset_server);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(80.0), Val::Px(20.0)),
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    ..default()
                },
                border: Rect::all(Val::Px(2.0)),
                ..default()
            },
            color: Color::rgb(0.4, 0.4, 1.0).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    color: Color::rgb(0.8, 0.8, 1.0).into(),
                    ..default()
                })
                .insert(ProgressBar);
        });

    add_quit_button(&mut commands, &asset_server);
    add_palette_buttons(&mut commands, &asset_server);
    add_difficulty_buttons(&mut commands, &asset_server);
}

fn update_progress_bar(level: Res<Level>, mut query: Query<&mut Style, With<ProgressBar>>) {
    // println!("dur: {}", level.timer.elapsed_secs());
    let bar = query.get_single_mut();
    // println!("bar: {bar:?}");
    if let Ok(mut bar) = bar {
        if let Level::Running { timer, .. } = level.as_ref() {
            bar.size.width = Val::Percent(timer.percent() * 100.);
        }
    }
}

/// A helper function to add a text component bundle with a variable number of text sections.
///
/// This function assumes the first section of the `text` is a section title, so it has bold style
/// with different color.
///
/// Additional components can be inserted with the 4th argument closure. We could return EntityCommands to
/// the caller to let them insert, but lifetime annotations are too annoying so that I used inner closure
/// to avoid them.
fn spawn_text(
    asset_server: &AssetServer,
    parent: &mut ChildBuilder,
    text: &[&str],
    components: impl FnOnce(EntityCommands),
) {
    let builder = parent.spawn_bundle(TextBundle {
        text: Text {
            sections: text
                .iter()
                .enumerate()
                .map(|(i, text)| TextSection {
                    value: text.to_string(),
                    style: TextStyle {
                        font: asset_server.load(if i == 0 {
                            "fonts/FiraSans-Bold.ttf"
                        } else {
                            "fonts/FiraMono-Medium.ttf"
                        }),
                        font_size: STATUS_FONT_SIZE,
                        color: if i == 0 { TEXT_COLOR } else { SCORE_COLOR },
                    },
                })
                .collect(),
            ..default()
        },
        ..default()
    });

    components(builder);
}
