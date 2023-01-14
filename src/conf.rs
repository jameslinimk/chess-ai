//! Static config values

use macroquad::prelude::{color_u8, Color};

// Config for screen
pub const EXTRA_WIDTH: f32 = 125.0;
pub const SQUARE_SIZE: f32 = 64.0;
pub const MARGIN: f32 = 16.0;
pub const WIDTH: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 3 + EXTRA_WIDTH as i32;
pub const HEIGHT: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;
pub const CENTER_WIDTH: i32 = WIDTH / 2;
pub const CENTER_HEIGHT: i32 = HEIGHT / 2;

// Colors
pub const COLOR_WHITE: Color = color_u8!(235, 216, 183, 255);
pub const COLOR_BLACK: Color = color_u8!(172, 136, 103, 255);
pub const COLOR_BACKGROUND: Color = color_u8!(0, 0, 0, 255);
pub const COLOR_SELECTED: Color = color_u8!(0, 0, 0, 128);
pub const COLOR_LAST_MOVE: Color = color_u8!(204, 208, 119, 128);
pub const COLOR_HIGHLIGHT: Color = color_u8!(238, 75, 43, 255);
pub const COLOR_ARROW: Color = color_u8!(238, 75, 43, 255);
pub const COLOR_BUTTON: Color = color_u8!(127, 166, 80, 255);
pub const COLOR_BUTTON_HOVER: Color = color_u8!(149, 187, 74, 255);
pub const COLOR_BUTTON_PRESSED: Color = color_u8!(138, 172, 70, 255);

// Config for board
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const TEST_FEN: &str = "r1bq1rk1/pppnnppp/8/3p4/3P2Q1/2P1P1P1/PP3PP1/RN2KBNR w KQ - 15 9";
