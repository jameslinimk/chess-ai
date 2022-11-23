use crate::board::Board;
use crate::util::Loc;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub fn get_moves(board: Board, location: Loc) {
    match board.raw[location.x][location.y] {
        Some((Piece::Pawn, _)) => {
            println!("Pawn");
        }
        Some((Piece::Knight, _)) => {
            println!("Knight");
        }
        Some((Piece::Bishop, _)) => {
            println!("Bishop");
        }
        Some((Piece::Rook, _)) => {
            println!("Rook");
        }
        Some((Piece::Queen, _)) => {
            println!("Queen");
        }
        Some((Piece::King, _)) => {
            println!("King");
        }
        None => {
            println!("None");
        }
    }
}
