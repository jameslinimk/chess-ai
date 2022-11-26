use macroquad::prelude::WHITE;
use macroquad::shapes::{draw_circle, draw_rectangle};
use macroquad::texture::draw_texture;

use crate::board::{Board, ChessColor};
use crate::conf::{COLOR_BLACK, COLOR_SELECTED, COLOR_WHITE, MARGIN, SQUARE_SIZE};
use crate::loc;
use crate::pieces::piece::{Piece, PieceNames};
use crate::util::{validate_fen, Loc};

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
        if !validate_fen(fen) {
            panic!("Invalid FEN string");
        }

        let mut board = Board::new();

        let mut x: usize = 0;
        let mut y: usize = 0;
        for c in fen.chars() {
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
                _ => panic!("Invalid FEN"),
            };
            board.raw[y][x] = Some(Piece::new(name, color, loc!(x, y)));
            x += 1;
        }

        board
    }

    /// Export the board into FEN
    pub fn as_fen(&self) -> String {
        let mut fen = vec![];

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
            fen.push(row_string);
        }

        fen.join("/")
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
        // for row in self.raw.iter() {
        //     for piece in row.iter() {
        //         match piece {
        //             Some(p) => {
        //                 let first_char = format!("{:?}", p.name).chars().next().unwrap();
        //                 print!(
        //                     "{}",
        //                     match p.color {
        //                         ChessColor::White => first_char.to_uppercase().to_string(),
        //                         ChessColor::Black => first_char.to_lowercase().to_string(),
        //                     }
        //                 )
        //             }
        //             None => print!("-"),
        //         }
        //     }
        //     println!();
        // }

        for y in 0..8 {
            for x in 0..8 {
                let white = self.attack_white.contains(&loc!(x, y));
                let black = self.attack_black.contains(&loc!(x, y));

                if white && black {
                    print!("o");
                } else if white {
                    print!("w");
                } else if black {
                    print!("b");
                } else {
                    print!("-");
                }
            }

            print!(" ");

            for piece in self.raw[y].iter() {
                match piece {
                    Some(p) => {
                        let char = name_to_fen(&p.name);
                        print!(
                            "{}",
                            match p.color {
                                ChessColor::White => char.to_uppercase().to_string(),
                                ChessColor::Black => char.to_lowercase().to_string(),
                            }
                        )
                    }
                    None => print!("-"),
                }
            }
            println!();
        }

        println!("self.blockers: {:?}", self.blockers);
    }

    /* ----------------------------- Util functions ----------------------------- */
    pub fn player_turn(&self) -> bool {
        self.turn == self.color_player
    }

    pub fn agent_turn(&self) -> bool {
        self.turn == self.color_agent
    }

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
}
