use derive_new::new;

use crate::pieces::piece::Piece;

#[derive(new)]
pub struct Board {
    /// Array with the raw 8x8 board data
    pub raw: [[Option<Piece>; 8]; 8],

    /// True if black can castle
    #[new(value = "true")]
    pub castle_black: bool,
    /// True if white can castle
    #[new(value = "true")]
    pub castle_white: bool,
}
