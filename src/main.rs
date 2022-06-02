use bevy::{
    prelude::{
        App,
        Plugin,
        default,
        Commands,
        OrthographicCameraBundle,
        Color,
        Transform
    },
    window::WindowDescriptor,
    core_pipeline::ClearColor,
    DefaultPlugins,
    sprite::{
        SpriteBundle,
        Sprite,
    },
    math::{
        Vec2,
        Vec3
    }
};
use crate::config::BACKGROUND_COLOR;

mod config;

fn main() {
    App::new()
        .add_plugin(GameSetup)
        .add_system(setup_field)
        .run();
}

// TODO: Move to separete file
struct GameSetup;

impl Plugin for GameSetup {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WindowDescriptor {
                title: "Game of Life".to_string(),
                // TODO: use value from config
                width: 300.0,
                // TODO: use value from config
                height: 300.0,
                ..default()
            })
            .add_startup_system(setup_camera)
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_plugins(DefaultPlugins);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

/// Spawn field, filled by lines, which create
/// an ordered bunch of squares
///
/// TODO: show only in debug mode
/// TODO: move to separete file?
fn setup_field(mut commands: Commands) {
    // stores Sprintes for creating a `net`
    let mut lines: Vec<SpriteBundle> = Vec::new();

    // because sprites are aligned from centre
    // left edge of window is window_size / 2 * -1
    // I use -16 in order to have left and top border
    //
    // TODO: create useful function to calculate range
    // TODO: can I do something with Sprite anchor?
    // TODO: reuse size from config
    for i in -16..15 {
        lines.push(
            SpriteBundle {
                sprite: Sprite {
                    // TODO: reuse colour from config
                    color: Color::rgb(255.0, 255.0, 255.0),
                    // TODO: use value from Window
                    custom_size: Some(Vec2::new(1.0, 300.0)),
                    ..default()
                },
                transform: Transform {
                    // TODO: change 10 to window scale param
                    translation: Vec3::new(i as f32 * 10.0, 0.0, 0.0),
                    ..default()
                },
                ..default()
            }
        );
        lines.push(
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(255.0, 255.0, 255.0),
                    custom_size: Some(Vec2::new(300.0, 1.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, i as f32 * 10.0, 0.0),
                    ..default()
                },
                ..default()
            }
        );
    }

    commands
        .spawn_batch(lines);
}
