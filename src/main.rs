// TODO: add behaviour
// TODO: user should have ability to add several cells during MousePressed
use bevy::{
    core_pipeline::ClearColor,
    input::mouse::MouseButtonInput,
    math::Vec3,
    prelude::{
        default, App, Commands, Component, CoreStage, Deref, DerefMut, Entity, EventReader,
        OrthographicCameraBundle, Plugin, Query, Res, ResMut, SystemSet, Transform, With,
    },
    sprite::{Sprite, SpriteBundle},
    window::{WindowDescriptor, Windows},
    DefaultPlugins,
};
use config::{BACKGROUND_COLOR, CELL_COLOR, LINES_COUNT, MAIN_COLOR, WINDOW_PADDING, WINDOW_SIZE};

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
                    .with_system(cell_translation)
                    .with_system(cell_size_scaling)
                    .with_system(cell_visibility)
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
struct Cell {}

#[derive(Component)]
struct CellProperty {
    is_active: bool,
}

#[derive(Component)]
struct CellCoordinates {
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
        for j in 0..LINES_COUNT as usize {
            rows[i].push(
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: CELL_COLOR,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(CellCoordinates {
                        x: i as f32,
                        y: j as f32,
                    })
                    .insert(CellProperty { is_active: false })
                    .insert(Cell {})
                    .id(),
            );
        }
    }

    *cells = Cells(rows);
}

fn cell_translation(windows: Res<Windows>, mut q: Query<(&CellCoordinates, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        assert!(bound_window > 0., "bound_window need to be more than 0");
        assert!(bound_game > 0., "bound_game should be greater than 0");

        let tile_size = bound_window / bound_game;

        // because Sprite renders from center, we need to calculate its position
        // in actual table.
        // at first we calculate new position with deleting cells count by 2 and then
        // increasing that number from position.
        // Then we mulitply it by tile_size and add half of tile_size for centering
        (pos - (bound_game / 2.)) * tile_size + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap();
    let padding_size = WINDOW_PADDING * 2.;
    assert!(padding_size > 0., "padding_size should be greater than 0");
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() - padding_size, LINES_COUNT),
            convert(pos.y as f32, window.width() - padding_size, LINES_COUNT),
            0.,
        )
    }
}

// Function gets every cell, calculates tile size and then decreases size by some padding
fn cell_size_scaling(windows: Res<Windows>, mut q: Query<(&Cell, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    let tile_padding = 5.;
    let padding_size = WINDOW_PADDING * 2.;
    let tile_size = (window.width() - padding_size) / LINES_COUNT;
    for (_, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(tile_size - tile_padding, tile_size - tile_padding, 1.);
    }
}

// Changes visibility of cell depends of cell active status `is_active`
fn cell_visibility(mut q: Query<(&CellProperty, With<Cell>, &mut Sprite)>) {
    for (prop, _, mut sprite) in q.iter_mut() {
        if prop.is_active {
            sprite.color = *CELL_COLOR.clone().as_rgba().set_a(1.);
        } else {
            sprite.color = *CELL_COLOR.clone().as_rgba().set_a(0.);
        }
    }
}

// ** Click Logic **
fn click_handler(
    windows: Res<Windows>,
    mut mouse_button_event: EventReader<MouseButtonInput>,
    mut q: Query<(&CellCoordinates, &mut CellProperty)>,
) {
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

                for (coordinates, mut prop) in q.iter_mut() {
                    if coordinates.x == x - 1. && coordinates.y == y - 1. {
                        if prop.is_active {
                            prop.is_active = false;
                        } else {
                            prop.is_active = true;
                        }
                    }
                }

                println!("x, y: {:#?}, {:#?}", x, y);
            } else {
                println!("cursor out of field");
            }
        }
    }
}

// ** Behaviour **
// TODO: I need to have global variable like start_game in order to stop/run the game. After that I can add UI

// If there is more than 3 heighbours -> cell will die, if there is less than 2 neighbours -> cell will die
// For cells at the edge I calculate neighbours from opposite edge -> I mean infinity field
fn is_cell_will_live(
    cells: &Cells,
    cellCoordinates: CellCoordinates,
    current_status: bool,
) -> bool {
    let mut neighbours = 0;

    // check 8 neigbours

    if current_status {
        neighbours == 2 || neighbours == 3
    } else {
        neighbours == 3
    }
}

// fn update_state
