use super::piece::Piece;
use super::util::{static_attacks, static_moves};
use crate::board::Board;
use crate::util::{BitBoard, Loc};

pub(crate) fn knight_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    static_moves(piece, board, &directions)
}

pub(crate) fn knight_attacks(piece: &Piece, attacks: &mut BitBoard) {
    let directions = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    static_attacks(piece, &directions, attacks)
}
