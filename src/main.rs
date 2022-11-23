#![allow(dead_code, unused_imports)]

use assets::load_image_owned;
use board::Board;
use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use macroquad::prelude::{next_frame, Conf};
use macroquad::window::clear_background;

mod assets;
mod board;
mod conf;
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

    let board = Board::new();
    loop {
        clear_background(COLOR_BACKGROUND);
        board.draw();
        next_frame().await;
    }
}
