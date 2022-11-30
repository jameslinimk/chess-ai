use std::process::exit;
use std::thread::spawn;

use crossbeam_channel::{unbounded, Receiver, Sender};
use derive_new::new;
use macroquad::prelude::{
    is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton, TextParams,
};

use crate::agent::{Agent, AGENTS};
use crate::board::{Board, ChessColor};
use crate::conf::{COLOR_WHITE, EXTRA_WIDTH, HEIGHT, MARGIN, SQUARE_SIZE, TEST_FEN, WASM};
use crate::pieces::piece::Piece;
use crate::util::{multiline_text_ex, touches, Button, Loc};
use crate::{get_font, loc};

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

    #[new(value = "Agent::Minimax")]
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

    #[new(value = "false")]
    pub waiting_on_agent: bool,

    #[new(value = "unbounded()")]
    #[allow(clippy::type_complexity)]
    pub agent_channel: (Sender<Option<(Loc, Loc)>>, Receiver<Option<(Loc, Loc)>>),
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
        if is_key_pressed(KeyCode::T) {
            println!();
            println!("{}", self.board.as_fen());
        }
        if is_key_pressed(KeyCode::R) {
            if self.waiting_on_agent {
                println!("Waiting on agent...");
            } else {
                self.reset();
            }
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
            &format!("Turn: {:?}\nWScore: {}", self.board.turn, self.board.score),
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

        if self.board.is_over() {
            println!("Game over: {:?}", self.board.state);
            exit(0);
        }

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
                    if piece.color == self.board.turn {
                        self.selected = Some(piece);
                        self.highlight = self.selected.unwrap().get_moves(&self.board);
                    }
                }
            }
        } else if self.waiting_on_agent {
            if let Ok(mov) = self.agent_channel.1.try_recv() {
                self.waiting_on_agent = false;
                if let Some(m) = mov {
                    self.move_piece(&m.0, &m.1);
                } else {
                    println!("No moves left!");
                    exit(0);
                }
            }
        } else {
            let agent = self.agent;
            let board = self.board.clone();
            let sender = self.agent_channel.0.clone();
            self.waiting_on_agent = true;
            if WASM {
                println!("1");
                sender.send(agent.get_move(&board)).unwrap();
            } else {
                println!("2");
                spawn(move || {
                    sender.send(agent.get_move(&board)).unwrap();
                });
            }
        }

        // Drawing
        self.board.draw(&self.highlight);
        self.draw_ui();
    }
}
