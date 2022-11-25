use super::piece::Piece;
use super::util::static_move;
use crate::board::Board;
use crate::util::Loc;

pub fn knight_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = vec![
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    static_move(piece, board, &directions)
}
