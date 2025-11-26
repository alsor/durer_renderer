//! Software rasterizer based on the amazing tutorial by 'thebennybox'
//! at https://www.youtube.com/playlist?list=PLEETnX-uPtBUbVOok816vTl1K9vV1GgH5

use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

struct BitMap {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

impl BitMap {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; (width * height * 3) as usize],
        }
    }

    fn clear(&mut self, shade: u8) {
        self.buffer.fill(shade);
    }

    fn draw_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        let index = ((y * self.width + x) * 3) as usize;
        self.buffer[index] = r;
        self.buffer[index + 1] = g;
        self.buffer[index + 2] = b;
    }
}

fn main() {
    let size = 900;
    let mut bitmap = BitMap::new(size, size);
    bitmap.clear(0x80);
    for x in 100..150 {
        for y in 100..150 {
            bitmap.draw_pixel(x, y, 0xFF, 0, 0);
        }
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Software Rendering", size, size)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_static(PixelFormatEnum::RGB24, bitmap.width, bitmap.height)
        .unwrap();
    texture.update(None, &bitmap.buffer, (bitmap.width * 3) as usize).unwrap();
    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }
    }
}
