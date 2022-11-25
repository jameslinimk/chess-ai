#![allow(dead_code)]

use assets::load_image_owned;
use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use game::Game;
use macroquad::prelude::{next_frame, Conf};
use macroquad::window::clear_background;

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

#[macroquad::main(config)]
async fn main() {
    // Load chess pieces
    for color in &["black", "white"] {
        for piece in &["pawn", "knight", "bishop", "rook", "queen", "king"] {
            load_image_owned(format!("assets/{}_{}.png", color, piece)).await;
        }
    }

    let mut game = Game::new();
    loop {
        clear_background(COLOR_BACKGROUND);
        game.update();
        next_frame().await;
    }
}
