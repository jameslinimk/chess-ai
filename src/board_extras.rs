//! Part of [Board], split for readability
//!
//! Extra fen and util functions for [Board]

use macroquad::prelude::{info, WHITE};
use macroquad::shapes::{draw_circle, draw_rectangle};
use macroquad::texture::draw_texture;
use rustc_hash::FxHashSet;

use crate::board::{Board, BoardState, ChessColor, SimpleBoard};
use crate::conf::{COLOR_BLACK, COLOR_SELECTED, COLOR_WHITE, MARGIN, SQUARE_SIZE};
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::{validate_fen, Loc};
use crate::{color_ternary, hashset, loc};

#[inline(always)]
fn name_to_fen(name: &PieceNames) -> char {
    match name {
        PieceNames::Pawn => 'p',
        PieceNames::Rook => 'r',
        PieceNames::Knight => 'n',
        PieceNames::Bishop => 'b',
        PieceNames::Queen => 'q',
        PieceNames::King => 'k',
    }
}

impl Board {
    /// Generate a new board given a FEN string
    pub fn from_fen(fen: &str) -> Board {
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
            let name = match c.to_ascii_lowercase() {
                'p' => PieceNames::Pawn,
                'n' => PieceNames::Knight,
                'b' => PieceNames::Bishop,
                'r' => PieceNames::Rook,
                'q' => PieceNames::Queen,
                'k' => PieceNames::King,
                _ => panic!("Invalid FEN (board)"),
            };
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
    pub fn as_fen(&self) -> String {
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

                        let char = name_to_fen(&p.name);
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
    pub fn draw(&self, highlight: &[Loc]) {
        for (y, row) in self.raw.iter().enumerate() {
            for (x, square) in row.iter().enumerate() {
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

                // Draw piece
                if let Some(piece) = square {
                    draw_texture(
                        piece.get_image(),
                        MARGIN + SQUARE_SIZE * x as f32,
                        MARGIN + SQUARE_SIZE * y as f32,
                        WHITE,
                    )
                }

                // Draw highlight
                if highlight.contains(&loc!(x, y)) {
                    draw_circle(
                        MARGIN + SQUARE_SIZE * x as f32 + SQUARE_SIZE / 2.0,
                        MARGIN + SQUARE_SIZE * y as f32 + SQUARE_SIZE / 2.0,
                        SQUARE_SIZE / 4.0,
                        COLOR_SELECTED,
                    );
                }
            }
        }
    }

    /// Prints board to console
    pub fn print(&self) {
        for row in self.raw.iter() {
            for piece in row.iter() {
                match piece {
                    Some(p) => {
                        let first_char = format!("{:?}", p.name).chars().next().unwrap();
                        info!(
                            "{}",
                            match p.color {
                                ChessColor::White => first_char.to_uppercase().to_string(),
                                ChessColor::Black => first_char.to_lowercase().to_string(),
                            }
                        )
                    }
                    None => info!("-"),
                }
            }
            info!("");
        }
    }

    /// Returns a tuple of the locations of the kings (white, black)
    pub fn get_kings(&self) -> (Option<Loc>, Option<Loc>) {
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

    /* ----------------------------- Util functions ----------------------------- */
    pub fn get(&self, loc: &Loc) -> Option<Piece> {
        self.raw[loc.y][loc.x]
    }

    pub fn set(&mut self, loc: &Loc, value: Option<Piece>) {
        self.raw[loc.y][loc.x] = value;
    }

    pub fn get_moves(&self, color: ChessColor) -> Vec<(Loc, Loc)> {
        let mut moves = vec![];
        for row in self.raw.iter() {
            for piece in row.iter().flatten() {
                if piece.color == color {
                    for m in piece.get_moves(self) {
                        moves.push((piece.pos, m));
                    }
                }
            }
        }
        moves
    }

    pub fn full_moves(&self) -> u32 {
        self.half_moves / 2
    }

    pub fn is_over(&self) -> bool {
        matches!(self.state, BoardState::Checkmate(_) | BoardState::Stalemate)
    }

    pub fn as_simple(&self) -> SimpleBoard {
        SimpleBoard {
            raw: self.raw,
            castle_black: self.castle_black,
            castle_white: self.castle_white,
            en_passent: self.en_passent,
        }
    }
}
