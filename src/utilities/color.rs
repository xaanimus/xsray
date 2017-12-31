//TODO before implementing advanced image processing, implement Color3: image::Pixel
extern crate image;

use std::f32;
use self::image::Rgb;

use super::math::{Vec3};

pub type Color3 = Vec3;

fn convert_float_to_char_pixel(from: f32) -> u8 {
    let clipped = from.min(1.0).max(0.0);
    (clipped * 255.0) as u8
}

pub trait RgbU8Convertible {
    fn to_rgb(&self) -> Rgb<u8>;
}

impl RgbU8Convertible for Color3 {
    fn to_rgb(&self) -> Rgb<u8> {
        Rgb {data: [
            convert_float_to_char_pixel(self.x),
            convert_float_to_char_pixel(self.y),
            convert_float_to_char_pixel(self.z)
        ]}
    }
}
