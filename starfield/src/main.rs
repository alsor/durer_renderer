//! Simple Starfield effect application.

use common::{Color, Pixel};
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::Duration;
use std::time::Instant;

struct Starfield {
    num_stars: usize,
    spread: f64,
    speed: f64,
    star_x: Vec<f64>,
    star_y: Vec<f64>,
    star_z: Vec<f64>,
}

impl Starfield {
    fn new(num_stars: usize, spread: f64, speed: f64) -> Self {
        let mut star_x = Vec::with_capacity(num_stars);
        let mut star_y = Vec::with_capacity(num_stars);
        let mut star_z = Vec::with_capacity(num_stars);

        for _ in 0..num_stars {
            let (x, y, z) = init_star(spread);
            star_x.push(x);
            star_y.push(y);
            star_z.push(z);
        }

        Self { num_stars, spread, speed, star_x, star_y, star_z }
    }

    fn update_and_render(&mut self, delta: &Duration, buffer: &mut Vec<u8>, size: usize) {
        let half_size = (size as f64) / 2.0;

        for i in 0..self.num_stars {
            let new_z = self.star_z[i] - ((delta.as_nanos() as f64) / 1000000000.0) * self.speed;

            if new_z <= 0.0 {
                self.new_star(i);
            } else {
                self.star_z[i] = new_z;
            }

            let screen_x = ((self.star_x[i] / self.star_z[i]) * half_size + half_size) as usize;
            let screen_y = ((self.star_y[i] / self.star_z[i]) * half_size + half_size) as usize;

            if screen_x >= size || screen_y >= size {
                self.new_star(i);
            } else {
                common::put_pixel_to_buffer(
                    buffer,
                    size,
                    Pixel {
                        x: screen_x,
                        y: screen_y,
                        color: Color { r: 255, g: 255, b: 255 },
                    },
                )
            }
        }
    }

    fn new_star(&mut self, i: usize) {
        let (x, y, z) = init_star(self.spread);
        self.star_x[i] = x;
        self.star_y[i] = y;
        self.star_z[i] = z;
    }
}

fn init_star(spread: f64) -> (f64, f64, f64) {
    let mut rng = rand::thread_rng();

    (
        2.0 * (rng.gen::<f64>() - 0.5) * spread,
        2.0 * (rng.gen::<f64>() - 0.5) * spread,
        (rng.gen::<f64>() + 0.0001) * spread,
    )
}

#[test]
fn test_init_star() {
    let spread = 10.0;

    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
}

fn main() {
    let size = 900;
    let mut buffer = vec![0u8; size * size * 3];
    let mut starfield = Starfield::new(1000, 5.0, 110000.0);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Durer", size as u32, size as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_static(PixelFormatEnum::RGB24, size as u32, size as u32)
        .unwrap();

    let mut now = Instant::now();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let delta = now.elapsed();

        buffer.fill(0);
        starfield.update_and_render(&delta, &mut buffer, size);

        texture.update(None, &buffer, size * 3).unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        match event_pump.poll_event() {
            Some(event) => {
                match event {
                    Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    }
                    _ => {}
                };
            }
            None => {}
        };

        now = Instant::now();
    }
}
