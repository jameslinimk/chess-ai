use derive_new::new;
use macroquad::texture::Texture2D;

use super::bishop::bishop_moves;
use super::king::king_moves;
use super::knight::knight_moves;
use super::pawn::pawn_moves;
use super::queen::queen_moves;
use super::rook::rook_moves;
use crate::assets::get_image;
use crate::board::{Board, ChessColor};
use crate::util::Loc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PieceNames {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, new, Debug)]
pub struct Piece {
    /// Type of piece
    pub name: PieceNames,
    /// The color of the piece
    pub color: ChessColor,
    /// Piece's current position on the board
    /// - Make sure to update this when moving the piece
    pub pos: Loc,
}

impl Piece {
    /// Get valid moves for this piece
    pub fn get_moves(&self, board: &Board) -> Vec<Loc> {
        if self.color != board.turn {
            return vec![];
        }

        match self.name {
            PieceNames::Pawn => pawn_moves(self, board),
            PieceNames::Knight => knight_moves(self, board),
            PieceNames::King => {
                let mut moves = king_moves(self, board);
                moves.retain(|&to| {
                    let attacks = if self.color == ChessColor::White {
                        &board.attack_black
                    } else {
                        &board.attack_white
                    };
                    !attacks.contains(&to)
                });
                moves
            }
            PieceNames::Rook => rook_moves(self, board),
            PieceNames::Bishop => bishop_moves(self, board),
            PieceNames::Queen => queen_moves(self, board),
        }
    }

    pub fn get_raw_moves(&self, board: &Board) -> Vec<Loc> {
        match self.name {
            PieceNames::Pawn => pawn_moves(self, board),
            PieceNames::Knight => knight_moves(self, board),
            PieceNames::King => king_moves(self, board),
            PieceNames::Rook => rook_moves(self, board),
            PieceNames::Bishop => bishop_moves(self, board),
            PieceNames::Queen => queen_moves(self, board),
        }
    }

    /// Get image texture for this piece
    pub fn get_image(&self) -> Texture2D {
        let path = match self.color {
            ChessColor::White => match self.name {
                PieceNames::Pawn => "assets/white_pawn.png",
                PieceNames::Rook => "assets/white_rook.png",
                PieceNames::Knight => "assets/white_knight.png",
                PieceNames::Bishop => "assets/white_bishop.png",
                PieceNames::Queen => "assets/white_queen.png",
                PieceNames::King => "assets/white_king.png",
            },
            ChessColor::Black => match self.name {
                PieceNames::Pawn => "assets/black_pawn.png",
                PieceNames::Rook => "assets/black_rook.png",
                PieceNames::Knight => "assets/black_knight.png",
                PieceNames::Bishop => "assets/black_bishop.png",
                PieceNames::Queen => "assets/black_queen.png",
                PieceNames::King => "assets/black_king.png",
            },
        };
        get_image(path)
    }

    /// Get the piece value
    pub fn get_value(&self) -> i32 {
        match self.name {
            PieceNames::Pawn => 1,
            PieceNames::Knight => 3,
            PieceNames::Bishop => 3,
            PieceNames::Rook => 5,
            PieceNames::Queen => 9,
            PieceNames::King => 100,
        }
    }
}
