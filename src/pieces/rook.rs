use super::piece::Piece;
use super::util::directional_move;
use crate::board::Board;
use crate::util::Loc;

pub fn rook_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [(0, -1), (0, 1), (1, 0), (-1, 0)];

    directional_move(piece, board, &directions)
}
