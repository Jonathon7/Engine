use crate::platform::tilemap::{WORLD_WIDTH, WORLD_HEIGHT, SCREEN_WIDTH, SCREEN_HEIGHT};

pub const CAMERA_SPEED: f32 = 400.0;

pub struct Camera {
    pub x: f32,
    pub y: f32
}

impl Camera {
    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        (world_x - self.x, world_y - self.y)
    }

    pub fn clamp_to_world(&mut self) {
        let max_x = (WORLD_WIDTH - SCREEN_WIDTH as f32).max(0.0);
        let max_y = (WORLD_HEIGHT - SCREEN_HEIGHT as f32).max(0.0);
        self.x = self.x.clamp(0.0, max_x);
        self.y = self.y.clamp(0.0, max_y);
    }
}
