//! Static config values

use macroquad::prelude::{color_u8, Color};

// Config for screen
pub(crate) const EXTRA_WIDTH: f32 = 125.0;
pub(crate) const SQUARE_SIZE: f32 = 60.0;
pub(crate) const MARGIN: f32 = 16.0;
pub(crate) const WIDTH: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 3 + EXTRA_WIDTH as i32;
pub(crate) const HEIGHT: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;
pub(crate) const CENTER_WIDTH: i32 = WIDTH / 2;
pub(crate) const CENTER_HEIGHT: i32 = HEIGHT / 2;

// Colors
pub(crate) const COLOR_WHITE: Color = color_u8!(235, 216, 183, 255);
pub(crate) const COLOR_BLACK: Color = color_u8!(172, 136, 103, 255);
pub(crate) const COLOR_BACKGROUND: Color = color_u8!(0, 0, 0, 255);
pub(crate) const COLOR_SELECTED: Color = color_u8!(0, 0, 0, 128);
pub(crate) const COLOR_LAST_MOVE: Color = color_u8!(204, 208, 119, 128);
pub(crate) const COLOR_HIGHLIGHT: Color = color_u8!(238, 75, 43, 255);
pub(crate) const COLOR_ARROW: Color = color_u8!(238, 75, 43, 255);
pub(crate) const COLOR_BUTTON: Color = color_u8!(127, 166, 80, 255);
pub(crate) const COLOR_BUTTON_HOVER: Color = color_u8!(149, 187, 74, 255);
pub(crate) const COLOR_BUTTON_PRESSED: Color = color_u8!(138, 172, 70, 255);

// Config for board
pub(crate) const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub(crate) const FEN: &str = DEFAULT_FEN;
