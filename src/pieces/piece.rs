use derive_new::new;
use macroquad::texture::Texture2D;

use super::bishop::{bishop_attacks, bishop_moves};
use super::king::{king_attacks, king_moves};
use super::knight::{knight_attacks, knight_moves};
use super::pawn::{pawn_attacks, pawn_moves};
use super::queen::{queen_attacks, queen_moves};
use super::rook::{rook_attacks, rook_moves};
use crate::assets::get_image;
use crate::board::{Board, ChessColor};
use crate::board_eval::piece_value;
use crate::color_ternary;
use crate::util::{BitBoard, Loc};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum PieceNames {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, new)]
pub(crate) struct Piece {
    /// Type of piece
    pub(crate) name: PieceNames,
    /// The color of the piece
    pub(crate) color: ChessColor,
    /// Piece's current position on the board
    /// - Make sure to update this when moving the piece
    pub(crate) pos: Loc,
}

impl Piece {
    /// Get valid moves for this piece
    pub(crate) fn moves(&self, board: &Board) -> Vec<Loc> {
        let mut temp_moves = match self.name {
            PieceNames::Pawn => pawn_moves(self, board),
            PieceNames::Knight => knight_moves(self, board),
            PieceNames::King => {
                let mut moves = king_moves(self, board);
                moves.retain(|&to| {
                    let attacks =
                        color_ternary!(self.color, &board.attacks_black, &board.attacks_white);
                    !attacks.contains(&to)
                });
                moves
            }
            PieceNames::Rook => rook_moves(self, board),
            PieceNames::Bishop => bishop_moves(self, board),
            PieceNames::Queen => queen_moves(self, board),
        };

        if board.blockers.contains(&self.pos)
            || color_ternary!(self.color, board.check_white, board.check_black)
            || (self.name == PieceNames::Pawn && board.en_passent.is_some())
        {
            temp_moves.retain(|&to| !board.leaves_king_attacked(&self.pos, &to, self.color));
        }

        temp_moves
    }

    /// Add the squares attacked by this piece to `attacks`
    ///
    /// - Writes straight into the set rather than returning a `Vec`, which was a heap allocation
    ///   per piece, twice per position, for every position the search looked at
    pub(crate) fn attacks(&self, board: &Board, attacks: &mut BitBoard) {
        match self.name {
            PieceNames::Pawn => pawn_attacks(self, attacks),
            PieceNames::Knight => knight_attacks(self, attacks),
            PieceNames::King => king_attacks(self, attacks),
            PieceNames::Rook => rook_attacks(self, board, attacks),
            PieceNames::Bishop => bishop_attacks(self, board, attacks),
            PieceNames::Queen => queen_attacks(self, board, attacks),
        }
    }

    /// Get image texture for this piece
    pub(crate) fn image(&self) -> Texture2D {
        let path = match self.color {
            ChessColor::White => match self.name {
                PieceNames::Pawn => "assets/pieces/white_pawn.png",
                PieceNames::Rook => "assets/pieces/white_rook.png",
                PieceNames::Knight => "assets/pieces/white_knight.png",
                PieceNames::Bishop => "assets/pieces/white_bishop.png",
                PieceNames::Queen => "assets/pieces/white_queen.png",
                PieceNames::King => "assets/pieces/white_king.png",
            },
            ChessColor::Black => match self.name {
                PieceNames::Pawn => "assets/pieces/black_pawn.png",
                PieceNames::Rook => "assets/pieces/black_rook.png",
                PieceNames::Knight => "assets/pieces/black_knight.png",
                PieceNames::Bishop => "assets/pieces/black_bishop.png",
                PieceNames::Queen => "assets/pieces/black_queen.png",
                PieceNames::King => "assets/pieces/black_king.png",
            },
        };
        get_image(path)
    }

    /// Get the piece value
    pub(crate) fn value(&self) -> i32 {
        piece_value(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, ChessColor};
    use crate::util::Loc;

    #[test]
    fn en_passent_cannot_expose_the_king_along_the_rank() {
        // Ka5, Pb5, pc5 (which just played c7-c5) and rh5 all share rank 5. Capturing bxc6 e.p.
        // takes *two* pawns off that rank at once, so the rook would be giving check. The pawn is
        // not itself attacked, so it never landed in `blockers` and the legality check was skipped
        let board = Board::from_fen("7k/8/8/KPp4r/8/8/8/8 w - c6 0 30");
        let ep = (Loc::from_notation("b5"), Loc::from_notation("c6"));

        assert!(!board.check_white, "white should not be in check yet");
        assert!(!board.moves(ChessColor::White).contains(&ep));
    }

    #[test]
    fn en_passent_is_still_legal_when_it_exposes_nothing() {
        let board = Board::from_fen("7k/8/8/1Pp5/8/8/8/K7 w - c6 0 30");
        let ep = (Loc::from_notation("b5"), Loc::from_notation("c6"));

        assert!(board.moves(ChessColor::White).contains(&ep));
    }
}
