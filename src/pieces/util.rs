use crate::board::Board;
use crate::util::Loc;

pub fn validate(location: &Loc) -> bool {
    !(location.x > 8 || location.y > 8)
}

/// Adds to moves if the move is on the board
pub fn add(location: Loc, moves: &mut Vec<Loc>) {
    if validate(&location) {
        moves.push(location);
    }
}

/// Adds to moves if the move is a capture
pub fn add_if_capture(board: &Board, location: Loc, moves: &mut Vec<Loc>) {
    if validate(&location) && board.raw[location.x][location.y].is_some() {
        moves.push(location);
    }
}
