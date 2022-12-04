use derive_new::new;
use rustc_hash::FxHashSet;

use crate::conf::{CASTLE_VALUE, CHECKMATE_VALUE, CHECK_VALUE, STALEMATE_VALUE};
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::Loc;
use crate::{color_ternary, hashset, loc};

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoardState {
    Normal,
    /// Attached color is who is in check
    Check(ChessColor),
    /// Attached color is who is in checkmate
    Checkmate(ChessColor),
    Stalemate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, new)]
pub struct SimpleBoard {
    pub raw: [[Option<Piece>; 8]; 8],
    pub castle_black: (bool, bool),
    pub castle_white: (bool, bool),
    pub en_passent: Option<(Loc, ChessColor)>,
}

/// Represents a chess board and metadata
#[derive(Debug, Clone, PartialEq, Eq, new)]
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
    pub attack_white: FxHashSet<Loc>,

    /// Which squares are under attack by black pieces
    #[new(value = "hashset! {}")]
    pub attack_black: FxHashSet<Loc>,

    /// Wether the white king is in check
    #[new(value = "false")]
    pub check_white: bool,

    /// Wether the black king is in check
    #[new(value = "false")]
    pub check_black: bool,

    /// Pieces that block attackers
    #[new(value = "hashset! {}")]
    pub blockers: FxHashSet<Loc>,

    /// Available moves for white
    #[new(value = "vec![]")]
    pub moves_white: Vec<(Loc, Loc)>,

    /// Available moves for black
    #[new(value = "vec![]")]
    pub moves_black: Vec<(Loc, Loc)>,

    #[new(value = "0")]
    pub half_moves: u32,

    #[new(value = "[None; 12]")]
    pub prev_states: [Option<SimpleBoard>; 12],

    /// Updates on piece capture or pawn move
    #[new(value = "0")]
    pub fifty_rule: u32,
}
impl Board {
    /// Moves the piece in `from` to `to`
    pub fn move_piece(&mut self, from: &Loc, to: &Loc, check_stale: bool) -> bool {
        // Moving piece
        let capture = self.move_actions(from, to);
        self.move_raw(from, to);

        // Update turn
        self.turn = match self.turn {
            ChessColor::Black => ChessColor::White,
            ChessColor::White => ChessColor::Black,
        };
        self.half_moves += 1;

        // 3fold repetition
        self.prev_states.rotate_right(1);
        self.prev_states[0] = Some(self.as_simple());

        // Fifty move rule (pawn move is done in move_actions)
        if capture {
            self.fifty_rule = self.half_moves;
        }

        // Update other metadata
        self.update_things(check_stale);

        capture
    }

    pub fn update_things(&mut self, check_stale: bool) {
        // Update attacks (relies on nothing)
        self.attack_white = self.get_attacks(ChessColor::White);
        self.attack_black = self.get_attacks(ChessColor::Black);

        // Update check (relies on attacks)
        let (white_king, black_king) = self.get_kings();
        if let Some(white_king) = white_king {
            self.check_white = self.attack_black.contains(&white_king);
        } else {
            self.check_white = true;
        }
        if let Some(black_king) = black_king {
            self.check_black = self.attack_white.contains(&black_king);
        } else {
            self.check_black = true;
        }

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
        if self.half_moves - self.fifty_rule >= 50 {
            self.state = BoardState::Stalemate;
            return;
        }

        let mut sum = 0;
        let self_simple = self.as_simple();
        for simple in self.prev_states.iter().flatten() {
            if simple == &self_simple {
                sum += 1;
                if sum >= 3 {
                    self.state = BoardState::Stalemate;
                    return;
                }
            }
        }

        match (self.check_white, self.check_black) {
            (true, false) => {
                if self.moves_white.is_empty() {
                    self.state = BoardState::Checkmate(ChessColor::White)
                } else {
                    self.state = BoardState::Check(ChessColor::White)
                }
            }
            (false, true) => {
                if self.moves_black.is_empty() {
                    self.state = BoardState::Checkmate(ChessColor::Black)
                } else {
                    self.state = BoardState::Check(ChessColor::Black)
                }
            }
            (false, false) => {
                if !check_stale {
                    return;
                }

                let moves = color_ternary!(self.turn, &self.moves_white, &self.moves_black);

                if moves.is_empty() {
                    self.state = BoardState::Stalemate
                } else {
                    self.state = BoardState::Normal
                }
            }
            _ => {}
        };
    }

    /// Returns a tuple of the locations of the kings (white, black)
    fn get_kings(&self) -> (Option<Loc>, Option<Loc>) {
        let mut white_king = None;
        let mut black_king = None;
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.name == PieceNames::King {
                    color_ternary!(
                        piece.color,
                        white_king = Some(piece.pos),
                        black_king = Some(piece.pos)
                    );
                }
            }
        }
        (white_king, black_king)
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

    pub fn get_attacks(&mut self, color: ChessColor) -> FxHashSet<Loc> {
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
    fn move_actions(&mut self, from: &Loc, to: &Loc) -> bool {
        let mut set_en_passent = false;
        let mut capture = self.get(to).is_some();

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
                                capture = true;
                            }
                        }
                    }

                    // Fifty move rule
                    self.fifty_rule = self.half_moves;
                }
                _ => {}
            }
        }

        // Reset en passent if it hasn't been set yet
        if !set_en_passent && self.en_passent.is_some() {
            self.en_passent = None;
        }

        capture
    }

    /// Calculates the score of the board, for the color specified
    fn get_score(&self) -> i32 {
        match self.state {
            BoardState::Checkmate(check_color) => {
                return color_ternary!(check_color, -CHECKMATE_VALUE, CHECKMATE_VALUE);
            }
            BoardState::Stalemate => return STALEMATE_VALUE,
            _ => {}
        }

        let mut score = 0;

        // Add value based on pieces
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                color_ternary!(
                    piece.color,
                    score += piece.get_value(),
                    score -= piece.get_value()
                );
            }
        }

        for (castle, color) in [
            (self.castle_white, ChessColor::White),
            (self.castle_black, ChessColor::Black),
        ]
        .iter()
        {
            let subtract = color_ternary!(*color, 1, -1);

            // Add value based on castling
            if castle.0 {
                score += CASTLE_VALUE * subtract;
            }
            if castle.1 {
                score += CASTLE_VALUE * subtract;
            }
        }

        if let BoardState::Check(check_color) = self.state {
            color_ternary!(check_color, score -= CHECK_VALUE, score += CHECK_VALUE);
        }

        score
    }
}
