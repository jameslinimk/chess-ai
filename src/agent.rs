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

use std::hash::{Hash, Hasher};

use lazy_static::lazy_static;
use macroquad::rand::ChooseRandom;
use rustc_hash::{FxHashMap, FxHasher};

use crate::agent_opens::OPENINGS;
use crate::board::{Board, ChessColor};
use crate::util::{choose_array, Loc};
use crate::{color_ternary, hashmap};

pub fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.get_moves(board.agent_color);
    moves.choose().copied()
}

pub const DEPTH: u8 = 4;

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
    if depth == DEPTH {
        let mut hasher = FxHasher::default();
        board.raw.hash(&mut hasher);
        if let Some(moves) = OPENINGS.get(&hasher.finish()) {
            let (name, mov) = choose_array(moves);
            println!("Opening found! {}", name);
            return (i32::MAX, Some(*mov));
        }
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

            if score == i32::MAX {
                return (score, Some((*from, *to)));
            }

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

            if score == i32::MAX {
                return (score, Some((*from, *to)));
            }

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
    let (_, best_move) = minimax(board, false, DEPTH, i32::MIN, i32::MAX);
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
