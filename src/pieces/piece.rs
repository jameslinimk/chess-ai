use derive_new::new;
use macroquad::texture::Texture2D;

use super::bishop::{bishop_attacks, bishop_moves};
use super::king::{king_attacks, king_moves};
use super::knight::{knight_attacks, knight_moves};
use super::pawn::{pawn_attacks, pawn_moves};
use super::queen::{queen_attacks, queen_moves};
use super::rook::{rook_attacks, rook_moves};
use crate::assets::get_image;
use crate::board::{Board, ChessColor};
use crate::board_eval::piece_value;
use crate::color_ternary;
use crate::util::Loc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum PieceNames {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, new)]
pub(crate) struct Piece {
    /// Type of piece
    pub(crate) name: PieceNames,
    /// The color of the piece
    pub(crate) color: ChessColor,
    /// Piece's current position on the board
    /// - Make sure to update this when moving the piece
    pub(crate) pos: Loc,
}

impl Piece {
    /// Get valid moves for this piece
    pub(crate) fn moves(&self, board: &Board) -> Vec<Loc> {
        let mut temp_moves = match self.name {
            PieceNames::Pawn => pawn_moves(self, board),
            PieceNames::Knight => knight_moves(self, board),
            PieceNames::King => {
                let mut moves = king_moves(self, board);
                moves.retain(|&to| {
                    let attacks =
                        color_ternary!(self.color, &board.attacks_black, &board.attacks_white);
                    !attacks.contains(&to)
                });
                moves
            }
            PieceNames::Rook => rook_moves(self, board),
            PieceNames::Bishop => bishop_moves(self, board),
            PieceNames::Queen => queen_moves(self, board),
        };

        if board.blockers.contains(&self.pos)
            || color_ternary!(self.color, board.check_white, board.check_black)
        {
            let new_board = board.clone();
            temp_moves.retain(|&to| {
                let mut new_board = new_board.clone();
                new_board.move_piece(&self.pos, &to, false);
                color_ternary!(self.color, !new_board.check_white, !new_board.check_black)
            });
        }

        temp_moves
    }

    /// Get squares that are attacked by this piece
    pub(crate) fn attacks(&self, board: &Board) -> Vec<Loc> {
        match self.name {
            PieceNames::Pawn => pawn_attacks(self),
            PieceNames::Knight => knight_attacks(self),
            PieceNames::King => king_attacks(self),
            PieceNames::Rook => rook_attacks(self, board),
            PieceNames::Bishop => bishop_attacks(self, board),
            PieceNames::Queen => queen_attacks(self, board),
        }
    }

    /// Get image texture for this piece
    pub(crate) fn image(&self) -> Texture2D {
        let path = match self.color {
            ChessColor::White => match self.name {
                PieceNames::Pawn => "assets/pieces/white_pawn.png",
                PieceNames::Rook => "assets/pieces/white_rook.png",
                PieceNames::Knight => "assets/pieces/white_knight.png",
                PieceNames::Bishop => "assets/pieces/white_bishop.png",
                PieceNames::Queen => "assets/pieces/white_queen.png",
                PieceNames::King => "assets/pieces/white_king.png",
            },
            ChessColor::Black => match self.name {
                PieceNames::Pawn => "assets/pieces/black_pawn.png",
                PieceNames::Rook => "assets/pieces/black_rook.png",
                PieceNames::Knight => "assets/pieces/black_knight.png",
                PieceNames::Bishop => "assets/pieces/black_bishop.png",
                PieceNames::Queen => "assets/pieces/black_queen.png",
                PieceNames::King => "assets/pieces/black_king.png",
            },
        };
        get_image(path)
    }

    /// Get the piece value
    pub(crate) fn value(&self) -> i32 {
        piece_value(&self.name)
    }
}
