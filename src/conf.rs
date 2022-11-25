use macroquad::color_u8;
use macroquad::prelude::Color;

pub const SQUARE_SIZE: f32 = 64.0;
pub const MARGIN: f32 = 16.0;
pub const WIDTH: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;
pub const HEIGHT: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;

pub const COLOR_WHITE: Color = color_u8!(238, 238, 210, 255);
pub const COLOR_BLACK: Color = color_u8!(118, 150, 86, 255);
pub const COLOR_BACKGROUND: Color = color_u8!(0, 0, 0, 255);
pub const COLOR_SELECTED: Color = color_u8!(0, 0, 0, 128);

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
pub const TEST_FEN: &str = "rkbqkbkr/pppp4/8/5pP1/3Pp1P1/2KQP2N/PPPBB2P/R3K2R";
