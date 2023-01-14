use derive_new::new;
use macroquad::prelude::{screen_height, screen_width, set_camera, vec2, Camera2D};

use crate::conf::{HEIGHT, WIDTH};

#[derive(Clone, Copy, new)]
pub struct Camera {
    #[new(value = "Camera2D {
        zoom: vec2(2.0 / screen_width(), -2.0 / screen_height()),
        target: vec2(screen_width() / 2.0, screen_height() / 2.0),
        ..Default::default()
    }")]
    camera: Camera2D,
}
impl Camera {
    fn update_camera(&self) {
        set_camera(&self.camera);
    }

    pub fn update(&mut self) {
        self.camera.target = vec2(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
        self.update_camera();
    }
}
