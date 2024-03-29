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
use colored::{Color, Colorize};
use conf::{COLOR_BACKGROUND, HEIGHT, WIDTH};
use game::Game;
use macroquad::prelude::{next_frame, Conf};
use macroquad::text::Font;
use macroquad::window::clear_background;

use crate::camera::camera;

pub(crate) mod agent;
pub(crate) mod agent_opens;
pub(crate) mod assets;
pub(crate) mod board;
pub(crate) mod board_eval;
pub(crate) mod board_extras;
pub(crate) mod camera;
pub(crate) mod conf;
pub(crate) mod game;
pub(crate) mod pieces;
pub(crate) mod util;

#[cfg(not(windows))]
fn config() -> Conf {
    Conf {
        window_title: "Chess AI".to_string(),
        window_width: WIDTH,
        window_height: HEIGHT,
        window_resizable: true,
        ..Default::default()
    }
}

#[cfg(windows)]
fn config() -> Conf {
    use std::io::Cursor;

    use image::io::Reader;
    use macroquad::miniquad::conf::Icon;

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
        window_resizable: true,
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
pub(crate) fn font() -> Font {
    unsafe { FONT.unwrap() }
}

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

#[cfg(not(target_family = "wasm"))]
const CONFIG_LINK: &str = "https://github.com/jameslinimk/chess-ai/raw/master/Cargo.toml";
#[cfg(not(target_family = "wasm"))]
const GITHUB_LINK: &str = "https://github.com/jameslinimk/chess-ai";

#[cfg(not(target_family = "wasm"))]
fn color_convert(color: macroquad::prelude::Color) -> Color {
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
        use std::thread::spawn;
        use std::time::{SystemTime, UNIX_EPOCH};

        use macroquad::prelude::GRAY;
        use macroquad::rand::srand;
        use reqwest::blocking::get;

        use crate::conf::{COLOR_BLACK, COLOR_WHITE};

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
        camera().update();
        next_frame().await;
    }
}
