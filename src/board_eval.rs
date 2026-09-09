//! Part of [Board], split for readability
//!
//! Contains all the functions related to calculating the score of the board / move. Used for the minimax search

use std::cmp::Reverse;

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
            [0,    0,    0,    0,    0,    0,    0,    0],
            [50,   50,   50,   50,   50,   50,   50,   50],
            [10,   10,   20,   30,   30,   20,   10,   10],
            [5,    5,    10,   25,   25,   10,   5,    5],
            [0,    0,    0,    20,   20,   0,    0,    0],
            [5,    -5,   -10,  0,    0,    -10,  -5,   5],
            [5,    10,   10,   -20,  -20,  10,   10,   5],
            [0,    0,    0,    0,    0,    0,    0,    0],
        ]),
        PieceNames::Bishop => rev_arrays!([
            [-20,  -10,  -10,  -10,  -10,  -10,  -10,  -20],
            [-10,  0,    0,    0,    0,    0,    0,    -10],
            [-10,  0,    5,    10,   10,   5,    0,    -10],
            [-10,  5,    5,    10,   10,   5,    5,    -10],
            [-10,  0,    10,   10,   10,   10,   0,    -10],
            [-10,  10,   10,   10,   10,   10,   10,   -10],
            [-10,  5,    0,    0,    0,    0,    5,    -10],
            [-20,  -10,  -50,  -10,  -10,  -50,  -10,  -20],
        ]),
        PieceNames::Knight => rev_arrays!([
            [-50,  -40,  -30,  -30,  -30,  -30,  -40,  -50],
            [-40,  -20,  0,    0,    0,    0,    -20,  -40],
            [-30,  0,    10,   15,   15,   10,   0,    -30],
            [-30,  5,    15,   20,   20,   15,   5,    -30],
            [-30,  0,    15,   20,   20,   15,   0,    -30],
            [-30,  5,    10,   15,   15,   10,   5,    -30],
            [-40,  -20,  0,    5,    5,    0,    -20,  -40],
            [-50,  -50,  -30,  -30,  -30,  -30,  -50,  -50],
        ]),
        PieceNames::Rook => rev_arrays!([
            [0,    0,    0,    0,    0,    0,    0,    0],
            [5,    10,   10,   10,   10,   10,   10,   5],
            [-5,   0,    0,    0,    0,    0,    0,    -5],
            [-5,   0,    0,    0,    0,    0,    0,    -5],
            [-5,   0,    0,    0,    0,    0,    0,    -5],
            [-5,   0,    0,    0,    0,    0,    0,    -5],
            [-5,   0,    0,    0,    0,    0,    0,    -5],
            [0,    0,    0,    5,    5,    0,    0,    0],
        ]),
        PieceNames::Queen => rev_arrays!([
            [-20,  -10,  -10,  5,    -5,   -10,  -10,  -20],
            [-10,  0,    0,    0,    0,    0,    0,    -10],
            [-10,  0,    0,    0,    0,    0,    0,    -10],
            [-5,   0,    0,    0,    0,    0,    0,    -5],
            [0,    0,    0,    0,    0,    0,    0,    -5],
            [-10,  0,    0,    0,    0,    5,    0,    -10],
            [-10,  0,    5,    0,    0,    0,    0,    -10],
            [-20,  -10,  -10,  5,    -5,   -10,  -10,  -20],
        ]),
    };

    /// (Middle game (`black`, `white`), End game (`black`, `white`))
    static ref KING_TABLE: ((Table, Table), (Table, Table)) = (
        rev_arrays!([
            [-30,  -40,  -40,  -50,  -50,  -40,  -40,  -30],
            [-30,  -40,  -40,  -50,  -50,  -40,  -40,  -30],
            [-30,  -40,  -40,  -50,  -50,  -40,  -40,  -30],
            [-30,  -40,  -40,  -50,  -50,  -40,  -40,  -30],
            [-20,  -30,  -30,  -40,  -40,  -30,  -30,  -20],
            [-10,  -20,  -20,  -20,  -20,  -20,  -20,  -10],
            [20,   20,   -30,  -30,  -30,  -30,  20,   20],
            [20,   30,   10,   -30,  0,    10,   30,   20],
        ]),
        rev_arrays!([
            [-50,  -40,  -30,  -20,  -20,  -30,  -40,  -50],
            [-30,  -20,  -10,  0,    0,    -10,  -20,  -30],
            [-30,  -10,  20,   30,   30,   20,   -10,  -30],
            [-30,  -10,  30,   40,   40,   30,   -10,  -30],
            [-30,  -10,  30,   40,   40,   30,   -10,  -30],
            [-30,  -10,  20,   30,   30,   20,   -10,  -30],
            [-30,  -30,  0,    0,    0,    0,    -30,  -30],
            [-50,  -30,  -30,  -30,  -30,  -30,  -30,  -50],
        ])
    );
}

fn piece_table(piece: &PieceNames, color: &ChessColor, endgame: bool) -> Table {
    let table = if piece == &PieceNames::King {
        ternary!(endgame, &KING_TABLE.1, &KING_TABLE.0)
    } else {
        &PIECE_TABLES[piece]
    };

    color_ternary!(*color, table.0, table.1)
}

fn table_value(piece: &Piece, endgame: bool) -> i32 {
    let table = piece_table(&piece.name, &piece.color, endgame);
    table[piece.pos.1][piece.pos.0]
}

pub(crate) fn piece_value(piece: &PieceNames) -> i32 {
    match piece {
        PieceNames::Pawn => 100,
        PieceNames::Knight => 320,
        PieceNames::Bishop => 330,
        PieceNames::Rook => 500,
        PieceNames::Queen => 900,
        PieceNames::King => 20000,
    }
}

fn full_piece_value(piece: &Piece, endgame: bool) -> i32 {
    piece.value() + table_value(piece, endgame)
}

const CHECK_VALUE: i32 = 50;
pub(crate) const CHECKMATE_VALUE: i32 = 20000;
/// A draw is worth the same to both sides, so it has to be neutral. A nonzero value here makes
/// whichever side is minimizing hunt for stalemates and repetitions
const STALEMATE_VALUE: i32 = 0;

impl Board {
    /// Order moves by `move_value`, best first
    ///
    /// - Scores each move once up front. Doing it inside the comparator re-evaluates every move
    ///   O(log n) times, in the hottest loop of the search
    fn sort_by_value(&self, moves: Vec<(Loc, Loc)>) -> Vec<(Loc, Loc)> {
        let mut scored = moves
            .into_iter()
            .map(|(from, to)| (self.move_value(&from, &to), (from, to)))
            .collect::<Vec<_>>();

        scored.sort_unstable_by_key(|(score, _)| Reverse(*score));

        scored.into_iter().map(|(_, m)| m).collect()
    }

    pub(crate) fn sorted_moves(&self, color: ChessColor) -> Vec<(Loc, Loc)> {
        self.sort_by_value(self.moves(color))
    }

    /// Legal captures and promotions only, ordered like [Board::sorted_moves]
    ///
    /// - This is what quiescence searches. Promotions are in here because they swing material as
    ///   hard as a capture does
    /// - Generates every legal move and filters, rather than having a second generator that only
    ///   produces captures. Move legality is where this engine's bugs lived and the perft suite
    ///   covers this path, so it is not worth duplicating for the speed
    pub(crate) fn sorted_captures(&self, color: ChessColor) -> Vec<(Loc, Loc)> {
        let captures = self
            .moves(color)
            .into_iter()
            .filter(|(from, to)| self.is_capture(from, to).is_some() || self.is_promotion(from, to))
            .collect();

        self.sort_by_value(captures)
    }

    /// Calculates the score of the board, for the white
    pub(crate) fn score(&self) -> i32 {
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

        // Add value based on attacks
        score += self.attacks_white.count() as i32;
        score -= self.attacks_black.count() as i32;

        score
    }

    fn move_value(&self, from: &Loc, to: &Loc) -> i32 {
        let piece = match self.get(from) {
            Some(piece) => piece,
            None => {
                warn!("Tried to get value of move from empty square");
                return -100;
            }
        };

        let mut score = 0;

        match piece.name {
            // Promoting pawn
            PieceNames::Pawn if to.1 == 7 || to.1 == 0 => {
                return i32::MAX;
            }
            // Moving king with no castle during the non-endgame
            PieceNames::King if !self.endgame && from.0.abs_diff(to.0) != 2 => {
                score -= 20;
            }
            // Moving king without developed minors
            PieceNames::Queen if self.agent_developments != ((true, true), (true, true)) => {
                score -= 100;
            }
            _ => {}
        }

        // Position change
        let table = piece_table(&piece.name, &piece.color, self.endgame);
        score += table[to.1][to.0] - table[from.1][from.0];

        // Add value based on capture (MVV-LVA style)
        if let Some(capture_pos) = self.is_capture(from, to) {
            let capture = self.get(&capture_pos).unwrap();
            score += capture.value() * 10 - piece.value();
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::util::Loc;

    #[test]
    fn a_draw_is_worth_nothing_to_either_side() {
        // Black to move and stalemated. Scoring a draw at -100 made whichever side was minimizing
        // (the agent, which plays black) steer into stalemates and repetitions
        let board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 30");

        assert_eq!(board.state, BoardState::Stalemate);
        assert_eq!(board.score, 0);
    }

    #[test]
    fn promotion_move_value_uses_destination_square() {
        let board = Board::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1");

        assert_eq!(
            board.move_value(&Loc::from_notation("e7"), &Loc::from_notation("e8")),
            i32::MAX
        );
    }

    #[test]
    fn move_ordering_prefers_big_captures_with_small_attackers() {
        let board = Board::from_fen("7k/8/8/3p1q2/4P3/8/8/3Q3K w - - 0 1");
        let pawn_capture = board.move_value(&Loc::from_notation("e4"), &Loc::from_notation("f5"));
        let queen_capture = board.move_value(&Loc::from_notation("d1"), &Loc::from_notation("d5"));

        assert!(pawn_capture > queen_capture);
        assert_eq!(
            board.sorted_moves(ChessColor::White).first(),
            Some(&(Loc::from_notation("e4"), Loc::from_notation("f5")))
        );
    }
}
