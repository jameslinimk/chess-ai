use derive_new::new;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
}
impl Loc {
    pub fn move_usize(&self, x_diff: usize, y_diff: usize) -> Loc {
        loc!(self.x + x_diff, self.y + y_diff)
    }

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
