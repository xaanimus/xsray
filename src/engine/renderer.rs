extern crate serde;
extern crate image;
extern crate time;
extern crate rand;

use self::image::{RgbImage, ImageBuffer, Rgb, Pixel};
use super::scene::*;
use super::integrator::*;

//clean
use super::color::*;
use utilities::math::*;
use utilities::codable::*;

fn gamma_correct(value: f32, gamma: f32) -> f32 {
    value.powf(1.0 / gamma)
}

fn convert_float_to_char_pixel(from: f32) -> u8 {
    let clipped = from.min(1.0).max(0.0);
    (clipped * 255.0) as u8
}

#[derive(Debug, Deserialize)]
pub struct RenderSettings {
    pub resolution_width: i32,
    pub resolution_height: i32,
}

impl RenderSettings {
    ///Maps x from [0, resolution_width) to [0, 1)
    ///and y from [0, resolution_height) to [1, 0)
    fn pixel_to_uv(&self, x: i32, y: i32) -> (f32, f32) {
        ((x as f32 + 0.5) / self.resolution_width as f32,
         ((self.resolution_height - 1 - y) as f32 + 0.5) / self.resolution_width as f32)
    }

    fn pixel_float_to_uv(&self, x: f32, y: f32) -> (f32, f32) {
        ((x + 0.5) / self.resolution_width as f32,
         ((self.resolution_height as f32 - 1.0 - y) + 0.5) / self.resolution_width as f32)
    }

    fn uv_pixel_info(&self) -> UvPixelInfo {
        UvPixelInfo {
            uv_pixel_width: 1.0 / self.resolution_width as f32,
            uv_pixel_height: 1.0 / self.resolution_height as f32
        }
    }
}

impl_deserialize!(CodableWrapper<Box<Integrator>>, |deserializer| {
                  let integrator = IntegratorSpec::deserialize(deserializer)?.into_integrator();
                  Ok(integrator.into())
});

#[derive(Debug, Deserialize)]
struct PostProcess {
    exposure: f32,
    gamma: f32
}

impl PostProcess {
    fn apply(&self, buffer: &mut Float32Image) {
        for pixel in buffer.pixels_mut() {
            *pixel = pixel.map(|value| {
                let pre_gamma = value * self.exposure;
                gamma_correct(pre_gamma, self.gamma)
            })
        }
    }
}

impl Default for PostProcess {
    fn default() -> Self {
        PostProcess {
            exposure: 1.0,
            gamma: 2.2
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub settings: RenderSettings,
    pub scene: Scene,
    integrator: CodableWrapper<Box<Integrator>>,
    post_process: PostProcess
}

type Float32Image = ImageBuffer<Rgb<f32>, Vec<f32>>;
fn image_f32_to_u8(src: &Float32Image) -> RgbImage {
    let mut result = RgbImage::new(src.width(), src.height());
    for y in 0..src.height() {
        for x in 0..src.width() {
            let float_pixel = src.get_pixel(x,y);
            let u8_pixel = Rgb {data: [
                convert_float_to_char_pixel(float_pixel.data[0]),
                convert_float_to_char_pixel(float_pixel.data[1]),
                convert_float_to_char_pixel(float_pixel.data[2]),
            ]};
            result.put_pixel(x, y, u8_pixel);
        }
    }
    result
}

impl Config {
    pub fn new(settings: RenderSettings, scene: Scene, integrator: Box<Integrator>) -> Config {
        Config {
            settings: settings,
            scene: scene,
            integrator: integrator.into(),
            post_process: PostProcess::default()
        }
    }

    pub fn render(&self) -> RgbImage {
        println!("rendering...");
        let start_time = time::precise_time_s();
        // make buffer
        let mut buffer = Float32Image::new(clamp_i32(self.settings.resolution_width),
                                           clamp_i32(self.settings.resolution_height));

        {
            let blocks = ImageBlockIterator::new(&buffer, 8, 8);
            for block in blocks {
                for x in block.start_x()..block.end_x() {
                    for y in block.start_y()..block.end_y() {
                        let mut pixel = buffer.get_pixel_mut(x,y);
                        let (u, v) = self.settings.pixel_to_uv(x as i32, y as i32);
                        let render_color = self.render_point(u,v);
                        pixel.data[0] = render_color.x;
                        pixel.data[1] = render_color.y;
                        pixel.data[2] = render_color.z;
                    }
                }
            }
        }

        let end_time = time::precise_time_s();
        println!("elapsed time: {}s", end_time - start_time);

        let post_process_start_time = time::precise_time_s();
        self.post_process.apply(&mut buffer);
        let result = image_f32_to_u8(&buffer);
        let post_process_end_time = time::precise_time_s();
        println!("post processing time: {}s", post_process_end_time - post_process_start_time);

        result
    }

    pub fn render_point(&self, u: f32, v: f32) -> Color3 {
        self.integrator.get_ref()
            .shade_camera_point(&self.scene, u, v, &self.settings.uv_pixel_info())
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
    fn new<Pr: image::Primitive + 'static>(buffer: &ImageBuffer<Rgb<Pr>, Vec<Pr>>,
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

