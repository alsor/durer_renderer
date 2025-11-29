//! Simple Starfield effect application.

use common::{Color, Pixel};
use rand::Rng;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::PixelFormat;
use std::time::{Duration, Instant};

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
        let fov: f64 = 120.0;
        let tan_half_fov: f64 = (fov / 2.0).to_radians().tan();

        for i in 0..self.num_stars {
            let new_z = self.star_z[i] - delta.as_secs_f64() * self.speed;

            if new_z <= 0.0 {
                self.new_star(i);
            } else {
                self.star_z[i] = new_z;
            }

            let screen_x =
                ((self.star_x[i] / (self.star_z[i] * tan_half_fov)) * half_size + half_size) as usize;
            let screen_y =
                ((self.star_y[i] / (self.star_z[i] * tan_half_fov)) * half_size + half_size) as usize;

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
                );
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
    let mut starfield = Starfield::new(5000, 5.0, 1.3);

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Durer", size as u32, size as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_static(PixelFormat::RGB24, size as u32, size as u32)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut previous_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_timer = Instant::now();
    let mut frame_times_ms = Vec::with_capacity(100); // For averaging frame time

    // 🔧 FPS limit setting: use `None` for unlimited
    const MAX_FPS: Option<u32> = Some(60); // Or `None` to disable limit
    let frame_duration = MAX_FPS.map(|fps| Duration::from_secs_f64(1.0 / fps as f64));

    'running: loop {
        let frame_start = Instant::now();
        let current_time = Instant::now();
        let delta = current_time - previous_time;
        previous_time = current_time;

        buffer.fill(0);
        starfield.update_and_render(&delta, &mut buffer, size);

        texture.update(None, &buffer, size * 3).unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        // 📏 Frame processing time (render + update + SDL)
        let frame_end = Instant::now();
        let frame_time_ms = (frame_end - frame_start).as_secs_f64() * 1000.0;
        frame_times_ms.push(frame_time_ms);

        // Increment frame counter
        frame_count += 1;

        // Output stats every second
        if fps_timer.elapsed().as_secs() >= 1 {
            let avg_frame_time_ms = frame_times_ms.iter().sum::<f64>() / frame_times_ms.len() as f64;
            println!(
                "FPS: {:>4}, Avg Frame Time: {:.2} ms",
                frame_count, avg_frame_time_ms
            );

            // Reset
            frame_count = 0;
            frame_times_ms.clear();
            fps_timer = Instant::now();
        }

        // ⏱️ FPS limiting: only applied if MAX_FPS is not None
        // if let Some(duration) = frame_duration {
        //     let frame_end_instant = previous_time + duration;
        //     if Instant::now() < frame_end_instant {
        //         std::thread::sleep(frame_end_instant - Instant::now());
        //     }
        // }

        // ⏱️ FPS limiting: More precise delay, but less efficient
        if let Some(duration) = frame_duration {
            let frame_end = previous_time + duration;
            while Instant::now() < frame_end {
                std::thread::yield_now();
            }
        }

        // Handle events
        match event_pump.poll_event() {
            Some(Event::Quit { .. }) | Some(Event::KeyDown { keycode: Some(Keycode::Escape), .. }) => {
                break 'running;
            }
            _ => {}
        }
    }
}
