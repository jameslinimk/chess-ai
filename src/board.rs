use derive_new::new;

use crate::pieces::piece::{Piece, PieceNames};
use crate::util::{BitBoard, Loc};
use crate::{color_ternary, loc, ternary};

/// How many half moves of history are kept for threefold repetition detection
const MAX_PREV_STATES: usize = 24;

/// Black or white, the colors of chess
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ChessColor {
    Black,
    White,
}

/// Board state IE (check, checkmate, etc)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BoardState {
    Normal,
    /// Attached color is who is in check
    Check(ChessColor),
    /// Attached color is who is in checkmate
    Checkmate(ChessColor),
    Stalemate,
    Draw,
}
impl BoardState {
    /// Returns the endgame message for the board state, panics if the game is not over
    pub(crate) fn message(&self, player_color: ChessColor) -> &'static str {
        match self {
            BoardState::Checkmate(color) => ternary!(
                &player_color != color,
                "Congrats! You won!\nPress \"r\" to restart!",
                "Dang, you lost\nPress \"r\" to restart!"
            ),
            BoardState::Stalemate => "Game over, stalemate\nPress \"r\" to restart!",
            BoardState::Draw => "Game over, draw\nPress \"r\" to restart!",
            _ => unreachable!(),
        }
    }
}

/// Represents a chess board and metadata
#[derive(Debug, Clone, PartialEq, Eq, new)]
pub(crate) struct Board {
    /// Array with the raw 8x8 board data
    #[new(value = "[[None; 8]; 8]")]
    pub(crate) raw: [[Option<Piece>; 8]; 8],

    /// Turn of the board
    #[new(value = "ChessColor::White")]
    pub(crate) turn: ChessColor,

    /// State of the board IE (check, checkmate, etc)
    #[new(value = "BoardState::Normal")]
    pub(crate) state: BoardState,

    /// Player color
    #[new(value = "ChessColor::White")]
    pub(crate) player_color: ChessColor,

    /// Agent color
    #[new(value = "ChessColor::Black")]
    pub(crate) agent_color: ChessColor,

    /// True if black can castle (queen side, king side)
    #[new(value = "(true, true)")]
    pub(crate) castle_black: (bool, bool),

    /// True if white can castle (queen side, king side)
    #[new(value = "(true, true)")]
    pub(crate) castle_white: (bool, bool),

    /// Last pawn move and color
    #[new(value = "None")]
    pub(crate) en_passent: Option<(Loc, ChessColor)>,

    /// Current score of board, for white
    #[new(value = "0")]
    pub(crate) score: i32,

    /// Which squares are under attack by white pieces
    #[new(value = "BitBoard::default()")]
    pub(crate) attacks_white: BitBoard,

    /// Which squares are under attack by black pieces
    #[new(value = "BitBoard::default()")]
    pub(crate) attacks_black: BitBoard,

    /// Wether the white king is in check
    #[new(value = "false")]
    pub(crate) check_white: bool,

    /// Wether the black king is in check
    #[new(value = "false")]
    pub(crate) check_black: bool,

    /// Pieces that block any attackers
    #[new(value = "BitBoard::default()")]
    pub(crate) blockers: BitBoard,

    /// Available moves for white
    #[new(value = "vec![]")]
    pub(crate) moves_white: Vec<(Loc, Loc)>,

    /// Available moves for black
    #[new(value = "vec![]")]
    pub(crate) moves_black: Vec<(Loc, Loc)>,

    /// Number of half moves (+1 per white *or* black turn)
    /// - Use `Board.full_moves()` for full moves
    #[new(value = "0")]
    pub(crate) half_moves: u32,

    /// Previous board states, used for 3fold check
    #[new(value = "Vec::with_capacity(MAX_PREV_STATES)")]
    pub(crate) prev_states: Vec<u64>,

    /// Value of `half_moves` the last time a piece was captured or a pawn moved
    /// - The FEN half move clock is `half_moves - fifty_rule`
    #[new(value = "0")]
    pub(crate) fifty_rule: u32,

    /// Wether the game is endgame or not
    #[new(value = "false")]
    pub(crate) endgame: bool,

    /// Hash of the board
    #[new(value = "0")]
    pub(crate) hash: u64,

    /// `((queen knight, king knight), (queen bishop, king bishop)))`
    /// - `true` if moved before, `false` if not
    #[new(value = "((false, false), (false, false))")]
    pub(crate) agent_developments: ((bool, bool), (bool, bool)),
}
impl Board {
    /// Moves the piece in `from` to `to`
    pub(crate) fn move_piece(&mut self, from: &Loc, to: &Loc, check_stale: bool) -> bool {
        if self.get(from).is_none() {
            return false;
        }

        let capture_info = self.is_capture(from, to);
        let capture = capture_info.is_some();

        // Special case where a castle rook is captured
        if let Some(capture_pos) = capture_info {
            let piece = self.get(&capture_pos).unwrap();
            if piece.name == PieceNames::Rook {
                match piece.color {
                    ChessColor::White => {
                        if piece.pos == loc!(0, 7) {
                            self.castle_white.0 = false;
                        } else if piece.pos == loc!(7, 7) {
                            self.castle_white.1 = false;
                        }
                    }
                    ChessColor::Black => {
                        if piece.pos == loc!(0, 0) {
                            self.castle_black.0 = false;
                        } else if piece.pos == loc!(7, 0) {
                            self.castle_black.1 = false;
                        }
                    }
                }
            }
        }

        // Moving piece
        let pawn_move = self.move_actions(from, to);
        self.move_raw(from, to);

        // Update turn
        self.turn = match self.turn {
            ChessColor::Black => ChessColor::White,
            ChessColor::White => ChessColor::Black,
        };
        self.half_moves += 1;

        // Set hash (relies on nothing)
        self.hash = self.hash();

        // Fifty move rule (relies on half_moves)
        if capture || pawn_move {
            self.fifty_rule = self.half_moves;
        }

        // 3fold repetition (relies on hash)
        // Captures and pawn moves are irreversible, so no earlier state can happen again
        if capture || pawn_move {
            self.prev_states.clear();
        }
        if self.prev_states.len() == MAX_PREV_STATES {
            self.prev_states.rotate_left(1);
            self.prev_states[MAX_PREV_STATES - 1] = self.hash;
        } else {
            self.prev_states.push(self.hash);
        }

        // Update other metadata
        self.update_things(check_stale);

        capture
    }

    /// Updates "things", such as the game state, checks, attacks, etc. Auto called by `move_piece`
    pub(crate) fn update_things(&mut self, check_stale: bool) {
        // Update attacks (relies on nothing)
        self.attacks_white = self.attacks(ChessColor::White);
        self.attacks_black = self.attacks(ChessColor::Black);

        // Update check (relies on attacks)
        let (white_king, black_king) = self.kings();
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
            self.moves_white = self.moves(ChessColor::White);
            self.moves_black = self.moves(ChessColor::Black);
        } else {
            // Nothing reads these unless `check_stale` is set. Leaving the parent's lists in place
            // meant every node of the search cloned two vectors it was never going to look at, and
            // it is what made the stale move lists reachable by `detect_state` in the first place
            self.moves_white.clear();
            self.moves_black.clear();
        }

        // Detect state (relies on check and moves)
        self.detect_state(check_stale);

        // Set endgame (relies on nothing)
        self.endgame = {
            let mut queens = 0;
            let mut minors = 0;

            for piece in self.raw.iter().flatten().flatten() {
                match piece.name {
                    PieceNames::Bishop | PieceNames::Knight => minors += 1,
                    PieceNames::Queen => queens += 1,
                    _ => {}
                }
            }

            queens == 0 || minors <= queens
        };

        // Set score (relies on state, endgame)
        self.score = self.score();
    }

    /// Detect wether the players are in check, checkmate or stalemate
    ///
    /// - Checkmate and stalemate can only be detected when `check_stale` is set, since they need
    ///   the move lists. During the search `check_stale` is false and the move lists are stale, so
    ///   only check / normal is reported here and [crate::agent::minimax] detects the terminal
    ///   positions itself from the move list it already generates
    fn detect_state(&mut self, check_stale: bool) {
        // Fifty move rule (100 half moves, ie 50 moves by each player)
        if self.half_moves.saturating_sub(self.fifty_rule) >= 100 {
            self.state = BoardState::Draw;
            return;
        }

        // 3fold repetition
        let mut sum = 0;
        for hash in self.prev_states.iter() {
            if hash == &self.hash {
                sum += 1;
                if sum >= 3 {
                    self.state = BoardState::Draw;
                    return;
                }
            }
        }

        // Draw by insufficient material: bare kings, or a lone minor piece, which cannot mate
        let (mut minors, mut majors) = (0, 0);
        for piece in self.raw.iter().flatten().flatten() {
            match piece.name {
                PieceNames::King => {}
                PieceNames::Bishop | PieceNames::Knight => minors += 1,
                _ => majors += 1,
            }
        }
        if majors == 0 && minors <= 1 {
            self.state = BoardState::Draw;
            return;
        }

        // Others (relies on the check flags, which are always up to date)
        let checked = match (self.check_white, self.check_black) {
            (true, _) => Some(ChessColor::White),
            (_, true) => Some(ChessColor::Black),
            _ => None,
        };

        // Always reassign, so a board cloned during the search never keeps its parent's state
        let state = match checked {
            Some(color) => {
                let moves = color_ternary!(color, &self.moves_white, &self.moves_black);
                if check_stale && moves.is_empty() {
                    BoardState::Checkmate(color)
                } else {
                    BoardState::Check(color)
                }
            }
            None => {
                let moves = color_ternary!(self.turn, &self.moves_white, &self.moves_black);
                if check_stale && moves.is_empty() {
                    BoardState::Stalemate
                } else {
                    BoardState::Normal
                }
            }
        };
        self.state = state;
    }

    /// Updates `self.blockers`
    fn update_blockers(&mut self) {
        self.blockers = BitBoard::default();

        for (attacks, blocked) in [
            (self.attacks_white, ChessColor::Black),
            (self.attacks_black, ChessColor::White),
        ] {
            for loc in attacks.iter() {
                if let Some(piece) = self.get(&loc) {
                    if piece.color == blocked {
                        self.blockers.insert(loc);
                    }
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

    pub(crate) fn is_capture(&self, from: &Loc, to: &Loc) -> Option<Loc> {
        if self.get(to).is_some() {
            return Some(*to);
        }

        if let Some(piece) = self.get(from) {
            if piece.name == PieceNames::Pawn && from.0.abs_diff(to.0) == 1 {
                if let Some((loc, color)) = self.en_passent {
                    if to.0 == loc.0 && to.1.abs_diff(loc.1) == 1 && piece.color != color {
                        return Some(loc);
                    }
                }
            };
        };

        None
    }

    /// Whether moving `from` to `to` promotes a pawn
    ///
    /// - `move_actions` always promotes to a queen, so this is only ever a yes or no. Quiescence
    ///   uses it to make sure promotions are searched alongside the captures
    pub(crate) fn is_promotion(&self, from: &Loc, to: &Loc) -> bool {
        (to.1 == 0 || to.1 == 7) && self.get(from).map(|piece| piece.name) == Some(PieceNames::Pawn)
    }

    /// Special actions that happen when moving a piece
    /// - IE: Castling, en passent, pawn promotion, etc...
    /// - Returns `true` if the piece moved was a pawn
    fn move_actions(&mut self, from: &Loc, to: &Loc) -> bool {
        let mut set_en_passent = false;
        let mut pawn_move = false;

        // Read before the piece is moved, so an en passent capture can be told apart from a normal
        // capture that happens to land a rank away from the en passent pawn, on the same file
        let dest_empty = self.get(to).is_none();

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
                    pawn_move = true;

                    // Promotion
                    if to.1 == 0 || to.1 == 7 {
                        piece.name = PieceNames::Queen;
                    }

                    // Setting en passent
                    if from.1.abs_diff(to.1) == 2 {
                        self.en_passent = Some((*to, piece.color));
                        set_en_passent = true;
                    }

                    // En passent capture, only when moving diagonally onto an *empty* square.
                    // Without the `dest_empty` check a normal capture on a square next to the en
                    // passent pawn would remove that pawn as well
                    if dest_empty && from.0.abs_diff(to.0) == 1 {
                        if let Some((loc, color)) = self.en_passent {
                            if to.0 == loc.0 && to.1.abs_diff(loc.1) == 1 && piece.color != color {
                                self.set(&loc, None);
                            }
                        }
                    }
                }
                PieceNames::Knight => {
                    let (queenside, kingside) = &mut self.agent_developments.0;
                    let y = color_ternary!(piece.color, 7, 0);
                    if from == &loc!(1, y) {
                        *queenside = true;
                    } else if from == &loc!(6, y) {
                        *kingside = true;
                    }
                }
                PieceNames::Bishop => {
                    let (queenside, kingside) = &mut self.agent_developments.1;
                    let y = color_ternary!(piece.color, 7, 0);
                    if from == &loc!(2, y) {
                        *queenside = true;
                    } else if from == &loc!(5, y) {
                        *kingside = true;
                    }
                }
                _ => {}
            }
        }

        // Reset en passent if it hasn't been set yet
        if !set_en_passent && self.en_passent.is_some() {
            self.en_passent = None;
        }

        pawn_move
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notation(n: &str) -> Loc {
        Loc::from_notation(n)
    }

    #[test]
    fn en_passent_does_not_capture_two_pieces() {
        // Black pushes e7-e5 past the white d3 pawn, but white takes the knight on e4 instead. The
        // e5 pawn is not the piece being captured, so it has to stay on the board
        let mut board = Board::from_fen("4k3/4p3/8/8/4n3/3P4/8/4K3 b - - 0 30");
        board.move_piece(&notation("e7"), &notation("e5"), true);
        assert_eq!(board.en_passent, Some((notation("e5"), ChessColor::Black)));

        board.move_piece(&notation("d3"), &notation("e4"), true);

        let e5 = board
            .get(&notation("e5"))
            .expect("the e5 pawn should survive");
        assert_eq!((e5.name, e5.color), (PieceNames::Pawn, ChessColor::Black));
        let e4 = board
            .get(&notation("e4"))
            .expect("the white pawn should be on e4");
        assert_eq!((e4.name, e4.color), (PieceNames::Pawn, ChessColor::White));
    }

    #[test]
    fn en_passent_still_captures_the_pawn() {
        let mut board = Board::from_fen("4k3/4p3/8/3P4/8/8/8/4K3 b - - 0 30");
        board.move_piece(&notation("e7"), &notation("e5"), true);

        assert!(board
            .moves(ChessColor::White)
            .contains(&(notation("d5"), notation("e6"))));

        board.move_piece(&notation("d5"), &notation("e6"), true);

        assert!(board.get(&notation("e5")).is_none());
        let e6 = board
            .get(&notation("e6"))
            .expect("the white pawn should be on e6");
        assert_eq!((e6.name, e6.color), (PieceNames::Pawn, ChessColor::White));
    }

    #[test]
    fn fifty_move_rule_needs_a_hundred_half_moves() {
        let almost = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 99 60");
        assert_ne!(almost.state, BoardState::Draw);

        let drawn = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 100 60");
        assert_eq!(drawn.state, BoardState::Draw);
    }

    #[test]
    fn half_move_clock_survives_a_fen_round_trip() {
        let board = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 40 30");

        assert!(board.as_fen().ends_with(" 40 30"), "got {}", board.as_fen());
        assert_ne!(board.state, BoardState::Draw);
    }

    #[test]
    fn inconsistent_half_move_clock_does_not_underflow() {
        // The clock claims more half moves than the move number allows. This used to underflow the
        // `half_moves - fifty_rule` subtraction, panicking in debug and drawing in release
        let board = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 5 1");

        assert_ne!(board.state, BoardState::Draw);
    }

    #[test]
    fn state_is_recomputed_for_search_boards() {
        // `move_piece(.., false)` is the path the search takes. The state used to be left at
        // whatever the parent board had, so a position where nobody is in check kept reporting one
        let mut board = Board::from_fen("4k3/8/8/8/7r/8/8/4K3 b - - 0 30");
        board.move_piece(&notation("h4"), &notation("e4"), true);
        assert_eq!(board.state, BoardState::Check(ChessColor::White));

        board.move_piece(&notation("e1"), &notation("d1"), false);

        assert!(!board.check_white);
        assert_eq!(board.state, BoardState::Normal);
    }

    /// Knight shuffle that returns both sides to the starting position every 4 half moves
    const SHUFFLE: [(&str, &str); 4] = [("g1", "f3"), ("g8", "f6"), ("f3", "g1"), ("f6", "g8")];
    const SHUFFLE_FEN: &str = "4k1n1/8/8/8/8/8/8/4K1N1 w - - 0 30";

    #[test]
    fn threefold_repetition_is_a_draw() {
        let mut board = Board::from_fen(SHUFFLE_FEN);

        for i in 0..9 {
            let (from, to) = SHUFFLE[i % SHUFFLE.len()];
            board.move_piece(&notation(from), &notation(to), true);

            // The 9th half move is the third time that position has been on the board
            if i < 8 {
                assert_ne!(
                    board.state,
                    BoardState::Draw,
                    "drawn too early, at half move {}",
                    i + 1
                );
            }
        }

        assert_eq!(board.state, BoardState::Draw);
    }

    #[test]
    fn repetition_history_keeps_the_newest_states() {
        // The buffer used to rotate the newest entry into slot 0 and then overwrite it, so once it
        // filled up it only ever remembered the oldest states
        let mut board = Board::from_fen(SHUFFLE_FEN);

        for i in 0..MAX_PREV_STATES + 4 {
            let (from, to) = SHUFFLE[i % SHUFFLE.len()];
            board.move_piece(&notation(from), &notation(to), false);
        }

        assert_eq!(board.prev_states.len(), MAX_PREV_STATES);
        assert_eq!(board.prev_states.last(), Some(&board.hash));
    }

    #[test]
    fn draws_by_insufficient_material() {
        // Only bare kings used to count, so the engine kept trying to win a king and knight ending
        for (fen, expected) in [
            ("4k3/8/8/8/8/8/8/4K3 w - - 0 30", true),     // K vs K
            ("4k3/8/8/8/8/8/8/4KN2 w - - 0 30", true),    // K+N vs K
            ("4k3/8/8/8/8/8/8/4KB2 w - - 0 30", true),    // K+B vs K
            ("4k3/8/8/8/8/8/8/4KNN1 w - - 0 30", false),  // K+2N: mate is possible
            ("4k3/8/8/8/8/8/8/4KR2 w - - 0 30", false),   // K+R: mate is forced
            ("4k3/8/8/8/8/8/4P3/4K3 w - - 0 30", false),  // K+P promotes
            ("4k3/8/8/8/8/8/8/2b1KN2 w - - 0 30", false), // one minor each
        ] {
            let board = Board::from_fen(fen);
            assert_eq!(board.state == BoardState::Draw, expected, "{}", fen);
        }
    }
}
