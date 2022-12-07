use super::piece::Piece;
use super::util::{static_attacks, static_moves};
use crate::board::Board;
use crate::util::Loc;
use crate::{color_ternary, loc};

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
    if color_ternary!(piece.color, board.check_white, board.check_black) {
        return moves;
    }

    let (queen_side, king_side) =
        color_ternary!(piece.color, board.castle_white, board.castle_black);
    macro_rules! clear_range {
        ($start: expr, $end: expr) => {
            'main: {
                for i in $start..=$end {
                    if board.get(&loc!(i, piece.pos.y)).is_some() {
                        break 'main true;
                    }
                }
                break 'main false;
            }
        };
    }

    if queen_side && !clear_range!(1, 3) {
        moves.push(loc!(2, piece.pos.y));
    }

    if king_side && !clear_range!(5, 6) {
        moves.push(loc!(6, piece.pos.y));
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
