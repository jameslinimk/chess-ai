#![allow(dead_code)]

use assets::load_image_owned;
use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use game::Game;
use macroquad::prelude::{next_frame, Conf};
use macroquad::text::{load_ttf_font, Font};
use macroquad::window::clear_background;

mod agent;
mod assets;
mod board;
mod board_extras;
mod conf;
mod game;
mod pieces;
mod util;

fn config() -> Conf {
    Conf {
        window_title: "Chess AI".to_owned(),
        window_width: WIDTH,
        window_height: HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

static mut FONT: Option<Font> = None;
pub fn get_font() -> Font {
    unsafe { FONT.unwrap() }
}

#[macroquad::main(config)]
async fn main() {
    // Load chess pieces
    for color in &["black", "white"] {
        for piece in &["pawn", "knight", "bishop", "rook", "queen", "king"] {
            load_image_owned(format!("assets/{}_{}.png", color, piece)).await;
        }
    }
    match load_ttf_font("assets/DejaVuSansMono-Bold.ttf").await {
        Ok(font) => unsafe {
            FONT = Some(font);
        },
        Err(_) => panic!("Failed to load font"),
    };

    let mut game = Game::new();
    loop {
        clear_background(COLOR_BACKGROUND);
        game.update();
        next_frame().await;
    }
}
