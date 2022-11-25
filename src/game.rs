use cli_clipboard::set_contents;
use derive_new::new;
use macroquad::prelude::{
    is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton,
};

use crate::board::Board;
use crate::conf::{TEST_FEN, MARGIN, SQUARE_SIZE};
use crate::loc;
use crate::pieces::piece::Piece;
use crate::util::Loc;

#[derive(new)]
pub struct Game {
    #[new(value = "Board::from_fen(TEST_FEN)")]
    pub board: Board,

    #[new(value = "vec![]")]
    pub move_history: Vec<(Loc, Loc)>,

    #[new(value = "None")]
    pub selected: Option<Piece>,

    #[new(value = "vec![]")]
    pub highlight: Vec<Loc>,
}
impl Game {
    fn get_clicked_square(&self) -> Option<Loc> {
        if is_mouse_button_pressed(MouseButton::Left) {
            for (y, row) in self.board.raw.iter().enumerate() {
                for (x, _) in row.iter().enumerate() {
                    let top_left = loc!(
                        MARGIN as usize + SQUARE_SIZE as usize * x,
                        MARGIN as usize + SQUARE_SIZE as usize * y
                    );
                    let size = SQUARE_SIZE;

                    // See if mouse intersects with rectangle
                    let mouse_pos = mouse_position();
                    if mouse_pos.0 > top_left.x as f32
                        && mouse_pos.0 < top_left.x as f32 + size
                        && mouse_pos.1 > top_left.y as f32
                        && mouse_pos.1 < top_left.y as f32 + size
                    {
                        return Some(loc!(x, y));
                    }
                }
            }
        }

        None
    }

    fn move_piece(&mut self, from: &Loc, to: &Loc) {
        self.move_history.push((*from, *to));
        self.board.move_piece(from, to);
        self.selected = None;
        self.highlight = vec![];
    }

    fn reset(&mut self) {
        *self = Game::new();
    }

    pub fn update(&mut self) {
        // Debug
        if is_key_pressed(KeyCode::F) {
            println!();
            self.board.print();
        }
        if is_key_pressed(KeyCode::E) {
            let fen = self.board.as_fen();
            match set_contents(fen) {
                Ok(_) => {}
                Err(_) => println!("Error while copying FEN!"),
            };
        }
        if is_key_pressed(KeyCode::R) {
            self.reset();
        }

        // if self.board.player_turn() {
        if let Some(clicked) = self.get_clicked_square() {
            // Click same place
            if self.selected.is_some() && self.selected.unwrap().pos == clicked {
                self.selected = None;
                self.highlight = vec![];
            // Move (Clicked highlighted piece)
            } else if self.highlight.contains(&clicked) {
                self.move_piece(&self.selected.unwrap().pos, &clicked);
                // Clicked a new place
            } else if let Some(piece) = self.board.get(&clicked) {
                self.selected = Some(piece);
                self.highlight = self.selected.unwrap().get_moves(&self.board);
            }
        }
        // }

        // Drawing
        self.board.draw(&self.highlight);
    }
}
