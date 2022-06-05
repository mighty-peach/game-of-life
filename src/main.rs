// TODO: add circels to field with calculated position and scaling
// TODO: add behaviour
use bevy::{
    core_pipeline::ClearColor,
    input::mouse::MouseButtonInput,
    math::{Vec2, Vec3},
    prelude::{
        default, App, Commands, Component, CoreStage, Deref, DerefMut, Entity, EventReader,
        OrthographicCameraBundle, Plugin, Query, Res, ResMut, SystemSet, Transform,
    },
    sprite::{Sprite, SpriteBundle},
    window::{WindowDescriptor, Windows},
    DefaultPlugins,
};
use config::{BACKGROUND_COLOR, LINES_COUNT, MAIN_COLOR, WINDOW_PADDING, WINDOW_SIZE};

mod config;

fn main() {
    App::new()
        .add_plugin(GameSetup)
        .add_system(click_handler)
        .insert_resource(Cells::default())
        .run();
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
            .add_startup_system(prefill_cells)
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

#[derive(Default, Deref, DerefMut)]
struct Lines(Vec<Entity>);

#[derive(Default, Deref, DerefMut)]
struct Cells(Vec<Vec<Entity>>);

#[derive(Component)]
struct CellSize {
    radius: f32,
}

#[derive(Component)]
struct CellProperty {
    is_active: bool,
}

#[derive(Component)]
struct CellCoordinate {
    x: f32,
    y: f32,
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

// ** Game Field **
// Bunch of vertical and horizontal lines, which create square table

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    // Gets position of line, calculates gap between lines and multiplies current position to the gap
    // Gap is size of window without padding divided on lines count
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        assert!(bound_window > 0., "bound_window need to be more than 0");
        assert!(bound_game > 0., "bound_game should be greater than 0");
        let tile_size = bound_window / bound_game;

        pos * tile_size
    }

    let window = windows.get_primary().unwrap();
    let padding_size = WINDOW_PADDING * 2.0;
    assert!(padding_size > 0.0, "padding_size should be greater than 0");
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() - padding_size, LINES_COUNT),
            convert(pos.y as f32, window.width() - padding_size, LINES_COUNT),
            0.0,
        )
    }
}

// Calculate line width
// Its scale property is window width without paddings
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

// Fills Lines with predefined count of lines
fn setup_field(mut commands: Commands, mut lines: ResMut<Lines>) {
    let half_of_field_lines_count = (LINES_COUNT / 2.) as i32;

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

// ** Cells logic **
fn prefill_cells(mut commands: Commands, mut cells: ResMut<Cells>) {
    let mut rows = Vec::new();

    for i in 0..LINES_COUNT as usize {
        rows.push(Vec::new());
        for _ in 0..LINES_COUNT as usize {
            rows[i].push(
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: MAIN_COLOR,
                            custom_size: Some(Vec2::new(10., 10.)),
                            ..default()
                        },
                        ..default()
                    })
                    .id(),
            );
        }
    }

    *cells = Cells(rows);
}

// ** Click Logic **
fn click_handler(windows: Res<Windows>, mut mouse_button_event: EventReader<MouseButtonInput>) {
    use bevy::input::ElementState;

    let window = windows.get_primary().expect("error in widnows.get_primary");
    for event in mouse_button_event.iter() {
        if event.state == ElementState::Pressed {
            let cursor_position = window
                .cursor_position()
                .expect("error of getting cursor position");
            let window_width = window.width();

            if cursor_position[0] > WINDOW_PADDING
                && cursor_position[1] > WINDOW_PADDING
                && cursor_position[0] < window_width - WINDOW_PADDING
                && cursor_position[1] < window_width - WINDOW_PADDING
            {
                let x = ((cursor_position[0] - WINDOW_PADDING)
                    / ((window_width - (WINDOW_PADDING * 2.)) / LINES_COUNT))
                    .ceil();
                let y = ((cursor_position[1] - WINDOW_PADDING)
                    / ((window_width - (WINDOW_PADDING * 2.)) / LINES_COUNT))
                    .ceil();
                println!("cursor position: {:#?}", window.cursor_position());
                println!("x, y: {:#?}, {:#?}", x, y);
            } else {
                println!("cursor out of field");
            }
        }
    }
}
