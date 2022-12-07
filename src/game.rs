use std::thread::spawn;

use crossbeam_channel::{unbounded, Receiver, Sender};
use derive_new::new;
use macroquad::audio::{play_sound, PlaySoundParams};
use macroquad::prelude::{
    info, is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton,
    TextParams, WHITE,
};
use macroquad::shapes::draw_rectangle;
use macroquad::text::measure_text;

use crate::agent::{Agent, AGENTS};
use crate::assets::get_audio;
use crate::board::{Board, BoardState};
use crate::conf::{
    COLOR_BACKGROUND, COLOR_WHITE, EXTRA_WIDTH, HEIGHT, MARGIN, SQUARE_SIZE, TEST_FEN, WASM, WIDTH,
};
use crate::pieces::piece::Piece;
use crate::util::{multiline_text_ex, touches, Button, Loc, Tween};
use crate::{get_font, loc, ternary};

#[derive(new)]
pub struct Game {
    #[new(value = "Board::from_fen(TEST_FEN)")]
    pub board: Board,

    #[new(value = "vec![]")]
    pub board_history: Vec<(Board, Option<(Loc, Loc)>)>,

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

    #[new(value = "None")]
    pub last_move: Option<(Loc, Loc)>,

    /// (loc of piece that is being tweened, tween)
    #[new(value = "None")]
    pub current_tween: Option<(Loc, Tween)>,

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
        if self.board.turn == self.board.player_color {
            self.board_history
                .push((self.board.clone(), self.last_move));
        }

        let capture = self.board.move_piece(from, to, true);
        self.selected = None;
        self.highlight = vec![];
        self.last_move = Some((*from, *to));
        self.current_tween = Some((*to, Tween::new(from.as_tuple(), to.as_tuple(), 20.0)));

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
        if is_key_pressed(KeyCode::E) {
            info!("self.board: {:#?}", self.board);
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
                self.highlight = vec![];
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
                "Agent: {:?}\nTurn: {:?}\nScore: {}\n\n{}Keybinds:\nR-Reset\nL-Last move",
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
                font_size: 25,
                font_scale: 1.0,
                color: COLOR_WHITE,
                font: get_font(),
                ..Default::default()
            },
        )
    }

    fn draw_end(&self) {
        let message = match self.board.state {
            BoardState::Checkmate(color) => ternary!(
                self.board.agent_color == color,
                "Congrats! You won!\nPress \"r\" to restart!",
                "Dang, you lost\nPress \"r\" to restart!"
            ),
            BoardState::Stalemate => "Game over, stalemate\nPress \"r\" to restart!",
            _ => panic!(),
        };

        let params = TextParams {
            font_size: 30,
            font_scale: 1.0,
            color: COLOR_BACKGROUND,
            font: get_font(),
            ..Default::default()
        };

        let dims = measure_text(
            message.lines().next().unwrap(),
            Some(params.font),
            params.font_size,
            params.font_scale,
        );

        draw_rectangle(
            0.0,
            (HEIGHT / 2) as f32 - dims.height / 2.0 - MARGIN / 2.0,
            WIDTH as f32,
            dims.height * 2.0 + MARGIN,
            WHITE,
        );

        multiline_text_ex(
            message,
            (WIDTH / 2) as f32 - dims.width / 2.0,
            (HEIGHT / 2) as f32 - dims.height / 2.0,
            params,
        );
    }

    pub fn update(&mut self) {
        self.update_keys();
        self.update_buttons();

        if self.board.turn == self.board.player_color {
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
                }
            }
        } else {
            let agent = self.agent;
            let board = self.board.clone();
            self.waiting_on_agent = true;
            if WASM {
                self.agent_channel.0.send(agent.get_move(&board)).unwrap();
            } else {
                let sender = self.agent_channel.0.clone();
                spawn(move || {
                    sender.send(agent.get_move(&board)).unwrap();
                });
            }
        }

        // Drawing
        self.board
            .draw(&self.highlight, &self.last_move, &mut self.current_tween);
        self.draw_ui();

        if self.board.is_over() {
            self.draw_end();
        }
    }
}
