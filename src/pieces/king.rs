use super::piece::Piece;
use super::util::{static_attacks, static_moves};
use crate::board::{Board, ChessColor};
use crate::color_ternary;
use crate::util::Loc;

pub fn king_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = vec![
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
    ];

    let mut moves = static_moves(piece, board, &directions);

    // Castling
    let can_castle = color_ternary!(piece.color, board.castle_white, board.castle_black);
    let mut directions = Vec::with_capacity(2);
    if can_castle.0 {
        directions.push(1);
    }
    if can_castle.1 {
        directions.push(-1);
    }
    for dir in directions.iter() {
        'main: {
            let l = if *dir == 1 { 2 } else { 3 };
            for i in 1..=l {
                let pos = piece.pos.copy_move_i32(i * dir, 0).0;
                if board.get(&pos).is_some() {
                    break 'main;
                }
            }

            let loc = piece.pos.copy_move_i32(2 * dir, 0).0;
            moves.push(loc);
        };
    }

    moves
}

pub fn king_attacks(piece: &Piece) -> Vec<Loc> {
    let directions = vec![
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
    ];

    static_attacks(piece, &directions)
}
