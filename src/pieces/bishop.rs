use super::piece::Piece;
use super::util::directional_move;
use crate::board::Board;
use crate::util::Loc;

pub fn bishop_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    directional_move(piece, board, &directions)
}
