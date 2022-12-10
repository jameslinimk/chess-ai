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

#![feature(future_join)]

#[cfg(not(target_family = "wasm"))]
use std::thread::spawn;
#[cfg(not(target_family = "wasm"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_family = "wasm"))]
use colored::{Color, Colorize};
use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use game::Game;
#[cfg(not(target_family = "wasm"))]
use macroquad::miniquad::conf::Icon;
use macroquad::prelude::{next_frame, Conf};
#[cfg(not(target_family = "wasm"))]
use macroquad::prelude::{Color as MacroColor, GRAY};
#[cfg(not(target_family = "wasm"))]
use macroquad::rand::srand;
use macroquad::text::Font;
use macroquad::window::clear_background;
#[cfg(not(target_family = "wasm"))]
use reqwest::blocking::get;

#[cfg(not(target_family = "wasm"))]
use crate::conf::{COLOR_BLACK, COLOR_WHITE};

pub mod agent;
pub mod agent_opens;
pub mod assets;
pub mod board;
pub mod board_eval;
pub mod board_extras;
pub mod conf;
pub mod game;
pub mod pieces;
pub mod util;

#[cfg(not(windows))]
fn config() -> Conf {
    Conf {
        window_title: "Chess AI".to_string(),
        window_width: WIDTH,
        window_height: HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

#[cfg(windows)]
fn config() -> Conf {
    use std::io::Cursor;

    use image::io::Reader;

    macro_rules! image {
        ($path: expr) => {
            Reader::new(Cursor::new(include_bytes!($path)))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
                .to_rgba8()
                .to_vec()
                .try_into()
                .unwrap()
        };
    }

    Conf {
        window_title: "Chess AI".to_string(),
        window_width: WIDTH,
        window_height: HEIGHT,
        window_resizable: false,
        icon: Some(Icon {
            small: image!("../assets/icon-16.png"),
            medium: image!("../assets/icon-32.png"),
            big: image!("../assets/icon-64.png"),
        }),
        ..Default::default()
    }
}

/// Font used throughout GUI, stored as static for accessibility
static mut FONT: Option<Font> = None;

/// Safely get [FONT] in safe code
pub fn get_font() -> Font {
    unsafe { FONT.unwrap() }
}

#[cfg(target_family = "wasm")]
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

#[cfg(not(target_family = "wasm"))]
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
            load_image_from_bytes(
                concat!("assets/pieces/", $path),
                include_bytes!(concat!("../assets/pieces/", $path)),
            )
        };
    }

    macro_rules! load_audio {
        ($path: expr) => {
            load_audio_from_bytes(
                concat!("assets/sounds/", $path),
                include_bytes!(concat!("../assets/sounds/", $path)),
            )
        };
    }

    join!(
        load!("black_pawn.png"),
        load!("black_knight.png"),
        load!("black_bishop.png"),
        load!("black_rook.png"),
        load!("black_queen.png"),
        load!("black_king.png"),
        load!("white_pawn.png"),
        load!("white_knight.png"),
        load!("white_bishop.png"),
        load!("white_rook.png"),
        load!("white_queen.png"),
        load!("white_king.png"),
        load_audio!("move.wav"),
        load_audio!("capture.wav"),
    )
    .await;
}

pub const CONFIG_LINK: &str = "https://github.com/jameslinimk/chess-ai/raw/master/Cargo.toml";
pub const GITHUB_LINK: &str = "https://github.com/jameslinimk/chess-ai";

#[cfg(not(target_family = "wasm"))]
fn color_convert(color: MacroColor) -> Color {
    Color::TrueColor {
        r: (color.r * 255.0) as u8,
        g: (color.g * 255.0) as u8,
        b: (color.b * 255.0) as u8,
    }
}

#[macroquad::main(config)]
async fn main() {
    #[cfg(not(target_family = "wasm"))]
    {
        println!(
            "{}\n{}\n{}\n{}",
            "=====================================================".color(color_convert(GRAY)),
            "░█████╗░██╗░░██╗███████╗░██████╗░██████╗  ░█████╗░██╗\n██╔══██╗██║░░██║██╔════╝██╔════╝██╔════╝  ██╔══██╗██║\n██║░░╚═╝███████║█████╗░░╚█████╗░╚█████╗░  ███████║██║\n██║░░██╗██╔══██║██╔══╝░░░╚═══██╗░╚═══██╗  ██╔══██║██║\n╚█████╔╝██║░░██║███████╗██████╔╝██████╔╝  ██║░░██║██║\n░╚════╝░╚═╝░░╚═╝╚══════╝╚═════╝░╚═════╝░  ╚═╝░░╚═╝╚═╝"
                .color(color_convert(COLOR_WHITE)),
            "     █▄▄ █▄█   ░░█ ▄▀█ █▀▄▀█ █▀▀ █▀   █░░ █ █▄░█\n     █▄█ ░█░   █▄█ █▀█ █░▀░█ ██▄ ▄█   █▄▄ █ █░▀█"
                .color(color_convert(COLOR_BLACK)),
            "=====================================================".color(color_convert(GRAY))
        );

        spawn(|| {
            if let Ok(res) = get(CONFIG_LINK) {
                for line in res.text().unwrap().lines().map(|l| l.replace(' ', "")) {
                    if !line.starts_with("version") {
                        continue;
                    }

                    let version = line.split('=').nth(1).unwrap().replace('"', "");
                    if version == env!("CARGO_PKG_VERSION") {
                        println!("{}", "Up to date!".blue());
                        break;
                    }

                    println!(
                        "{} {}\n {} {}",
                        "Update available:".green(),
                        version.to_string().bright_green(),
                        "Download here:".truecolor(169, 169, 169),
                        GITHUB_LINK.truecolor(128, 128, 128)
                    );
                }
            }
        });

        let start = SystemTime::now();
        let seed = start.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        srand(seed);
    }

    load_images().await;

    let mut game = Game::new();
    loop {
        clear_background(COLOR_BACKGROUND);
        game.update();
        next_frame().await;
    }
}
