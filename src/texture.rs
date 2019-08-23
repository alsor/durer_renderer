use image::DynamicImage;
use image::GenericImageView;

use crate::Color;

pub struct Texture {
    img: DynamicImage,
    width: u32,
    height: u32
}

impl Texture {
    pub fn get_texel(&self, u: f64, v: f64) -> Color {
        let x = (((self.width - 1) as f64) * u) as u32;
        let y = (((self.height - 1) as f64) * v) as u32;
        let pixel = self.img.get_pixel(x, y);

        Color { r: pixel[0], g: pixel[1], b: pixel[2] }
    }
}

pub fn load_from_file(filename: &str) -> Texture {
    let img = image::open(filename).unwrap();
    let (width, height) = img.dimensions();

    Texture { img, width, height }
}