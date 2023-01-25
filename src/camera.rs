use derive_new::new;
use macroquad::prelude::{
    mouse_position, screen_height, screen_width, set_camera, vec2, Camera2D, Vec2,
};

use crate::conf::{HEIGHT, WIDTH};

static mut CAMERA: Option<Camera> = None;
pub(crate) fn camera() -> &'static mut Camera {
    unsafe {
        if CAMERA.is_none() {
            CAMERA = Some(Camera::new());
        }
        CAMERA.as_mut().unwrap()
    }
}

/// Struct for controlling the camera
#[derive(Clone, Copy, new)]
pub(crate) struct Camera {
    #[new(value = "Camera2D {
        zoom: vec2(2.0 / WIDTH as f32, -2.0 / HEIGHT as f32),
        target: vec2(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
        ..Default::default()
    }")]
    camera: Camera2D,

    #[new(value = "1.0")]
    scale: f32,
}
impl Camera {
    /// Set the current camera to the macroquad camera
    fn update_camera(&self) {
        set_camera(&self.camera);
    }

    /// Updates the camera, fits its to the screen size while keeping the aspect ratio
    pub(crate) fn update(&mut self) {
        let width_height_ratio = WIDTH as f32 / HEIGHT as f32;
        if screen_width() / screen_height() > width_height_ratio {
            self.scale = screen_height() / HEIGHT as f32;
        } else {
            self.scale = screen_width() / WIDTH as f32;
        }

        self.camera.zoom = vec2(2.0 / screen_width(), -2.0 / screen_height()) * self.scale;

        self.update_camera();
    }

    /// Gets the mouse position
    pub(crate) fn mouse_position(&self) -> Vec2 {
        self.camera.screen_to_world(mouse_position().into())
    }
}
