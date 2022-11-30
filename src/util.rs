use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result};

use macroquad::prelude::{
    is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton,
};
use macroquad::rand::gen_range;
use macroquad::shapes::draw_rectangle;
use macroquad::text::{draw_text_ex, measure_text, TextDimensions, TextParams};

use crate::conf::{COLOR_BUTTON, COLOR_BUTTON_HOVER, COLOR_BUTTON_PRESSED, COLOR_WHITE};
use crate::get_font;

pub fn validate_fen(fen: &str) -> bool {
    let rows = fen.split('/');
    let mut rows_len = 0;

    for row in rows {
        rows_len += 1;

        let mut sum = 0;
        for c in row.chars() {
            if c.is_ascii_digit() {
                sum += c.to_digit(10).unwrap();
            } else {
                sum += 1;
            }
        }

        if sum != 8 {
            return false;
        }
    }

    if rows_len != 8 {
        return false;
    }
    true
}

#[macro_export]
macro_rules! loc {
    ($x: expr, $y: expr) => {
        Loc { x: $x, y: $y }
    };
}

macro_rules! clamp_negative {
    ($number: expr) => {
        if $number.is_negative() {
            0
        } else {
            $number
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
}
impl Debug for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
impl Loc {
    pub fn copy_move_i32(&self, x_diff: i32, y_diff: i32) -> Loc {
        loc!(
            clamp_negative!(self.x as i32 + x_diff) as usize,
            clamp_negative!(self.y as i32 + y_diff) as usize
        )
    }

    pub fn move_i32(&mut self, x_diff: i32, y_diff: i32) -> bool {
        let new_x = self.x as i32 + x_diff;
        let new_y = self.y as i32 + y_diff;

        if new_x < 0 || new_y < 0 {
            self.x = 0;
            self.y = 0;
            return false;
        }

        self.x = new_x as usize;
        self.y = new_y as usize;
        true
    }

    pub fn as_notation(&self) -> String {
        let x = match self.x {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => panic!(),
        };
        let y = match self.y {
            0 => '8',
            1 => '7',
            2 => '6',
            3 => '5',
            4 => '4',
            5 => '3',
            6 => '2',
            7 => '1',
            _ => panic!(),
        };
        format!("{}{}", x, y)
    }
}

pub fn touches(point: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    point.0 >= rect.0
        && point.0 <= rect.0 + rect.2
        && point.1 >= rect.1
        && point.1 <= rect.1 + rect.3
}

pub fn multiline_text_ex(text: &str, x: f32, y: f32, params: TextParams) {
    let height = measure_text(text, Some(params.font), params.font_size, params.font_scale).height;
    for (i, line) in text.lines().enumerate() {
        draw_text_ex(line, x, y + height * (i as f32 + 1.0), params);
    }
}

pub fn debug_print(highlights: &HashSet<Loc>) {
    for y in 0..8 {
        for x in 0..8 {
            if highlights.contains(&loc!(x, y)) {
                print!("x");
            } else {
                print!("-");
            }
        }
        println!();
    }
}

pub fn choose_array<T>(arr: &[T]) -> &T {
    let index = gen_range(0, arr.len());
    &arr[index]
}

#[macro_export]
macro_rules! color_ternary {
    ($cond: expr, $if_white: expr, $if_black: expr) => {
        if $cond == ChessColor::White {
            $if_white
        } else {
            $if_black
        }
    };
}

pub struct Button {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &'static str,
    hover: bool,
    pressed: bool,
    dims: TextDimensions,
    params: TextParams,
}
impl Button {
    pub fn new(x: f32, y: f32, w: f32, h: f32, text: &'static str) -> Button {
        let params = TextParams {
            font_size: 30,
            font_scale: 1.0,
            color: COLOR_WHITE,
            font: get_font(),
            ..Default::default()
        };
        Button {
            x,
            y,
            w,
            h,
            text,
            hover: false,
            pressed: false,
            params,
            dims: measure_text(text, Some(params.font), params.font_size, params.font_scale),
        }
    }

    // FIXME clicking doesn't work
    pub fn update(&mut self) -> bool {
        self.hover = touches(mouse_position(), (self.x, self.y, self.w, self.h));
        if self.hover {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.pressed = true;
                return true;
            } else if is_mouse_button_down(MouseButton::Left) {
                self.pressed = true;
            } else {
                self.pressed = false;
            }
        } else {
            self.pressed = false;
        }

        false
    }

    pub fn draw(&self) {
        let color = match (self.hover, self.pressed) {
            (true, true) => COLOR_BUTTON_PRESSED,
            (true, false) => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON,
        };

        draw_rectangle(self.x, self.y, self.w, self.h, color);

        // Draw centered text
        draw_text_ex(
            self.text,
            self.x + self.w / 2.0 - self.dims.width / 2.0,
            self.y + self.h / 2.0 + self.dims.height / 2.0,
            self.params,
        );
    }
}
