use std::process::exit;

use derive_new::new;
use macroquad::prelude::{
    is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position, KeyCode,
    MouseButton, BLACK,
};
use macroquad::shapes::draw_rectangle;
use macroquad::text::{draw_text_ex, measure_text, TextParams};

use crate::agent::{Agent, AGENTS};
use crate::board::{Board, BoardState, ChessColor};
use crate::conf::{
    BUTTON_HEIGHT, COLOR_BUTTON, COLOR_BUTTON_HOVER, COLOR_BUTTON_PRESSED, MARGIN, SQUARE_SIZE,
    TEST_FEN,
};
use crate::pieces::piece::Piece;
use crate::util::{touches, Loc};
use crate::{get_font, loc};

#[derive(new)]
pub struct Button {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &'static str,
    #[new(value = "false")]
    hover: bool,
    #[new(value = "false")]
    pressed: bool,
}
impl Button {
    pub fn update(&mut self) -> bool {
        self.hover = touches(mouse_position(), (self.x, self.y, self.w, self.h));
        if self.hover {
            if is_mouse_button_down(MouseButton::Left) {
                self.pressed = true;
            } else if is_mouse_button_pressed(MouseButton::Left) {
                self.pressed = true;
                return true;
            } else {
                self.pressed = false;
            }
        } else {
            self.pressed = false;
        }

        false
    }

    // FIXME
    pub fn draw(&self) {
        let color = match (self.hover, self.pressed) {
            (true, true) => COLOR_BUTTON_PRESSED,
            (true, false) => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON,
        };

        draw_rectangle(self.x, self.y, self.w, self.h, color);

        let params = TextParams {
            font_size: 20,
            font_scale: 1.0,
            color: BLACK,
            font: get_font(),
            ..Default::default()
        };

        let width = measure_text(
            self.text,
            Some(params.font),
            params.font_size,
            params.font_scale,
        )
        .width;

        draw_text_ex(self.text, self.x + width / 2.0, self.y, params);
    }
}

pub enum EndState {
    Checkmate(ChessColor),
    Stalemate,
}

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

    #[new(value = "Agent::Random")]
    pub agent: Agent,

    #[new(value = "{
        let mut temp = vec![];

        let button_width = 8.0 * SQUARE_SIZE;

        for (i, (key, value)) in AGENTS.iter().enumerate() {
            temp.push((
                Button::new(
                    MARGIN,
                    button_width + MARGIN + BUTTON_HEIGHT * i as f32,
                    button_width,
                    BUTTON_HEIGHT,
                    key,
                ),
                *value,
            ));
        }

        temp
    }")]
    pub agent_buttons: Vec<(Button, Agent)>,
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
                    if touches(
                        mouse_position(),
                        (top_left.x as f32, top_left.y as f32, size, size),
                    ) {
                        return Some(loc!(x, y));
                    }
                }
            }
        }

        None
    }

    fn move_piece(&mut self, from: &Loc, to: &Loc) {
        self.move_history.push((*from, *to));
        self.board.move_piece(from, to, true);
        self.selected = None;
        self.highlight = vec![];
    }

    fn reset(&mut self) {
        *self = Game::new();
    }

    fn update_debug(&mut self) {
        if is_key_pressed(KeyCode::F) {
            println!();
            self.board.print();
        }
        if is_key_pressed(KeyCode::R) {
            self.reset();
        }
    }

    fn update_buttons(&mut self) {
        for (button, agent) in self.agent_buttons.iter_mut() {
            if button.update() {
                self.agent = *agent;
            }
            button.draw();
        }
    }

    pub fn update(&mut self) {
        self.update_debug();
        self.update_buttons();

        if self.board.state == BoardState::Normal {
            if self.board.player_turn() {
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
            } else {
                let m = self.agent.get_move(&self.board);
                if let Some(m) = m {
                    self.move_piece(&m.0, &m.1);
                } else {
                    println!("No moves left!");
                    exit(0);
                }
            }
        } else {
            println!("Game over {:?}", self.board.state);
            exit(0);
        }

        // Drawing
        self.board.draw(&self.highlight);
    }
}
