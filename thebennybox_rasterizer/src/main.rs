//! Software rasterizer based on the amazing tutorial by 'thebennybox'
//! at https://www.youtube.com/playlist?list=PLEETnX-uPtBUbVOok816vTl1K9vV1GgH5

mod indexed_model;
mod matrix4f;
mod obj_model;
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

    fn copy_pixels(
        &mut self,
        dest_x: usize,
        dest_y: usize,
        src_x: usize,
        src_y: usize,
        src: &Bitmap,
        light_amount: f64,
    ) {
        let dest_index = (dest_y * self.width + dest_x) * 3;
        let src_index = (src_y * src.width + src_x) * 3;
        self.buffer[dest_index] = ((src.buffer[src_index] as f64) * light_amount) as u8;
        self.buffer[dest_index + 1] = ((src.buffer[src_index + 1] as f64) * light_amount) as u8;
        self.buffer[dest_index + 2] = ((src.buffer[src_index + 2] as f64) * light_amount) as u8;
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    pos: Vector4f,
    tex_coords: Vector4f,
    normal: Vector4f,
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

    fn transform(&self, transform: Matrix4f, normal_transform: Matrix4f) -> Self {
        Vertex {
            pos: transform.transform(self.pos),
            tex_coords: self.tex_coords,
            normal: normal_transform.transform(self.normal),
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
            normal: self.normal,
        }
    }

    #[must_use]
    fn lerp(self, other: Vertex, lerp_amount: f64) -> Self {
        Vertex {
            pos: self.pos.lerp(other.pos, lerp_amount),
            tex_coords: self.tex_coords.lerp(other.tex_coords, lerp_amount),
            normal: self.normal.lerp(other.normal, lerp_amount),
        }
    }

    fn get(self, index: usize) -> f64 {
        match index {
            0 => self.pos.x,
            1 => self.pos.y,
            2 => self.pos.z,
            3 => self.pos.w,
            _ => panic!("Invalid index"),
        }
    }

    fn is_inside_view_frustum(&self) -> bool {
        self.pos.w > 0.0
            && self.pos.x >= -self.pos.w
            && self.pos.x <= self.pos.w
            && self.pos.y >= -self.pos.w
            && self.pos.y <= self.pos.w
            && self.pos.z >= -self.pos.w
            && self.pos.z <= self.pos.w
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
    depth: f64,
    depth_step: f64,
    light_amount: f64,
    light_amount_step: f64,
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

        let depth = gradients.depth[start_index]
            + gradients.depth_x_step * x_prestep
            + gradients.depth_y_step * y_prestep;
        let depth_step = gradients.depth_y_step + gradients.depth_x_step * x_step;

        let light_amount = gradients.light_amount[start_index]
            + gradients.light_amount_x_step * x_prestep
            + gradients.light_amount_y_step * y_prestep;
        let light_amount_step = gradients.light_amount_y_step + gradients.light_amount_x_step * x_step;

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
            depth,
            depth_step,
            light_amount,
            light_amount_step,
        }
    }

    fn step(&mut self) {
        self.x += self.x_step;
        self.tex_coord_x += self.tex_coord_x_step;
        self.tex_coord_y += self.tex_coord_y_step;
        self.inv_z += self.inv_z_step;
        self.depth += self.depth_step;
        self.light_amount += self.light_amount_step;
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

fn fill_triangle(
    bitmap: &mut Bitmap,
    v1: Vertex,
    v2: Vertex,
    v3: Vertex,
    texture: &Bitmap,
    z_buffer: &mut Vec<f64>,
) {
    let screen_space_transform =
        Matrix4f::init_screen_space_transform((bitmap.width as f64) / 2.0, (bitmap.height as f64) / 2.0);
    let identity = Matrix4f::init_identity();
    let mut min_y = v1.transform(screen_space_transform, identity).perspective_divide();
    let mut mid_y = v2.transform(screen_space_transform, identity).perspective_divide();
    let mut max_y = v3.transform(screen_space_transform, identity).perspective_divide();

    if min_y.triangle_area_times_two(max_y, mid_y) >= 0.0 {
        return;
    }

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
    scan_triangle(bitmap, min_y, mid_y, max_y, short_is_left, texture, z_buffer);
}

fn draw_triangle(
    bitmap: &mut Bitmap,
    v1: Vertex,
    v2: Vertex,
    v3: Vertex,
    texture: &Bitmap,
    z_buffer: &mut Vec<f64>,
) {
    if v1.is_inside_view_frustum() && v2.is_inside_view_frustum() && v3.is_inside_view_frustum() {
        fill_triangle(bitmap, v1, v2, v3, texture, z_buffer);
        return;
    }

    let mut vertices = vec![v1, v2, v3];
    let mut auxillary_vec: Vec<Vertex> = Vec::new();

    if clip_polygon_axis(&mut vertices, &mut auxillary_vec, 0)
        && clip_polygon_axis(&mut vertices, &mut auxillary_vec, 1)
        && clip_polygon_axis(&mut vertices, &mut auxillary_vec, 2)
    {
        if vertices.len() < 3 {
            return;
        }

        let initial_vertex = vertices[0];
        for i in 1..vertices.len() - 1 {
            fill_triangle(
                bitmap,
                initial_vertex,
                vertices[i],
                vertices[i + 1],
                texture,
                z_buffer,
            );
        }
    }
}

fn scan_triangle(
    bitmap: &mut Bitmap,
    min_y: Vertex,
    mid_y: Vertex,
    max_y: Vertex,
    short_is_left: bool,
    texture: &Bitmap,
    z_buffer: &mut Vec<f64>,
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
        z_buffer,
    );
    scan_edges(
        bitmap,
        &mut top_to_bottom,
        &mut middle_to_bottom,
        short_is_left,
        texture,
        z_buffer,
    );
}

fn scan_edges(
    bitmap: &mut Bitmap,
    long: &mut Edge,
    short: &mut Edge,
    short_if_left: bool,
    texture: &Bitmap,
    z_buffer: &mut Vec<f64>,
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
        draw_scan_line(bitmap, left, right, j, texture, z_buffer);
        left.step();
        right.step();
    }
}

fn draw_scan_line(
    bitmap: &mut Bitmap,
    left: &mut Edge,
    right: &Edge,
    j: usize,
    texture: &Bitmap,
    z_buffer: &mut Vec<f64>,
) {
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
    let depth_x_step = if x_dist.abs() > f64::EPSILON {
        (right.depth - left.depth) / x_dist
    } else {
        0.0
    };
    let light_amount_x_step = if x_dist.abs() > f64::EPSILON {
        (right.light_amount - left.light_amount) / x_dist
    } else {
        0.0
    };

    let mut tex_coord_x = left.tex_coord_x + tex_coord_x_x_step * x_prestep;
    let mut tex_coord_y = left.tex_coord_y + tex_coord_y_x_step * x_prestep;
    let mut inv_z = left.inv_z + inv_z_x_step * x_prestep;
    let mut depth = left.depth + depth_x_step * x_prestep;
    let mut light_amount = left.light_amount + light_amount_x_step * x_prestep;

    for i in x_min..x_max {
        let index = i + j * bitmap.width;

        if depth < z_buffer[index] {
            z_buffer[index] = depth;
            let z = 1.0 / inv_z;
            let src_x = (((tex_coord_x * z) * (texture.width - 1) as f64) + 0.5) as usize;
            let src_y = (((tex_coord_y * z) * (texture.height - 1) as f64) + 0.5) as usize;

            bitmap.copy_pixels(i, j, src_x, src_y, texture, light_amount);
        }

        tex_coord_x += tex_coord_x_x_step;
        tex_coord_y += tex_coord_y_x_step;
        inv_z += inv_z_x_step;
        depth += depth_x_step;
        light_amount += light_amount_x_step;
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
    depth: [f64; 3],
    depth_x_step: f64,
    depth_y_step: f64,
    light_amount: [f64; 3],
    light_amount_x_step: f64,
    light_amount_y_step: f64,
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

        let depth0 = min_y.pos.z;
        let depth1 = mid_y.pos.z;
        let depth2 = max_y.pos.z;

        let light_direction = Vector4f::new(1.0, 0.0, 0.0, 0.0);
        let light_amount0 = min_y.normal.dot(light_direction).clamp(0.0, 1.0) * 0.9 + 0.1;
        let light_amount1 = mid_y.normal.dot(light_direction).clamp(0.0, 1.0) * 0.9 + 0.1;
        let light_amount2 = max_y.normal.dot(light_direction).clamp(0.0, 1.0) * 0.9 + 0.1;

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

        let depth_x_step = Self::calc_step_x(depth0, depth1, depth2, dy1, dy2, inv_dx);
        let depth_y_step = Self::calc_step_y(depth0, depth1, depth2, dx2, dx1, inv_dy);

        let light_amount_x_step =
            Self::calc_step_x(light_amount0, light_amount1, light_amount2, dy1, dy2, inv_dx);
        let light_amount_y_step =
            Self::calc_step_y(light_amount0, light_amount1, light_amount2, dx2, dx1, inv_dy);

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
            depth: [depth0, depth1, depth2],
            depth_x_step,
            depth_y_step,
            light_amount: [light_amount0, light_amount1, light_amount2],
            light_amount_x_step,
            light_amount_y_step,
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

fn load_texture_from_file<P>(file_path: P) -> Bitmap
where
    P: AsRef<std::path::Path>,
{
    let path = file_path.as_ref();
    let image = image::open(path).expect(&format!("Cannot load texture from file: {}", path.display()));

    let width = image.width() as usize;
    let height = image.height() as usize;

    let rgb_image = image.into_rgb8();
    let buffer = rgb_image.into_raw();

    Bitmap { width, height, buffer }
}

struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<usize>,
}

impl Mesh {
    fn new<P: AsRef<std::path::Path>>(file_name: P) -> Self {
        let model = obj_model::OBJModel::new(file_name).unwrap().to_indexed_model();

        let mut vertices: Vec<Vertex> = Vec::with_capacity(model.positions.len());
        for i in 0..model.positions.len() {
            vertices.push(Vertex {
                pos: model.positions[i],
                tex_coords: model.tex_coords[i],
                normal: model.normals[i],
            });
        }
        let indices = model.indices;

        Self { vertices, indices }
    }
}

fn draw_mesh(
    mesh: &Mesh,
    texture: &Bitmap,
    view_projection: Matrix4f,
    transform: Matrix4f,
    screen: &mut Bitmap,
    z_buffer: &mut Vec<f64>,
) {
    let mvp = view_projection.mul(transform);

    for chunk in mesh.indices.chunks(3) {
        draw_triangle(
            screen,
            mesh.vertices[chunk[0]].transform(mvp, transform),
            mesh.vertices[chunk[1]].transform(mvp, transform),
            mesh.vertices[chunk[2]].transform(mvp, transform),
            texture,
            z_buffer,
        );
    }
}

// Простой моноширинный шрифт 8x8 (только ASCII 32-126)
const FONT: [[u8; 8]; 95] = [
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // ' '
    [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00], // '!'
    [0x36, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00], // '"'
    [0x36, 0x36, 0x7F, 0x36, 0x7F, 0x36, 0x36, 0x00], // '#'
    [0x0C, 0x3E, 0x03, 0x1E, 0x30, 0x1F, 0x0C, 0x00], // '$'
    [0x1E, 0x33, 0x18, 0x0C, 0x18, 0x33, 0x1E, 0x00], // '%'
    [0x0C, 0x1E, 0x33, 0x33, 0x3F, 0x1C, 0x38, 0x00], // '&'
    [0x18, 0x18, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00], // '''
    [0x0E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x0E, 0x00], // '('
    [0x70, 0x18, 0x18, 0x18, 0x18, 0x18, 0x70, 0x00], // ')'
    [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00], // '*'
    [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00], // '+'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30], // ','
    [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00], // '-'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00], // '.'
    [0x06, 0x0C, 0x18, 0x30, 0x60, 0x40, 0x00, 0x00], // '/'
    [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // '0'
    [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00], // '1'
    [0x3C, 0x66, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00], // '2'
    [0x3C, 0x66, 0x0C, 0x38, 0x0C, 0x66, 0x3C, 0x00], // '3'
    [0x0C, 0x1C, 0x2C, 0x4C, 0x7C, 0x0C, 0x0C, 0x00], // '4'
    [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00], // '5'
    [0x38, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x3C, 0x00], // '6'
    [0x7C, 0x66, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00], // '7'
    [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00], // '8'
    [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x0C, 0x38, 0x00], // '9'
    [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00], // ':'
    [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x30], // ';'
    [0x00, 0x18, 0x30, 0x60, 0x30, 0x18, 0x00, 0x00], // '<'
    [0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00], // '='
    [0x00, 0x60, 0x30, 0x18, 0x30, 0x60, 0x00, 0x00], // '>'
    [0x3C, 0x66, 0x0C, 0x18, 0x30, 0x00, 0x30, 0x00], // '?'
    [0x3C, 0x66, 0x66, 0x66, 0x7E, 0x6C, 0x36, 0x00], // '@'
    [0x1C, 0x36, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00], // 'A'
    [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00], // 'B'
    [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00], // 'C'
    [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00], // 'D'
    [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x7E, 0x00], // 'E'
    [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x00], // 'F'
    [0x3C, 0x66, 0x60, 0x60, 0x66, 0x66, 0x3C, 0x0E], // 'G'
    [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00], // 'H'
    [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'I'
    [0x0F, 0x06, 0x06, 0x06, 0x06, 0x66, 0x3C, 0x00], // 'J'
    [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00], // 'K'
    [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00], // 'L'
    [0x63, 0x77, 0x7F, 0x7F, 0x63, 0x63, 0x63, 0x00], // 'M'
    [0x66, 0x6E, 0x76, 0x7E, 0x6E, 0x66, 0x66, 0x00], // 'N'
    [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // 'O'
    [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00], // 'P'
    [0x3C, 0x66, 0x66, 0x66, 0x6E, 0x6C, 0x3D, 0x00], // 'Q'
    [0x7C, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x00], // 'R'
    [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00], // 'S'
    [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00], // 'T'
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00], // 'U'
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00], // 'V'
    [0x63, 0x63, 0x63, 0x6B, 0x6B, 0x6B, 0x46, 0x00], // 'W'
    [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00], // 'X'
    [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00], // 'Y'
    [0x7E, 0x66, 0x30, 0x18, 0x0C, 0x66, 0x7E, 0x00], // 'Z'
    [0x3C, 0x60, 0x60, 0x60, 0x60, 0x60, 0x3C, 0x00], // '['
    [0x60, 0x30, 0x18, 0x0C, 0x18, 0x30, 0x60, 0x00], // '\'
    [0x3C, 0x06, 0x06, 0x06, 0x06, 0x06, 0x3C, 0x00], // ']'
    [0x18, 0x3C, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00], // '^'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E], // '_'
    [0x18, 0x0C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00], // '`'
    [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00], // 'a'
    [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x00], // 'b'
    [0x00, 0x00, 0x3C, 0x66, 0x60, 0x66, 0x3C, 0x00], // 'c'
    [0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3E, 0x00], // 'd'
    [0x00, 0x00, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00], // 'e'
    [0x1C, 0x30, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x00], // 'f'
    [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x7C], // 'g'
    [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00], // 'h'
    [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'i'
    [0x06, 0x00, 0x0E, 0x06, 0x06, 0x06, 0x06, 0x3C], // 'j'
    [0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x00], // 'k'
    [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00], // 'l'
    [0x00, 0x00, 0x6C, 0x7E, 0x7E, 0x66, 0x66, 0x00], // 'm'
    [0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00], // 'n'
    [0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00], // 'o'
    [0x00, 0x00, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60], // 'p'
    [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x06], // 'q'
    [0x00, 0x00, 0x7C, 0x66, 0x60, 0x60, 0x60, 0x00], // 'r'
    [0x00, 0x00, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x00], // 's'
    [0x10, 0x30, 0x30, 0x7C, 0x30, 0x30, 0x1C, 0x00], // 't'
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3E, 0x00], // 'u'
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00], // 'v'
    [0x00, 0x00, 0x66, 0x66, 0x6E, 0x6E, 0x46, 0x00], // 'w'
    [0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00], // 'x'
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x7C], // 'y'
    [0x00, 0x00, 0x7E, 0x18, 0x30, 0x60, 0x7E, 0x00], // 'z'
    [0x1C, 0x30, 0x30, 0x60, 0x30, 0x30, 0x1C, 0x00], // '{'
    [0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x00], // '|'
    [0x70, 0x30, 0x30, 0x18, 0x30, 0x30, 0x70, 0x00], // '}'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // '~'
];

fn draw_char(bitmap: &mut Bitmap, ch: char, x: usize, y: usize, r: u8, g: u8, b: u8) {
    let idx = (ch as u8).saturating_sub(32) as usize;
    if idx >= FONT.len() {
        return;
    }
    let glyph = &FONT[idx];
    for row in 0..8 {
        let byte = glyph[row];
        for col in 0..8 {
            if (byte >> (7 - col)) & 1 == 1 {
                let px = x + col;
                let py = y + row;
                if px < bitmap.width && py < bitmap.height {
                    let index = (py * bitmap.width + px) * 3;
                    bitmap.buffer[index] = r;
                    bitmap.buffer[index + 1] = g;
                    bitmap.buffer[index + 2] = b;
                }
            }
        }
    }
}

fn draw_string(bitmap: &mut Bitmap, text: &str, x: usize, y: usize, r: u8, g: u8, b: u8) {
    let mut cx = x;
    for ch in text.chars() {
        draw_char(bitmap, ch, cx, y, r, g, b);
        cx += 8; // шаг по ширине
        if cx >= bitmap.width {
            break;
        }
    }
}

fn clip_polygon_component(
    vertices: &Vec<Vertex>,
    component_index: usize,
    component_factor: f64,
    result: &mut Vec<Vertex>,
) {
    let mut previous_vertex = vertices[vertices.len() - 1];
    let mut previous_component = previous_vertex.get(component_index) * component_factor;
    let mut previous_inside = previous_component <= previous_vertex.pos.w;

    for vertex in vertices {
        let current_vertex = *vertex;
        let current_component = current_vertex.get(component_index) * component_factor;
        let current_inside = current_component <= current_vertex.pos.w;

        if current_inside ^ previous_inside {
            let a = previous_vertex.pos.w - previous_component;
            let b = current_vertex.pos.w - current_component;
            let denom = a - b;
            let lerp_amount = if denom.abs() > f64::EPSILON {
                (a / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            result.push(previous_vertex.lerp(current_vertex, lerp_amount));
        }

        if current_inside {
            result.push(current_vertex);
        }

        previous_vertex = current_vertex;
        previous_component = current_component;
        previous_inside = current_inside;
    }
}

fn clip_polygon_axis(
    vertices: &mut Vec<Vertex>,
    auxillary_vec: &mut Vec<Vertex>,
    component_index: usize,
) -> bool {
    clip_polygon_component(vertices, component_index, 1.0, auxillary_vec);
    vertices.clear();

    if auxillary_vec.is_empty() {
        return false;
    }

    clip_polygon_component(auxillary_vec, component_index, -1.0, vertices);
    auxillary_vec.clear();

    !vertices.is_empty()
}

fn main() {
    let width: u32 = 900;
    let height: u32 = 900;
    let mut bitmap = Bitmap::new(width, height);
    let mut z_buffer = vec![std::f64::MAX; (width * height) as usize];
    let mut fps = 0;

    // let texture = create_random_texture(32, 32);
    // let texture = load_texture_from_file("resources/bricks.jpg");
    // let texture = load_texture_from_file("resources/simpbricks.png");
    let texture = load_texture_from_file("resources/bricks2.jpg");
    // let mesh = Mesh::new("resources/icosphere.obj");
    // let mesh = Mesh::new("resources/monkey2.obj");
    let mesh = Mesh::new("resources/smoothMonkey2.obj");
    println!(
        "Vertices: {}, polygons: {}",
        mesh.vertices.len(),
        mesh.indices.len() / 3
    );

    // let v1 = Vertex {
    //     pos: Vector4f::new(-1.0, -1.0, 0.0, 1.0),
    //     tex_coords: Vector4f::new(0.0, 0.0, 0.0, 0.0), // UV: (0,0)
    // };
    // let v2 = Vertex {
    //     pos: Vector4f::new(0.0, 1.0, 0.0, 1.0),
    //     tex_coords: Vector4f::new(0.5, 1.0, 0.0, 0.0), // UV: (0.5,1)
    // };
    // let v3 = Vertex {
    //     pos: Vector4f::new(1.0, -1.0, 0.0, 1.0),
    //     tex_coords: Vector4f::new(1.0, 0.0, 0.0, 0.0), // UV: (1,0)
    // };

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
        let translation = Matrix4f::init_translation(0.0, 0.0, 3.0 - 3.0 * rotation_counter.sin());
        let rotation = Matrix4f::init_rotation_euler(rotation_counter, 0.0, rotation_counter);
        let transform = translation.mul(rotation);

        bitmap.clear(0);
        z_buffer.fill(std::f64::MAX);

        draw_mesh(&mesh, &texture, projection, transform, &mut bitmap, &mut z_buffer);
        // fill_triangle(
        //     &mut bitmap,
        //     v1.transform(transform),
        //     v2.transform(transform),
        //     v3.transform(transform),
        //     &texture,
        // );

        draw_string(&mut bitmap, "THEBENNYBOX SOFTWARE RASTERIZER", 10, 10, 0, 255, 0); // Зелёный

        let fps_text = format!("FPS: {}", fps);
        draw_string(&mut bitmap, &fps_text, 10, 30, 255, 255, 255); // Белый текст

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
            fps = frame_count;
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
