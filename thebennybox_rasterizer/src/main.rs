//! Software rasterizer based on the amazing tutorial by 'thebennybox'
//! at https://www.youtube.com/playlist?list=PLEETnX-uPtBUbVOok816vTl1K9vV1GgH5

mod matrix4f;
mod vector4f;

use matrix4f::Matrix4f;
use sdl3::{event::Event, keyboard::Keycode, pixels::PixelFormat};
use std::fmt;
use std::time::{Duration, Instant};
use vector4f::Vector4f;

struct BitMap {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
}

impl BitMap {
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
}

#[derive(Clone, Copy)]
struct Vertex {
    pos: Vector4f,
    color: Vector4f,
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
        Vertex { pos: matrix.transform(self.pos), color: self.color }
    }

    fn perspective_divide(&self) -> Self {
        Vertex {
            pos: Vector4f {
                x: self.pos.x / self.pos.w,
                y: self.pos.y / self.pos.w,
                z: self.pos.z / self.pos.w,
                w: self.pos.w,
            },
            color: self.color,
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
    color: Vector4f,
    color_step: Vector4f,
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

        let color = gradients.colors[start_index]
            .add(gradients.color_y_step * y_prestep)
            .add(gradients.color_x_step * x_prestep);

        let color_step = gradients.color_y_step + gradients.color_x_step * x_step;

        Self { x, x_step, y_start, y_end, color, color_step }
    }

    fn step(&mut self) {
        self.x += self.x_step;
        self.color = self.color.add(self.color_step);
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

fn fill_triangle(bitmap: &mut BitMap, v1: Vertex, v2: Vertex, v3: Vertex) {
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
    scan_triangle(bitmap, min_y, mid_y, max_y, short_is_left);
}

fn scan_triangle(bitmap: &mut BitMap, min_y: Vertex, mid_y: Vertex, max_y: Vertex, short_is_left: bool) {
    let gradients = Gradients::new(min_y, mid_y, max_y);

    let mut top_to_bottom = Edge::new(gradients, min_y, max_y, 0);
    let mut top_to_middle = Edge::new(gradients, min_y, mid_y, 0);
    let mut middle_to_bottom = Edge::new(gradients, mid_y, max_y, 1);

    scan_edges(
        bitmap,
        gradients,
        &mut top_to_bottom,
        &mut top_to_middle,
        short_is_left,
    );
    scan_edges(
        bitmap,
        gradients,
        &mut top_to_bottom,
        &mut middle_to_bottom,
        short_is_left,
    );
}

fn scan_edges(
    bitmap: &mut BitMap,
    gradients: Gradients,
    long: &mut Edge,
    short: &mut Edge,
    short_if_left: bool,
) {
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
        draw_scan_line(bitmap, gradients, left, right, j);
        left.step();
        right.step();
    }
}

fn draw_scan_line(bitmap: &mut BitMap, gradients: Gradients, left: &mut Edge, right: &Edge, j: usize) {
    let x_min = left.x.ceil() as usize;
    let x_max = right.x.ceil() as usize;
    let x_prestep = x_min as f64 - left.x;

    let min_color = left.color.add(gradients.color_x_step.mul_scalar(x_prestep));
    let max_color = right.color.add(gradients.color_y_step.mul_scalar(x_prestep));

    let mut lerp_amount = 0.0;
    let lerp_step = 1.0 / (x_max as f64 - x_min as f64);

    for i in x_min..x_max {
        let color = min_color.lerp(max_color, lerp_amount);

        let r = (color.x * 255.0 + 0.5) as u8;
        let g = (color.y * 255.0 + 0.5) as u8;
        let b = (color.z * 255.0 + 0.5) as u8;

        bitmap.draw_pixel(i, j, r, g, b);
        lerp_amount += lerp_step;
    }
}

#[derive(Copy, Clone)]
struct Gradients {
    colors: [Vector4f; 3], // [minY, midY, maxY]
    color_x_step: Vector4f,
    color_y_step: Vector4f,
}

impl Gradients {
    /// Создаёт градиенты по трём вершинам (отсортированным по Y)
    fn new(min_y: Vertex, mid_y: Vertex, max_y: Vertex) -> Self {
        let c0 = min_y.color;
        let c1 = mid_y.color;
        let c2 = max_y.color;

        let dx1 = mid_y.x() - max_y.x();
        let dy1 = min_y.y() - max_y.y();
        let dx2 = min_y.x() - max_y.x();
        let dy2 = mid_y.y() - max_y.y();

        let det = dx1 * dy1 - dx2 * dy2;

        let inv_dx = if det.abs() > f64::EPSILON { 1.0 / det } else { 0.0 };
        let inv_dy = -inv_dx;

        let color_x_step = (c1.sub(c2).mul_scalar(dy1)).sub(c0.sub(c2).mul_scalar(dy2)).mul_scalar(inv_dx);
        let color_y_step = (c1.sub(c2).mul_scalar(dx2)).sub(c0.sub(c2).mul_scalar(dx1)).mul_scalar(inv_dy);

        Self { colors: [c0, c1, c2], color_x_step, color_y_step }
    }
}

impl fmt::Display for Gradients {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Gradients {{")?;
        writeln!(f, "  colors[0]: {}", self.colors[0])?;
        writeln!(f, "  colors[1]: {}", self.colors[1])?;
        writeln!(f, "  colors[2]: {}", self.colors[2])?;
        writeln!(f, "  color_x_step: {}", self.color_x_step)?;
        writeln!(f, "  color_y_step: {}", self.color_y_step)?;
        write!(f, "}}")
    }
}

fn main() {
    let width: u32 = 900;
    let height: u32 = 900;
    let mut bitmap = BitMap::new(width, height);

    bitmap.clear(0x80);

    let v1 = Vertex {
        pos: Vector4f::new(-1.0, -1.0, 0.0, 1.0),
        color: Vector4f::new(1.0, 0.0, 0.0, 0.0),
    };
    let v2 = Vertex {
        pos: Vector4f::new(0.0, 1.0, 0.0, 1.0),
        color: Vector4f::new(0.0, 1.0, 0.0, 0.0),
    };
    let v3 = Vertex {
        pos: Vector4f::new(1.0, -1.0, 0.0, 1.0),
        color: Vector4f::new(0.0, 0.0, 1.0, 0.0),
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
    let mut texture = texture_creator.create_texture_static(PixelFormat::RGB24, width, height).unwrap();
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
        fill_triangle(
            &mut bitmap,
            v1.transform(transform),
            v2.transform(transform),
            v3.transform(transform),
        );

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
        // Handle events
        match event_pump.poll_event() {
            Some(Event::Quit { .. }) | Some(Event::KeyDown { keycode: Some(Keycode::Escape), .. }) => {
                break 'running;
            }
            _ => {}
        }
    }
}
