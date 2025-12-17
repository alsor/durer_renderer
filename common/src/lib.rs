pub mod vectors;

#[derive(Copy, Clone)]
pub struct Vector3f {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3f {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: x,
            y: y,
            z: z,
        }
    }

    pub fn zero_vector() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn from_vec(vec: [f64; 3]) -> Self {
        Vector3f { x: vec[0], y: vec[1], z: vec[2] }
    }

    pub fn to_vec(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Copy, Clone)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
    pub color: Color,
}

#[derive(Copy, Clone)]
pub enum Light {
    Ambient { intensity: f64 },
    Point { intensity: f64, position: Vector3f },
    Directional { intensity: f64, direction: Vector3f },
}

pub fn multiply_color(k: f64, color: Color) -> Color {
    Color {
        r: multiply_channel(k, color.r),
        g: multiply_channel(k, color.g),
        b: multiply_channel(k, color.b),
    }
}

fn multiply_channel(k: f64, channel: u8) -> u8 {
    let scaled = channel as f64 * k;
    if scaled > 255.0 {
        255
    } else if scaled < 0.0 {
        0
    } else {
        scaled as u8
    }
}

pub fn put_pixel_to_buffer(buffer: &mut Vec<u8>, size: usize, pixel: Pixel) {
    let offset = pixel.y * size * 3 + pixel.x * 3;
    //        if offset >= self.buffer.len() {
    //            println!("was going to draw pixel {} {} with buffer offset {}",
    //                     pixel.x, pixel.y, offset);
    //            return;
    //        }

    buffer[offset] = pixel.color.r;
    buffer[offset + 1] = pixel.color.g;
    buffer[offset + 2] = pixel.color.b;
}
