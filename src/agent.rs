use macroquad::rand::ChooseRandom;

use crate::board::Board;
use crate::util::Loc;

pub fn random_agent(board: &Board) -> Option<(Loc, Loc)> {
    let moves = board.get_moves(board.color_agent);
    moves.choose().copied()
}
