//! Agents for [Board]. Has a minimax agent and a random agent. Change between agents in the GUI or editing `Board.agent`
//!
//! # Minimax
//!
//! - Alpha-beta pruning
//! - Sorted move ordering
//! - Stored openings
//!
//! # Random
//!
//! - Just picks a valid move by random

use std::str::from_utf8;

use lazy_static::lazy_static;
use macroquad::rand::ChooseRandom;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::board::{Board, ChessColor};
use crate::board_extras::char_to_piece;
use crate::conf::TEST_FEN;
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::{choose_array, Loc};
use crate::{color_ternary, hashmap, loc, ternary};

#[derive(Serialize, Deserialize, Debug)]
pub struct Opening {
    name: String,
    code: String,
    moves: Vec<String>,
}

lazy_static! {
    static ref OPENINGS: Vec<Opening> =
        from_str(from_utf8(include_bytes!("../assets/openings.json")).unwrap()).unwrap();
}

// Only things needed:
// - Pawn moves
// - Normal piece moves
// - Castling
// - Pawn captures
// - Normal piece captures

#[test]
fn test() {
    for opening in OPENINGS.iter() {
        let mut board = Board::from_fen(TEST_FEN);
        #[allow(clippy::never_loop)]
        for (i, raw_ms) in opening.moves.iter().enumerate() {
            let turn = ternary!(i % 2 == 0, ChessColor::White, ChessColor::Black);

            let legal_moves = color_ternary!(
                board.turn,
                board.get_moves(ChessColor::White),
                board.get_moves(ChessColor::Black)
            );

            let (from, to) = 'main: {
                let move_string = if raw_ms.ends_with('+') || raw_ms.ends_with('#') {
                    &raw_ms[1..raw_ms.len() - 1]
                } else {
                    raw_ms
                };

                // FIXME: maybe this is problem
                if move_string == "O-O" || move_string == "O-O-O" {
                    let y = color_ternary!(turn, 7, 0);
                    let x = ternary!(move_string == "O-O", 6, 2);
                    break 'main (loc!(x, y), loc!(x, y));
                }

                if move_string.len() == 2 {
                    let pos = Loc::from_notation(move_string);
                    let dir = color_ternary!(turn, 1, -1);

                    let mut i = 0;
                    loop {
                        if let Some(piece) = board.get(&pos.copy_move_i32(0, dir * i).0) {
                            if piece.color == turn && piece.name == PieceNames::Pawn {
                                break 'main (piece.pos, pos);
                            }
                        }
                        i += 1;
                    }
                }

                let debug = move_string == "Qe8";

                if move_string.len() == 3 {
                    let mut chars = move_string.chars();
                    let name = char_to_piece(&chars.next().unwrap());
                    let pos = Loc::from_notation(&chars.collect::<String>());

                    for mov in legal_moves.iter() {
                        if let Some(piece) = board.get(&mov.0) {
                            if debug {
                                println!("piece: {:?}", piece);
                            }
                            if piece.name == name && mov.1 == pos {
                                break 'main *mov;
                            }
                        }
                    }

                    if debug {
                        board.print();
                    }
                }

                if move_string.len() == 4 {
                    if move_string.chars().next().unwrap().is_ascii_lowercase() {
                        let mut chars = move_string.chars();
                        let x = chars.next().unwrap() as u32 - 97;
                        let pos = loc!(x as usize, 0);
                        let mut i = 0;
                        let killer = loop {
                            if let Some(piece) = board.get(&pos.copy_move_i32(0, i).0) {
                                if piece.color == turn && piece.name == PieceNames::Pawn {
                                    break piece.pos;
                                }
                            }
                            i += 1;
                        };
                        chars.next();

                        let pos = Loc::from_notation(&chars.collect::<String>());
                        for mov in legal_moves.iter() {
                            if let Some(piece) = board.get(&mov.0) {
                                if piece.name == PieceNames::Pawn
                                    && piece.pos == killer
                                    && mov.1 == pos
                                {
                                    break 'main *mov;
                                }
                            }
                        }
                    } else {
                        let mut chars = move_string.chars();
                        let killer = char_to_piece(&chars.next().unwrap());
                        chars.next();

                        let pos = Loc::from_notation(&chars.collect::<String>());
                        for mov in legal_moves.iter() {
                            if let Some(piece) = board.get(&mov.0) {
                                if piece.name == killer && mov.1 == pos {
                                    break 'main *mov;
                                }
                            }
                        }
                    }
                }

                panic!("Not implemented! {} {} {}", move_string, i, opening.name);
            };

            board.move_piece(&from, &to, true);
        }
    }
}

pub fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.get_moves(board.agent_color);
    moves.choose().copied()
}

const OPENINGS_BLACK: [(Loc, Loc); 4] = [
    (loc!(2, 1), loc!(2, 3)),
    (loc!(3, 1), loc!(3, 3)),
    (loc!(4, 1), loc!(4, 3)),
    (loc!(5, 1), loc!(5, 3)),
];

const OPENINGS_WHITE: [(Loc, Loc); 4] = [
    (loc!(2, 6), loc!(2, 4)),
    (loc!(3, 6), loc!(3, 4)),
    (loc!(4, 6), loc!(4, 4)),
    (loc!(5, 6), loc!(5, 4)),
];

/// Minimax agent with alpha-beta pruning and sorted move ordering
fn minimax(
    board: &Board,
    maximizing: bool,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
) -> (i32, Option<(Loc, Loc)>) {
    // Base case
    if depth == 0 || board.is_over() {
        return (board.score, None);
    }

    // First move for white
    if board.full_moves() == 0 {
        let openings = color_ternary!(board.agent_color, &OPENINGS_WHITE, &OPENINGS_BLACK);
        return (0, Some(*choose_array(openings)));
    }

    let moves = color_ternary!(
        board.turn,
        board.get_sorted_moves(ChessColor::White),
        board.get_sorted_moves(ChessColor::Black)
    );

    if maximizing {
        let mut max_score = i32::MIN;
        let mut best_move = None;

        for (from, to) in moves.iter() {
            let mut test_board = board.clone();
            test_board.move_piece(from, to, false);

            let (score, _) = minimax(&test_board, !maximizing, depth - 1, alpha, beta);

            if score > max_score {
                max_score = score;
                best_move = Some((*from, *to));
            }

            alpha = alpha.max(max_score);
            if beta <= alpha {
                break;
            }
        }

        (max_score, best_move)
    } else {
        let mut min_score = i32::MAX;
        let mut best_move = None;

        for (from, to) in moves.iter() {
            let mut test_board = board.clone();
            test_board.move_piece(from, to, false);

            let (score, _) = minimax(&test_board, !maximizing, depth - 1, alpha, beta);

            if score < min_score {
                min_score = score;
                best_move = Some((*from, *to));
            }

            beta = beta.min(min_score);
            if beta <= alpha {
                break;
            }
        }

        (min_score, best_move)
    }
}

/// Wrapper for minimax
pub fn minimax_agent(board: &Board) -> Option<(Loc, Loc)> {
    let (_, best_move) = minimax(board, false, 4, i32::MIN, i32::MAX);
    best_move
}

#[derive(Clone, Copy, Debug)]
/// List of agents for [Board] to use
pub enum Agent {
    Minimax,
    Random,
}
impl Agent {
    pub fn get_move(&self, board: &Board) -> Option<(Loc, Loc)> {
        match self {
            Agent::Minimax => minimax_agent(board),
            Agent::Random => random_agent(board),
        }
    }
}

lazy_static! {
    pub static ref AGENTS: FxHashMap<&'static str, Agent> = hashmap! {
        "Minimax" => Agent::Minimax,
        "Random" => Agent::Random,
    };
}
