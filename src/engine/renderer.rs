extern crate image;

use std::cmp::max;
use self::image::{RgbImage};
use super::scene::*;
use super::misc::*;

fn i32_to_u32(x: i32) -> u32 {
    max(x, 0) as u32
}

#[derive(Debug)]
pub struct RenderSettings {
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub exposure: f32,
}

impl RenderSettings {
    fn pixel_to_uv(&self, x: i32, y: i32) -> (f32, f32) {
        ((x as f32 + 0.5) / self.resolution_width as f32,
         ((self.resolution_height - 1 - y) as f32 + 0.5) / self.resolution_width as f32)
    }
}

#[derive(Debug)]
pub struct Config {
    pub settings: RenderSettings,
    pub scene: Scene,
}

impl Config {
    pub fn render(&self) -> RgbImage {
        // make buffer
        let mut buffer = RgbImage::new(i32_to_u32(self.settings.resolution_width),
                                   i32_to_u32(self.settings.resolution_height));

        //render each pixel
        for x in 0..buffer.width() {
            for y in 0..buffer.height() {
                let mut pixel = buffer.get_pixel_mut(x,y);
                let (u, v) = self.settings.pixel_to_uv(x as i32, y as i32);
                let render_color = self.process_color(self.render_point(u,v));
                let (r, g, b) = render_color.pixel_rgb8_values();
                pixel.data[0] = r;
                pixel.data[1] = g;
                pixel.data[2] = b;
            }
        }

        buffer
    }

    pub fn render_point(&self, u: f32, v: f32) -> Color3 {
        let ray = self.scene.camera.shoot_ray(u,v);
        //try to intersect with object.
        if let Some((rec, shader)) = self.scene.intersect(&ray) {
            shader.shade(&rec, &self.scene)
        } else {
            //if no intersection, output background
            self.scene.background_color
        }
    }

    pub fn process_color(&self, color: Color3) -> Color3 {
        color * self.settings.exposure
    }
}




