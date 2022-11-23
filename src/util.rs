use derive_new::new;

#[derive(new, Clone, Copy)]
pub struct Loc {
    pub x: usize,
    pub y: usize,
}
impl Loc {
    pub fn copy_move(&self, x_diff: usize, y_diff: usize) -> Loc {
        Loc::new(self.x + x_diff, self.y + y_diff)
    }

    pub fn copy_move_i32(&self, x_diff: i32, y_diff: i32) -> Loc {
        macro_rules! clamp_negative {
            ($number: expr) => {
                if $number.is_negative() {
                    0
                } else {
                    $number
                }
            };
        }

        Loc::new(
            clamp_negative!(self.x as i32 + x_diff) as usize,
            clamp_negative!(self.y as i32 + y_diff) as usize,
        )
    }
}
