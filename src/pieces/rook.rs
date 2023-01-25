use super::piece::Piece;
use super::util::{directional_attacks, directional_moves};
use crate::board::Board;
use crate::util::Loc;

pub(crate) fn rook_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [(0, -1), (0, 1), (1, 0), (-1, 0)];

    directional_moves(piece, board, &directions)
}

pub(crate) fn rook_attacks(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [(0, -1), (0, 1), (1, 0), (-1, 0)];

    directional_attacks(piece, board, &directions)
}
