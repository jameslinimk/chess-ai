use super::piece::Piece;
use crate::board::{Board, ChessColor};
use crate::util::Loc;

pub fn valid_pos(location: &Loc) -> bool {
    !(location.x >= 8 || location.y >= 8)
}

/// Adds to moves if the move is valid
pub fn add(board: &Board, color: &ChessColor, location: Loc, moves: &mut Vec<Loc>) {
    if valid_pos(&location) {
        if let Some(piece) = board.get(&location) {
            if &piece.color == color {
                return;
            }
        }
        moves.push(location);
    }
}

pub fn static_move(piece: &Piece, board: &Board, directions: &[(i32, i32)]) -> Vec<Loc> {
    let mut moves = vec![];
    for (x, y) in directions.iter() {
        let loc = piece.pos.copy_move_i32(*x, *y);
        add(board, &piece.color, loc, &mut moves);
    }
    moves
}

pub fn directional_move(piece: &Piece, board: &Board, directions: &[(i32, i32)]) -> Vec<Loc> {
    let mut moves = vec![];
    for (x, y) in directions.iter() {
        let mut loc = piece.pos.copy_move_i32(*x, *y);
        while valid_pos(&loc) {
            if let Some(capture) = board.get(&loc) {
                if capture.color != piece.color {
                    moves.push(loc);
                }
                break;
            }
            moves.push(loc);
            let end = loc.move_i32(*x, *y);
            if !end {
                break;
            }
        }
    }
    moves
}
