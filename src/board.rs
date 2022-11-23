use derive_new::new;
use macroquad::prelude::{BLACK, WHITE};
use macroquad::shapes::draw_rectangle;
use macroquad::texture::draw_texture;

use crate::conf::{get_piece_image, COLOR_BLACK, COLOR_WHITE, MARGIN, SQUARE_SIZE};
use crate::pieces::piece::Piece;
use crate::util::Loc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

/// Represents a chess board and metadata
#[derive(new)]
pub struct Board {
    /// Array with the raw 8x8 board data
    #[new(value = "[[None; 8]; 8]")]
    pub raw: [[Option<(Piece, Color)>; 8]; 8],

    /// True if black can castle
    #[new(value = "true")]
    pub castle_black: bool,

    /// True if white can castle
    #[new(value = "true")]
    pub castle_white: bool,

    #[new(value = "vec![]")]
    pub move_history: Vec<(Loc, Loc)>,
}
impl Board {
    /// Generate a board given a fen string
    pub fn from_fen(fen: &str) {
        let segments: Vec<&str> = fen.split(' ').collect();

        let board = segments[0];
        for i in 1..=8 {}
    }

    pub fn move_piece(&mut self, from: Loc, to: Loc) {
        self.move_history.push((from, to));
        self.raw[to.y][to.x] = self.raw[from.y][from.x];
        self.raw[from.y][from.x] = None;
    }

    pub fn draw(&self) {
        for (y, row) in self.raw.iter().enumerate() {
            for (x, square) in row.iter().enumerate() {
                let color = if (x + y) % 2 == 0 {
                    COLOR_WHITE
                } else {
                    COLOR_BLACK
                };
                draw_rectangle(
                    MARGIN + SQUARE_SIZE * x as f32,
                    MARGIN + SQUARE_SIZE * y as f32,
                    SQUARE_SIZE,
                    SQUARE_SIZE,
                    color,
                );

                if let Some(piece) = square {
                    draw_texture(
                        get_piece_image(*piece),
                        MARGIN + SQUARE_SIZE * x as f32,
                        MARGIN + SQUARE_SIZE * y as f32,
                        WHITE,
                    )
                }
            }
        }
    }
}
