#![allow(dead_code)]

use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use game::Game;
use macroquad::prelude::{next_frame, Conf};
use macroquad::text::{load_ttf_font, Font};
use macroquad::window::clear_background;

use crate::assets::{load_audio, load_image};

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

#[test]
fn test() {
    let test1 = vec![
        Some("2ewdfsdfeqfvsdvqefwef".to_string()),
        Some("qwfwsgaegaerg".to_string()),
        Some("eabsefdnaenrsn".to_string()),
    ];

    let test2 = [
        Some("2ewdfsdfeqfvsdvqefwef".to_string()),
        Some("qwfwsgaegaerg".to_string()),
        Some("eabsefdnaenrsn".to_string()),
    ];

    let iters = 10000;

    let mut test_sum = 0;
    let mut test2_sum = 0;

    for _ in 0..=iters {
        let now = std::time::Instant::now();
        let _ = test1.clone();
        test_sum += now.elapsed().as_nanos();

        let now2 = std::time::Instant::now();
        let _ = test2.clone();
        test2_sum += now2.elapsed().as_nanos();
    }

    println!("clone vec took {}ns", test_sum / iters);
    println!("clone array took {}ns", test2_sum / iters);
}
