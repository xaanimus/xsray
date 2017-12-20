extern crate image;

use std::cmp::max;
use self::image::{RgbImage};
use super::scene::*;
use super::integrator::*;

//clean
use super::color::*;
use super::math::*;

fn i32_to_u32(x: i32) -> u32 {
    max(x, 0) as u32
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Config {
    pub settings: RenderSettings,
    pub scene: Scene,
}

impl Config {
    pub fn new(settings: RenderSettings, scene: Scene) -> Config {
        Config {
            settings: settings,
            scene: scene
        }
    }

    pub fn render(&self) -> RgbImage {
        // make buffer
        let mut buffer = RgbImage::new(i32_to_u32(self.settings.resolution_width),
                                   i32_to_u32(self.settings.resolution_height));

        {
            let blocks = ImageBlockIterator::new(&buffer, 8, 8);
            for block in blocks {
                for x in block.start_x()..block.end_x() {
                    for y in block.start_y()..block.end_y() {
                        let mut pixel = buffer.get_pixel_mut(x,y);
                        let (u, v) = self.settings.pixel_to_uv(x as i32, y as i32);
                        let render_color = self.render_point(u,v);
                        let (r, g, b) = render_color.pixel_rgb8_values();
                        pixel.data[0] = r;
                        pixel.data[1] = g;
                        pixel.data[2] = b;
                    }
                }
            }
        }

        buffer
    }

    pub fn render_point(&self, u: f32, v: f32) -> Color3 {
        //let ray = self.scene.camera.shoot_ray(u,v);
        let integrator = PathTracerIntegrator {
            max_bounces: 0,
            number_samples: 1 
        };
        integrator.shade_camera_point(&self.scene, u, v)
    }

    pub fn process_color(&self, color: Color3) -> Color3 {
        color * self.settings.exposure
    }
}

#[derive(Debug)]
struct ImageBlock {
    block_width: u32,
    block_height: u32,
    pixel_x: u32,
    pixel_y: u32
}

impl ImageBlock {
    fn start_x(&self) -> u32 { self.pixel_x }
    fn start_y(&self) -> u32 { self.pixel_y }
    fn end_x(&self) -> u32 { self.pixel_x + self.block_width }
    fn end_y(&self) -> u32 { self.pixel_y + self.block_height }
}

struct ImageBlockIterator {
    buffer_width: u32,
    buffer_height: u32,
    current_pixel_x: u32,
    current_pixel_y: u32,
    block_width: u32,
    block_height: u32
}

impl ImageBlockIterator {
    fn new(buffer: &RgbImage,
           block_width: u32, block_height: u32) -> ImageBlockIterator {
        ImageBlockIterator {
            buffer_width: buffer.width(),
            buffer_height: buffer.height(),
            current_pixel_x: 0,
            current_pixel_y: 0,
            block_width: block_width,
            block_height: block_height
        }
    }
}

impl Iterator for ImageBlockIterator {
    type Item = ImageBlock;
    fn next(&mut self) -> Option<Self::Item> {
        let new_x: u32;
        let new_y: u32;
        if self.current_pixel_x + self.block_width >= self.buffer_width {
            new_x = 0;
            new_y = self.current_pixel_y + self.block_height;
        } else {
            new_x = self.current_pixel_x + self.block_width;
            new_y = self.current_pixel_y;
        };
        //due to previous if, new_width should never be negative
        let new_width = (self.buffer_width - new_x).min(self.block_width);
        let new_height = (self.buffer_height - new_y).min(self.block_height);
        if new_height <= 0 {
            None
        } else {
            self.current_pixel_x = new_x;
            self.current_pixel_y = new_y;
            Some(ImageBlock {
                block_width: new_width,
                block_height: new_height,
                pixel_x: new_x,
                pixel_y: new_y
            })
        }
    }
}

