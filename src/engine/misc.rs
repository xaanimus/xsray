
extern crate cgmath;
use std::ops::Range;
use std::f32;

pub type Vec3 = cgmath::Vector3<f32>;
pub type Matrix3 = cgmath::Matrix3<f32>;
pub type Color3 = Vec3;

pub use self::cgmath::Zero;
pub use self::cgmath::SquareMatrix;

fn f32_to_u8_color(x: f32) -> u8 {
    f32::max(0f32, f32::min(x * 255f32, 255f32)) as u8
}

pub struct Ray {
    pub position: Vec3,
    pub direction: Vec3,
    pub t_range: Range<f32>,
}

impl Ray {
    pub fn new(position: Vec3, direction: Vec3) -> Ray {
        Ray {
            position: position,
            direction: direction,
            t_range: 0.0..(f32::INFINITY)
        }
    }
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
