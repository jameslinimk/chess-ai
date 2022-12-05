use std::fmt;

use macroquad::prelude::{
    is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton,
};
use macroquad::rand::gen_range;
use macroquad::shapes::draw_rectangle;
use macroquad::text::{draw_text_ex, measure_text, TextDimensions, TextParams};

use crate::conf::{COLOR_BUTTON, COLOR_BUTTON_HOVER, COLOR_BUTTON_PRESSED, COLOR_WHITE};
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
        $crate::util::Loc { x: $x, y: $y }
    };
}

/// Create [std::collections::HashMap]'s using a readable syntax, similar to dicts in python or objects in js. Adapted from maplit to support `FxHashMap`
#[macro_export]
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
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

    ($($key:expr,)+) => { hashset!($($key),+) };
    ($($key:expr),*) => {
        {
            let _cap = hashset!(@count $($key),*);
            let mut _set = rustc_hash::FxHashSet::with_capacity_and_hasher(_cap, Default::default());
            $(
                let _ = _set.insert($key);
            )*
            _set
        }
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
impl fmt::Debug for Loc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
impl Loc {
    /// Create a new `Loc`, shifted `x_diff` and `y_diff` away from the current pos
    pub fn copy_move_i32(&self, x_diff: i32, y_diff: i32) -> (Loc, bool) {
        let mut new_x = self.x as i32 + x_diff;
        let mut new_y = self.y as i32 + y_diff;

        if new_x < 0 || new_y < 0 {
            new_x = clamp_negative!(new_x);
            new_y = clamp_negative!(new_y);
            return (loc!(new_x as usize, new_y as usize), true);
        }

        (loc!(new_x as usize, new_y as usize), false)
    }

    /// Move the current pos by `x_diff` and `y_diff`
    pub fn move_i32(&mut self, x_diff: i32, y_diff: i32) -> bool {
        let new_x = self.x as i32 + x_diff;
        let new_y = self.y as i32 + y_diff;

        if new_x < 0 || new_y < 0 {
            self.x = clamp_negative!(new_x) as usize;
            self.y = clamp_negative!(new_y) as usize;
            return false;
        }

        self.x = new_x as usize;
        self.y = new_y as usize;
        true
    }

    /// Get the location as chess notation IE (`(0, 0)` becomes `"A8"`)
    pub fn as_notation(&self) -> String {
        let x = char::from_u32(self.x as u32 + 97).unwrap();
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
