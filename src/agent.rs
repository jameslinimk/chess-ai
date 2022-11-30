use std::collections::HashMap;

use lazy_static::lazy_static;
use macroquad::rand::ChooseRandom;
use maplit::hashmap;

use crate::board::{Board, ChessColor};
use crate::util::{choose_array, Loc};
use crate::{color_ternary, loc};

pub fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.get_moves(board.color_agent);
    moves.choose().copied()
}

const OPENINGS_BLACK: [(Loc, Loc); 4] = [
    (loc!(2, 1), loc!(2, 3)),
    (loc!(3, 1), loc!(3, 3)),
    (loc!(5, 1), loc!(5, 3)),
    (loc!(6, 1), loc!(6, 3)),
];

const OPENINGS_WHITE: [(Loc, Loc); 4] = [
    (loc!(2, 6), loc!(2, 4)),
    (loc!(3, 6), loc!(3, 4)),
    (loc!(4, 6), loc!(4, 4)),
    (loc!(5, 6), loc!(5, 4)),
];

fn minimax(
    board: &Board,
    player: bool,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
) -> (i32, Option<(Loc, Loc)>) {
    // Base case
    if depth == 0 || board.is_over() {
        return (board.score, None);
    }

    // First move for white
    if board.move_count() == 0 {
        let openings = color_ternary!(board.color_agent, &OPENINGS_WHITE, &OPENINGS_BLACK);
        return (0, Some(*choose_array(openings)));
    }

    let moves = color_ternary!(
        board.turn,
        board.get_moves(ChessColor::White),
        board.get_moves(ChessColor::Black)
    );

    if color_ternary!(board.turn, true, false) == player {
        let mut max_score = i32::MIN;
        let mut best_move = None;

        for (from, to) in moves.iter() {
            let mut test_board = board.clone();
            test_board.move_piece(from, to, true);

            let (score, _) = minimax(&test_board, !player, depth - 1, alpha, beta);

            alpha = alpha.max(score);
            if beta <= alpha {
                break;
            }

            if score >= max_score {
                max_score = score;
                best_move = Some((*from, *to));
            }
        }

        (max_score, Some(best_move.unwrap()))
    } else {
        let mut min_score = i32::MAX;
        let mut best_move = None;

        for (from, to) in moves.iter() {
            let mut test_board = board.clone();
            test_board.move_piece(from, to, false);

            let (score, _) = minimax(&test_board, player, depth - 1, alpha, beta);

            beta = beta.min(score);
            if beta <= alpha {
                break;
            }

            if score <= min_score {
                min_score = score;
                best_move = Some((*from, *to));
            }
        }

        (min_score, Some(best_move.unwrap()))
    }
}

pub fn minimax_agent(board: &Board) -> Option<(Loc, Loc)> {
    let (_, best_move) = minimax(board, false, 3, i32::MIN, i32::MAX);
    best_move
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
            Agent::Minimax => minimax_agent(board),
        }
    }
}

lazy_static! {
    pub static ref AGENTS: HashMap<&'static str, Agent> = hashmap! {
        "Random" => Agent::Random,
        "Minimax" => Agent::Minimax,
    };
}
