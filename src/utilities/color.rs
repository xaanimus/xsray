use std::f32;

use super::math::{Vec3};

pub type Color3 = Vec3;

fn f32_to_u8_color(x: f32) -> u8 {
    f32::max(0f32, f32::min(x * 255f32, 255f32)) as u8
}

pub trait PixelRgb8Extractable {
    fn pixel_rgb8_values(&self) -> (u8, u8, u8);
}

impl PixelRgb8Extractable for Color3 {
    fn pixel_rgb8_values(&self) -> (u8, u8, u8) {
        (f32_to_u8_color(self.x),
         f32_to_u8_color(self.y),
         f32_to_u8_color(self.z))
    }
}
