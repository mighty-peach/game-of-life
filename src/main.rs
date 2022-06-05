// TODO: Or there is too much lines with window resize, or clearcolor doesn't happened
use bevy::{
    core_pipeline::ClearColor,
    math::Vec3,
    prelude::{
        default, App, Commands, Component, CoreStage, Deref, DerefMut, Entity,
        OrthographicCameraBundle, Plugin, Query, Res, ResMut, SystemSet, Transform,
    },
    sprite::{Sprite, SpriteBundle},
    window::{WindowDescriptor, Windows},
    DefaultPlugins,
};
use config::{BACKGROUND_COLOR, MAIN_COLOR, WINDOW_PADDING, WINDOW_SIZE};

mod config;

fn main() {
    App::new().add_plugin(GameSetup).run();
}

struct GameSetup;

impl Plugin for GameSetup {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .insert_resource(WindowDescriptor {
                title: "Game of Life".to_string(),
                width: WINDOW_SIZE,
                height: WINDOW_SIZE,
                ..default()
            })
            .add_startup_system(setup_camera)
            .add_startup_system(setup_field)
            .insert_resource(Lines::default())
            .add_plugins(DefaultPlugins)
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_system(position_translation)
                    .with_system(size_scaling),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct LineProperty {
    is_vertical: bool,
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;

        pos * tile_size
    }

    let window = windows.get_primary().unwrap();
    let padding_size = WINDOW_PADDING * 2.0;
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() - padding_size, 30.0),
            convert(pos.y as f32, window.width() - padding_size, 30.0),
            0.0,
        )
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&LineProperty, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    let padding_size = WINDOW_PADDING * 2.0;
    for (props, mut transform) in q.iter_mut() {
        if props.is_vertical {
            transform.scale = Vec3::new(1.0, (window.width() as f32) - padding_size, 1.0);
        } else {
            transform.scale = Vec3::new((window.width() as f32) - padding_size, 1.0, 1.0);
        }
    }
}

#[derive(Default, Deref, DerefMut)]
struct Lines(Vec<Entity>);

fn setup_field(mut commands: Commands, mut lines: ResMut<Lines>) {
    let cells_count = 30.0;
    let half_of_field_lines_count = (cells_count / 2.) as i32;

    // calculates steps for iterator for creating lines
    // at first it calculates centre of field and than it finds
    // starting position of lines and end
    let calculate_range = || {
        (
            -1 * half_of_field_lines_count,
            half_of_field_lines_count + 1,
        )
    };
    let range = calculate_range();

    for i in range.0..range.1 {
        *lines = Lines(vec![
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: MAIN_COLOR,
                        ..default()
                    },
                    ..default()
                })
                .insert(Position {
                    x: i as f32,
                    y: 0.0,
                })
                .insert(LineProperty { is_vertical: true })
                .id(),
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: MAIN_COLOR,
                        ..default()
                    },
                    ..default()
                })
                .insert(LineProperty { is_vertical: false })
                .insert(Position {
                    x: 0.0,
                    y: i as f32,
                })
                .id(),
        ]);
    }
}
