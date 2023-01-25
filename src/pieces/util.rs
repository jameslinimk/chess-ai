//! Helper functions for creating pieces
//!
//! - Static pieces are pieces that have set moves and can take or go to a place
//! - Directional pieces are pieces that slide on the board and can take or move in rays

use super::piece::Piece;
use crate::board::{Board, ChessColor};
use crate::util::Loc;

pub(crate) fn valid_pos(location: &Loc) -> bool {
    !(location.0 >= 8 || location.1 >= 8)
}

/// Adds to moves if the move is valid and doesn't capture friendly
pub(crate) fn add(board: &Board, color: &ChessColor, location: Loc, moves: &mut Vec<Loc>) {
    if valid_pos(&location) {
        if let Some(piece) = board.get(&location) {
            if &piece.color == color {
                return;
            }
        }
        moves.push(location);
    }
}

/// Adds to moves if the move is valid
pub(crate) fn add_ff(location: Loc, moves: &mut Vec<Loc>) {
    if valid_pos(&location) {
        moves.push(location);
    }
}

/// Get all moves for static pieces
pub(crate) fn static_moves(piece: &Piece, board: &Board, directions: &[(i32, i32)]) -> Vec<Loc> {
    let mut moves = vec![];
    for (x, y) in directions.iter() {
        let (loc, out) = piece.pos.copy_move_i32(*x, *y);
        if out {
            continue;
        }
        add(board, &piece.color, loc, &mut moves);
    }
    moves
}

/// Get all attack squares for static pieces
pub(crate) fn static_attacks(piece: &Piece, directions: &[(i32, i32)]) -> Vec<Loc> {
    let mut moves = vec![];
    for (x, y) in directions.iter() {
        let (loc, out) = piece.pos.copy_move_i32(*x, *y);
        if out {
            continue;
        }
        add_ff(loc, &mut moves);
    }
    moves
}

/// Get all moves for directional pieces
pub(crate) fn directional_moves(
    piece: &Piece,
    board: &Board,
    directions: &[(i32, i32)],
) -> Vec<Loc> {
    let mut moves = vec![];
    for (x, y) in directions.iter() {
        let (mut loc, out) = piece.pos.copy_move_i32(*x, *y);
        if out {
            continue;
        }
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

/// Get all attack squares for directional pieces
pub(crate) fn directional_attacks(
    piece: &Piece,
    board: &Board,
    directions: &[(i32, i32)],
) -> Vec<Loc> {
    let mut moves = vec![];
    for (x, y) in directions.iter() {
        let (mut loc, out) = piece.pos.copy_move_i32(*x, *y);
        if out {
            continue;
        }
        while valid_pos(&loc) {
            if board.get(&loc).is_some() {
                moves.push(loc);
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
