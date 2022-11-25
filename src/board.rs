use std::collections::HashSet;

use derive_new::new;
use maplit::hashset;

use crate::loc;
use crate::pieces::piece::{Piece, PieceNames};
use crate::pieces::util::valid_pos;
use crate::util::Loc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ChessColor {
    Black,
    White,
}

/// Represents a chess board and metadata
#[derive(Clone, new)]
pub struct Board {
    /// Array with the raw 8x8 board data
    #[new(value = "[[None; 8]; 8]")]
    pub raw: [[Option<Piece>; 8]; 8],

    /// Turn of the board
    #[new(value = "ChessColor::White")]
    pub turn: ChessColor,

    /// Player color
    #[new(value = "ChessColor::White")]
    pub color_player: ChessColor,

    /// Agent color
    #[new(value = "ChessColor::Black")]
    pub color_agent: ChessColor,

    /// True if black can castle (queen side, king side)
    #[new(value = "(true, true)")]
    pub castle_black: (bool, bool),

    /// True if white can castle (queen side, king side)
    #[new(value = "(true, true)")]
    pub castle_white: (bool, bool),

    /// Last pawn move and color
    #[new(value = "None")]
    pub en_passent: Option<(Loc, ChessColor)>,

    /// Current score of board, for white
    #[new(value = "0")]
    pub score_white: i32,

    /// Current score of board, for black
    #[new(value = "0")]
    pub score_black: i32,

    /// Which squares are under attack by white pieces
    #[new(value = "hashset! {}")]
    pub attack_white: HashSet<Loc>,

    /// Which squares are under attack by black pieces
    #[new(value = "hashset! {}")]
    pub attack_black: HashSet<Loc>,
}
impl Board {
    /// Moves the piece in `from` to `to`
    pub fn move_piece(&mut self, from: &Loc, to: &Loc) {
        // Moving piece
        self.move_actions(from, to);
        self.move_raw(from, to);

        // Update turn
        self.turn = match self.turn {
            ChessColor::Black => ChessColor::White,
            ChessColor::White => ChessColor::Black,
        };

        // Set score
        self.score_white = self.get_score(ChessColor::White);
        self.score_black = self.get_score(ChessColor::Black);

        // Update attacks
        self.attack_white = self.get_attacks(ChessColor::White);
        self.attack_black = self.get_attacks(ChessColor::Black);
    }

    pub fn get_attacks(&self, color: ChessColor) -> HashSet<Loc> {
        let mut attacks = hashset! {};
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.color == color {
                    match piece.name {
                        PieceNames::Pawn => {
                            let direction = if piece.color == ChessColor::White {
                                -1
                            } else {
                                1
                            };

                            for pos in [
                                piece.pos.copy_move_i32(1, direction),
                                piece.pos.copy_move_i32(-1, direction),
                            ] {
                                if valid_pos(&pos) {
                                    attacks.insert(pos);
                                }
                            }
                        }
                        _ => attacks.extend(piece.get_raw_moves(self)),
                    }
                }
            }
        }
        attacks
    }

    // Move the piece in `from` to `to` without updating anything
    fn move_raw(&mut self, from: &Loc, to: &Loc) {
        self.set(to, self.get(from));
        self.set(from, None);
    }

    /// Special actions that happen when moving a piece
    /// - IE: Castling, en passent, pawn promotion, etc...
    fn move_actions(&mut self, from: &Loc, to: &Loc) {
        let mut set_en_passent = false;

        if let Some(piece) = self.raw[from.y][from.x].as_mut() {
            piece.pos = *to;

            match piece.name {
                // Castle check
                PieceNames::King => {
                    match piece.color {
                        ChessColor::Black => self.castle_black = (false, false),
                        ChessColor::White => self.castle_white = (false, false),
                    }

                    if from.x.abs_diff(to.x) == 2 {
                        let (rook_from, rook_to) = match to.x {
                            2 => (loc!(0, to.y), loc!(3, to.y)),
                            6 => (loc!(7, to.y), loc!(5, to.y)),
                            _ => panic!(),
                        };

                        self.move_raw(&rook_from, &rook_to);
                    }
                }
                PieceNames::Rook => match from {
                    loc!(0, 0) => self.castle_black.0 = false,
                    loc!(7, 0) => self.castle_black.1 = false,
                    loc!(0, 7) => self.castle_white.0 = false,
                    loc!(7, 7) => self.castle_white.1 = false,
                    _ => (),
                },

                // En passent check
                PieceNames::Pawn => {
                    // Promotion
                    if to.y == 0 || to.y == 7 {
                        piece.name = PieceNames::Queen;
                    }

                    // Setting en passent
                    if from.y.abs_diff(to.y) == 2 {
                        self.en_passent = Some((*to, piece.color));
                        set_en_passent = true;
                    }

                    // En passent capture
                    if from.x.abs_diff(to.x) == 1 {
                        if let Some((loc, color)) = self.en_passent {
                            if to.x == loc.x && to.y.abs_diff(loc.y) == 1 && piece.color != color {
                                self.raw[loc.y][loc.x] = None;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Reset en passent if it hasn't been set yet
        if !set_en_passent && self.en_passent.is_some() {
            self.en_passent = None;
        }
    }

    /// Calculates the score of the board, for the color specified
    fn get_score(&self, color: ChessColor) -> i32 {
        let mut score = 0;

        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.color == color {
                    score += piece.get_value();
                } else {
                    score -= piece.get_value();
                }
            }
        }

        score
    }
}
