//! Agents for [Board]. Has a minimax agent and a random agent. Change between agents in the GUI or editing `Board.agent`
//!
//! # Minimax
//!
//! - Stored openings
//! - Alpha-beta pruning
//! - Sorted move ordering
//! - Transposition table
//!
//! # Random
//!
//! - Just picks a valid move by random
//!
//! # Control
//!
//! - Manually control the agent by clicking on the board

use macroquad::prelude::info;
use macroquad::rand::ChooseRandom;
use rustc_hash::FxHashMap;

use crate::agent_opens::OPENINGS;
use crate::board::{Board, ChessColor};
use crate::pieces::piece::PieceNames;
use crate::util::{choose_array, Loc};
use crate::{color_ternary, hashmap, ternary};

fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.moves(board.agent_color);
    moves.choose().copied()
}

const DEPTH: u8 = 4;
const MAX: i32 = i32::MAX - 1;

/// Minimax agent with alpha-beta pruning and sorted move ordering
#[allow(clippy::type_complexity)]
fn minimax(
    board: &Board,
    maximizing: bool,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    trans_table: &mut FxHashMap<u64, (u8, i32, Option<(Loc, Loc)>)>,
) -> (i32, Option<(Loc, Loc)>) {
    if maximizing {
        assert_eq!(board.turn, ChessColor::White);
    } else {
        assert_eq!(board.turn, ChessColor::Black);
    }

    // Base case
    if depth == 0 || board.is_over() {
        return (board.score, None);
    }

    // Openings
    if depth == DEPTH {
        // Very first move
        if board.full_moves() == 0 && board.agent_color == ChessColor::Black {
            macro_rules! responses {
                ($($key:expr => $value:expr,)+) => { responses!($($key => $value),+) };
                ($($key:expr => $value:expr),*) => {
                    $(
                        if let Some(piece) = board.get(&Loc::from_notation($key.1)) {
                            if piece.name == $key.0 {
                                let m = choose_array(&$value);
                                info!("First move found!");
                                return (MAX, Some((Loc::from_notation(m.0), Loc::from_notation(m.1))));
                            }
                        }
                    )*
                };
            }

            responses! {
                // e4 -> e5, e6, c5
                (PieceNames::Pawn, "e4") => [("e7", "e5"), ("e7", "e6"), ("c7", "c5")],
                // d4 -> d5, c6, Nf6, Nc6
                (PieceNames::Pawn, "d4") => [("d7", "d5"), ("g8", "f6"), ("b8", "c6")],
                // c4 -> e5, Nf6
                (PieceNames::Pawn, "c4") => [("e7", "e5"), ("g8", "f6")],
                // Nf3 -> e5, Nf6
                (PieceNames::Knight, "f3") => [("e7", "e5"), ("g8", "f6")],
            };
        }

        if let Some(moves) = OPENINGS.get(&board.hash) {
            let (opening, name) = choose_array(moves);
            info!("Opening found! {}", name);
            return (MAX, Some(*opening));
        }
    }

    // Check if the current board state is already stored in the transposition table
    let stored_data = trans_table.get(&board.hash);
    let mut greater_depth = false;
    if let Some((stored_depth, stored_score, stored_best)) = stored_data {
        if stored_depth >= &depth {
            return (*stored_score, *stored_best);
        }
        greater_depth = true;
    }

    // Get the sorted legal moves for the current turn
    let moves = color_ternary!(
        board.turn,
        board.sorted_moves(ChessColor::White),
        board.sorted_moves(ChessColor::Black)
    );

    let mut best_score = ternary!(maximizing, i32::MIN, i32::MAX);
    let mut best_move = None;

    // Iterate through the moves and apply minimax
    for (from, to) in moves.iter() {
        let mut test_board = board.clone();
        test_board.move_piece(from, to, false);

        let (score, _) = minimax(
            &test_board,
            !maximizing,
            depth - 1,
            alpha,
            beta,
            trans_table,
        );

        if score == MAX {
            return (score, Some((*from, *to)));
        }

        // Update the best score and best move
        if ternary!(maximizing, score > best_score, score < best_score) {
            best_score = score;
            best_move = Some((*from, *to));
        }

        // Update alpha and beta
        if maximizing {
            alpha = alpha.max(score);
        } else {
            beta = beta.min(score);
        }

        // Prune the search if alpha is greater than or equal to beta
        if alpha >= beta {
            break;
        }
    }

    // Store the data in the transposition table
    if greater_depth {
        trans_table.insert(board.hash, (depth, best_score, best_move));
    }
    (best_score, best_move)
}

/// Wrapper for minimax
fn minimax_agent(board: &Board) -> Option<(Loc, Loc)> {
    if board.is_over() {
        return None;
    }
    let (_, best_move) = minimax(board, false, DEPTH, i32::MIN, i32::MAX, &mut hashmap! {});
    best_move
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// List of agents for [Board] to use
pub(crate) enum Agent {
    Minimax,
    Control,
    Random,
}
impl Agent {
    pub(crate) fn get_move(&self, board: &Board) -> Option<(Loc, Loc)> {
        match self {
            Agent::Minimax => minimax_agent(board),
            Agent::Random => random_agent(board),
            Agent::Control => None,
        }
    }
}

pub(crate) const AGENTS: [(&str, Agent); 3] = [
    ("Random", Agent::Random),
    ("Control", Agent::Control),
    ("Minimax", Agent::Minimax),
];
