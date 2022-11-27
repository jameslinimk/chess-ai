use std::collections::HashMap;

use lazy_static::lazy_static;
use macroquad::color_u8;
use macroquad::prelude::Color;
use maplit::hashmap;

use crate::pieces::piece::PieceNames;

pub const SQUARE_SIZE: f32 = 64.0;
pub const MARGIN: f32 = 16.0;
pub const WIDTH: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;
pub const HEIGHT: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;

pub const COLOR_WHITE: Color = color_u8!(238, 238, 210, 255);
pub const COLOR_BLACK: Color = color_u8!(118, 150, 86, 255);
pub const COLOR_BACKGROUND: Color = color_u8!(0, 0, 0, 255);
pub const COLOR_SELECTED: Color = color_u8!(0, 0, 0, 128);

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
pub const TEST_FEN: &str = "k7/4q3/8/8/4R3/8/4K3/7P";

pub const CASTLE_VALUE: i32 = 5;
pub const CHECK_VALUE: i32 = 50;
pub const CHECKMATE_VALUE: i32 = 100;
pub const STALEMATE_VALUE: i32 = 0;

lazy_static! {
    pub static ref PIECE_VALUES: HashMap<PieceNames, i32> = hashmap! {
        PieceNames::Pawn => 1,
        PieceNames::Knight => 3,
        PieceNames::Bishop => 3,
        PieceNames::Rook => 5,
        PieceNames::Queen => 9,
        PieceNames::King => 100,
    };
}
