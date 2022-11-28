use std::cmp::{max, min};
use std::collections::HashMap;

use lazy_static::lazy_static;
use macroquad::rand::ChooseRandom;
use maplit::hashmap;

use crate::board::{Board, ChessColor};
use crate::util::Loc;

pub fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.get_moves(board.color_agent);
    moves.choose().copied()
}

fn minimax(depth: u8, board: &Board, maximizing: bool) -> i32 {
    if depth == 0 {
        return board.score;
    }

    let mut best_score = if maximizing { i32::MIN } else { i32::MAX };
    for (from, to) in board.get_moves(if maximizing {
        board.color_agent
    } else {
        board.color_player
    }) {
        let mut board = board.clone();
        board.move_piece(&from, &to, true);
        let score = minimax(depth - 1, &board, !maximizing);
        if maximizing {
            best_score = max(best_score, score);
        } else {
            best_score = min(best_score, score);
        }
    }

    return best_score;
}

pub fn minimax_agent(board: &Board) -> Option<(Loc, Loc)> {
    let depth = 3;
    let maximizing = if board.color_agent == ChessColor::White {
        true
    } else {
        false
    };

    for (from, to) in board.get_moves(board.color_agent) {
        let mut board = board.clone();
        board.move_piece(&from, &to, true);
        minimax(3, &board, !maximizing);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Agent {
    Random,
    Minimax,
}
impl Agent {
    pub fn get_move(&self, board: &Board) -> Option<(Loc, Loc)> {
        match self {
            Agent::Random => random_agent(board),
            Agent::Minimax => todo!(),
        }
    }
}

lazy_static! {
    pub static ref AGENTS: HashMap<&'static str, Agent> = hashmap! {
        "Random" => Agent::Random,
        "Minimax" => Agent::Minimax,
    };
}
