//! Software rasterizer based on the amazing tutorial by 'thebennybox'
//! at https://www.youtube.com/playlist?list=PLEETnX-uPtBUbVOok816vTl1K9vV1GgH5

mod matrix4f;
mod vector4f;

use matrix4f::Matrix4f;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use std::time::{Duration, Instant};
use vector4f::Vector4f;

struct BitMap {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
    scan_buffer: Vec<usize>,
}

impl BitMap {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width: width as usize,
            height: height as usize,
            buffer: vec![0; (width * height * 3) as usize],
            scan_buffer: vec![0; (height * 2) as usize],
        }
    }

    fn clear(&mut self, shade: u8) {
        self.buffer.fill(shade);
    }

    fn draw_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let index = (y * self.width + x) * 3;
        self.buffer[index] = r;
        self.buffer[index + 1] = g;
        self.buffer[index + 2] = b;
    }

    fn draw_scan_buffer(&mut self, y: usize, x_min: usize, x_max: usize) {
        self.scan_buffer[y * 2] = x_min;
        self.scan_buffer[y * 2 + 1] = x_max;
    }

    fn fill_shape(&mut self, y_min: usize, y_max: usize) {
        for j in y_min..y_max {
            let x_min = self.scan_buffer[j * 2];
            let x_max = self.scan_buffer[j * 2 + 1];
            for i in x_min..x_max {
                self.draw_pixel(i, j, 0xFF, 0xFF, 0xFF);
            }
        }
    }

    fn scan_convert_line(&mut self, min_y: Vertex, max_y: Vertex, which_side: usize) {
        let y_start = min_y.y() as usize;
        let y_end = max_y.y() as usize;
        let x_start = min_y.x() as usize;
        let x_end = max_y.x() as usize;

        let y_dist = y_end - y_start;
        let x_dist = x_end as i32 - x_start as i32;

        if y_dist == 0 {
            return;
        }

        let x_step: f64 = x_dist as f64 / y_dist as f64;
        let mut cur_x: f64 = x_start as f64;

        for j in y_start..y_end {
            self.scan_buffer[j * 2 + which_side] = cur_x as usize;
            cur_x += x_step;
        }
    }

    fn scan_convert_triangle(&mut self, min_y: Vertex, mid_y: Vertex, max_y: Vertex, handedness: usize) {
        self.scan_convert_line(min_y, max_y, 0 + handedness);
        self.scan_convert_line(min_y, mid_y, 1 - handedness);
        self.scan_convert_line(mid_y, max_y, 1 - handedness);
    }

    fn fill_triangle(&mut self, v1: Vertex, v2: Vertex, v3: Vertex) {
        let screen_space_transform =
            Matrix4f::init_screen_space_transform((self.width as f64) / 2.0, (self.height as f64) / 2.0);
        let mut min_y = v1.transform(screen_space_transform).perspective_divide();
        let mut mid_y = v2.transform(screen_space_transform).perspective_divide();
        let mut max_y = v3.transform(screen_space_transform).perspective_divide();

        if max_y.y() < mid_y.y() {
            std::mem::swap(&mut max_y, &mut mid_y);
        }

        if mid_y.y() < min_y.y() {
            std::mem::swap(&mut mid_y, &mut min_y);
        }

        if max_y.y() < mid_y.y() {
            std::mem::swap(&mut max_y, &mut mid_y);
        }

        let area = min_y.triangle_area_times_two(max_y, mid_y);
        let handedness = if area >= 0.0 { 1 } else { 0 };

        self.scan_convert_triangle(min_y, mid_y, max_y, handedness);
        self.fill_shape(min_y.y() as usize, max_y.y() as usize);
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    pos: Vector4f,
}

impl Vertex {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { pos: Vector4f { x, y, z, w: 1.0 } }
    }

    fn x(&self) -> f64 {
        self.pos.x
    }

    fn y(&self) -> f64 {
        self.pos.y
    }

    fn triangle_area_times_two(&self, b: Vertex, c: Vertex) -> f64 {
        let x1 = b.x() - self.x();
        let y1 = b.y() - self.y();

        let x2 = c.x() - self.x();
        let y2 = c.y() - self.y();

        x1 * y2 - x2 * y1
    }

    fn transform(&self, matrix: Matrix4f) -> Self {
        Vertex { pos: matrix.transform(self.pos) }
    }

    fn perspective_divide(&self) -> Self {
        Vertex {
            pos: Vector4f {
                x: self.pos.x / self.pos.w,
                y: self.pos.y / self.pos.w,
                z: self.pos.z / self.pos.w,
                w: self.pos.w,
            },
        }
    }
}

fn main() {
    let width: u32 = 900;
    let height: u32 = 900;
    let mut bitmap = BitMap::new(width, height);

    bitmap.clear(0x80);

    let min_y = Vertex::new(-1.0, -1.0, 0.0);
    let mid_y = Vertex::new(0.0, 1.0, 0.0);
    let max_y = Vertex::new(1.0, -1.0, 0.0);

    let projection = Matrix4f::init_perspective(
        70.0_f64.to_radians(),
        (width as f64) / (height as f64),
        0.1,
        1000.0,
    );

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Software Rendering", width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_static(PixelFormatEnum::RGB24, width, height).unwrap();
    texture.update(None, &bitmap.buffer, bitmap.width * 3).unwrap();
    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut previous_time = Instant::now();
    let mut frame_count = 0;
    let mut fps_timer = Instant::now();
    let mut frame_times_ms = Vec::with_capacity(100); // For averaging frame time

    // 🔧 FPS limit setting: use `None` for unlimited
    const MAX_FPS: Option<u32> = Some(90); // Or `None` to disable limit
    let frame_duration = MAX_FPS.map(|fps| Duration::from_secs_f64(1.0 / fps as f64));

    let mut rotation_counter: f64 = 0.0;

    'running: loop {
        let frame_start = Instant::now();
        let current_time = Instant::now();
        let delta = current_time - previous_time;
        previous_time = current_time;

        // Update and render
        rotation_counter += delta.as_secs_f64(); // Adjust rotation speed
        let translation = Matrix4f::init_translation(0.0, 0.0, 3.0);
        let rotation = Matrix4f::init_rotation_euler(0.0, rotation_counter, 0.0);
        let transform = projection.mul(translation.mul(rotation));

        bitmap.clear(0);
        bitmap.fill_triangle(
            max_y.transform(transform),
            min_y.transform(transform),
            mid_y.transform(transform),
        );
        // bitmap.fill_triangle(max_y, min_y, mid_y);

        texture.update(None, &bitmap.buffer, bitmap.width * 3).unwrap();
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
