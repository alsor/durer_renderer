//! Software rasterizer based on the amazing tutorial by 'thebennybox'
//! at https://www.youtube.com/playlist?list=PLEETnX-uPtBUbVOok816vTl1K9vV1GgH5

use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

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
        let y_start = min_y.y as usize;
        let y_end = max_y.y as usize;
        let x_start = min_y.x as usize;
        let x_end = max_y.x as usize;

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
        let mut min_y = v1;
        let mut mid_y = v2;
        let mut max_y = v3;

        if max_y.y < mid_y.y {
            std::mem::swap(&mut max_y, &mut mid_y);
        }

        if mid_y.y < min_y.y {
            std::mem::swap(&mut mid_y, &mut min_y);
        }

        if max_y.y < mid_y.y {
            std::mem::swap(&mut max_y, &mut mid_y);
        }

        let area = min_y.triangle_area(max_y, mid_y);
        let handedness = if area >= 0.0 { 1 } else { 0 };

        self.scan_convert_triangle(min_y, mid_y, max_y, handedness);
        self.fill_shape(min_y.y as usize, max_y.y as usize);
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    x: f64,
    y: f64,
}

impl Vertex {
    fn triangle_area(&self, b: Vertex, c: Vertex) -> f64 {
        let x1 = b.x - self.x;
        let y1 = b.y - self.y;

        let x2 = c.x - self.x;
        let y2 = c.y - self.y;

        x1 * y2 - x2 * y1
    }
}

fn main() {
    let width: u32 = 900;
    let height: u32 = 900;
    let mut bitmap = BitMap::new(width, height);
    
    bitmap.clear(0x80);

    let min_y = Vertex { x: 100.0, y: 100.0 };
    let mid_y = Vertex { x: 150.0, y: 200.0 };
    let max_y = Vertex { x: 80.0, y: 300.0 };

    bitmap.fill_triangle(max_y, min_y, mid_y);

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
    'running: loop {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }
    }
}
