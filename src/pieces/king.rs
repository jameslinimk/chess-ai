use super::piece::Piece;
use super::util::{static_attacks, static_moves};
use crate::board::Board;
use crate::util::{BitBoard, Loc};
use crate::{color_ternary, loc};

pub(crate) fn king_moves(piece: &Piece, board: &Board) -> Vec<Loc> {
    let directions = [
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

    // Castling, can't castle out of check
    if color_ternary!(piece.color, board.check_white, board.check_black) {
        return moves;
    }

    // Can't castle with a displaced king, which a hand written FEN can produce
    if piece.pos.0 != 4 {
        return moves;
    }

    let (queen_side, king_side) =
        color_ternary!(piece.color, board.castle_white, board.castle_black);
    let enemy_attacks = color_ternary!(piece.color, &board.attacks_black, &board.attacks_white);

    macro_rules! clear_range {
        ($start: expr, $end: expr) => {
            'main: {
                for i in $start..=$end {
                    if board.get(&loc!(i, piece.pos.1)).is_some() {
                        break 'main true;
                    }
                }
                break 'main false;
            }
        };
    }

    // The square the king passes over can't be attacked either, else the king castles *through*
    // check. The destination square is already filtered out by `Piece::moves`
    macro_rules! passes_through_check {
        ($x: expr) => {
            enemy_attacks.contains(&loc!($x, piece.pos.1))
        };
    }

    if queen_side && !clear_range!(1, 3) && !passes_through_check!(3) {
        moves.push(loc!(2, piece.pos.1));
    }

    if king_side && !clear_range!(5, 6) && !passes_through_check!(5) {
        moves.push(loc!(6, piece.pos.1));
    }

    moves
}

pub(crate) fn king_attacks(piece: &Piece, attacks: &mut BitBoard) {
    let directions = [
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
    ];

    static_attacks(piece, &directions, attacks)
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, ChessColor};
    use crate::util::Loc;

    /// `(queen side available, king side available)` for white
    fn white_castles(fen: &str) -> (bool, bool) {
        let board = Board::from_fen(fen);
        let moves = board.moves(ChessColor::White);
        let e1 = Loc::from_notation("e1");

        (
            moves.contains(&(e1, Loc::from_notation("c1"))),
            moves.contains(&(e1, Loc::from_notation("g1"))),
        )
    }

    #[test]
    fn both_castles_are_available_when_nothing_is_in_the_way() {
        assert_eq!(
            white_castles("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 30"),
            (true, true)
        );
    }

    #[test]
    fn cannot_castle_through_an_attacked_square() {
        // The rook on d7 covers d1, which the king passes over when castling queen side. Only the
        // destination square used to be checked, so this castle was allowed
        assert_eq!(
            white_castles("4k3/3r4/8/8/8/8/8/R3K2R w KQ - 0 30"),
            (false, true)
        );
    }

    #[test]
    fn cannot_castle_out_of_check() {
        assert_eq!(
            white_castles("4k3/4r3/8/8/8/8/8/R3K2R w KQ - 0 30"),
            (false, false)
        );
    }
}
