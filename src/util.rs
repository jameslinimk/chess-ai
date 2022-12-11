use std::f32::consts::PI;

use derive_new::new;
use macroquad::prelude::{
    is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton,
};
use macroquad::rand::gen_range;
use macroquad::shapes::draw_rectangle;
use macroquad::text::{draw_text_ex, measure_text, TextDimensions, TextParams};
use macroquad::time::get_frame_time;
use serde::{Deserialize, Serialize};

use crate::conf::{
    COLOR_BUTTON, COLOR_BUTTON_HOVER, COLOR_BUTTON_PRESSED, COLOR_WHITE, MARGIN, SQUARE_SIZE,
};
use crate::get_font;

/// Makes sure the board part of fen is valid, doesn't check if there are 5 kings, 500 pawns, etc
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

/// Shorthand for creating a `Loc`
#[macro_export]
macro_rules! loc {
    ($x: expr, $y: expr) => {
        $crate::util::Loc($x, $y)
    };
}

/// Create [std::collections::HashMap]'s using a readable syntax, similar to dicts in python or objects in js. Adapted from maplit to support `FxHashMap`
#[macro_export]
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { $crate::hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = $crate::hashmap!(@count $($key),*);
            let mut _map = rustc_hash::FxHashMap::with_capacity_and_hasher(_cap, Default::default());
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

/// Create [std::collections::HashSet]'s using a readable syntax. Adapted from maplit to support `FxHashSet`
#[macro_export]
macro_rules! hashset {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashset!(@single $rest)),*]));

    ($($key:expr,)+) => { $crate::hashset!($($key),+) };
    ($($key:expr),*) => {
        {
            let _cap = $crate::hashset!(@count $($key),*);
            let mut _set = rustc_hash::FxHashSet::with_capacity_and_hasher(_cap, Default::default());
            $(
                let _ = _set.insert($key);
            )*
            _set
        }
    };
}

/// A Vec2 with usize values and utility functions for chess board stuff
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Loc(pub usize, pub usize);
impl Loc {
    /// Create a new `Loc`, shifted `x_diff` and `y_diff` away from the current pos
    pub fn copy_move_i32(&self, x_diff: i32, y_diff: i32) -> (Loc, bool) {
        let mut new_x = self.0 as i32 + x_diff;
        let mut new_y = self.1 as i32 + y_diff;

        if new_x < 0 || new_y < 0 {
            new_x = new_x.clamp(0, i32::MAX);
            new_y = new_y.clamp(0, i32::MAX);
            return (loc!(new_x as usize, new_y as usize), true);
        }

        (loc!(new_x as usize, new_y as usize), false)
    }

    /// Move the current pos by `x_diff` and `y_diff`
    pub fn move_i32(&mut self, x_diff: i32, y_diff: i32) -> bool {
        let new_x = self.0 as i32 + x_diff;
        let new_y = self.1 as i32 + y_diff;

        if new_x < 0 || new_y < 0 {
            self.0 = new_x.clamp(0, i32::MAX) as usize;
            self.1 = new_y.clamp(0, i32::MAX) as usize;
            return false;
        }

        self.0 = new_x as usize;
        self.1 = new_y as usize;
        true
    }

    /// Get the location as chess notation IE (`(0, 0)` becomes `"A8"`)
    pub fn as_notation(&self) -> String {
        let x = char::from_u32(self.0 as u32 + 97).unwrap();
        let y = match self.1 {
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

    /// Creates a `Loc` from a chess notation string IE (`"A8"` becomes `(0, 0)`)
    pub fn from_notation(notation: &str) -> Loc {
        let mut chars = notation.chars();
        let x = chars.next().unwrap() as u32 - 97;
        let y = match chars.next().unwrap() {
            '8' => 0,
            '7' => 1,
            '6' => 2,
            '5' => 3,
            '4' => 4,
            '3' => 5,
            '2' => 6,
            '1' => 7,
            _ => panic!(),
        };
        loc!(x as usize, y)
    }

    /// Convert the `Loc` to a `(f32, f32)`
    pub fn as_f32(&self) -> (f32, f32) {
        (self.0 as f32, self.1 as f32)
    }
}

/// Sees if a rectangle contains a point
pub fn touches(point: (f32, f32), rect: (f32, f32, f32, f32)) -> bool {
    point.0 >= rect.0
        && point.0 <= rect.0 + rect.2
        && point.1 >= rect.1
        && point.1 <= rect.1 + rect.3
}

/// Write multiple lines of text that are automatically spaced
pub fn multiline_text_ex(text: &str, x: f32, y: f32, params: TextParams) {
    let height = measure_text(text, Some(params.font), params.font_size, params.font_scale).height;
    for (i, line) in text.lines().enumerate() {
        draw_text_ex(line, x, y + height * (i as f32 + 1.0), params);
    }
}

/// Get a random element from an array
pub fn choose_array<T>(arr: &[T]) -> &T {
    let index = gen_range(0, arr.len());
    &arr[index]
}

/// If `cond` is `ChessColor::White`, then do `if_white`, else `if_black`
#[macro_export]
macro_rules! color_ternary {
    ($cond: expr, $if_white: expr, $if_black: expr) => {
        if $cond == $crate::board::ChessColor::White {
            $if_white
        } else {
            $if_black
        }
    };
}

/// Shorthand for `if $cond { $true } else { $false }` or the ternary operator in C style languages
#[macro_export]
macro_rules! ternary {
    ($cond: expr, $true: expr, $false: expr) => {
        if $cond {
            $true
        } else {
            $false
        }
    };
}

/// Convert a position on the screen to a board location
pub fn pos_to_board(pos: (f32, f32)) -> Option<Loc> {
    let x = (pos.0 - MARGIN) / SQUARE_SIZE;
    let y = (pos.1 - MARGIN) / SQUARE_SIZE;

    if x < 0.0 || y < 0.0 || x > 8.0 || y > 8.0 {
        return None;
    }

    Some(loc!(x as usize, y as usize))
}

/// Converts a board location to a position on the screen
pub fn board_to_pos_center(loc: &Loc) -> (f32, f32) {
    (
        loc.0 as f32 * SQUARE_SIZE + MARGIN + SQUARE_SIZE / 2.0,
        loc.1 as f32 * SQUARE_SIZE + MARGIN + SQUARE_SIZE / 2.0,
    )
}

/// Creates a button that can be clicked
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

// Gets the angle between two points
pub fn angle(origin: (f32, f32), dest: (f32, f32)) -> f32 {
    let x_dist = dest.0 - origin.0;
    let y_dist = dest.1 - origin.1;

    (-y_dist).atan2(x_dist) % (2.0 * PI)
}

/// Returns a new point that is the distance away from the original point in the direction of the angle
pub fn project(origin: (f32, f32), angle: f32, distance: f32) -> (f32, f32) {
    (
        origin.0 + (angle.cos() * distance),
        origin.1 - (angle.sin() * distance),
    )
}

/// Gets the distance between two points
pub fn distance(p1: (f32, f32), p2: (f32, f32)) -> f32 {
    ((p1.0 - p2.0).powf(2.0) + (p1.1 - p2.1).powf(2.0)).sqrt()
}

#[derive(new)]
/// A tween that moves from one point to another linearly
pub struct Tween {
    start: (f32, f32),
    end: (f32, f32),
    #[new(value = "angle(start, end)")]
    angle: f32,
    speed: f32,
}
impl Tween {
    pub fn update(&mut self) -> (f32, f32) {
        self.start = project(self.start, self.angle, self.speed * get_frame_time());
        if distance(self.start, self.end) <= self.speed * get_frame_time() {
            self.start = self.end;
            self.speed = 0.0;
        }

        self.start
    }
}
