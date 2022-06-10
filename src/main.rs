use bevy::{
    core_pipeline::ClearColor,
    hierarchy::{BuildChildren, Children},
    input::mouse::MouseButtonInput,
    math::{Rect, Size, Vec3},
    prelude::{
        default, App, AssetServer, Button, ButtonBundle, Changed, Color, Commands, Component,
        CoreStage, Deref, DerefMut, Entity, EventReader, OrthographicCameraBundle, Plugin, Query,
        Res, ResMut, State, SystemSet, SystemStage, TextBundle, Transform, UiCameraBundle, With,
    },
    sprite::{Sprite, SpriteBundle},
    text::{Text, TextStyle},
    ui::{AlignItems, Interaction, JustifyContent, Style, UiColor, Val},
    window::{WindowDescriptor, Windows},
    DefaultPlugins,
};
use config::{
    BACKGROUND_COLOR, CELL_COLOR, HOVERED_BUTTON, LINES_COUNT, MAIN_COLOR, NORMAL_BUTTON,
    PRESSED_BUTTON, WINDOW_PADDING, WINDOW_SIZE,
};
use iyes_loopless::prelude::*;
use std::time::Duration;

mod config;

fn main() {
    let mut update_state_stage = SystemStage::parallel();
    update_state_stage.add_system(update_state.run_in_bevy_state(GameState::Play));

    App::new()
        .add_plugin(GameSetup)
        .add_system(click_handler)
        .add_stage_after(
            CoreStage::Update,
            "UPDATE_STATE",
            FixedTimestepStage::new(Duration::from_millis(100)).with_stage(update_state_stage),
        )
        .add_system(button_system)
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(cell_translation)
                .with_system(cell_size_scaling)
                .with_system(cell_visibility)
                .with_system(size_scaling),
        )
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
                resizable: false,
                ..default()
            })
            .add_startup_system(setup_camera)
            .add_startup_system(setup_field)
            .add_startup_system(prefill_cells)
            .add_startup_system(setup_button)
            .insert_resource(Lines::default())
            .insert_resource(Cells::default())
            .add_state(GameState::Stop)
            .add_plugins(DefaultPlugins);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn setup_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: Rect {
                    bottom: Val::Px(20.0),
                    left: Val::Auto,
                    top: Val::Auto,
                    right: Val::Auto,
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Start!",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    },
                    Default::default(),
                ),
                ..default()
            });
        });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut game_state: ResMut<State<GameState>>,
) {
    for (interaction, mut color, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                if game_state.current() == &GameState::Stop {
                    text.sections[0].value = "Stop".to_string();
                    game_state.set(GameState::Play).unwrap();
                } else {
                    text.sections[0].value = "Start".to_string();
                    game_state.set(GameState::Stop).unwrap();
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Stop,
    Play,
}

#[derive(Default, Deref, DerefMut)]
struct Lines(Vec<Entity>);

#[derive(Debug, Default, Deref, DerefMut)]
struct Cells(Vec<Vec<Entity>>);

#[derive(Component)]
struct Cell {}

#[derive(Component, Debug)]
struct CellProperty {
    is_active: bool,
}

#[derive(Component, Debug)]
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

                println!("x, y: {:#?}, {:#?}", x - 1., y - 1.);
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
fn update_state(
    mut commands: Commands,
    mut cells: ResMut<Cells>,
    q: Query<(&CellCoordinates, &mut CellProperty)>,
) {
    // new array with updated state of cells
    let mut new_cells: Vec<Vec<Entity>> = Vec::new();

    // prefill new state with empty data
    for i in 0..LINES_COUNT as usize {
        new_cells.push(Vec::new());
        for _ in 0..LINES_COUNT as usize {
            new_cells[i].push(
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: CELL_COLOR,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(Cell {})
                    .id(),
            );
        }
    }

    // Calculate count of neighbours for every cell
    // if cell's neighbour out of field - we will check first cell from opposite edge
    for (coordinate, prop) in q.iter() {
        let mut neighbours = 0;

        let y_max = if coordinate.y == LINES_COUNT - 1. {
            0.
        } else {
            coordinate.y + 1.
        };
        let y_min = if coordinate.y == 0. {
            LINES_COUNT - 1.
        } else {
            coordinate.y - 1.
        };
        let y_range = vec![y_min as usize, coordinate.y as usize, y_max as usize];

        let x_max = if coordinate.x == LINES_COUNT - 1. {
            0.
        } else {
            coordinate.x + 1.
        };
        let x_min = if coordinate.x == 0. {
            LINES_COUNT - 1.
        } else {
            coordinate.x - 1.
        };
        let x_range = vec![x_min as usize, coordinate.x as usize, x_max as usize];

        for i in 0..3 {
            for j in 0..3 {
                // Current cell
                if i == 1 && j == 1 {
                    continue;
                }

                let neighbour_prop = q
                    .get_component::<CellProperty>(cells[x_range[i]][y_range[j]])
                    .expect("Error getting component");

                if neighbour_prop.is_active {
                    neighbours += 1;
                }
            }
        }

        if prop.is_active {
            if neighbours == 2 || neighbours == 3 {
                // alive -> alive
                commands
                    .entity(new_cells[coordinate.x as usize][coordinate.y as usize])
                    .insert(CellCoordinates {
                        x: coordinate.x as f32,
                        y: coordinate.y as f32,
                    })
                    .insert(CellProperty { is_active: true });
            } else {
                // alive -> die
                commands
                    .entity(new_cells[coordinate.x as usize][coordinate.y as usize])
                    .insert(CellCoordinates {
                        x: coordinate.x as f32,
                        y: coordinate.y as f32,
                    })
                    .insert(CellProperty { is_active: false });
            }
        } else {
            if neighbours == 3 {
                // die -> alive
                commands
                    .entity(new_cells[coordinate.x as usize][coordinate.y as usize])
                    .insert(CellCoordinates {
                        x: coordinate.x as f32,
                        y: coordinate.y as f32,
                    })
                    .insert(CellProperty { is_active: true });
            } else {
                // die -> die
                commands
                    .entity(new_cells[coordinate.x as usize][coordinate.y as usize])
                    .insert(CellCoordinates {
                        x: coordinate.x as f32,
                        y: coordinate.y as f32,
                    })
                    .insert(CellProperty { is_active: false });
            }
        }
    }

    // Delete previous state of cells
    for i in 0..LINES_COUNT as usize {
        for j in 0..LINES_COUNT as usize {
            commands.entity(cells[i][j]).despawn();
        }
    }

    // Set new state
    *cells = Cells(new_cells);
}
