use std::collections::HashMap;

use lazy_static::lazy_static;
use macroquad::rand::ChooseRandom;
use maplit::hashmap;

use crate::board::Board;
use crate::util::Loc;

pub fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.get_moves(board.color_agent);
    moves.choose().copied()
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
