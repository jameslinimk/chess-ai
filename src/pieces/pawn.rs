use super::piece::Piece;
use super::util::{add, add_if_capture};
use crate::board::{Board, Color};
use crate::util::Loc;

pub fn pawn_moves(board: Board, loc: Loc) -> Option<Vec<Loc>> {
    if let Some(piece) = board.raw[loc.y][loc.x] {
        let direction = if piece.1 == Color::White { -1 } else { 1 };

        let mut moves = vec![];
        add(loc.copy_move_i32(0, direction), &mut moves);
        add(loc.copy_move_i32(0, 2 * direction), &mut moves);
        add_if_capture(&board, loc.copy_move_i32(1, direction), &mut moves);
        add_if_capture(&board, loc.copy_move_i32(-1, direction), &mut moves);

        // En passent
        if let Some(last_move) = board.move_history.last() {
            if let Some(piece) = board.raw[last_move.0.y][last_move.0.x] {
                if piece.0 == Piece::Pawn && last_move.0.y == loc.y {
                    if last_move.1.x == loc.x + 1 {
                        add(loc.copy_move_i32(1, direction), &mut moves);
                    } else if last_move.1.x == loc.x - 1 {
                        add(loc.copy_move_i32(-1, direction), &mut moves);
                    }
                }
            }
        }

        Option::from(moves)
    } else {
        None
    }
}
