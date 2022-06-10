use bevy::{prelude::Color, ui::UiColor};

pub const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const CELL_COLOR: Color = Color::rgb(0.3, 0.8, 0.5);
pub const MAIN_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.1);
pub const WINDOW_SIZE: f32 = 800.0;
pub const WINDOW_PADDING: f32 = 100.0;
pub const LINES_COUNT: f32 = 40.0;

pub const NORMAL_BUTTON: UiColor = UiColor(Color::rgb(0.15, 0.15, 0.15));
pub const HOVERED_BUTTON: UiColor = UiColor(Color::rgb(0.25, 0.25, 0.25));
pub const PRESSED_BUTTON: UiColor = UiColor(Color::rgb(0.35, 0.75, 0.35));
