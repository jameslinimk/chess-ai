//! Parse and store ECO opening database

use std::str::from_utf8;

use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::from_str;

use crate::board::Board;
use crate::util::Loc;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Opening {
    /// Name of the opening
    pub name: String,
    /// List of moves the the full opening
    pub moves: Vec<(Loc, Loc)>,
}

impl Board {
    pub fn update_opens(&mut self) {
        self.openings.retain(|open| {
            open.moves.starts_with(&self.move_history) && open.moves.len() > self.move_history.len()
        });
    }
}

lazy_static! {
    pub static ref OPENINGS: Vec<Opening> =
        from_str(from_utf8(include_bytes!("../assets/openings.json")).unwrap()).unwrap();
}

#[test]
fn create_openings() {
    use std::fs::{read_to_string, write};

    use serde::{Deserialize, Serialize};
    use serde_json::{from_str, to_string};

    use crate::board::{Board, ChessColor};
    use crate::board_extras::char_to_piece;
    use crate::conf::TEST_FEN;
    use crate::pieces::piece::PieceNames;
    use crate::{color_ternary, loc, ternary};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Opening {
        name: String,
        code: String,
        moves: Vec<String>,
    }

    #[derive(Serialize, Debug)]
    pub struct NewOpening {
        name: String,
        moves: Vec<(Loc, Loc)>,
    }

    let mut new_openings: Vec<NewOpening> = vec![];

    let openings =
        from_str::<Vec<Opening>>(&read_to_string("openings/openings.json").unwrap()).unwrap();

    for opening in openings.iter() {
        let mut board = Board::from_fen(TEST_FEN);

        let mut new_moves = vec![];
        #[allow(clippy::never_loop)]
        for (i, raw_ms) in opening.moves.iter().enumerate() {
            let turn = ternary!(i % 2 == 0, ChessColor::White, ChessColor::Black);

            let legal_moves = color_ternary!(
                board.turn,
                board.get_moves(ChessColor::White),
                board.get_moves(ChessColor::Black)
            );

            let (from, to) = 'main: {
                let move_string = if raw_ms.ends_with('+') || raw_ms.ends_with('#') {
                    &raw_ms[0..raw_ms.len() - 1]
                } else {
                    raw_ms
                };

                // Castling
                if move_string == "O-O" || move_string == "O-O-O" {
                    let y = color_ternary!(turn, 7, 0);
                    let x = ternary!(move_string == "O-O", 6, 2);
                    break 'main (loc!(4, y), loc!(x, y));
                }

                // Pawn moves, ie "e4" or "d5"
                if move_string.len() == 2 {
                    let pos = Loc::from_notation(move_string);
                    let dir = color_ternary!(turn, 1, -1);

                    let mut i = 0;
                    loop {
                        if let Some(piece) = board.get(&pos.copy_move_i32(0, dir * i).0) {
                            if piece.color == turn && piece.name == PieceNames::Pawn {
                                break 'main (piece.pos, pos);
                            }
                        }
                        i += 1;
                    }
                }

                // Normal piece moves, ie "Nf3" or "Qe2"
                if move_string.len() == 3 {
                    let mut chars = move_string.chars();
                    let name = char_to_piece(&chars.next().unwrap());
                    let pos = Loc::from_notation(&chars.collect::<String>());

                    for mov in legal_moves.iter() {
                        if let Some(piece) = board.get(&mov.0) {
                            if piece.name == name && mov.1 == pos {
                                break 'main *mov;
                            }
                        }
                    }
                }

                // Takes notation, ie "exd5" or "Nxd5"
                if move_string.len() == 4 && move_string.chars().nth(1).unwrap() == 'x' {
                    if move_string.chars().next().unwrap().is_ascii_lowercase() {
                        let mut chars = move_string.chars();
                        let x = chars.next().unwrap() as u32 - 97;
                        let pos = loc!(x as usize, 0);
                        let mut i = 0;
                        let killer = loop {
                            if let Some(piece) = board.get(&pos.copy_move_i32(0, i).0) {
                                if piece.color == turn && piece.name == PieceNames::Pawn {
                                    break piece.pos;
                                }
                            }
                            i += 1;
                        };
                        chars.next();

                        let pos = Loc::from_notation(&chars.collect::<String>());
                        for mov in legal_moves.iter() {
                            if let Some(piece) = board.get(&mov.0) {
                                if piece.name == PieceNames::Pawn
                                    && piece.pos == killer
                                    && mov.1 == pos
                                {
                                    break 'main *mov;
                                }
                            }
                        }
                    } else {
                        let mut chars = move_string.chars();
                        let killer = char_to_piece(&chars.next().unwrap());
                        chars.next();

                        let pos = Loc::from_notation(&chars.collect::<String>());
                        for mov in legal_moves.iter() {
                            if let Some(piece) = board.get(&mov.0) {
                                if piece.name == killer && mov.1 == pos {
                                    break 'main *mov;
                                }
                            }
                        }
                    }
                }

                // Where 2 knights or rooks can move to the name place, ie "N3d2"
                if move_string.len() == 4 && move_string.starts_with('N')
                    || move_string.starts_with('R')
                {
                    let mut chars = move_string.chars();
                    let name = char_to_piece(&chars.next().unwrap());
                    let raw = chars.next().unwrap();
                    if raw.is_ascii_digit() {
                        let y = raw.to_digit(10).unwrap() as usize;
                        let pos = Loc::from_notation(&chars.collect::<String>());
                        for mov in legal_moves.iter() {
                            if let Some(piece) = board.get(&mov.0) {
                                if piece.name == name && mov.1 == pos && piece.pos.y == y {
                                    break 'main *mov;
                                }
                            }
                        }
                    } else {
                        let x = raw as usize - 97;
                        let pos = Loc::from_notation(&chars.collect::<String>());
                        for mov in legal_moves.iter() {
                            if let Some(piece) = board.get(&mov.0) {
                                if piece.name == name && mov.1 == pos && piece.pos.x == x {
                                    break 'main *mov;
                                }
                            }
                        }
                    }
                }

                panic!("Not implemented! {} {} {}", move_string, i, opening.name);
            };

            board.move_piece(&from, &to, true);
            new_moves.push((from, to));
        }

        new_openings.push(NewOpening {
            name: opening.name.clone(),
            moves: new_moves,
        });
    }

    write(
        "assets/openings.json",
        to_string(&new_openings).unwrap().as_bytes(),
    )
    .unwrap();
}
