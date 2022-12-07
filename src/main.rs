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

#![feature(future_join)]

use std::time::{SystemTime, UNIX_EPOCH};

use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use game::Game;
use macroquad::prelude::{next_frame, Conf};
use macroquad::rand::srand;
use macroquad::text::Font;
use macroquad::window::clear_background;

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

#[cfg(target_arch = "wasm32")]
async fn load_images() {
    use macroquad::text::load_ttf_font;

    use crate::assets::{load_audio, load_image};

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
}

#[cfg(not(target_arch = "wasm32"))]
async fn load_images() {
    use std::future::join;

    use macroquad::text::load_ttf_font_from_bytes;

    use crate::assets::{load_audio_from_bytes, load_image_from_bytes};

    match load_ttf_font_from_bytes(include_bytes!("../assets/fonts/DejaVuSansMono-Bold.ttf")) {
        Ok(font) => unsafe {
            FONT = Some(font);
        },
        Err(_) => panic!("Failed to load font"),
    };

    macro_rules! load {
        ($path: expr) => {
            load_image_from_bytes($path, include_bytes!(concat!("../", $path)))
        };
    }

    macro_rules! load_audio {
        ($path: expr) => {
            load_audio_from_bytes($path, include_bytes!(concat!("../", $path)))
        };
    }

    join!(
        load!("assets/pieces/black_pawn.png"),
        load!("assets/pieces/black_knight.png"),
        load!("assets/pieces/black_bishop.png"),
        load!("assets/pieces/black_rook.png"),
        load!("assets/pieces/black_queen.png"),
        load!("assets/pieces/black_king.png"),
        load!("assets/pieces/white_pawn.png"),
        load!("assets/pieces/white_knight.png"),
        load!("assets/pieces/white_bishop.png"),
        load!("assets/pieces/white_rook.png"),
        load!("assets/pieces/white_queen.png"),
        load!("assets/pieces/white_king.png"),
        load_audio!("assets/sounds/move.wav"),
        load_audio!("assets/sounds/capture.wav"),
    )
    .await;
}

#[macroquad::main(config)]
async fn main() {
    let start = SystemTime::now();
    let seed = start.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    srand(seed);

    load_images().await;

    let mut game = Game::new();
    loop {
        clear_background(COLOR_BACKGROUND);
        game.update();
        next_frame().await;
    }
}
