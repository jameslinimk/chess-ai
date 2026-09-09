use super::piece::Piece;
use super::util::{add, add_ff, valid_pos};
use crate::board::{Board, ChessColor};
use crate::color_ternary;
use crate::util::{BitBoard, Loc};

/// Adds to moves if the move is on the board and is empty
/// - Returns true if added, false else
pub(crate) fn add_if_empty(board: &Board, location: Loc, moves: &mut Vec<Loc>) -> bool {
    if valid_pos(&location) && board.get(&location).is_none() {
        moves.push(location);
        return true;
    }
    false
}

/// Adds to moves if the move is a capture
pub(crate) fn add_if_capture(
    board: &Board,
    color: &ChessColor,
    location: Loc,
    moves: &mut Vec<Loc>,
) {
    if valid_pos(&location) {
        if let Some(capture) = board.get(&location) {
            if &capture.color != color {
                moves.push(location);
            }
        }
    }
}

pub(crate) fn pawn_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let mut moves = vec![];
    let direction = color_ternary!(piece.color, -1, 1);

    // Forward movement
    let empty_ahead = add_if_empty(board, piece.pos.copy_move_i32(0, direction).0, &mut moves);
    // The double push is only available from the color's *own* starting rank
    let start_rank = color_ternary!(piece.color, 6, 1);
    if empty_ahead && piece.pos.1 == start_rank {
        add_if_empty(
            board,
            piece.pos.copy_move_i32(0, direction * 2).0,
            &mut moves,
        );
    }

    // Diagonal captures
    let left_side = piece.pos.copy_move_i32(-1, direction);
    if !left_side.1 {
        add_if_capture(board, &piece.color, left_side.0, &mut moves);
    }
    let right_side = piece.pos.copy_move_i32(1, direction);
    if !right_side.1 {
        add_if_capture(board, &piece.color, right_side.0, &mut moves);
    }

    // En passent
    if let Some(en_passent) = board.en_passent {
        if en_passent.1 != piece.color && en_passent.0 .1 == piece.pos.1 {
            if en_passent.0 .0 == piece.pos.0 + 1 {
                add(
                    board,
                    &piece.color,
                    piece.pos.copy_move_i32(1, direction).0,
                    &mut moves,
                );
            } else if piece.pos.0 != 0 && en_passent.0 .0 == piece.pos.0 - 1 {
                add(
                    board,
                    &piece.color,
                    piece.pos.copy_move_i32(-1, direction).0,
                    &mut moves,
                );
            }
        }
    }

    moves
}

pub(crate) fn pawn_attacks(piece: &Piece, attacks: &mut BitBoard) {
    let direction = color_ternary!(piece.color, -1, 1);

    // `copy_move_i32` clamps a negative coordinate to 0 instead of failing, so the out of bounds
    // flag has to be honored. Without it an a-file pawn "attacks" the square straight ahead of it
    for (pos, out) in [
        piece.pos.copy_move_i32(1, direction),
        piece.pos.copy_move_i32(-1, direction),
    ] {
        if out {
            continue;
        }
        add_ff(pos, attacks)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, ChessColor};
    use crate::util::Loc;

    #[test]
    fn a_file_pawn_does_not_attack_the_square_ahead_of_it() {
        // `copy_move_i32` clamps a negative file back onto the board instead of failing, so an a4
        // pawn used to be recorded as attacking a5
        let board = Board::from_fen("4k3/8/8/8/P7/8/8/4K3 w - - 0 30");

        assert!(board.attacks_white.contains(&Loc::from_notation("b5")));
        assert!(!board.attacks_white.contains(&Loc::from_notation("a5")));
    }

    #[test]
    fn a_pawn_does_not_check_the_king_in_front_of_it() {
        let board = Board::from_fen("8/8/8/k7/P7/8/8/4K3 b - - 0 30");

        assert!(!board.check_black);
    }

    #[test]
    fn pawns_only_double_push_from_their_own_start_rank() {
        // A white pawn on rank 7 is one square from promoting and has no double push. The rank used
        // to be checked without regard to color, which duplicated the single push
        let board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 30");
        let a7 = Loc::from_notation("a7");
        let pushes = board
            .moves(ChessColor::White)
            .iter()
            .filter(|(from, _)| *from == a7)
            .count();

        assert_eq!(pushes, 1);
    }
}
