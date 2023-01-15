use derive_new::new;
use macroquad::prelude::{mouse_position, screen_height, screen_width, set_camera, vec2, Camera2D, Vec2};

use crate::conf::{HEIGHT, WIDTH};

#[derive(Clone, Copy, new)]
pub struct Camera {
    #[new(value = "Camera2D {
        zoom: vec2(2.0 / screen_width(), -2.0 / screen_height()),
        target: vec2(screen_width() / 2.0, screen_height() / 2.0),
        ..Default::default()
    }")]
    camera: Camera2D,

    #[new(value = "1.0")]
    scale: f32,
}
impl Camera {
    fn update_camera(&self) {
        set_camera(&self.camera);
    }

    pub fn update(&mut self) {
        let width_height_ratio = WIDTH as f32 / HEIGHT as f32;
        if screen_width() / screen_height() > width_height_ratio {
            self.scale = screen_height() / HEIGHT as f32;
        } else {
            self.scale = screen_width() / WIDTH as f32;
        }

        self.camera.zoom = vec2(2.0 / screen_width(), -2.0 / screen_height()) * self.scale;

        self.update_camera();
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.camera.screen_to_world(mouse_position().into())
    }
}
