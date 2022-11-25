use super::piece::Piece;
use super::util::static_move;
use crate::board::{Board, ChessColor};
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

    let mut moves = static_move(piece, board, &directions);

    // Castling
    let can_castle = if piece.color == ChessColor::White {
        board.castle_white
    } else {
        board.castle_black
    };
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
                let pos = piece.pos.copy_move_i32(i * dir, 0);
                if board.get(&pos).is_some() {
                    break 'main;
                }
            }

            let loc = piece.pos.copy_move_i32(2 * dir, 0);
            moves.push(loc);
        };
    }

    moves
}
