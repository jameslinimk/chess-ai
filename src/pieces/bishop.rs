use super::piece::Piece;
use super::util::{directional_attacks, directional_moves};
use crate::board::Board;
use crate::util::Loc;

pub fn bishop_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    directional_moves(piece, board, &directions)
}

pub fn bishop_attacks(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    directional_attacks(piece, board, &directions)
}
