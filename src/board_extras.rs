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
use crate::util::{angle, board_to_pos_center, distance, project, validate_fen, Loc, Tween};
use crate::{color_ternary, hashset, loc};

#[rustfmt::skip]
const ENUMERATES: [(usize, usize); 64] = [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1), (0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2), (6, 2), (7, 2), (0, 3), (1, 3), (2, 3), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3), (0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (5, 4), (6, 4), (7, 4), (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (0, 6), (1, 6), (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6), (0, 7), (1, 7), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 7)];

impl Board {
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
                let loc = Loc::from_notation(en_passant);
                board.en_passent = Some((
                    loc,
                    board
                        .get(&loc)
                        .unwrap_or_else(|| panic!("Invalid FEN! (en passent)"))
                        .color,
                ));
            }
        }

        board.fifty_rule = fen_parts
            .next()
            .unwrap_or_else(|| panic!("Invalid FEN!"))
            .parse()
            .unwrap_or_else(|_| panic!("Invalid FEN! (fifty rule)"));
        let full_moves: u32 = fen_parts
            .next()
            .unwrap_or_else(|| panic!("Invalid FEN!"))
            .parse()
            .unwrap_or_else(|_| panic!("Invalid FEN! (full moves)"));
        board.half_moves =
            color_ternary!(board.turn, (full_moves - 1) * 2, (full_moves - 1) * 2 + 1);

        board.update_things(true);
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
            if self.castle_white.0 {
                fen.push('K');
            }
            if self.castle_white.1 {
                fen.push('Q');
            }
            if self.castle_black.0 {
                fen.push('k');
            }
            if self.castle_black.1 {
                fen.push('q');
            }
        }

        fen.push(' ');
        if let Some(en_passent) = self.en_passent {
            fen.push_str(&en_passent.0.as_notation())
        } else {
            fen.push('-');
        }

        fen.push(' ');
        fen.push_str(&(self.half_moves - self.fifty_rule).to_string());

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
                            piece.get_image(),
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

    pub(crate) fn attacks(&mut self, color: ChessColor) -> FxHashSet<Loc> {
        let mut attacks = hashset! {};
        for piece in self.raw.iter().flatten().flatten() {
            if piece.color == color {
                attacks.extend(piece.attacks(self));
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

    /// Returns a hash of the board, with castling and en passent included
    pub(crate) fn hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.raw.hash(&mut hasher);
        self.castle_white.hash(&mut hasher);
        self.castle_black.hash(&mut hasher);
        self.en_passent.hash(&mut hasher);
        hasher.finish()
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
