// TODO: Or there is too much lines with window resize, or clearcolor doesn't happened
use bevy::{
    core_pipeline::ClearColor,
    math::{Vec2, Vec3},
    prelude::{
        default, App, Commands, CoreStage, Deref, DerefMut, OrthographicCameraBundle, Plugin, Res,
        ResMut, SystemSet, Transform,
    },
    sprite::{Sprite, SpriteBundle},
    window::{WindowDescriptor, Windows},
    DefaultPlugins,
};
use config::{BACKGROUND_COLOR, MAIN_COLOR, WINDOW_PADDING, WINDOW_SIZE};

mod config;

fn main() {
    App::new()
        .add_plugin(GameSetup)
        .add_system(setup_field)
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
            .add_plugins(DefaultPlugins)
            .insert_resource(StepSize(10.0))
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new().with_system(size_scaling),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

#[derive(Default, Deref, DerefMut)]
struct StepSize(f32);

fn size_scaling(windows: Res<Windows>, mut step_size: ResMut<StepSize>) {
    let window = windows
        .get_primary()
        .expect("error while getting primary window");

    *step_size = StepSize((window.width() - (WINDOW_PADDING * 2.)) / 30.);
}

/// Spawn field, filled by lines, which create
/// an ordered bunch of squares
///
/// TODO: move to separete file?
fn setup_field(mut commands: Commands, step_size: Res<StepSize>) {
    const FIELD_SIZE: f32 = WINDOW_SIZE - WINDOW_PADDING;
    let half_of_field: i32 = (FIELD_SIZE / step_size.0 / 2.0) as i32;
    // Store lines with predefined capacity that equals lines count
    let mut lines: Vec<SpriteBundle> = Vec::with_capacity(
        (half_of_field / 2 + 2)
            .try_into()
            .expect("converting i32 to usize error"),
    );

    // calculates steps for iterator for creating lines
    // at first it calculates centre of field and than it finds
    // starting position of lines and end
    let calculate_range = || (-1 * half_of_field, half_of_field + 1);
    let range = calculate_range();

    // TODO: refactor this, initialise global structure Size and add system that calculates
    // size depend of window size, padding and cells_count
    fn get_line_sprite(position_x: f32, position_y: f32, is_vertical: bool) -> SpriteBundle {
        let size: Vec2 = match is_vertical {
            true => Vec2::new(FIELD_SIZE, 1.0),
            false => Vec2::new(1.0, FIELD_SIZE),
        };

        SpriteBundle {
            sprite: Sprite {
                color: MAIN_COLOR,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(position_x, position_y, 0.0),
                ..default()
            },
            ..default()
        }
    }

    // because sprites are aligned from centre
    // left edge of window is window_size / 2 * -1
    // I use 16 in order to have right and top border
    for i in range.0..range.1 {
        lines.push(get_line_sprite(i as f32 * step_size.0, 0.0, false));
        lines.push(get_line_sprite(0.0, i as f32 * step_size.0, true));
    }

    commands.spawn_batch(lines);
}
