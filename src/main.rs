//! A fully rust chess engine + AI + GUI written in Rust and Macroquad (a graphics library)
//!
//! AI is a minimax search with alpha-beta pruning, move-ordering, and Tomasz Michniewski's simplified evaluation function
//!
//! - Release hosted at <https://chess.jamesalin.com>
//! - Source here <https://github.com/jameslinimk/chess-ai>
//!
//! # Building
//!
//! Clone and build using `cargo build`
//!
//! **Make sure you copy `/assets` from Github (above) and put it in base directory, else rust will panic!**

use std::time::{SystemTime, UNIX_EPOCH};

use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use game::Game;
use macroquad::prelude::{next_frame, Conf};
use macroquad::rand::srand;
use macroquad::text::{load_ttf_font, Font};
use macroquad::window::clear_background;

use crate::assets::{load_audio, load_image};

pub mod agent;
pub mod assets;
pub mod board;
pub mod board_eval;
pub mod board_extras;
pub mod conf;
pub mod game;
pub mod pieces;
pub mod util;

/// Macroquad config function
fn config() -> Conf {
    Conf {
        window_title: "Chess AI".to_owned(),
        window_width: WIDTH,
        window_height: HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

/// Font used throughout GUI, stored as static for accessibility
static mut FONT: Option<Font> = None;

/// Safely get [FONT] in safe code
pub fn get_font() -> Font {
    unsafe { FONT.unwrap() }
}

#[macroquad::main(config)]
async fn main() {
    let start = SystemTime::now();
    let seed = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;
    srand(seed);

    // Load chess pieces
    for color in ["black", "white"].iter() {
        for piece in ["pawn", "knight", "bishop", "rook", "queen", "king"].iter() {
            load_image(&format!("assets/pieces/{}_{}.png", color, piece)).await;
        }
    }
    match load_ttf_font("assets/fonts/DejaVuSansMono-Bold.ttf").await {
        Ok(font) => unsafe {
            FONT = Some(font);
        },
        Err(_) => panic!("Failed to load font"),
    };
    load_audio("assets/sounds/move.wav").await;
    load_audio("assets/sounds/capture.wav").await;

    let mut game = Game::new();
    loop {
        clear_background(COLOR_BACKGROUND);
        game.update();
        next_frame().await;
    }
}
