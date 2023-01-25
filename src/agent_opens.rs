//! Parse and store ECO opening database

use std::str::from_utf8;

use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use serde_json::from_str;

use crate::util::Loc;

type Openings = FxHashMap<u64, Vec<((Loc, Loc), String)>>;

#[cfg(target_pointer_width = "64")]
lazy_static! {
    pub(crate) static ref OPENINGS: Openings =
        from_str(from_utf8(include_bytes!("../assets/openings_64.json")).unwrap()).unwrap();
}

#[cfg(target_pointer_width = "32")]
lazy_static! {
    pub(crate) static ref OPENINGS: Openings =
        from_str(from_utf8(include_bytes!("../assets/openings_32.json")).unwrap()).unwrap();
}

#[test]
fn create_openings() {
    use serde::{Deserialize, Serialize};
    use serde_json::to_string;

    use crate::board::{Board, ChessColor};
    use crate::board_extras::char_to_piece;
    use crate::conf::FEN;
    use crate::pieces::piece::PieceNames;
    use crate::{color_ternary, hashmap, loc, ternary};

    #[derive(Serialize, Deserialize, Debug)]
    struct RawOpening {
        name: String,
        code: String,
        moves: Vec<String>,
    }

    let mut new_openings: Openings = hashmap! {};

    let openings = from_str::<Vec<RawOpening>>(
        from_utf8(include_bytes!("../openings/openings.json")).unwrap(),
    )
    .unwrap();

    for opening in openings.iter() {
        let mut board = Board::from_fen(FEN);

        for (i, raw_ms) in opening.moves.iter().enumerate() {
            let turn = ternary!(i % 2 == 0, ChessColor::White, ChessColor::Black);

            let legal_moves = color_ternary!(
                board.turn,
                board.moves(ChessColor::White),
                board.moves(ChessColor::Black)
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
                                if piece.name == name && mov.1 == pos && piece.pos.1 == y {
                                    break 'main *mov;
                                }
                            }
                        }
                    } else {
                        let x = raw as usize - 97;
                        let pos = Loc::from_notation(&chars.collect::<String>());
                        for mov in legal_moves.iter() {
                            if let Some(piece) = board.get(&mov.0) {
                                if piece.name == name && mov.1 == pos && piece.pos.0 == x {
                                    break 'main *mov;
                                }
                            }
                        }
                    }
                }

                panic!("Not implemented! {} {} {}", move_string, i, opening.name);
            };

            if let Some(vec) = new_openings.get_mut(&board.hash) {
                vec.push(((from, to), opening.name.to_owned()));
            } else {
                new_openings.insert(board.hash, vec![((from, to), opening.name.to_owned())]);
            }

            board.move_piece(&from, &to, true);
        }
    }

    #[cfg(target_pointer_width = "64")]
    {
        let path = "assets/openings_64.json";
        std::fs::write(path, to_string(&new_openings).unwrap().as_bytes()).unwrap();
    }
    #[cfg(target_pointer_width = "32")]
    {
        macroquad::prelude::info!("{}", to_string(&new_openings).unwrap());
    }
}
