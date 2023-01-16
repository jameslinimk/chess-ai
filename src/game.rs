#[cfg(not(target_family = "wasm"))]
use std::thread::spawn;

use crossbeam_channel::{unbounded, Receiver, Sender};
use derive_new::new;
use macroquad::audio::{play_sound, PlaySoundParams};
use macroquad::prelude::{
    info, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, KeyCode, MouseButton,
    TextParams, WHITE,
};
use macroquad::shapes::draw_rectangle;
use macroquad::text::measure_text;
use rustc_hash::FxHashSet;

use crate::agent::{Agent, AGENTS};
use crate::assets::get_audio;
use crate::board::Board;
use crate::camera::Camera;
use crate::conf::{
    CENTER_HEIGHT, CENTER_WIDTH, COLOR_BACKGROUND, COLOR_WHITE, EXTRA_WIDTH, FEN, HEIGHT, MARGIN,
    SQUARE_SIZE,
};
use crate::pieces::piece::Piece;
use crate::util::{multiline_text_ex, pos_to_board, Button, Loc, Tween};
use crate::{get_font, hashset, ternary};

#[derive(new)]
pub struct Game {
    #[new(value = "Board::from_fen(FEN)")]
    pub board: Board,

    #[new(value = "vec![]")]
    pub board_history: Vec<(Board, Option<(Loc, Loc)>)>,

    #[new(value = "None")]
    pub selected: Option<Piece>,

    #[new(value = "vec![]")]
    pub highlight_moves: Vec<Loc>,

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

    #[new(value = "None")]
    pub last_move: Option<(Loc, Loc)>,

    /// (loc of piece that is being tweened, tween)
    #[new(value = "None")]
    pub current_tween: Option<(Loc, Tween)>,

    #[new(value = "vec![]")]
    pub arrows: Vec<(Loc, Loc)>,

    #[new(value = "None")]
    pub drag_start: Option<Loc>,

    #[new(value = "None")]
    pub drag_end: Option<Loc>,

    #[new(value = "hashset!{}")]
    pub highlights: FxHashSet<Loc>,

    #[allow(clippy::type_complexity)]
    #[new(value = "unbounded()")]
    pub agent_channel: (Sender<Option<(Loc, Loc)>>, Receiver<Option<(Loc, Loc)>>),

    #[new(value = "Camera::new()")]
    pub camera: Camera,
}
impl Game {
    fn get_clicked_square(&self, button: MouseButton) -> Option<Loc> {
        if is_mouse_button_pressed(button) {
            return pos_to_board(self.camera.mouse_position().into());
        }

        None
    }

    fn move_piece(&mut self, from: &Loc, to: &Loc) {
        if self.board.turn == self.board.player_color {
            self.board_history
                .push((self.board.clone(), self.last_move));
        }

        let capture = self.board.move_piece(from, to, true);
        self.selected = None;
        self.highlight_moves.clear();
        self.highlights.clear();
        self.arrows.clear();
        self.last_move = Some((*from, *to));
        self.current_tween = Some((*to, Tween::new(from.as_f32(), to.as_f32(), 20.0)));

        // See if move was capture
        if capture {
            play_sound(
                get_audio("assets/sounds/capture.wav"),
                PlaySoundParams::default(),
            );
        } else {
            play_sound(
                get_audio("assets/sounds/move.wav"),
                PlaySoundParams::default(),
            );
        }
    }

    fn reset(&mut self) {
        *self = Game::new();
    }

    fn update_keys(&mut self) {
        if is_key_pressed(KeyCode::F) {
            self.board.print();
        }
        if is_key_pressed(KeyCode::T) {
            info!("{}", self.board.as_fen());
        }
        if is_key_pressed(KeyCode::R) {
            if self.waiting_on_agent {
                info!("Waiting on agent...");
            } else {
                self.reset();
            }
        }
        if is_key_pressed(KeyCode::L) {
            if self.waiting_on_agent {
                info!("Waiting on agent...");
            } else if let Some((board, last_move)) = self.board_history.pop() {
                self.board = board;
                self.selected = None;
                self.last_move = last_move;
                self.highlight_moves.clear();

                self.clear_arrows_highlights();
            }
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

    fn draw_ui(&self) {
        multiline_text_ex(
            &format!(
                "Agent: {:?}\nTurn: {:?}\nScore: {}\n\n{}Keybinds:\nR-Reset\nL-Takeback",
                self.agent,
                self.board.turn,
                self.board.score,
                ternary!(
                    self.board.turn == self.board.agent_color,
                    "Computer is\nthinking...\n\n",
                    ""
                )
            ),
            SQUARE_SIZE * 8.0 + MARGIN * 2.0,
            MARGIN,
            TextParams {
                font_size: 15,
                font_scale: 1.0,
                color: COLOR_WHITE,
                font: get_font(),
                ..Default::default()
            },
        )
    }

    fn draw_end(&self) {
        let message = self.board.state.message(self.board.player_color);

        let params = TextParams {
            font_size: 30,
            font_scale: 1.0,
            color: COLOR_BACKGROUND,
            font: get_font(),
            ..Default::default()
        };

        let mut width = 0.0;
        let mut height = 0.0;
        for line in message.lines() {
            let dims = measure_text(line, Some(params.font), params.font_size, params.font_scale);
            width = dims.width.max(width);
            height += dims.height;
        }

        draw_rectangle(
            CENTER_WIDTH as f32 - width / 2.0 - MARGIN / 2.0,
            CENTER_HEIGHT as f32 - height / 2.0 - MARGIN / 4.0,
            width + MARGIN,
            height + MARGIN,
            WHITE,
        );

        multiline_text_ex(
            message,
            (CENTER_WIDTH) as f32 - width / 2.0,
            (CENTER_HEIGHT) as f32 - height / 2.0,
            params,
        );
    }

    fn clear_arrows_highlights(&mut self) {
        self.highlights.clear();
        self.arrows.clear();
        self.drag_end = None;
        self.drag_start = None;
    }

    pub fn update_arrows_highlights(&mut self) {
        if is_mouse_button_down(MouseButton::Left) {
            self.clear_arrows_highlights();
        }

        if is_mouse_button_down(MouseButton::Right) {
            if self.drag_start.is_none() {
                self.drag_start = pos_to_board(self.camera.mouse_position().into());
                return;
            }

            let pos = pos_to_board(self.camera.mouse_position().into());
            if self.drag_start != pos {
                self.drag_end = pos;
            }
        } else {
            if let (Some(start), Some(end)) = (self.drag_start, self.drag_end) {
                let index = self.arrows.iter().position(|&arrow| arrow == (start, end));
                if let Some(i) = index {
                    self.arrows.remove(i);
                } else {
                    self.arrows.push((start, end));
                }
            }

            if let Some(start) = self.drag_start {
                if self.drag_end.is_none() {
                    if self.highlights.contains(&start) {
                        self.highlights.remove(&start);
                    } else {
                        self.highlights.insert(start);
                    }
                }
            }

            self.drag_start = None;
            self.drag_end = None;
        }
    }

    pub fn update(&mut self) {
        self.camera.update();
        self.update_keys();
        self.update_buttons();
        self.update_arrows_highlights();

        if self.agent == Agent::Control || self.board.turn == self.board.player_color {
            if let Some(clicked) = self.get_clicked_square(MouseButton::Left) {
                // Click same place
                if self.selected.is_some() && self.selected.unwrap().pos == clicked {
                    self.selected = None;
                    self.highlight_moves.clear();
                // Move (Clicked highlighted piece)
                } else if self.highlight_moves.contains(&clicked) {
                    self.move_piece(&self.selected.unwrap().pos, &clicked);
                    // Clicked a new place
                } else if let Some(piece) = self.board.get(&clicked) {
                    if piece.color == self.board.turn {
                        self.selected = Some(piece);
                        self.highlight_moves = self.selected.unwrap().get_moves(&self.board);
                    }
                }
            }
        } else if self.waiting_on_agent {
            if let Ok(mov) = self.agent_channel.1.try_recv() {
                self.waiting_on_agent = false;
                if let Some(m) = mov {
                    self.move_piece(&m.0, &m.1);
                }
            }
        } else {
            let agent = self.agent;
            let board = self.board.clone();
            self.waiting_on_agent = true;
            #[cfg(target_family = "wasm")]
            {
                self.agent_channel.0.send(agent.get_move(&board)).unwrap();
            }
            #[cfg(not(target_family = "wasm"))]
            {
                let sender = self.agent_channel.0.clone();
                spawn(move || {
                    sender.send(agent.get_move(&board)).unwrap();
                });
            }
        }

        // Drawing
        self.board.draw(
            &self.highlight_moves,
            &self.last_move,
            &self.highlights,
            &self.arrows,
            &mut self.current_tween,
        );
        self.draw_ui();

        if self.board.is_over() {
            self.draw_end();
        }
    }
}
