//! Software rasterizer based on the amazing tutorial by 'thebennybox'
//! at https://www.youtube.com/playlist?list=PLEETnX-uPtBUbVOok816vTl1K9vV1GgH5

mod matrix4f;
mod vector4f;

use image::RgbImage;
use matrix4f::Matrix4f;
use rand::Rng;
use sdl3::{event::Event, keyboard::Keycode, pixels::PixelFormat};
use std::fmt;
use std::time::{Duration, Instant};
use vector4f::Vector4f;

struct Bitmap {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
}

impl Bitmap {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width: width as usize,
            height: height as usize,
            buffer: vec![0; (width * height * 3) as usize],
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

    fn copy_pixels(&mut self, dest_x: usize, dest_y: usize, src_x: usize, src_y: usize, src: &Bitmap) {
        let dest_index = (dest_y * self.width + dest_x) * 3;
        let src_index = (src_y * src.width + src_x) * 3;
        self.buffer[dest_index] = src.buffer[src_index];
        self.buffer[dest_index + 1] = src.buffer[src_index + 1];
        self.buffer[dest_index + 2] = src.buffer[src_index + 2];
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    pos: Vector4f,
    tex_coords: Vector4f,
}

impl Vertex {
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
        Vertex {
            pos: matrix.transform(self.pos),
            tex_coords: self.tex_coords,
        }
    }

    fn perspective_divide(&self) -> Self {
        Vertex {
            pos: Vector4f {
                x: self.pos.x / self.pos.w,
                y: self.pos.y / self.pos.w,
                z: self.pos.z / self.pos.w,
                w: self.pos.w,
            },
            tex_coords: self.tex_coords,
        }
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vertex(x: {:.2}, y: {:.2}, z: {:.2})",
            self.pos.x, self.pos.y, self.pos.z
        )
    }
}

struct Edge {
    x: f64,
    x_step: f64,
    y_start: usize,
    y_end: usize,
    tex_coord_x: f64,
    tex_coord_x_step: f64,
    tex_coord_y: f64,
    tex_coord_y_step: f64,
    inv_z: f64,
    inv_z_step: f64,
}

impl Edge {
    fn new(gradients: Gradients, start: Vertex, end: Vertex, start_index: usize) -> Self {
        let y_start = start.y().ceil() as usize;
        let y_end = end.y().ceil() as usize;

        let x_dist = end.x() - start.x();
        let y_dist = end.y() - start.y();

        let y_prestep = y_start as f64 - start.y();
        let x_step = x_dist / y_dist;
        let x = start.x() + y_prestep * x_step;
        let x_prestep = x - start.x();

        let tex_coord_x = gradients.tex_coord_x[start_index]
            + gradients.tex_coord_x_x_step * x_prestep
            + gradients.tex_coord_x_y_step * y_prestep;
        let tex_coord_x_step = gradients.tex_coord_x_y_step + gradients.tex_coord_x_x_step * x_step;

        let tex_coord_y = gradients.tex_coord_y[start_index]
            + gradients.tex_coord_y_x_step * x_prestep
            + gradients.tex_coord_y_y_step * y_prestep;
        let tex_coord_y_step = gradients.tex_coord_y_y_step + gradients.tex_coord_y_x_step * x_step;

        let inv_z = gradients.inv_z[start_index]
            + gradients.inv_z_x_step * x_prestep
            + gradients.inv_z_y_step * y_prestep;
        let inv_z_step = gradients.inv_z_y_step + gradients.inv_z_x_step * x_step;

        Self {
            x,
            x_step,
            y_start,
            y_end,
            tex_coord_x,
            tex_coord_x_step,
            tex_coord_y,
            tex_coord_y_step,
            inv_z,
            inv_z_step,
        }
    }

    fn step(&mut self) {
        self.x += self.x_step;
        self.tex_coord_x += self.tex_coord_x_step;
        self.tex_coord_y += self.tex_coord_y_step;
        self.inv_z += self.inv_z_step;
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Edge(x: {:.2}, x_step: {:.4}, y_start: {}, y_end: {})",
            self.x, self.x_step, self.y_start, self.y_end
        )
    }
}

fn fill_triangle(bitmap: &mut Bitmap, v1: Vertex, v2: Vertex, v3: Vertex, texture: &Bitmap) {
    let screen_space_transform =
        Matrix4f::init_screen_space_transform((bitmap.width as f64) / 2.0, (bitmap.height as f64) / 2.0);
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

    let short_is_left = min_y.triangle_area_times_two(max_y, mid_y) >= 0.0;
    scan_triangle(bitmap, min_y, mid_y, max_y, short_is_left, texture);
}

fn scan_triangle(
    bitmap: &mut Bitmap,
    min_y: Vertex,
    mid_y: Vertex,
    max_y: Vertex,
    short_is_left: bool,
    texture: &Bitmap,
) {
    let gradients = Gradients::new(min_y, mid_y, max_y);

    let mut top_to_bottom = Edge::new(gradients, min_y, max_y, 0);
    let mut top_to_middle = Edge::new(gradients, min_y, mid_y, 0);
    let mut middle_to_bottom = Edge::new(gradients, mid_y, max_y, 1);

    scan_edges(
        bitmap,
        &mut top_to_bottom,
        &mut top_to_middle,
        short_is_left,
        texture,
    );
    scan_edges(
        bitmap,
        &mut top_to_bottom,
        &mut middle_to_bottom,
        short_is_left,
        texture,
    );
}

fn scan_edges(bitmap: &mut Bitmap, long: &mut Edge, short: &mut Edge, short_if_left: bool, texture: &Bitmap) {
    let y_start = short.y_start;
    let y_end = short.y_end;

    let left;
    let right;

    if short_if_left {
        left = short;
        right = long;
    } else {
        left = long;
        right = short;
    }

    for j in y_start..y_end {
        draw_scan_line(bitmap, left, right, j, texture);
        left.step();
        right.step();
    }
}

fn draw_scan_line(bitmap: &mut Bitmap, left: &mut Edge, right: &Edge, j: usize, texture: &Bitmap) {
    let x_min = left.x.ceil() as usize;
    let x_max = right.x.ceil() as usize;
    let x_prestep = x_min as f64 - left.x;

    let x_dist = right.x - left.x;
    let tex_coord_x_x_step = if x_dist.abs() > f64::EPSILON {
        (right.tex_coord_x - left.tex_coord_x) / x_dist
    } else {
        0.0
    };
    let tex_coord_y_x_step = if x_dist.abs() > f64::EPSILON {
        (right.tex_coord_y - left.tex_coord_y) / x_dist
    } else {
        0.0
    };
    let inv_z_x_step = if x_dist.abs() > f64::EPSILON {
        (right.inv_z - left.inv_z) / x_dist
    } else {
        0.0
    };

    let mut tex_coord_x = left.tex_coord_x + tex_coord_x_x_step * x_prestep;
    let mut tex_coord_y = left.tex_coord_y + tex_coord_y_x_step * x_prestep;
    let mut inv_z = left.inv_z + inv_z_x_step * x_prestep;

    for i in x_min..x_max {
        let z = 1.0 / inv_z;
        let src_x = (((tex_coord_x * z) * (texture.width - 1) as f64) + 0.5) as usize;
        let src_y = (((tex_coord_y * z) * (texture.height - 1) as f64) + 0.5) as usize;

        bitmap.copy_pixels(i, j, src_x, src_y, texture);

        tex_coord_x += tex_coord_x_x_step;
        tex_coord_y += tex_coord_y_x_step;
        inv_z += inv_z_x_step;
    }
}

#[derive(Copy, Clone)]
struct Gradients {
    tex_coord_x: [f64; 3],
    tex_coord_x_x_step: f64,
    tex_coord_x_y_step: f64,
    tex_coord_y: [f64; 3],
    tex_coord_y_x_step: f64,
    tex_coord_y_y_step: f64,
    inv_z: [f64; 3],
    inv_z_x_step: f64,
    inv_z_y_step: f64,
}

impl Gradients {
    /// Создаёт градиенты по трём вершинам (отсортированным по Y)
    fn new(min_y: Vertex, mid_y: Vertex, max_y: Vertex) -> Self {
        let z0 = 1.0 / min_y.pos.w;
        let z1 = 1.0 / mid_y.pos.w;
        let z2 = 1.0 / max_y.pos.w;

        let x0 = min_y.tex_coords.x * z0;
        let x1 = mid_y.tex_coords.x * z1;
        let x2 = max_y.tex_coords.x * z2;

        let y0 = min_y.tex_coords.y * z0;
        let y1 = mid_y.tex_coords.y * z1;
        let y2 = max_y.tex_coords.y * z2;

        let dx1 = mid_y.x() - max_y.x();
        let dy1 = min_y.y() - max_y.y();
        let dx2 = min_y.x() - max_y.x();
        let dy2 = mid_y.y() - max_y.y();

        let det = dx1 * dy1 - dx2 * dy2;

        let inv_dx = if det.abs() > f64::EPSILON { 1.0 / det } else { 0.0 };
        let inv_dy = -inv_dx;

        let tex_coord_x_x_step = Self::calc_step_x(x0, x1, x2, dy1, dy2, inv_dx);
        let tex_coord_x_y_step = Self::calc_step_y(x0, x1, x2, dx2, dx1, inv_dy);

        let tex_coord_y_x_step = Self::calc_step_x(y0, y1, y2, dy1, dy2, inv_dx);
        let tex_coord_y_y_step = Self::calc_step_y(y0, y1, y2, dx2, dx1, inv_dy);

        let inv_z_x_step = Self::calc_step_x(z0, z1, z2, dy1, dy2, inv_dx);
        let inv_z_y_step = Self::calc_step_y(z0, z1, z2, dx2, dx1, inv_dy);

        Self {
            tex_coord_x: [x0, x1, x2],
            tex_coord_y: [y0, y1, y2],
            tex_coord_x_x_step,
            tex_coord_x_y_step,
            tex_coord_y_x_step,
            tex_coord_y_y_step,
            inv_z: [z0, z1, z2],
            inv_z_x_step,
            inv_z_y_step,
        }
    }

    /// Вычисляет градиент изменения величины вдоль оси X (по dy)
    /// Используется для шага по X: ∂/∂x
    fn calc_step_x(a0: f64, a1: f64, a2: f64, dy1: f64, dy2: f64, inv_dx: f64) -> f64 {
        (((a1 - a2) * dy1) - ((a0 - a2) * dy2)) * inv_dx
    }

    /// Вычисляет градиент изменения величины вдоль оси Y (по dx)
    /// Используется для шага по Y: ∂/∂y
    fn calc_step_y(a0: f64, a1: f64, a2: f64, dx2: f64, dx1: f64, inv_dy: f64) -> f64 {
        (((a1 - a2) * dx2) - ((a0 - a2) * dx1)) * inv_dy
    }
}

fn create_random_texture(width: u32, height: u32) -> Bitmap {
    let mut rng = rand::thread_rng();
    let mut texture = Bitmap::new(width, height);

    for j in 0..height as usize {
        for i in 0..width as usize {
            let r = rng.gen_range(0..=255);
            let g = rng.gen_range(0..=255);
            let b = rng.gen_range(0..=255);
            // В вашем `BitMap` используется RGB (3 байта), а не RGBA
            texture.draw_pixel(i, j, r, g, b);
        }
    }

    texture
}

fn main() {
    let width: u32 = 900;
    let height: u32 = 900;
    let mut bitmap = Bitmap::new(width, height);

    let texture = create_random_texture(32, 32);

    let v1 = Vertex {
        pos: Vector4f::new(-1.0, -1.0, 0.0, 1.0),
        tex_coords: Vector4f::new(0.0, 0.0, 0.0, 0.0), // UV: (0,0)
    };
    let v2 = Vertex {
        pos: Vector4f::new(0.0, 1.0, 0.0, 1.0),
        tex_coords: Vector4f::new(0.5, 1.0, 0.0, 0.0), // UV: (0.5,1)
    };
    let v3 = Vertex {
        pos: Vector4f::new(1.0, -1.0, 0.0, 1.0),
        tex_coords: Vector4f::new(1.0, 0.0, 0.0, 0.0), // UV: (1,0)
    };

    let projection = Matrix4f::init_perspective(
        70.0_f64.to_radians(),
        (width as f64) / (height as f64),
        0.1,
        1000.0,
    );

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Software Rendering", width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();
    let mut screen_texture =
        texture_creator.create_texture_static(PixelFormat::RGB24, width, height).unwrap();
    screen_texture.update(None, &bitmap.buffer, bitmap.width * 3).unwrap();
    canvas.clear();
    canvas.copy(&screen_texture, None, None).unwrap();
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
        let rotation = Matrix4f::init_rotation_euler(rotation_counter, rotation_counter, rotation_counter);
        let transform = projection.mul(translation.mul(rotation));

        bitmap.clear(0);
        fill_triangle(
            &mut bitmap,
            v1.transform(transform),
            v2.transform(transform),
            v3.transform(transform),
            &texture,
        );

        screen_texture.update(None, &bitmap.buffer, bitmap.width * 3).unwrap();
        canvas.clear();
        canvas.copy(&screen_texture, None, None).unwrap();
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

        // Ожидаем событие (блокирующий вызов)
        // for event in event_pump.wait_iter() {
        //     match event {
        //         Event::Quit { .. } => break 'running,
        //         Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
        //         Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
        //             // Реагируем на нажатие Пробела
        //             break;
        //         }
        //         _ => continue,
        //     }
        // }

        // Обработка событий
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    // Сохраняем скриншот
                    let img = RgbImage::from_raw(width as u32, height as u32, bitmap.buffer.clone())
                        .expect("Невозможно создать изображение из буфера");

                    if let Err(e) = img.save("screenshot.png") {
                        eprintln!("Ошибка при сохранении скриншота: {}", e);
                    } else {
                        println!("Скриншот сохранён как screenshot.png");
                    }
                }
                _ => {}
            }
        }
    }
}
