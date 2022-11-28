use macroquad::prelude::{color_u8, Color};

// Config for screen
pub const BUTTON_HEIGHT: f32 = 50.0;
pub const SQUARE_SIZE: f32 = 64.0;
pub const MARGIN: f32 = 16.0;
pub const WIDTH: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;
pub const HEIGHT: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 3 + BUTTON_HEIGHT as i32 * 2;

// Colors
pub const COLOR_WHITE: Color = color_u8!(238, 238, 210, 255);
pub const COLOR_BLACK: Color = color_u8!(118, 150, 86, 255);
pub const COLOR_BACKGROUND: Color = color_u8!(0, 0, 0, 255);
pub const COLOR_SELECTED: Color = color_u8!(0, 0, 0, 128);
pub const COLOR_BUTTON: Color = color_u8!(127, 166, 80, 255);
pub const COLOR_BUTTON_HOVER: Color = color_u8!(149, 187, 74, 255);
pub const COLOR_BUTTON_PRESSED: Color = color_u8!(138, 172, 70, 255);

// Config for board
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
pub const TEST_FEN: &str = "k7/4q3/8/8/4R3/8/4K3/7P";

// Game state values
pub const CASTLE_VALUE: i32 = 5;
pub const CHECK_VALUE: i32 = 50;
pub const CHECKMATE_VALUE: i32 = 100;
pub const STALEMATE_VALUE: i32 = 0;
