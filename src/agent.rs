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

use macroquad::prelude::info;
use macroquad::rand::ChooseRandom;
use rustc_hash::FxHashMap;

use crate::agent_opens::OPENINGS;
use crate::board::{Board, ChessColor};
use crate::util::{choose_array, Loc};
use crate::{color_ternary, hashmap, ternary};

pub fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.get_moves(board.agent_color);
    moves.choose().copied()
}

pub const DEPTH: u8 = 4;

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
    // Base case
    if depth == 0 || board.is_over() {
        return (board.score, None);
    }

    // Openings
    if depth == DEPTH {
        if let Some(moves) = OPENINGS.get(&board.hash) {
            info!("Opening found!");
            return (i32::MAX, Some(*choose_array(moves)));
        }
    }

    if let Some((dep, score, best)) = trans_table.get(&board.hash) {
        if dep >= &depth {
            return (*score, *best);
        }
    }

    let moves = color_ternary!(
        board.turn,
        board.get_sorted_moves(ChessColor::White),
        board.get_sorted_moves(ChessColor::Black)
    );

    let mut best_score = ternary!(maximizing, i32::MIN, i32::MAX);
    let mut best_move = None;

    for (from, to) in moves.iter() {
        let mut test_board = board.clone();
        test_board.move_piece(from, to, false);

        let (score, _) = minimax(&test_board, !maximizing, depth - 1, alpha, beta, trans_table);

        if score == i32::MAX {
            return (score, Some((*from, *to)));
        }

        if ternary!(maximizing, score > best_score, score < best_score) {
            best_score = score;
            best_move = Some((*from, *to));
        }

        if maximizing {
            alpha = alpha.max(best_score);
            if beta <= alpha {
                break;
            }
        } else {
            beta = beta.min(best_score);
            if beta <= alpha {
                break;
            }
        }
    }

    trans_table.insert(board.hash, (depth, best_score, best_move));
    (best_score, best_move)
}

/// Wrapper for minimax
pub fn minimax_agent(board: &Board) -> Option<(Loc, Loc)> {
    let (_, best_move) = minimax(board, false, DEPTH, i32::MIN, i32::MAX, &mut hashmap! {});
    best_move
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// List of agents for [Board] to use
pub enum Agent {
    Minimax,
    Control,
    Random,
}
impl Agent {
    pub fn get_move(&self, board: &Board) -> Option<(Loc, Loc)> {
        match self {
            Agent::Minimax => minimax_agent(board),
            Agent::Random => random_agent(board),
            Agent::Control => None,
        }
    }
}

pub const AGENTS: [(&str, Agent); 3] = [
    ("Random", Agent::Random),
    ("Control", Agent::Control),
    ("Minimax", Agent::Minimax),
];
