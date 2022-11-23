use std::collections::HashMap;

use lazy_static::lazy_static;
use macroquad::color_u8;
use macroquad::prelude::Color;
use macroquad::texture::Texture2D;
use maplit::hashmap;

use crate::assets::get_image;
use crate::board::Color as BoardColor;
use crate::pieces::piece::Piece;

pub const SQUARE_SIZE: f32 = 64.0;
pub const MARGIN: f32 = 16.0;
pub const WIDTH: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;
pub const HEIGHT: i32 = SQUARE_SIZE as i32 * 8 + MARGIN as i32 * 2;

pub const COLOR_WHITE: Color = color_u8!(238, 238, 210, 255);
pub const COLOR_BLACK: Color = color_u8!(118, 150, 86, 255);
pub const COLOR_BACKGROUND: Color = color_u8!(0, 0, 0, 255);

lazy_static! {
    pub static ref IMAGES_WHITE: HashMap<Piece, &'static str> = hashmap! {
        Piece::Pawn => "assets/white_pawn.png",
        Piece::Rook => "assets/white_rook.png",
        Piece::Knight => "assets/white_knight.png",
        Piece::Bishop => "assets/white_bishop.png",
        Piece::Queen => "assets/white_queen.png",
        Piece::King => "assets/white_king.png",
    };
    pub static ref IMAGES_BLACK: HashMap<Piece, &'static str> = hashmap! {
        Piece::Pawn => "assets/black_pawn.png",
        Piece::Rook => "assets/black_rook.png",
        Piece::Knight => "assets/black_knight.png",
        Piece::Bishop => "assets/black_bishop.png",
        Piece::Queen => "assets/black_queen.png",
        Piece::King => "assets/black_king.png",
    };
}

/// Get the texture for a piece and color
pub fn get_piece_image(piece: (Piece, BoardColor)) -> Texture2D {
    let name = match piece.1 {
        BoardColor::White => IMAGES_WHITE.get(&piece.0).unwrap(),
        BoardColor::Black => IMAGES_BLACK.get(&piece.0).unwrap(),
    };
    get_image(name)
}
