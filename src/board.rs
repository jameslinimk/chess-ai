use derive_new::new;
use rustc_hash::FxHashSet;

use crate::pieces::piece::{Piece, PieceNames};
use crate::util::Loc;
use crate::{color_ternary, hashset, loc};

/// Black or white, the colors of chess
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

    pub fn display(&self) -> &'static str {
        match self {
            ChessColor::Black => "black",
            ChessColor::White => "white",
        }
    }
}

/// Board state IE (check, checkmate, etc)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoardState {
    Normal,
    /// Attached color is who is in check
    Check(ChessColor),
    /// Attached color is who is in checkmate
    Checkmate(ChessColor),
    Stalemate,
    Draw,
}

/// Board that is stripped of non-vital information to compare two boards, used for 3fold repetition check
#[derive(Clone, Copy, Debug, PartialEq, Eq, new)]
pub struct SimpleBoard {
    pub raw: [[Option<Piece>; 8]; 8],
    pub castle_black: (bool, bool),
    pub castle_white: (bool, bool),
    pub en_passent: Option<(Loc, ChessColor)>,
}
impl PartialEq<Board> for SimpleBoard {
    fn eq(&self, other: &Board) -> bool {
        self.raw == other.raw
            && self.castle_black == other.castle_black
            && self.castle_white == other.castle_white
            && self.en_passent == other.en_passent
    }
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

    /// State of the board IE (check, checkmate, etc)
    #[new(value = "BoardState::Normal")]
    pub state: BoardState,

    /// Player color
    #[new(value = "ChessColor::White")]
    pub player_color: ChessColor,

    /// Agent color
    #[new(value = "ChessColor::Black")]
    pub agent_color: ChessColor,

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
    pub attacks_white: FxHashSet<Loc>,

    /// Which squares are under attack by black pieces
    #[new(value = "hashset! {}")]
    pub attacks_black: FxHashSet<Loc>,

    /// Wether the white king is in check
    #[new(value = "false")]
    pub check_white: bool,

    /// Wether the black king is in check
    #[new(value = "false")]
    pub check_black: bool,

    /// Pieces that block any attackers
    #[new(value = "hashset! {}")]
    pub blockers: FxHashSet<Loc>,

    /// Available moves for white
    #[new(value = "vec![]")]
    pub moves_white: Vec<(Loc, Loc)>,

    /// Available moves for black
    #[new(value = "vec![]")]
    pub moves_black: Vec<(Loc, Loc)>,

    /// Number of half moves (+1 per white *or* black turn)
    /// - Use `Board.full_moves()` for full moves
    #[new(value = "0")]
    pub half_moves: u32,

    /// Previous board states, used for 3fold check
    #[new(value = "[None; 12]")]
    pub prev_states: [Option<SimpleBoard>; 12],

    /// Updates on piece capture or pawn move
    #[new(value = "0")]
    pub fifty_rule: u32,

    /// Wether the game is endgame or not
    #[new(value = "false")]
    pub endgame: bool,
}
impl Board {
    /// Moves the piece in `from` to `to`
    pub fn move_piece(&mut self, from: &Loc, to: &Loc, check_stale: bool) -> bool {
        if self.get(from).is_none() {
            return false;
        }

        let (capture, capture_pos) = self.is_capture(from, to);

        // Special case where a castle rook is captured
        if capture {
            let piece = self.get(&capture_pos).unwrap();
            if piece.name == PieceNames::Rook {
                match piece.color {
                    ChessColor::White => {
                        if capture_pos == loc!(0, 0) {
                            self.castle_white.0 = false;
                        } else if capture_pos == loc!(7, 0) {
                            self.castle_white.1 = false;
                        }
                    }
                    ChessColor::Black => {
                        if capture_pos == loc!(0, 7) {
                            self.castle_black.0 = false;
                        } else if capture_pos == loc!(7, 7) {
                            self.castle_black.1 = false;
                        }
                    }
                }
            }
        }

        // Moving piece
        self.move_actions(from, to);
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

    /// Updates "things", such as the game state, checks, attacks, etc. Auto called by `move_piece`
    pub fn update_things(&mut self, check_stale: bool) {
        // Update attacks (relies on nothing)
        self.attacks_white = self.get_attacks(ChessColor::White);
        self.attacks_black = self.get_attacks(ChessColor::Black);

        // Update check (relies on attacks)
        let (white_king, black_king) = self.get_kings();
        if let Some(white_king) = white_king {
            self.check_white = self.attacks_black.contains(&white_king);
        } else {
            self.check_white = true;
        }
        if let Some(black_king) = black_king {
            self.check_black = self.attacks_white.contains(&black_king);
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

        // Set endgame (relies on nothing)
        self.endgame = {
            let mut queens = 0;
            let mut minors = 0;

            for row in self.raw.iter() {
                for piece in row.iter().flatten() {
                    match piece.name {
                        PieceNames::Bishop | PieceNames::Knight => minors += 1,
                        PieceNames::Queen => queens += 1,
                        _ => {}
                    }
                }
            }

            queens == 0 || minors <= queens
        };

        // Set score (relies on state, endgame)
        self.score = self.get_score();
    }

    /// Detect wether the players are in check, checkmate or stalemate
    fn detect_state(&mut self, check_stale: bool) {
        // Fifty move rule
        if self.half_moves - self.fifty_rule >= 50 {
            self.state = BoardState::Draw;
            return;
        }

        // 3fold repetition
        let mut sum = 0;
        for simple in self.prev_states.iter().flatten() {
            if simple == self {
                sum += 1;
                if sum >= 3 {
                    self.state = BoardState::Draw;
                    return;
                }
            }
        }

        // Others
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

    /// Updates `self.blockers`
    fn update_blockers(&mut self) {
        self.blockers = hashset! {};
        for loc in self.attacks_white.iter() {
            if let Some(piece) = self.get(loc) {
                if piece.color == ChessColor::Black {
                    self.blockers.insert(*loc);
                }
            }
        }
        for loc in self.attacks_black.iter() {
            if let Some(piece) = self.get(loc) {
                if piece.color == ChessColor::White {
                    self.blockers.insert(*loc);
                }
            }
        }
    }

    // Move the piece in `from` to `to` without updating anything
    fn move_raw(&mut self, from: &Loc, to: &Loc) {
        if let Some(piece) = self.raw[from.1][from.0].as_mut() {
            piece.pos = *to;
        }

        self.set(to, self.get(from));
        self.set(from, None);
    }

    pub fn is_capture(&self, from: &Loc, to: &Loc) -> (bool, Loc) {
        if self.get(to).is_some() {
            return (true, *to);
        }

        if let Some(piece) = self.get(from) {
            if piece.name == PieceNames::Pawn && from.0.abs_diff(to.0) == 1 {
                if let Some((loc, color)) = self.en_passent {
                    if to.0 == loc.0 && to.1.abs_diff(loc.1) == 1 && piece.color != color {
                        return (true, loc);
                    }
                }
            };
        };

        (false, loc!(0, 0))
    }

    /// Special actions that happen when moving a piece
    /// - IE: Castling, en passent, pawn promotion, etc...
    fn move_actions(&mut self, from: &Loc, to: &Loc) {
        let mut set_en_passent = false;

        if let Some(piece) = self.raw[from.1][from.0].as_mut() {
            piece.pos = *to;

            match piece.name {
                // Castle check
                PieceNames::King => {
                    match piece.color {
                        ChessColor::Black => self.castle_black = (false, false),
                        ChessColor::White => self.castle_white = (false, false),
                    }

                    if from.0.abs_diff(to.0) == 2 {
                        let (rook_from, rook_to) = match to.0 {
                            2 => (loc!(0, to.1), loc!(3, to.1)),
                            6 => (loc!(7, to.1), loc!(5, to.1)),
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
                    if to.1 == 0 || to.1 == 7 {
                        piece.name = PieceNames::Queen;
                    }

                    // Setting en passent
                    if from.1.abs_diff(to.1) == 2 {
                        self.en_passent = Some((*to, piece.color));
                        set_en_passent = true;
                    }

                    // En passent capture
                    if from.0.abs_diff(to.0) == 1 {
                        if let Some((loc, color)) = self.en_passent {
                            if to.0 == loc.0 && to.1.abs_diff(loc.1) == 1 && piece.color != color {
                                self.set(&loc, None);
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
    }
}
