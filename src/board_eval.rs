//! Part of [Board], split for readability
//!
//! Contains all the functions related to calculating the score of the board / move. Used for the minimax search

use lazy_static::lazy_static;
use macroquad::prelude::warn;
use rustc_hash::FxHashMap;

use crate::board::{Board, BoardState, ChessColor};
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::Loc;
use crate::{color_ternary, hashmap, ternary};

#[macro_export]
macro_rules! rev_arrays {
    ($arr:expr) => {{
        let mut rev = $arr;
        rev.reverse();
        ($arr, rev)
    }};
}

type Table = [[i32; 8]; 8];
lazy_static! {
    /// (`white`, `black`)
    static ref PIECE_TABLES: FxHashMap<PieceNames, (Table, Table)> = hashmap! {
        PieceNames::Pawn => rev_arrays!([
            [0, 0, 0, 0, 0, 0, 0, 0],
            [50, 50, 50, 50, 50, 50, 50, 50],
            [10, 10, 20, 30, 30, 20, 10, 10],
            [5, 5, 10, 25, 25, 10, 5, 5],
            [0, 0, 0, 20, 20, 0, 0, 0],
            [5, -5, -10, 0, 0, -10, -5, 5],
            [5, 10, 10, -20, -20, 10, 10, 5],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ]),
        PieceNames::Bishop => rev_arrays!([
            [-20, -10, -10, -10, -10, -10, -10, -20],
            [-10, 0, 0, 0, 0, 0, 0, -10],
            [-10, 0, 5, 10, 10, 5, 0, -10],
            [-10, 5, 5, 10, 10, 5, 5, -10],
            [-10, 0, 10, 10, 10, 10, 0, -10],
            [-10, 10, 10, 10, 10, 10, 10, -10],
            [-10, 5, 0, 0, 0, 0, 5, -10],
            [-20, -10, -10, -10, -10, -10, -10, -20],
        ]),
        PieceNames::Knight => rev_arrays!([
            [-50, -40, -30, -30, -30, -30, -40, -50],
            [-40, -20, 0, 0, 0, 0, -20, -40],
            [-30, 0, 10, 15, 15, 10, 0, -30],
            [-30, 5, 15, 20, 20, 15, 5, -30],
            [-30, 0, 15, 20, 20, 15, 0, -30],
            [-30, 5, 10, 15, 15, 10, 5, -30],
            [-40, -20, 0, 5, 5, 0, -20, -40],
            [-50, -40, -30, -30, -30, -30, -40, -50],
        ]),
        PieceNames::Rook => rev_arrays!([
            [0, 0, 0, 0, 0, 0, 0, 0],
            [5, 10, 10, 10, 10, 10, 10, 5],
            [-5, 0, 0, 0, 0, 0, 0, -5],
            [-5, 0, 0, 0, 0, 0, 0, -5],
            [-5, 0, 0, 0, 0, 0, 0, -5],
            [-5, 0, 0, 0, 0, 0, 0, -5],
            [-5, 0, 0, 0, 0, 0, 0, -5],
            [0, 0, 0, 5, 5, 0, 0, 0],
        ]),
        PieceNames::Queen => rev_arrays!([
            [-20, -10, -10, -5, -5, -10, -10, -20],
            [-10, 0, 0, 0, 0, 0, 0, -10],
            [-10, 0, 5, 5, 5, 5, 0, -10],
            [-5, 0, 5, 5, 5, 5, 0, -5],
            [0, 0, 5, 5, 5, 5, 0, -5],
            [-10, 5, 5, 5, 5, 5, 0, -10],
            [-10, 0, 5, 0, 0, 0, 0, -10],
            [-20, -10, -10, -5, -5, -10, -10, -20],
        ]),
    };

    /// (Middle game (`black`, `white`), End game (`black`, `white`))
    static ref KING_TABLE: ((Table, Table), (Table, Table)) = (
        rev_arrays!([
            [-30, -40, -40, -50, -50, -40, -40, -30],
            [-30, -40, -40, -50, -50, -40, -40, -30],
            [-30, -40, -40, -50, -50, -40, -40, -30],
            [-30, -40, -40, -50, -50, -40, -40, -30],
            [-20, -30, -30, -40, -40, -30, -30, -20],
            [-10, -20, -20, -20, -20, -20, -20, -10],
            [20, 20, -10, -10, -10, -10, 20, 20],
            [20, 30, 10, 0, 0, 10, 30, 20],
        ]),
        rev_arrays!([
            [-50, -40, -30, -20, -20, -30, -40, -50],
            [-30, -20, -10, 0, 0, -10, -20, -30],
            [-30, -10, 20, 30, 30, 20, -10, -30],
            [-30, -10, 30, 40, 40, 30, -10, -30],
            [-30, -10, 30, 40, 40, 30, -10, -30],
            [-30, -10, 20, 30, 30, 20, -10, -30],
            [-30, -30, 0, 0, 0, 0, -30, -30],
            [-50, -30, -30, -30, -30, -30, -30, -50],
        ])
    );
}

fn table_value(piece: &Piece, endgame: bool) -> i32 {
    let table = if piece.name == PieceNames::King {
        ternary!(endgame, &KING_TABLE.1, &KING_TABLE.0)
    } else {
        &PIECE_TABLES[&piece.name]
    };

    color_ternary!(piece.color, table.0, table.1)[piece.pos.1][piece.pos.0]
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
    piece_value(&piece.name) + table_value(piece, endgame)
}

pub const CHECK_VALUE: i32 = 50;
pub const CHECKMATE_VALUE: i32 = 20000;
pub const STALEMATE_VALUE: i32 = -100;

impl Board {
    pub fn get_sorted_moves(&self, color: ChessColor) -> Vec<(Loc, Loc)> {
        let mut moves = self.get_moves(color);

        color_ternary!(
            color,
            moves.sort_unstable_by(|a, b| {
                self.move_value(&a.0, &a.1)
                    .cmp(&self.move_value(&b.0, &b.1))
            }),
            moves.sort_unstable_by(|a, b| {
                self.move_value(&b.0, &b.1)
                    .cmp(&self.move_value(&a.0, &a.1))
            })
        );

        moves
    }

    /// Calculates the score of the board, for the white
    pub fn get_score(&self) -> i32 {
        let mut score = 0;

        match self.state {
            BoardState::Checkmate(check_color) => {
                return color_ternary!(check_color, -CHECKMATE_VALUE, CHECKMATE_VALUE);
            }
            BoardState::Stalemate | BoardState::Draw => {
                return STALEMATE_VALUE;
            }
            BoardState::Check(check_color) => {
                color_ternary!(check_color, score -= CHECK_VALUE, score += CHECK_VALUE);
            }
            _ => {}
        }

        // Add value based on pieces
        for piece in self.raw.iter().flatten().flatten() {
            let value = full_piece_value(piece, self.endgame);
            color_ternary!(piece.color, score += value, score -= value);
        }

        score
    }

    pub fn move_value(&self, from: &Loc, to: &Loc) -> i32 {
        let piece = match self.get(from) {
            Some(piece) => piece,
            None => {
                warn!("Tried to get value of move from empty square");
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
        if let Some(capture_pos) = self.is_capture(from, to) {
            score += piece.get_value() - self.get(&capture_pos).unwrap().get_value();
        }

        score
    }
}
