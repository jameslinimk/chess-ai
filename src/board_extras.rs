//! Part of [Board], split for readability
//!
//! Extra fen and util functions for [Board]

use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, PI};
use std::hash::{Hash, Hasher};

use macroquad::prelude::{vec2, WHITE};
use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_triangle};
#[cfg(not(target_family = "wasm"))]
use macroquad::texture::draw_texture;
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use rustc_hash::{FxHashSet, FxHasher};

use crate::board::{Board, BoardState, ChessColor};
use crate::conf::{
    COLOR_ARROW, COLOR_BLACK, COLOR_HIGHLIGHT, COLOR_LAST_MOVE, COLOR_SELECTED, COLOR_WHITE,
    MARGIN, SQUARE_SIZE,
};
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::{
    angle, board_to_pos_center, distance, project, validate_fen, BitBoard, Loc, Tween,
};
use crate::{color_ternary, loc, ternary};

#[rustfmt::skip]
const ENUMERATES: [(usize, usize); 64] = [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2), (7, 2), (0, 3), (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), (0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (5, 4), (6, 4), (7, 4), (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (0, 6), (1, 6), (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6), (0, 7), (1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 7)];

/// Raw 8x8 board data, without any of the metadata [Board] carries
pub(crate) type Raw = [[Option<Piece>; 8]; 8];

/// Whether `color` attacks `target` on `raw`
///
/// - Walks outwards from `target` looking for attackers, rather than building the attack set for
///   the whole side, so it bails out as soon as it finds one and never allocates
pub(crate) fn square_attacked(raw: &Raw, target: Loc, color: ChessColor) -> bool {
    use crate::pieces::util::valid_pos;

    macro_rules! attacker {
        ($dx: expr, $dy: expr) => {{
            let (loc, out) = target.copy_move_i32($dx, $dy);
            if out || !valid_pos(&loc) {
                None
            } else {
                raw[loc.1][loc.0].filter(|piece| piece.color == color)
            }
        }};
    }

    // Knights
    for (dx, dy) in [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ] {
        if let Some(piece) = attacker!(dx, dy) {
            if piece.name == PieceNames::Knight {
                return true;
            }
        }
    }

    // King
    for (dx, dy) in [
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
    ] {
        if let Some(piece) = attacker!(dx, dy) {
            if piece.name == PieceNames::King {
                return true;
            }
        }
    }

    // Pawns. A white pawn attacks towards a smaller y, so it sits below the square it attacks
    let pawn_dy = color_ternary!(color, 1, -1);
    for dx in [-1, 1] {
        if let Some(piece) = attacker!(dx, pawn_dy) {
            if piece.name == PieceNames::Pawn {
                return true;
            }
        }
    }

    // Sliders, out along each ray until something blocks it
    for (directions, slider) in [
        ([(0, -1), (0, 1), (1, 0), (-1, 0)], PieceNames::Rook),
        ([(1, 1), (1, -1), (-1, 1), (-1, -1)], PieceNames::Bishop),
    ] {
        for (dx, dy) in directions {
            let (mut loc, out) = target.copy_move_i32(dx, dy);
            if out {
                continue;
            }

            while valid_pos(&loc) {
                if let Some(piece) = raw[loc.1][loc.0] {
                    if piece.color == color
                        && (piece.name == slider || piece.name == PieceNames::Queen)
                    {
                        return true;
                    }
                    break;
                }

                if !loc.move_i32(dx, dy) {
                    break;
                }
            }
        }
    }

    false
}

impl Board {
    /// Whether moving `from` to `to` would leave `color`'s own king attacked
    ///
    /// - Applies the move to a copy of the raw squares and asks one question of it. Doing this with
    ///   `move_piece` instead rebuilt both attack sets, the blockers, the score and the hash, for
    ///   every candidate move of every piece, at every node of the search
    pub(crate) fn leaves_king_attacked(&self, from: &Loc, to: &Loc, color: ChessColor) -> bool {
        let mut raw = self.raw;

        let mut piece = match raw[from.1][from.0] {
            Some(piece) => piece,
            None => return false,
        };

        // En passent captures a pawn that is not on the destination square
        if piece.name == PieceNames::Pawn && from.0 != to.0 && raw[to.1][to.0].is_none() {
            if let Some((loc, ep_color)) = self.en_passent {
                if to.0 == loc.0 && to.1.abs_diff(loc.1) == 1 && ep_color != piece.color {
                    raw[loc.1][loc.0] = None;
                }
            }
        }

        // Castling brings the rook along, and the rook can block a check on the new king square
        if piece.name == PieceNames::King && from.0.abs_diff(to.0) == 2 {
            let (rook_from, rook_to) = ternary!(
                to.0 == 2,
                (loc!(0, to.1), loc!(3, to.1)),
                (loc!(7, to.1), loc!(5, to.1))
            );

            if let Some(mut rook) = raw[rook_from.1][rook_from.0] {
                rook.pos = rook_to;
                raw[rook_to.1][rook_to.0] = Some(rook);
                raw[rook_from.1][rook_from.0] = None;
            }
        }

        piece.pos = *to;
        raw[to.1][to.0] = Some(piece);
        raw[from.1][from.0] = None;

        let king = if piece.name == PieceNames::King {
            *to
        } else {
            match raw
                .iter()
                .flatten()
                .flatten()
                .find(|piece| piece.name == PieceNames::King && piece.color == color)
            {
                Some(king) => king.pos,
                None => return true,
            }
        };

        let enemy = color_ternary!(color, ChessColor::Black, ChessColor::White);
        square_attacked(&raw, king, enemy)
    }

    /// Generate a new board given a FEN string
    pub(crate) fn from_fen(fen: &str) -> Board {
        let mut fen_parts = fen.split_whitespace();

        /* -------------------------------- Board fen ------------------------------- */
        let board_fen = fen_parts.next().unwrap_or_else(|| panic!("Invalid FEN!"));

        if !validate_fen(board_fen) {
            panic!("Invalid FEN! (board)");
        }

        let mut board = Board::new();
        let mut x: usize = 0;
        let mut y: usize = 0;
        for c in board_fen.chars() {
            // Check end of row
            if c == '/' {
                x = 0;
                y += 1;
                continue;
            }

            // Check for empty squares
            if c.is_ascii_digit() {
                x += c.to_digit(10).unwrap() as usize;
                continue;
            }

            // Check for piece
            let color = if c.is_uppercase() {
                ChessColor::White
            } else {
                ChessColor::Black
            };
            let name = char_to_piece(&c);
            board.set(&loc!(x, y), Some(Piece::new(name, color, loc!(x, y))));
            x += 1;
        }

        /* ----------------------------- Extra fen data ----------------------------- */
        board.turn = match fen_parts.next().unwrap_or_else(|| panic!("Invalid FEN!")) {
            "w" => ChessColor::White,
            "b" => ChessColor::Black,
            _ => panic!("Invalid FEN (turn)"),
        };

        board.castle_white = (false, false);
        board.castle_black = (false, false);
        let castle_fen = fen_parts.next().unwrap_or_else(|| panic!("Invalid FEN!"));
        for char in castle_fen.chars() {
            match char {
                'K' => board.castle_white.1 = true,
                'Q' => board.castle_white.0 = true,
                'k' => board.castle_black.1 = true,
                'q' => board.castle_black.0 = true,
                '-' => {}
                _ => panic!("Invalid FEN (castling)"),
            }
        }

        match fen_parts.next().unwrap_or_else(|| panic!("Invalid FEN!")) {
            "-" => {}
            en_passant => {
                let target = Loc::from_notation(en_passant);
                let (pawn_loc, out, pawn_color) = match board.turn {
                    ChessColor::White => {
                        let (loc, out) = target.copy_move_i32(0, 1);
                        (loc, out, ChessColor::Black)
                    }
                    ChessColor::Black => {
                        let (loc, out) = target.copy_move_i32(0, -1);
                        (loc, out, ChessColor::White)
                    }
                };

                if out
                    || board.get(&pawn_loc).map(|piece| (piece.name, piece.color))
                        != Some((PieceNames::Pawn, pawn_color))
                {
                    panic!("Invalid FEN! (en passent)");
                }

                board.en_passent = Some((pawn_loc, pawn_color));
            }
        }

        let half_move_clock: u32 = fen_parts
            .next()
            .unwrap_or_else(|| panic!("Invalid FEN!"))
            .parse()
            .unwrap_or_else(|_| panic!("Invalid FEN! (fifty rule)"));
        let full_moves: u32 = fen_parts
            .next()
            .unwrap_or_else(|| panic!("Invalid FEN!"))
            .parse()
            .unwrap_or_else(|_| panic!("Invalid FEN! (full moves)"));
        if full_moves == 0 {
            panic!("Invalid FEN! (full moves)");
        }
        board.half_moves =
            color_ternary!(board.turn, (full_moves - 1) * 2, (full_moves - 1) * 2 + 1);
        // `fifty_rule` is the `half_moves` the clock last reset at, not the clock itself, which is
        // what `as_fen` writes back out as `half_moves - fifty_rule`
        board.fifty_rule = board.half_moves.saturating_sub(half_move_clock);

        board.update_things(true);
        board.hash = board.hash();
        board
    }

    /// Export the board into FEN
    pub(crate) fn as_fen(&self) -> String {
        let mut fen = "".to_string();

        let mut board_fen = vec![];
        for row in self.raw.iter() {
            let mut row_string = "".to_string();

            let mut empty = 0;
            for (i, piece) in row.iter().enumerate() {
                match piece {
                    Some(p) => {
                        if empty != 0 {
                            row_string.push_str(&empty.to_string());
                        }
                        empty = 0;

                        let char = piece_to_char(&p.name);
                        row_string.push_str(&match p.color {
                            ChessColor::White => char.to_uppercase().to_string(),
                            ChessColor::Black => char.to_lowercase().to_string(),
                        })
                    }
                    None => {
                        empty += 1;
                        if i == 7 {
                            row_string.push_str(&empty.to_string());
                        }
                    }
                }
            }
            board_fen.push(row_string);
        }

        fen.push_str(&board_fen.join("/"));

        fen.push(' ');
        fen.push(color_ternary!(self.turn, 'w', 'b'));

        fen.push(' ');
        if !self.castle_white.0
            && !self.castle_white.1
            && !self.castle_black.0
            && !self.castle_black.1
        {
            fen.push('-');
        } else {
            if self.castle_white.1 {
                fen.push('K');
            }
            if self.castle_white.0 {
                fen.push('Q');
            }
            if self.castle_black.1 {
                fen.push('k');
            }
            if self.castle_black.0 {
                fen.push('q');
            }
        }

        fen.push(' ');
        if let Some((en_passent, color)) = self.en_passent {
            let (target, out) = color_ternary!(
                color,
                en_passent.copy_move_i32(0, 1),
                en_passent.copy_move_i32(0, -1)
            );
            if out {
                panic!("Invalid en passent state");
            }
            fen.push_str(&target.as_notation())
        } else {
            fen.push('-');
        }

        fen.push(' ');
        fen.push_str(&self.half_moves.saturating_sub(self.fifty_rule).to_string());

        fen.push(' ');
        fen.push_str(&(self.full_moves() + 1).to_string());

        fen
    }

    /// Draws the board to the screen
    #[allow(unused_variables)]
    pub(crate) fn draw(
        &self,
        highlight_moves: &[Loc],
        last_move: &Option<(Loc, Loc)>,
        highlights: &FxHashSet<Loc>,
        arrows: &[(Loc, Loc)],
        current_tween: &mut Option<(Loc, Tween)>,
    ) {
        for (x, y) in ENUMERATES {
            let color = if (x + y) % 2 == 0 {
                COLOR_WHITE
            } else {
                COLOR_BLACK
            };

            draw_rectangle(
                MARGIN + SQUARE_SIZE * x as f32,
                MARGIN + SQUARE_SIZE * y as f32,
                SQUARE_SIZE,
                SQUARE_SIZE,
                color,
            );

            if let Some(last_move) = last_move {
                if last_move.0 == loc!(x, y) || last_move.1 == loc!(x, y) {
                    draw_rectangle(
                        MARGIN + SQUARE_SIZE * x as f32,
                        MARGIN + SQUARE_SIZE * y as f32,
                        SQUARE_SIZE,
                        SQUARE_SIZE,
                        COLOR_LAST_MOVE,
                    );
                }
            }
        }

        for (y, row) in self.raw.iter().enumerate() {
            for (x, square) in row.iter().enumerate() {
                // Draw piece
                if let Some(piece) = square {
                    #[cfg(target_family = "wasm")]
                    {
                        draw_texture_ex(
                            piece.image(),
                            MARGIN + SQUARE_SIZE * x as f32,
                            MARGIN + SQUARE_SIZE * y as f32,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(SQUARE_SIZE, SQUARE_SIZE)),
                                ..Default::default()
                            },
                        )
                    }

                    #[cfg(not(target_family = "wasm"))]
                    {
                        let mut tweened = false;

                        if let Some((loc, tween)) = current_tween {
                            if loc == &loc!(x, y) {
                                let (x, y) = tween.update();
                                draw_texture(
                                    piece.image(),
                                    MARGIN + SQUARE_SIZE * x,
                                    MARGIN + SQUARE_SIZE * y,
                                    WHITE,
                                );
                                tweened = true;
                            }
                        }

                        if !tweened {
                            draw_texture_ex(
                                piece.image(),
                                MARGIN + SQUARE_SIZE * x as f32,
                                MARGIN + SQUARE_SIZE * y as f32,
                                WHITE,
                                DrawTextureParams {
                                    dest_size: Some(vec2(SQUARE_SIZE, SQUARE_SIZE)),
                                    ..Default::default()
                                },
                            )
                        }
                    }
                }
            }
        }

        for (x, y) in ENUMERATES {
            // Draw highlight
            if highlight_moves.contains(&loc!(x, y)) {
                draw_circle(
                    MARGIN + SQUARE_SIZE * x as f32 + SQUARE_SIZE / 2.0,
                    MARGIN + SQUARE_SIZE * y as f32 + SQUARE_SIZE / 2.0,
                    SQUARE_SIZE / 5.0,
                    COLOR_SELECTED,
                );
            }

            if highlights.contains(&loc!(x, y)) {
                draw_circle_lines(
                    MARGIN + SQUARE_SIZE * x as f32 + SQUARE_SIZE / 2.0,
                    MARGIN + SQUARE_SIZE * y as f32 + SQUARE_SIZE / 2.0,
                    SQUARE_SIZE / 2.0 - 2.5,
                    5.0,
                    COLOR_HIGHLIGHT,
                );
            }
        }

        for arrow in arrows.iter() {
            let start = board_to_pos_center(&arrow.0);
            let end = board_to_pos_center(&arrow.1);
            let angle = angle(start, end);

            let left_angle = (angle - FRAC_PI_2 - FRAC_PI_3) % (2.0 * PI);
            let left_point = project(end, left_angle, 25.0);

            let right_angle = (angle + FRAC_PI_2 + FRAC_PI_3) % (2.0 * PI);
            let right_point = project(end, right_angle, 25.0);

            let top_end = project(end, angle, -7.0);

            draw_triangle(
                top_end.into(),
                left_point.into(),
                right_point.into(),
                COLOR_ARROW,
            );

            let new_start = project(start, angle, SQUARE_SIZE / 3.0);
            let new_end = project(start, angle, distance(start, end) - 15.0);
            draw_line(
                new_start.0,
                new_start.1,
                new_end.0,
                new_end.1,
                10.0,
                COLOR_ARROW,
            );
            draw_circle(new_start.0, new_start.1, 5.0, COLOR_ARROW);
        }
    }

    /// Prints board to console
    pub(crate) fn print(&self) {
        for row in self.raw.iter() {
            for piece in row.iter() {
                match piece {
                    Some(p) => {
                        let first_char = piece_to_char(&p.name);
                        print!(
                            "{}",
                            match p.color {
                                ChessColor::White => first_char.to_uppercase().to_string(),
                                ChessColor::Black => first_char.to_lowercase().to_string(),
                            }
                        )
                    }
                    None => print!("-"),
                }
            }
            println!();
        }
    }

    /// Returns a tuple of the locations of the kings (white, black)
    pub(crate) fn kings(&self) -> (Option<Loc>, Option<Loc>) {
        let mut white_king = None;
        let mut black_king = None;
        for piece in self.raw.iter().flatten().flatten() {
            if piece.name == PieceNames::King {
                color_ternary!(
                    piece.color,
                    white_king = Some(piece.pos),
                    black_king = Some(piece.pos)
                );
            }
        }
        (white_king, black_king)
    }

    pub(crate) fn attacks(&self, color: ChessColor) -> BitBoard {
        let mut attacks = BitBoard::default();
        for piece in self.raw.iter().flatten().flatten() {
            if piece.color == color {
                piece.attacks(self, &mut attacks);
            }
        }
        attacks
    }

    /* ----------------------------- Util functions ----------------------------- */
    pub(crate) fn get(&self, loc: &Loc) -> Option<Piece> {
        self.raw[loc.1][loc.0]
    }

    pub(crate) fn set(&mut self, loc: &Loc, value: Option<Piece>) {
        self.raw[loc.1][loc.0] = value;
    }

    pub(crate) fn moves(&self, color: ChessColor) -> Vec<(Loc, Loc)> {
        let mut moves = vec![];
        for piece in self.raw.iter().flatten().flatten() {
            if piece.color == color {
                for m in piece.moves(self) {
                    moves.push((piece.pos, m));
                }
            }
        }
        moves
    }

    /// Returns the number of full moves
    pub(crate) fn full_moves(&self) -> u32 {
        self.half_moves / 2
    }

    /// Checks if the game is over
    pub(crate) fn is_over(&self) -> bool {
        matches!(
            self.state,
            BoardState::Checkmate(_) | BoardState::Stalemate | BoardState::Draw
        )
    }

    /// Returns a hash of the board, with turn, castling and en passent included
    pub(crate) fn hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.raw.hash(&mut hasher);
        self.turn.hash(&mut hasher);
        self.castle_white.hash(&mut hasher);
        self.castle_black.hash(&mut hasher);
        self.en_passent.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    /// Counts the leaf nodes of the move tree, the standard move generation sanity check
    fn perft(board: &Board, depth: u32) -> u64 {
        let moves = board.moves(board.turn);
        if depth <= 1 {
            return moves.len() as u64;
        }

        moves
            .iter()
            .map(|(from, to)| {
                let mut next = board.clone();
                next.move_piece(from, to, false);
                perft(&next, depth - 1)
            })
            .sum()
    }

    fn check_perft(name: &str, fen: &str, expected: &[u64]) {
        let board = Board::from_fen(fen);
        for (i, want) in expected.iter().enumerate() {
            assert_eq!(
                perft(&board, i as u32 + 1),
                *want,
                "{} at depth {}",
                name,
                i + 1
            );
        }
    }

    /// Node counts against the published values for the standard test positions
    #[test]
    fn perft_matches_the_reference_counts() {
        check_perft("start", crate::conf::DEFAULT_FEN, &[20, 400, 8902, 197_281]);

        // Pawn and en passent heavy
        check_perft(
            "position 3",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            &[14, 191, 2812, 43238, 674_624],
        );

        // Middlegame with both sides castled
        check_perft(
            "position 6",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            &[46, 2079, 89890],
        );
    }

    /// Pins the one place the move generator knowingly departs from the rules
    ///
    /// - `move_actions` always promotes to a queen, and a move is a bare `(from, to)` pair with
    ///   nowhere to record the chosen piece, so the three underpromotions are never generated.
    ///   Position 5 has exactly one promotion available, dxc8, which the rules count as four moves
    /// - The reference counts here are 44 and 4085603. If underpromotion is ever added, these
    ///   numbers should become the reference ones
    #[test]
    fn underpromotions_are_the_only_missing_moves() {
        let board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft(&board, 1), 44 - 3);

        // Kiwipete, whose only remaining gap at depth 4 is 11379 unreachable underpromotions
        let kiwipete =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&kiwipete, 3), 97862);
        assert_eq!(perft(&kiwipete, 4), 4_085_603 - 11_379);
    }

    #[test]
    fn hash_includes_side_to_move() {
        let white = Board::from_fen("8/8/8/8/8/8/4P3/K6k w - - 0 1");
        let black = Board::from_fen("8/8/8/8/8/8/4P3/K6k b - - 0 1");

        assert_ne!(white.hash, black.hash);
    }

    #[test]
    fn fen_parses_no_castling_rights() {
        let board = Board::from_fen("8/8/8/8/8/8/4P3/K6k w - - 0 1");

        assert_eq!(board.castle_white, (false, false));
        assert_eq!(board.castle_black, (false, false));
    }

    #[test]
    fn fen_writes_castling_in_standard_order() {
        let mut board = Board::from_fen("8/8/8/8/8/8/4P3/K6k w - - 0 1");
        board.castle_white = (true, false);
        board.castle_black = (false, true);

        assert_eq!(board.as_fen(), "8/8/8/8/8/8/4P3/K6k w Qk - 0 1");
    }

    #[test]
    fn fen_parses_and_writes_en_passant_target_square() {
        let board = Board::from_fen("4k3/8/8/4p3/8/8/8/4K3 w - e6 0 1");

        assert_eq!(
            board.en_passent,
            Some((Loc::from_notation("e5"), ChessColor::Black))
        );
        assert_eq!(board.as_fen(), "4k3/8/8/4p3/8/8/8/4K3 w - e6 0 1");
    }
}

/// Converts a piece name to a char
fn piece_to_char(name: &PieceNames) -> char {
    match name {
        PieceNames::Pawn => 'p',
        PieceNames::Rook => 'r',
        PieceNames::Knight => 'n',
        PieceNames::Bishop => 'b',
        PieceNames::Queen => 'q',
        PieceNames::King => 'k',
    }
}

/// Converts a string to a piece
pub(crate) fn char_to_piece(c: &char) -> PieceNames {
    match c.to_ascii_lowercase() {
        'p' => PieceNames::Pawn,
        'n' => PieceNames::Knight,
        'b' => PieceNames::Bishop,
        'r' => PieceNames::Rook,
        'q' => PieceNames::Queen,
        'k' => PieceNames::King,
        _ => panic!("Invalid piece"),
    }
}
