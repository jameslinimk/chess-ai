use super::piece::Piece;
use super::util::{add, add_ff, valid_pos};
use crate::board::{Board, ChessColor};
use crate::util::Loc;

/// Adds to moves if the move is on the board and is empty
/// - Returns true if added, false else
pub fn add_if_empty(board: &Board, location: Loc, moves: &mut Vec<Loc>) -> bool {
    if valid_pos(&location) && board.get(&location).is_none() {
        moves.push(location);
        return true;
    }
    false
}

/// Adds to moves if the move is a capture
pub fn add_if_capture(board: &Board, color: &ChessColor, location: Loc, moves: &mut Vec<Loc>) {
    if valid_pos(&location) {
        if let Some(capture) = board.get(&location) {
            if &capture.color != color {
                moves.push(location);
            }
        }
    }
}

pub fn pawn_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let mut moves = vec![];
    let direction = if piece.color == ChessColor::White {
        -1
    } else {
        1
    };

    // Forward movement
    let blocked = add_if_empty(board, piece.pos.copy_move_i32(0, direction), &mut moves);
    if blocked && piece.pos.y == 1 || piece.pos.y == 6 {
        add_if_empty(board, piece.pos.copy_move_i32(0, direction * 2), &mut moves);
    }

    // Diagonal captures
    add_if_capture(
        board,
        &piece.color,
        piece.pos.copy_move_i32(1, direction),
        &mut moves,
    );
    add_if_capture(
        board,
        &piece.color,
        piece.pos.copy_move_i32(-1, direction),
        &mut moves,
    );

    // En passent
    if let Some(en_passent) = board.en_passent {
        if en_passent.1 != piece.color && en_passent.0.y == piece.pos.y {
            if en_passent.0.x == piece.pos.x + 1 {
                add(
                    board,
                    &piece.color,
                    piece.pos.copy_move_i32(1, direction),
                    &mut moves,
                );
            } else if en_passent.0.x == piece.pos.x - 1 {
                add(
                    board,
                    &piece.color,
                    piece.pos.copy_move_i32(-1, direction),
                    &mut moves,
                );
            }
        }
    }

    moves
}

pub fn pawn_attacks(piece: &Piece) -> Vec<Loc> {
    let mut moves = vec![];
    let direction = if piece.color == ChessColor::White {
        -1
    } else {
        1
    };

    for pos in [
        piece.pos.copy_move_i32(1, direction),
        piece.pos.copy_move_i32(-1, direction),
    ] {
        add_ff(pos, &mut moves)
    }

    moves
}
