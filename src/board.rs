use std::collections::HashSet;

use derive_new::new;
use maplit::hashset;

use crate::conf::{CASTLE_VALUE, CHECKMATE_VALUE, CHECK_VALUE, STALEMATE_VALUE};
use crate::loc;
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::Loc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ChessColor {
    Black,
    White,
}
impl ChessColor {
    /// Returns the opposite color
    pub fn other(&self) -> ChessColor {
        match self {
            ChessColor::Black => ChessColor::White,
            ChessColor::White => ChessColor::Black,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoardState {
    Normal,
    /// Attached color is who is in check
    Check(ChessColor),
    /// Attached color is who is in checkmate
    Checkmate(ChessColor),
    Stalemate,
}

/// Represents a chess board and metadata
#[derive(Debug, Clone, new)]
pub struct Board {
    /// Array with the raw 8x8 board data
    #[new(value = "[[None; 8]; 8]")]
    pub raw: [[Option<Piece>; 8]; 8],

    /// Turn of the board
    #[new(value = "ChessColor::White")]
    pub turn: ChessColor,

    /// State of the board
    #[new(value = "BoardState::Normal")]
    pub state: BoardState,

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
    pub score: i32,

    /// Which squares are under attack by white pieces
    #[new(value = "hashset! {}")]
    pub attack_white: HashSet<Loc>,

    /// Which squares are under attack by black pieces
    #[new(value = "hashset! {}")]
    pub attack_black: HashSet<Loc>,

    /// Wether the white king is in check
    #[new(value = "false")]
    pub check_white: bool,

    /// Wether the black king is in check
    #[new(value = "false")]
    pub check_black: bool,

    /// Pieces that block attackers
    #[new(value = "hashset! {}")]
    pub blockers: HashSet<Loc>,

    /// Available moves for white
    #[new(value = "vec![]")]
    pub moves_white: Vec<(Loc, Loc)>,

    /// Available moves for black
    #[new(value = "vec![]")]
    pub moves_black: Vec<(Loc, Loc)>,
}
impl Board {
    /// Moves the piece in `from` to `to`
    pub fn move_piece(&mut self, from: &Loc, to: &Loc, check_stale: bool) {
        // Moving piece (relies on nothing)
        self.move_actions(from, to);
        self.move_raw(from, to);

        // Update turn (relies on nothing)
        self.turn = match self.turn {
            ChessColor::Black => ChessColor::White,
            ChessColor::White => ChessColor::Black,
        };

        // Update other metadata
        self.update_things(check_stale);
    }

    pub fn update_things(&mut self, check_stale: bool) {
        // Update attacks (relies on nothing)
        self.attack_white = self.get_attacks(ChessColor::White);
        self.attack_black = self.get_attacks(ChessColor::Black);

        // Update check (relies on attacks)
        let (white_king, black_king) = self.get_kings();
        self.check_white = self.attack_black.contains(&white_king);
        self.check_black = self.attack_white.contains(&black_king);

        // Update blockers (relies on attacks)
        self.update_blockers();

        // Update moves (relies on attacks and blockers)
        if check_stale {
            self.moves_white = self.get_moves(ChessColor::White);
            self.moves_black = self.get_moves(ChessColor::Black);
        }

        // Detect state (relies on check and moves)
        self.detect_state(check_stale);

        // Set score (relies on state)
        self.score = self.get_score();
    }

    /// Detect wether the players are in check, checkmate or stalemate
    fn detect_state(&mut self, check_stale: bool) {
        self.state = match (self.check_white, self.check_black) {
            (true, true) => {
                panic!("Both kings are in check!");
            }
            (true, false) => {
                if self.moves_white.is_empty() {
                    BoardState::Checkmate(ChessColor::White)
                } else {
                    BoardState::Check(ChessColor::White)
                }
            }
            (false, true) => {
                if self.moves_black.is_empty() {
                    BoardState::Checkmate(ChessColor::Black)
                } else {
                    BoardState::Check(ChessColor::Black)
                }
            }
            (false, false) => {
                if !check_stale {
                    return;
                }

                let moves = if self.turn == ChessColor::White {
                    &self.moves_white
                } else {
                    &self.moves_black
                };

                if moves.is_empty() {
                    BoardState::Stalemate
                } else {
                    BoardState::Normal
                }
            }
        };
    }

    /// Returns a tuple of the locations of the kings (white, black)
    fn get_kings(&self) -> (Loc, Loc) {
        let mut white_king = None;
        let mut black_king = None;
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.name == PieceNames::King {
                    if piece.color == ChessColor::White {
                        white_king = Some(piece.pos);
                    } else {
                        black_king = Some(piece.pos);
                    }
                }
            }
        }
        (white_king.unwrap(), black_king.unwrap())
    }

    fn update_blockers(&mut self) {
        self.blockers = hashset! {};
        for loc in self.attack_white.iter() {
            if let Some(piece) = self.get(loc) {
                if piece.color == ChessColor::Black {
                    self.blockers.insert(*loc);
                }
            }
        }
        for loc in self.attack_black.iter() {
            if let Some(piece) = self.get(loc) {
                if piece.color == ChessColor::White {
                    self.blockers.insert(*loc);
                }
            }
        }
    }

    pub fn get_attacks(&mut self, color: ChessColor) -> HashSet<Loc> {
        let mut attacks = hashset! {};
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.color == color {
                    attacks.extend(piece.get_attacks(self));
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
                                self.set(&loc, None);
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
    fn get_score(&self) -> i32 {
        match self.state {
            BoardState::Checkmate(check_color) => {
                if check_color == ChessColor::White {
                    return -CHECKMATE_VALUE;
                } else {
                    return CHECKMATE_VALUE;
                }
            }
            BoardState::Check(check_color) => {
                if check_color == ChessColor::White {
                    return -CHECK_VALUE;
                } else {
                    return CHECK_VALUE;
                }
            }
            BoardState::Stalemate => return STALEMATE_VALUE,
            _ => {}
        }

        let mut score = 0;

        // Add value based on pieces
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.color == ChessColor::White {
                    score += piece.get_value();
                } else {
                    score -= piece.get_value();
                }
            }
        }

        for (castle, attacks, color) in &[
            (self.castle_white, &self.attack_white, ChessColor::White),
            (self.castle_black, &self.attack_black, ChessColor::Black),
        ] {
            let subtract = if color == &ChessColor::White { 1 } else { -1 };

            // Add value based on castling
            if castle.0 {
                score += CASTLE_VALUE * subtract;
            }
            if castle.1 {
                score += CASTLE_VALUE * subtract;
            }

            // Add value based on attacks
            score += attacks.len() as i32 * subtract;
        }

        score
    }
}
