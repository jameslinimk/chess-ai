use std::process::exit;

use derive_new::new;
use macroquad::prelude::{
    draw_rectangle, draw_text_ex, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
    measure_text, mouse_position, KeyCode, MouseButton, TextDimensions, TextParams,
};

use crate::agent::{Agent, AGENTS};
use crate::board::{Board, BoardState, ChessColor};
use crate::conf::{
    COLOR_BUTTON, COLOR_BUTTON_HOVER, COLOR_BUTTON_PRESSED, COLOR_WHITE, EXTRA_WIDTH, HEIGHT,
    MARGIN, SQUARE_SIZE, TEST_FEN,
};
use crate::pieces::piece::Piece;
use crate::util::{multiline_text_ex, touches, Loc};
use crate::{get_font, loc};

pub struct Button {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &'static str,
    hover: bool,
    pressed: bool,
    dims: TextDimensions,
    params: TextParams,
}
impl Button {
    pub fn new(x: f32, y: f32, w: f32, h: f32, text: &'static str) -> Button {
        let params = TextParams {
            font_size: 30,
            font_scale: 1.0,
            color: COLOR_WHITE,
            font: get_font(),
            ..Default::default()
        };
        Button {
            x,
            y,
            w,
            h,
            text,
            hover: false,
            pressed: false,
            params,
            dims: measure_text(text, Some(params.font), params.font_size, params.font_scale),
        }
    }

    // FIXME clicking doesn't work
    pub fn update(&mut self) -> bool {
        self.hover = touches(mouse_position(), (self.x, self.y, self.w, self.h));
        if self.hover {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.pressed = true;
                return true;
            } else if is_mouse_button_down(MouseButton::Left) {
                self.pressed = true;
            } else {
                self.pressed = false;
            }
        } else {
            self.pressed = false;
        }

        false
    }

    pub fn draw(&self) {
        let color = match (self.hover, self.pressed) {
            (true, true) => COLOR_BUTTON_PRESSED,
            (true, false) => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON,
        };

        draw_rectangle(self.x, self.y, self.w, self.h, color);

        // Draw centered text
        draw_text_ex(
            self.text,
            self.x + self.w / 2.0 - self.dims.width / 2.0,
            self.y + self.h / 2.0 + self.dims.height / 2.0,
            self.params,
        );
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

        for (i, (key, value)) in AGENTS.iter().enumerate() {
            temp.push((
                Button::new(
                    SQUARE_SIZE * 8.0 + MARGIN * 2.0,
                    HEIGHT as f32 - (50.0 + MARGIN) * (i as f32 + 1.0),
                    EXTRA_WIDTH,
                    50.0,
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
        if is_key_pressed(KeyCode::E) {
            println!();
            println!("self.board: {:#?}", self.board);
        }
        if is_key_pressed(KeyCode::R) {
            self.reset();
        }
    }

    fn update_buttons(&mut self) {
        for (button, agent) in self.agent_buttons.iter_mut() {
            if button.update() {
                self.agent = *agent;
                println!("Agent set to {:?}", agent);
            }
            button.draw();
        }
    }

    fn draw_ui(&self) {
        multiline_text_ex(
            &format!(
                "Turn: {:?}\nWScore: {}\nBScore: {}",
                self.board.turn, self.board.score_white, self.board.score_black
            ),
            SQUARE_SIZE * 8.0 + MARGIN * 2.0,
            MARGIN,
            TextParams {
                font_size: 30,
                font_scale: 1.0,
                color: COLOR_WHITE,
                font: get_font(),
                ..Default::default()
            },
        )
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
        self.draw_ui();
    }
}
