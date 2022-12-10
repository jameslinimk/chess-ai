//! Part of [Board], split for readability
//!
//! Contains all the functions related to calculating the score of the board / move. Used for the minimax search

use crate::board::{Board, BoardState, ChessColor};
use crate::color_ternary;
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::Loc;

/// Taken from <https://www.chessprogramming.org/Simplified_Evaluation_Function>
fn piece_table(piece: &Piece, endgame: bool) -> [[i32; 8]; 8] {
    match piece.name {
        PieceNames::Pawn => color_ternary!(
            piece.color,
            [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [50, 50, 50, 50, 50, 50, 50, 50],
                [10, 10, 20, 30, 30, 20, 10, 10],
                [5, 5, 10, 25, 25, 10, 5, 5],
                [0, 0, 0, 20, 20, 0, 0, 0],
                [5, -5, -10, 0, 0, -10, -5, 5],
                [5, 10, 10, -20, -20, 10, 10, 5],
                [0, 0, 0, 0, 0, 0, 0, 0],
            ],
            [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [5, 10, 10, -20, -20, 10, 10, 5],
                [5, -5, -10, 0, 0, -10, -5, 5],
                [0, 0, 0, 20, 20, 0, 0, 0],
                [5, 5, 10, 25, 25, 10, 5, 5],
                [10, 10, 20, 30, 30, 20, 10, 10],
                [50, 50, 50, 50, 50, 50, 50, 50],
                [0, 0, 0, 0, 0, 0, 0, 0]
            ]
        ),
        PieceNames::Bishop => color_ternary!(
            piece.color,
            [
                [-20, -10, -10, -10, -10, -10, -10, -20],
                [-10, 0, 0, 0, 0, 0, 0, -10],
                [-10, 0, 5, 10, 10, 5, 0, -10],
                [-10, 5, 5, 10, 10, 5, 5, -10],
                [-10, 0, 10, 10, 10, 10, 0, -10],
                [-10, 10, 10, 10, 10, 10, 10, -10],
                [-10, 5, 0, 0, 0, 0, 5, -10],
                [-20, -10, -10, -10, -10, -10, -10, -20],
            ],
            [
                [-20, -10, -10, -10, -10, -10, -10, -20],
                [-10, 5, 0, 0, 0, 0, 5, -10],
                [-10, 10, 10, 10, 10, 10, 10, -10],
                [-10, 0, 10, 10, 10, 10, 0, -10],
                [-10, 5, 5, 10, 10, 5, 5, -10],
                [-10, 0, 5, 10, 10, 5, 0, -10],
                [-10, 0, 0, 0, 0, 0, 0, -10],
                [-20, -10, -10, -10, -10, -10, -10, -20]
            ]
        ),
        PieceNames::Knight => color_ternary!(
            piece.color,
            [
                [-50, -40, -30, -30, -30, -30, -40, -50],
                [-40, -20, 0, 0, 0, 0, -20, -40],
                [-30, 0, 10, 15, 15, 10, 0, -30],
                [-30, 5, 15, 20, 20, 15, 5, -30],
                [-30, 0, 15, 20, 20, 15, 0, -30],
                [-30, 5, 10, 15, 15, 10, 5, -30],
                [-40, -20, 0, 5, 5, 0, -20, -40],
                [-50, -40, -30, -30, -30, -30, -40, -50],
            ],
            [
                [-50, -40, -30, -30, -30, -30, -40, -50],
                [-40, -20, 0, 5, 5, 0, -20, -40],
                [-30, 5, 10, 15, 15, 10, 5, -30],
                [-30, 0, 15, 20, 20, 15, 0, -30],
                [-30, 5, 15, 20, 20, 15, 5, -30],
                [-30, 0, 10, 15, 15, 10, 0, -30],
                [-40, -20, 0, 0, 0, 0, -20, -40],
                [-50, -40, -30, -30, -30, -30, -40, -50]
            ]
        ),
        PieceNames::Rook => color_ternary!(
            piece.color,
            [
                [0, 0, 0, 0, 0, 0, 0, 0],
                [5, 10, 10, 10, 10, 10, 10, 5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [0, 0, 0, 5, 5, 0, 0, 0],
            ],
            [
                [0, 0, 0, 5, 5, 0, 0, 0],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [-5, 0, 0, 0, 0, 0, 0, -5],
                [5, 10, 10, 10, 10, 10, 10, 5],
                [0, 0, 0, 0, 0, 0, 0, 0]
            ]
        ),
        PieceNames::Queen => color_ternary!(
            piece.color,
            [
                [-20, -10, -10, -5, -5, -10, -10, -20],
                [-10, 0, 0, 0, 0, 0, 0, -10],
                [-10, 0, 5, 5, 5, 5, 0, -10],
                [-5, 0, 5, 5, 5, 5, 0, -5],
                [0, 0, 5, 5, 5, 5, 0, -5],
                [-10, 5, 5, 5, 5, 5, 0, -10],
                [-10, 0, 5, 0, 0, 0, 0, -10],
                [-20, -10, -10, -5, -5, -10, -10, -20],
            ],
            [
                [-20, -10, -10, -5, -5, -10, -10, -20],
                [-10, 0, 5, 0, 0, 0, 0, -10],
                [-10, 5, 5, 5, 5, 5, 0, -10],
                [0, 0, 5, 5, 5, 5, 0, -5],
                [-5, 0, 5, 5, 5, 5, 0, -5],
                [-10, 0, 5, 5, 5, 5, 0, -10],
                [-10, 0, 0, 0, 0, 0, 0, -10],
                [-20, -10, -10, -5, -5, -10, -10, -20]
            ]
        ),
        PieceNames::King => match endgame {
            false => color_ternary!(
                piece.color,
                [
                    [-30, -40, -40, -50, -50, -40, -40, -30],
                    [-30, -40, -40, -50, -50, -40, -40, -30],
                    [-30, -40, -40, -50, -50, -40, -40, -30],
                    [-30, -40, -40, -50, -50, -40, -40, -30],
                    [-20, -30, -30, -40, -40, -30, -30, -20],
                    [-10, -20, -20, -20, -20, -20, -20, -10],
                    [20, 20, 0, 0, 0, 0, 20, 20],
                    [20, 30, 10, 0, 0, 10, 30, 20],
                ],
                [
                    [20, 30, 10, 0, 0, 10, 30, 20],
                    [20, 20, 0, 0, 0, 0, 20, 20],
                    [-10, -20, -20, -20, -20, -20, -20, -10],
                    [-20, -30, -30, -40, -40, -30, -30, -20],
                    [-30, -40, -40, -50, -50, -40, -40, -30],
                    [-30, -40, -40, -50, -50, -40, -40, -30],
                    [-30, -40, -40, -50, -50, -40, -40, -30],
                    [-30, -40, -40, -50, -50, -40, -40, -30]
                ]
            ),
            true => color_ternary!(
                piece.color,
                [
                    [-50, -40, -30, -20, -20, -30, -40, -50],
                    [-30, -20, -10, 0, 0, -10, -20, -30],
                    [-30, -10, 20, 30, 30, 20, -10, -30],
                    [-30, -10, 30, 40, 40, 30, -10, -30],
                    [-30, -10, 30, 40, 40, 30, -10, -30],
                    [-30, -10, 20, 30, 30, 20, -10, -30],
                    [-30, -30, 0, 0, 0, 0, -30, -30],
                    [-50, -30, -30, -30, -30, -30, -30, -50],
                ],
                [
                    [-50, -30, -30, -30, -30, -30, -30, -50],
                    [-30, -30, 0, 0, 0, 0, -30, -30],
                    [-30, -10, 20, 30, 30, 20, -10, -30],
                    [-30, -10, 30, 40, 40, 30, -10, -30],
                    [-30, -10, 30, 40, 40, 30, -10, -30],
                    [-30, -10, 20, 30, 30, 20, -10, -30],
                    [-30, -20, -10, 0, 0, -10, -20, -30],
                    [-50, -40, -30, -20, -20, -30, -40, -50]
                ]
            ),
        },
    }
}

pub fn piece_value(piece: &PieceNames) -> i32 {
    match piece {
        PieceNames::Pawn => 100,
        PieceNames::Knight => 320,
        PieceNames::Bishop => 330,
        PieceNames::Rook => 500,
        PieceNames::Queen => 900,
        PieceNames::King => 20000,
    }
}

pub fn full_piece_value(piece: &Piece, endgame: bool) -> i32 {
    piece_value(&piece.name) + piece_table(piece, endgame)[piece.pos.1][piece.pos.0]
}

pub const CHECK_VALUE: i32 = 50;
pub const CHECKMATE_VALUE: i32 = 20000;
pub const STALEMATE_VALUE: i32 = -100;

impl Board {
    pub fn get_sorted_moves(&self, color: ChessColor) -> Vec<(Loc, Loc)> {
        let mut moves = vec![];
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.color == color {
                    for m in piece.get_moves(self) {
                        moves.push((piece.pos, m));
                    }
                }
            }
        }
        moves.sort_unstable_by(|a, b| {
            self.move_value(&b.0, &b.1)
                .cmp(&self.move_value(&a.0, &a.1))
        });

        moves
    }

    /// Calculates the score of the board, for the color specified
    pub fn get_score(&self) -> i32 {
        match self.state {
            BoardState::Checkmate(check_color) => {
                return color_ternary!(check_color, -CHECKMATE_VALUE, CHECKMATE_VALUE);
            }
            BoardState::Stalemate | BoardState::Draw => {
                return STALEMATE_VALUE;
            }
            _ => {}
        }

        let mut score = 0;

        // Add value based on pieces
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                let value = full_piece_value(piece, self.endgame);
                color_ternary!(piece.color, score += value, score -= value);
            }
        }

        if let BoardState::Check(check_color) = self.state {
            color_ternary!(check_color, score -= CHECK_VALUE, score += CHECK_VALUE);
        }

        score
    }

    pub fn move_value(&self, from: &Loc, to: &Loc) -> i32 {
        let piece = match self.get(from) {
            Some(piece) => piece,
            None => {
                return -100;
            }
        };

        // Pawn promotion
        if piece.name == PieceNames::Pawn && (piece.pos.1 == 7 || piece.pos.1 == 0) {
            return i32::MAX;
        }

        let mut score = 0;

        // Position change
        let temp_piece = Piece { pos: *to, ..piece };
        score +=
            full_piece_value(&piece, self.endgame) - full_piece_value(&temp_piece, self.endgame);

        // Add value based on capture
        let (capture, capture_pos) = self.is_capture(from, to);
        if capture {
            score += piece.get_value() - self.get(&capture_pos).unwrap().get_value();
        }

        score
    }
}
