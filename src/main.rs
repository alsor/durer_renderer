extern crate image;
extern crate sdl2;
extern crate mpeg_encoder;
extern crate rand;

mod ray_tracing;
mod vectors;
mod projective_camera;
mod buffer_canvas;
mod model;
mod instance;
mod ply2;
mod matrix44f;
mod vector4f;
mod plane;
mod texture;
mod uv;

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;
use std::f64;
use std::io::prelude::*;
use std::str::FromStr;

use ray_tracing::Sphere;
use buffer_canvas::BufferCanvas;
use projective_camera::ProjectiveCamera;
use model::Model;
use vector4f::Vector4f;
use texture::Texture;
use self::rand::Rng;

#[derive(Copy, Clone)]
pub struct Point3D { x: f64, y: f64, z: f64 }

impl Point3D {
    pub fn from_vec(vec: [f64; 3]) -> Self {
        Point3D { x: vec[0], y: vec[1], z: vec[2] }
    }

    pub fn to_vec(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    pub fn from_vector4f(vector: Vector4f) -> Self {
        Point3D { x: vector.x, y: vector.y, z: vector.z }
    }

    pub fn to_vector4f(&self) -> Vector4f {
        Vector4f { x: self.x, y: self.y, z: self.z, w: 1.0 }
    }
}

#[derive(Copy, Clone)]
struct Point2D { x: f64, y: f64 }

#[derive(Copy, Clone)]
struct Point { x: i32, y: i32, h: f64, z: f64 }

#[derive(Copy, Clone)]
struct Frame { x_min: f64, x_max: f64, y_min: f64, y_max: f64 }

#[derive(Copy, Clone)]
pub struct Color {
    r: u8, g: u8, b: u8
}

#[derive(Copy, Clone)]
pub struct Pixel {
    pub x: usize,
    pub y: usize,
    pub color: Color
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub indexes: [usize; 3],
    pub normals: [Point3D; 3],
    pub calculated_normal: Point3D
}

impl Triangle {
    pub fn new_with_calculated_normals(vertices: &Vec<Point3D>, indexes: [usize; 3]) -> Self {
        let calculated_normal = vectors::normalize(Self::calculate_normal_in_left(
            indexes,
            vertices
        ));

        Self {
            indexes,
            normals: [calculated_normal, calculated_normal, calculated_normal],
            calculated_normal
        }
    }

    pub fn new_with_provided_normals(
        vertices: &Vec<Point3D>,
        indexes: [usize; 3],
        normal_directions: [Point3D; 3]
    )
    -> Self {
        let calculated_normal = vectors::normalize(Self::calculate_normal_in_left(
            indexes,
            vertices
        ));

        let normals= [
            vectors::normalize(normal_directions[0]),
            vectors::normalize(normal_directions[1]),
            vectors::normalize(normal_directions[2]),
        ];

        Self { indexes, normals, calculated_normal }
    }

    fn calculate_normal_in_left(indexes: [usize; 3], vertices: &Vec<Point3D>) -> Point3D {
        let vector1 = vectors::difference(vertices[indexes[2]], vertices[indexes[1]]);
        let vector2 = vectors::difference(vertices[indexes[1]], vertices[indexes[0]]);
        vectors::cross_product(vector2, vector1)
    }

}

#[test]
fn test_new_with_calculated_normal() {
    let triangle = Triangle::new_with_calculated_normals(
        &vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 5.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 5.0 },
        ],
        [2, 1, 0]
    );
    assert!(::tests::roughly_equals(triangle.normals[0].x, 0.0));
    assert!(::tests::roughly_equals(triangle.normals[0].y, 1.0));
    assert!(::tests::roughly_equals(triangle.normals[0].z, 0.0));

    println!("normal: {:.2} {:.2} {:.2}", triangle.normals[0].x, triangle.normals[0].y, triangle.normals[0].z);
}


#[derive(Copy, Clone)]
pub struct Triangle4f {
    pub a: Vector4f,
    pub b: Vector4f,
    pub c: Vector4f,
    pub color: Color,
    pub normals: [Point3D; 3]
}

#[derive(Copy, Clone)]
pub enum Light {
    Ambient { intensity: f64 },
    Point { intensity: f64, position: Point3D },
    Directional { intensity: f64, direction: Point3D }
}

#[derive(Copy, Clone)]
pub enum ShadingModel {
    Flat, Gouraud, Phong
}

pub struct RenderingSettings {
    pub shading_model: ShadingModel,
    pub show_normals: bool,
    pub backface_culling: bool
}

fn project(point3d: Point3D) -> Point2D {
    Point2D { x: -point3d.x / point3d.z, y: -point3d.y / point3d.z }
}

fn normalize(point2d: Point2D, frame: Frame) -> Point2D {
    Point2D {
        x: 1.0 - (point2d.x - frame.x_min) / (frame.x_max - frame.x_min),
        y: (point2d.y - frame.y_min) / (frame.y_max - frame.y_min)
    }
}

fn rasterize(point: Point2D, size: usize) -> Pixel {
    let x = (size as f64 * point.x) as usize;
    let y = (size as f64 * point.y) as usize;
    Pixel { x, y, color: Color { r: 255, g: 255, b: 255 } }
}

pub fn rasterize_line(start: Pixel, end: Pixel, buffer: &mut [u8], size: usize) {
    let x1 = start.x as i32;
    let y1 = start.y as i32;
    let x2 = end.x as i32;
    let y2 = end.y as i32;

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x2 >= x1 {
        1
    } else {
        -1
    };
    let sy = if y2 >= y1 {
        1
    } else {
        -1
    };

    if dy <= dx {
        let mut d = (dy << 1) - dx;
        let d1 = dy << 1;
        let d2 = (dy - dx) << 1;

        put_pixel(start, buffer, size);

        let mut x = x1 + sx;
        let mut y = y1;
        for i in 1..dx {
            if d > 0 {
                d = d + d2;
                y = y + sy;
            } else {
                d = d + d1;
            }

            put_pixel(
                Pixel { x: x as usize, y: y as usize, color: start.color },
                buffer,
                size
            );

            x = x + sx;
        }
    } else {
        let mut d = (dx << 1) - dy;
        let d1 = dx << 1;
        let d2 = (dx - dy) << 1;

        put_pixel(start, buffer, size);

        let mut x = x1;
        let mut y = y1 + sy;
        for i in 1..dy {
            if d > 0 {
                d = d + d2;
                x = x + sx;
            } else {
                d = d + d1;
            }

            put_pixel(
                Pixel { x: x as usize, y: y as usize, color: start.color },
                buffer,
                size
            );

            y = y + sy;
        }
    }
}

fn simple_line(start: Pixel, end: Pixel, buffer: &mut [u8], size: usize) {
    let xa = start.x as i32;
    let ya = start.y as i32;
    let xb = end.x as i32;
    let yb = end.y as i32;

    let k = (yb - ya) as f64 / (xb - xa) as f64;
    let b = ya as f64 - k * xa as f64;

    for x in xa..xb {
        let y = (k * x as f64 + b) as i32;
        put_pixel(
            Pixel { x: x as usize, y: y as usize, color: Color { r: 255, g: 255, b: 255 } },
            buffer,
            size
        );
    }
}

fn put_pixel(pixel: Pixel, buffer: &mut [u8], size: usize) {
    let offset = pixel.y * size * 3 + pixel.x * 3;
    buffer[offset] = pixel.color.r;
    buffer[offset + 1] = pixel.color.g;
    buffer[offset + 2] = pixel.color.b;
}

fn write_image(buffer: &[u8], size: usize) -> Result<(), std::io::Error> {
    let output = File::create("target/result.png")?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&buffer, size as u32, size as u32, ColorType::RGB(8))?;

    Ok(())
}

fn face_visible(face: &Vec<i32>, vertices: &[Point3D]) -> bool {
    let vector1 = vectors::difference(vertices[face[2] as usize], vertices[face[1] as usize]);
    let vector2 = vectors::difference(vertices[face[1] as usize], vertices[face[0] as usize]);
    let face_vector = vectors::cross_product(
        vector1,
        vector2
    );

    vectors::dot_product(vertices[face[0] as usize], face_vector) < 0.0
}

fn face_visible2(face: &Vec<i32>, vertices: &[Point3D]) -> bool {
    let vector1 = vectors::difference(vertices[face[2] as usize], vertices[face[1] as usize]);
    let vector2 = vectors::difference(vertices[face[1] as usize], vertices[face[0] as usize]);
    let face_vector = vectors::cross_product(
        vector2,
        vector1
    );

    vectors::dot_product(vertices[face[0] as usize], face_vector) < 0.0
}

fn face_visible_4f(vertex: Point3D, normal_direction: Point3D) -> bool {
    vectors::dot_product(vertex, normal_direction) < 0.0
}

fn draw_face(face: &Vec<i32>,
             vertex_pixels: &Vec<Pixel>,
             buffer: &mut [u8],
             size: usize) {
    for vertex_index in 0..face.len() {
        let start_vertex;
        let end_vertex;
        if vertex_index + 1 < face.len() {
            start_vertex = vertex_index;
            end_vertex = vertex_index + 1;
        } else {
            start_vertex = vertex_index;
            end_vertex = 0;
        }
        rasterize_line(
            vertex_pixels[face[start_vertex] as usize],
            vertex_pixels[face[end_vertex] as usize],
            buffer, size
        );
    }
}

fn transform(vertices: &Vec<Point3D>, vector: Point3D) -> Vec<Point3D> {
    let mut result = Vec::new();
    for i in 0..vertices.len() {
        result.push(
            Point3D {
                x: vertices[i].x + vector.x,
                y: vertices[i].y + vector.y,
                z: vertices[i].z + vector.z
            }
        );
    }
    result
}

fn rotated_cube_vertices(t: f64) -> [Point3D; 8] {
    let radius = (2.0 as f64).sqrt() / 2.0;
    [
        // 0
        Point3D {
            x: (5.0 * f64::consts::PI / 4.0 + t).cos() * radius,
            y: -0.5,
            z: (5.0 * f64::consts::PI / 4.0 + t).sin() * radius
        },
        // 1
        Point3D {
            x: (5.0 * f64::consts::PI / 4.0 + t).cos() * radius,
            y: 0.5,
            z: (5.0 * f64::consts::PI / 4.0 + t).sin() * radius
        },
        // 2
        Point3D {
            x: (7.0 * f64::consts::PI / 4.0 + t).cos() * radius,
            y: 0.5,
            z: (7.0 * f64::consts::PI / 4.0 + t).sin() * radius
        },
        // 3
        Point3D {
            x: (7.0 * f64::consts::PI / 4.0 + t).cos() * radius,
            y: -0.5,
            z: (7.0 * f64::consts::PI / 4.0 + t).sin() * radius
        },
        // 4
        Point3D {
            x: (3.0 * f64::consts::PI / 4.0 + t).cos() * radius,
            y: -0.5,
            z: (3.0 * f64::consts::PI / 4.0 + t).sin() * radius
        },
        // 5
        Point3D {
            x: (3.0 * f64::consts::PI / 4.0 + t).cos() * radius,
            y: 0.5,
            z: (3.0 * f64::consts::PI / 4.0 + t).sin() * radius
        },
        // 6
        Point3D {
            x: (f64::consts::PI / 4.0 + t).cos() * radius,
            y: 0.5,
            z: (f64::consts::PI / 4.0 + t).sin() * radius
        },
        // 7
        Point3D {
            x: (f64::consts::PI / 4.0 + t).cos() * radius,
            y: -0.5,
            z: (f64::consts::PI / 4.0 + t).sin() * radius
        },
    ]
}

fn rotated_cube(t: f64) -> (Vec<Point3D>, Vec<Vec<i32>>) {
    let cube_vertices = rotated_cube_vertices(t).to_vec();

    let vertices = transform(&cube_vertices, Point3D { x: 0.0, y: 0.0, z: 5.0 });

    let faces = vec![
        vec![0, 3, 2, 1],
        vec![3, 7, 6, 2],
        vec![7, 4, 5, 6],
        vec![4, 0, 1, 5],
        vec![0, 4, 7, 3],
        vec![1, 2, 6, 5],
    ];

    (vertices, faces)
}

fn triangle<'a>(size: f64) -> Model<'a> {
    let half_size = size / 2.0;

    let vertices = vec![
        Point3D { x: half_size, y: half_size, z: 0.0 },
        Point3D { x: -half_size, y: half_size, z: 0.0 },
        Point3D { x: -half_size, y: -half_size, z: 0.0 },
    ];

    let triangles = vec![
        Triangle::new_with_calculated_normals(&vertices, [2, 1, 0])
    ];

    let mut rng = rand::thread_rng();
//    let colors = vec![
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//    ];
    let colors = vec![
        Color { r: 119, g: 136, b: 153 },
    ];

    Model { vertices, triangles, colors, textures: None, uvs: None }
}

fn cube<'a>(size: f64) -> Model<'a> {
    let half_size = size / 2.0;

    let vertices = vec![
        Point3D { x: half_size, y: half_size, z: half_size },
        Point3D { x: -half_size, y: half_size, z: half_size },
        Point3D { x: -half_size, y: -half_size, z: half_size },
        Point3D { x: half_size, y: -half_size, z: half_size },
        Point3D { x: half_size, y: half_size, z: -half_size },
        Point3D { x: -half_size, y: half_size, z: -half_size },
        Point3D { x: -half_size, y: -half_size, z: -half_size },
        Point3D { x: half_size, y: -half_size, z: -half_size },
    ];

    let triangles = vec![
        Triangle::new_with_calculated_normals(&vertices, [0, 1, 2]),
        Triangle::new_with_calculated_normals(&vertices, [0, 2, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 0, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 3, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 4, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 7, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 5, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 6, 2]),
        Triangle::new_with_calculated_normals(&vertices, [4, 5, 1]),
        Triangle::new_with_calculated_normals(&vertices, [4, 1, 0]),
        Triangle::new_with_calculated_normals(&vertices, [2, 6, 7]),
        Triangle::new_with_calculated_normals(&vertices, [2, 7, 3]),
    ];

    let mut rng = rand::thread_rng();
//    let colors = vec![
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//        Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
//    ];
    let colors = vec![
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
    ];

    Model { vertices, triangles, colors, textures: None, uvs: None }
}

fn textured_cube(size: f64, texture: &Texture) -> Model {
    let half_size = size / 2.0;

    let vertices = vec![
        Point3D { x: half_size, y: half_size, z: half_size },
        Point3D { x: -half_size, y: half_size, z: half_size },
        Point3D { x: -half_size, y: -half_size, z: half_size },
        Point3D { x: half_size, y: -half_size, z: half_size },
        Point3D { x: half_size, y: half_size, z: -half_size },
        Point3D { x: -half_size, y: half_size, z: -half_size },
        Point3D { x: -half_size, y: -half_size, z: -half_size },
        Point3D { x: half_size, y: -half_size, z: -half_size },
    ];

    let triangles = vec![
        Triangle::new_with_calculated_normals(&vertices, [0, 1, 2]),
        Triangle::new_with_calculated_normals(&vertices, [0, 2, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 0, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 3, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 4, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 7, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 5, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 6, 2]),
        Triangle::new_with_calculated_normals(&vertices, [1, 0, 5]),
        Triangle::new_with_calculated_normals(&vertices, [5, 0, 4]),
        Triangle::new_with_calculated_normals(&vertices, [2, 6, 7]),
        Triangle::new_with_calculated_normals(&vertices, [2, 7, 3]),
    ];

    let colors = vec![
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
    ];

    let textures = vec![
        texture,
        texture,
        texture,
        texture,
        texture,
        texture,
        texture,
        texture,
        texture,
        texture,
        texture,
        texture,
    ];

    let uvs = vec![
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 1.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 0.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
    ];

    Model { vertices, triangles, colors, textures: Some(textures), uvs: Some(uvs) }
}

fn sphere<'a>(divs: i32) -> Model<'a> {
    let mut vertices= Vec::<Point3D>::new();
    let mut triangles = Vec::<Triangle>::new();
    let mut colors = Vec::<Color>::new();

    let delta_angle = 2.0 * f64::consts::PI / (divs as f64);

    // generate vertices
    for d in 0..(divs + 1) {
        let y = (2.0 / (divs as f64)) * ((d as f64) - (divs as f64) / 2.0);
        let radius = (1.0 - y * y).sqrt();

        for i in 0..divs {
            let x = radius * ((i as f64) * delta_angle).cos();
            let z = radius * ((i as f64) * delta_angle).sin();
            vertices.push(Point3D { x, y, z });
//            println!("generated vertex: {:.2}, {:.2}, {:.2}", x, y, z)
        }
    }

    // generate triangles
    for d in 0..divs {
        for i in 0..(divs - 1) {
            let i0 = d * divs + i;

            triangles.push(Triangle::new_with_provided_normals(
                &vertices,
                [i0 as usize, (i0 + divs + 1) as usize, (i0 + 1) as usize],
                [vertices[i0 as usize], vertices[(i0 + divs + 1) as usize], vertices[(i0 + 1) as usize]]
            ));
            colors.push(Color { r: 119, g: 136, b: 153 });

            triangles.push(Triangle::new_with_provided_normals(
                &vertices,
                [i0 as usize, (i0 + divs) as usize, (i0 + divs + 1) as usize],
                [vertices[i0 as usize], vertices[(i0 + divs) as usize], vertices[(i0 + divs + 1) as usize]]
            ));
            colors.push(Color { r: 119, g: 136, b: 153 });
        }
    }

    Model { vertices, triangles, colors, textures: None, uvs: None }
}

fn two_unit_cube<'a>() -> Model<'a> {
    cube(2.0)
}

fn enclosing_frame(vertices: &Vec<Point3D>) -> Frame {
    let mut x_min = 0.0;
    let mut x_max = 0.0;
    let mut y_min = 0.0;
    let mut y_max = 0.0;
    for v in vertices {
        if v.x < x_min {
            x_min = v.x;
        }
        if v.x > x_max {
            x_max = v.x;
        }
        if v.y < y_min {
            y_min = v.y;
        }
        if v.y > y_max {
            y_max = v.y;
        }
    }

    let size_x = (x_max - x_min).abs();
    let size_y = (y_max - y_min).abs();
    let size = if size_x > size_y {
        size_x
    } else {
        size_y
    };


    println!("calculated size {:.2}", size);

    let margin = size * 0.0;
    println!("calculated margin {:.2}", margin);

    let half = size / 3.0 + margin;
//    let margin = 0.0;

//    Frame {
//        x_min: x_min - margin,
//        x_max: x_max + margin,
//        y_min: y_min - margin,
//        y_max: y_max + margin
//    }
    Frame {
        x_min: -half,
        x_max: half,
        y_min: -half,
        y_max: half
    }
}

fn find_z_transform(vertices: &Vec<Point3D>) -> f64 {
    let mut z_min = 0.0;
    let mut z_max = 0.0;

    for v in vertices {
        if v.z < z_min {
            z_min = v.z;
        }
        if v.z > z_max {
            z_max = v.z;
        }
    }

    let size = (z_max - z_min).abs();

    1.0 - z_min
}

fn render_model_to_buffer(
    buffer: &mut [u8],
    size: usize,
    vertices: Vec<Point3D>,
    faces: Vec<Vec<i32>>,
    frame: Frame) {

    //
    // y
    // ^
    // |
    // |
    //  ------> x
    //
    // z - deeper into the screen
    //

    let mut vertex_pixels = Vec::new();

    for point3d in &vertices {
        let point2d = project(*point3d);
        let norm_point = normalize(point2d, frame);
        //        println!("normalized Point2D: {:.2} {:.2}", norm_point.x, norm_point.y);
        let pixel = rasterize(norm_point, size);
        //        println!("Pixel: {:?} {:?}", pixel.x, pixel.y);
        vertex_pixels.push(pixel);
    };

    // line from eye to vertex
//    rasterize_line(
//        rasterize(normalize(project(Point3D { x: 0.0, y: 0.0, z: 1.1 }), frame), size),
//        rasterize(normalize(project(vertices[3]), frame), size),
//        &mut buffer,
//        size
//    );

    // draw faces
    for face in &faces {
        if face_visible(face, &vertices) {
            draw_face(face, &vertex_pixels, buffer, size);
        };
    };

    //        simple_line(Pixel { x: 60, y: 40 }, Pixel { x: 120, y: 50 }, &mut buffer, size);
    //    simple_line(Pixel { x: 120, y: 50 }, Pixel { x: 60, y: 40 }, &mut buffer, size);
    //    rasterize_line(Pixel { x: 60, y: 50 }, Pixel { x: 120, y: 60 }, &mut buffer, size);
}

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn show_buffer_in_window(buffer: &mut [u8], size: usize) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Durer", size as u32, size as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    //    let mut texture = texture_creator.create_texture_streaming(
    //        PixelFormatEnum::RGB24, 256, 256
    //    ).unwrap();

    //    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //        for y in 0..256 {
    //            for x in 0..256 {
    //                let offset = y * pitch + x * 3;
    //                buffer[offset] = x as u8;
    //                buffer[offset + 1] = y as u8;
    //                buffer[offset + 2] = 0;
    //            }
    //        }
    //    }).unwrap();

    let mut texture = texture_creator.create_texture_static(
        PixelFormatEnum::RGB24,
        size as u32,
        size as u32
    ).unwrap();

    texture.update(None, &buffer, size * 3).unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    //    canvas.copy_ex(&texture, None, Some(Rect::new(450, 100, 256, 256)), 30.0, None, false, false).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
    }
}

fn rotating_cube_window(buffer: &mut [u8], size: usize) {
    let mut t = 0.8;
    let (vertices, faces) = rotated_cube(t);

    // Z-transform
    let z_transform = find_z_transform(&vertices);
    println!("calculated z transform: {:.2}", z_transform);
    let z_transform = 2.0;
    let mut transform_vector = Point3D { x: 0.0, y: -1.0, z: z_transform };

    let vertices = transform(&vertices, transform_vector);

    // Frame
    let frame = enclosing_frame(&vertices);
    println!("calculated frame: {:.2} {:.2} {:.2} {:.2}",
             frame.x_min, frame.x_max, frame.y_min, frame.y_max);
    let half = 0.3;
    let frame = Frame { x_min: -half, x_max: half, y_min: -half, y_max: half };


    render_model_to_buffer(buffer, size, vertices, faces, frame);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Durer", size as u32, size as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    //    let mut texture = texture_creator.create_texture_streaming(
    //        PixelFormatEnum::RGB24, 256, 256
    //    ).unwrap();

    //    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //        for y in 0..256 {
    //            for x in 0..256 {
    //                let offset = y * pitch + x * 3;
    //                buffer[offset] = x as u8;
    //                buffer[offset + 1] = y as u8;
    //                buffer[offset + 2] = 0;
    //            }
    //        }
    //    }).unwrap();

    let mut texture = texture_creator.create_texture_static(
        PixelFormatEnum::RGB24,
        size as u32,
        size as u32
    ).unwrap();

    texture.update(None, buffer, size * 3).unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    //    canvas.copy_ex(&texture, None, Some(Rect::new(450, 100, 256, 256)), 30.0, None, false, false).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                    t += 0.1;
                },
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    t -= 0.1;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    transform_vector.x += 0.1;
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    transform_vector.x -= 0.1;
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    transform_vector.y += 0.1;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    transform_vector.y -= 0.1;
                },
                Event::KeyDown { keycode: Some(Keycode::Comma), .. } => {
                    transform_vector.z += 0.1;
                },
                Event::KeyDown { keycode: Some(Keycode::Period), .. } => {
                    transform_vector.z -= 0.1;
                },
                _ => {}
            }

            let mut buffer = vec![0u8; size as usize * size as usize * 3];
            let (vertices, faces) = rotated_cube(t);
            let vertices = transform(&vertices, transform_vector);

            render_model_to_buffer(&mut buffer, size, vertices, faces, frame);

            texture.update(None, &buffer, size * 3).unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
    }
}

fn three_spheres_window(buffer: &mut [u8], size: usize) {
    let mut x_position = 0.0;
    let mut z_position = 0.0;

    let mut angle = 0.0;

    let origin = Point3D { x: x_position, y: 0.0, z: z_position };
    let rotation = vectors::rotation_around_y(angle);

    let spheres = vec![
        Sphere {
            center: Point3D { x: 0.0, y: -1.0, z: 3.0 },
            radius: 1.0,
            color: Color { r: 255, g: 0, b: 0 },
            specular: 200,
            reflective: 0.0
        },
        Sphere {
            center: Point3D { x: -2.0, y: 0.0, z: 4.0 },
            radius: 1.0,
            color: Color { r: 150, g: 150, b: 150 },
            specular: 200,
            reflective: 0.5
        },
        Sphere {
            center: Point3D { x: 2.0, y: 0.0, z: 4.0 },
            radius: 1.0,
            color: Color { r: 0, g: 0, b: 255 },
            specular: 200,
            reflective: 0.3
        },
        Sphere {
            center: Point3D { x: 0.0, y: -5001.0, z: 0.0 },
            radius: 5000.0,
            color: Color { r: 100, g: 100, b: 0 },
            specular: 0,
            reflective: 0.0
        },
    ];

    let lights = vec![
        Light::Ambient { intensity: 0.2 },
        Light::Directional {
            intensity: 0.8,
            direction: Point3D { x: 1.0, y: 4.0, z: 4.0 }
        }
    ];

    ray_tracing::render_scene_to_buffer(
        &spheres,
        &lights,
        buffer,
        size,
        origin,
        rotation
    );

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Durer", size as u32, size as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    //    let mut texture = texture_creator.create_texture_streaming(
    //        PixelFormatEnum::RGB24, 256, 256
    //    ).unwrap();

    //    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //        for y in 0..256 {
    //            for x in 0..256 {
    //                let offset = y * pitch + x * 3;
    //                buffer[offset] = x as u8;
    //                buffer[offset + 1] = y as u8;
    //                buffer[offset + 2] = 0;
    //            }
    //        }
    //    }).unwrap();

    let mut texture = texture_creator.create_texture_static(
        PixelFormatEnum::RGB24,
        size as u32,
        size as u32
    ).unwrap();

    texture.update(None, buffer, size * 3).unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    //    canvas.copy_ex(&texture, None, Some(Rect::new(450, 100, 256, 256)), 30.0, None, false, false).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                    angle += 10.0;
                },
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    angle -= 10.5;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    x_position += 0.5;
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    x_position -= 0.5;
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    z_position += 0.5;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    z_position -= 0.5;
                },
                _ => {}
            }

            let mut buffer = vec![0u8; size as usize * size as usize * 3];
            let origin = Point3D { x: x_position, y: 0.0, z: z_position };
            let rotation = vectors::rotation_around_y(angle);

            ray_tracing::render_scene_to_buffer(
                &spheres,
                &lights,
                &mut buffer,
                size,
                origin,
                rotation
            );

            texture.update(None, &buffer, size * 3).unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
    }
}

fn render_video() {
//    let mut encoder = mpeg_encoder::Encoder::new("target/shperes.mpeg", size, size);


    // scene goes here



//        encoder.encode_rgb(size, size, &buffer, false);

}

fn draw_wireframe_triangle(
    p0: Point,
    p1: Point,
    p2: Point,
    color: Color,
    canvas: &mut BufferCanvas
) {
    canvas.draw_line(p0, p1, color);
    canvas.draw_line(p1, p2, color);
    canvas.draw_line(p2, p0, color);
}


pub fn screen_x(x_canvas: i32, canvas_width: i32) -> usize {
    (canvas_width / 2 + x_canvas) as usize
}

pub fn screen_y(y_canvas: i32, canvas_height: i32) -> usize {
    (canvas_height / 2 - y_canvas - 1) as usize
}

//fn point_to_pixel(point: Point, color: Color, size: usize) -> Pixel {
//    let canvas_width = size as i32;
//    let canvas_height = size as i32;
//    Pixel {
//        x: screen_x(point.x, canvas_width),
//        y: screen_y(point.y, canvas_height),
//        color
//    }
//}
//
//fn draw_point(point: Point, color: Color, buffer: &mut Vec<u8>, size: usize) {
//    put_pixel(point_to_pixel(point, color, size), buffer, size);
//}

//fn draw_line(start: Point, end: Point, color: Color, buffer: &mut [u8], size: usize) {
//    rasterize_line(
//        point_to_pixel(start, color, size),
//        point_to_pixel(end, color, size),
//        buffer,
//        size
//    );
//}

//fn draw_wireframe_triangle(
//    p0: Point,
//    p1: Point,
//    p2: Point,
//    color: Color,
//    buffer: &mut [u8],
//    size: usize,
//) {
//    draw_line(p0, p1, color, buffer, size);
//    draw_line(p1, p2, color, buffer, size);
//    draw_line(p2, p0, color, buffer, size);
//}

use instance::Instance;
use matrix44f::Matrix44f;
mod rendering;
use std::{thread, time};
use plane::Plane;
use uv::UV;

#[macro_use] extern crate log;
extern crate env_logger;

fn main() {
    env_logger::init();

    let mut rendering_settings = RenderingSettings {
        shading_model: ShadingModel::Phong,
        show_normals: false,
        backface_culling: true
    };
    let mut buffer_canvas = BufferCanvas::new(750);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window(
        "Durer",
        buffer_canvas.size as u32,
        buffer_canvas.size as u32,
    ).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_static(
        PixelFormatEnum::RGB24,
        buffer_canvas.size as u32,
        buffer_canvas.size as u32
    ).unwrap();

    texture.update(None, &buffer_canvas.buffer, buffer_canvas.size * 3).unwrap();
    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    let viewport_size_delta = 0.1;
    let mut viewport_size = 1.0;
    let projection_plane_z_delta = 0.1;
    let mut projection_plane_z = 1.0;
    let mut x_position = 0.0;
    let mut y_position = 0.0;
    let mut z_position = 0.0;
    let mut angle = 0.0;

    let red = Color { r: 255, g: 0, b: 0 };
    let green = Color { r: 0, g: 255, b: 0 };
    let blue = Color { r: 0, g: 0, b: 255 };
    let white = Color { r: 255, g: 255, b: 255 };

    let wooden_crate = texture::load_from_file("resources/textures/wooden-crate.jpg");
    let bricks = texture::load_from_file("resources/textures/bricks.jpg");

//    let cube = two_unit_cube();
    let sphere = sphere(35);
    let cube = cube(0.9);
    let wooden_cube = textured_cube(0.9, &wooden_crate);
    let brick_cube = textured_cube(1.0, &bricks);
//    let triangle = triangle(5.0);
    let torus = ply2::load_model("resources/torus.ply2");
//    let twirl = ply2::load_model("resources/twirl.ply2");
//    let octo_flower = ply2::load_model("resources/octa-flower.ply2");
//    let statue = ply2::load_model("resources/statue.ply2");

    let instances = vec![
//        Instance::new(
//            &triangle,
//            Some(Vector4f { x: 0.0, y: 0.0, z: 10.0, w: 0.0 }),
//            None,
//            Some(Matrix44f::rotation_x(90.0))
//        ),

        Instance::new(
            &wooden_cube,
            Some(Vector4f { x: 1.0, y: 1.0, z: 4.0, w: 0.0 }),
            None,
            Some(Matrix44f::rotation_y(-30.0).multiply(Matrix44f::rotation_z(-30.0)))
        ),
        Instance::new(
            &brick_cube,
            Some(Vector4f { x: -0.3, y: -0.4, z: 3.0, w: 0.0 }),
            None,
            Some(Matrix44f::rotation_y(20.0).
                multiply(Matrix44f::rotation_z(10.0)).
                multiply(Matrix44f::rotation_x(25.0))),
        ),

//        Instance::new(
//            &cube,
//            Some(Vector4f { x: 0.0, y: 0.0, z: 4.0, w: 0.0 }),
//            None,
//            None
//        ),
//        Instance::new(
//            &cube,
//            Some(Vector4f { x: 2.0, y: -2.0, z: 4.5, w: 0.0 }),
//            None,
//            Some(Matrix44f::rotation_y(-30.0).multiply(Matrix44f::rotation_z(-30.0)))
//        ),
//        Instance::new(
//            &torus,
//            Some(Vector4f { x: 0.0, y: 3.0, z: 0.0, w: 0.0 }),
//            Some(0.2),
//            Some(Matrix44f::rotation_y(0.0).multiply(Matrix44f::rotation_x(90.0)))
//        ),
//        Instance::new(
//            &torus,
//            Some(Vector4f { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }),
//            Some(0.1),
//            Some(Matrix44f::rotation_x(0.0).multiply(Matrix44f::rotation_y(0.0)))
//        ),
//
//        Instance::new(
//            &sphere,
//            Some(Vector4f { x: 0.0, y: 0.0, z: 5.0, w: 0.0 }),
//            Some(1.3),
//            Some(Matrix44f::rotation_y(-45.0))
//        ),

//        Instance::new(
//            &octo_flower,
//            Some(Vector4f { x: 0.0, y: 0.0, z: 70.0, w: 0.0 }),
//            None,
//            Some(Matrix44f::rotation_x(0.0).multiply(Matrix44f::rotation_y(0.0)))
//        ),
//        Instance::new(
//            &twirl,
//            Some(Vector4f { x: 0.0, y: 0.0, z: 30.0, w: 0.0 }),
//            None,
//            Some(Matrix44f::rotation_x(0.0).multiply(Matrix44f::rotation_y(0.0)))
//        ),
//        Instance::new(
//            &statue,
//            Some(Vector4f { x: 0.0, y: 0.0, z: 10.0, w: 0.0 }),
//            None,
//            Some(Matrix44f::rotation_x(30.0).multiply(Matrix44f::rotation_y(135.0)))
//        ),
    ];


    let lights = vec![
        Light::Ambient { intensity: 0.2 },
//        Light::Directional {
//            intensity: 0.5,
//            direction: Point3D { x: 1.0, y: 1.0, z: 0.5 }
//        },
        Light::Point {
            intensity: 0.7,
            position: Point3D { x: -4.0, y: 4.0, z: -0.5 }
        }
    ];


//    rendering::render_scene(&scene, &camera, &mut buffer_canvas, &clipping_planes);
//    texture.update(None, &buffer_canvas.buffer, buffer_canvas.size * 3).unwrap();
//    canvas.clear();
//    canvas.copy(&texture, None, None).unwrap();
//    canvas.present();
//

    let step_increase = 0.005;
    let angle_increase = 0.1;
    let mut delta_x;
    let mut delta_y;
    let mut delta_z;
    let mut delta_angle;
    if cfg!(feature = "smooth_animation") {
        println!("configured to smooth animation");
        delta_x = 0.0;
        delta_y = 0.0;
        delta_z = 0.0;
        delta_angle = 0.0;
    } else {
        println!("configured to by step animation");
        delta_x = 0.1;
        delta_y = 0.1;
        delta_z = 0.1;
        delta_angle = 1.0;
    };

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {

        trace!("z_position: {:.2}", z_position);

        if cfg!(feature = "smooth_animation") {
            x_position += delta_x;
            y_position += delta_y;
            z_position += delta_z;
            angle += delta_angle;
        };

        let camera = ProjectiveCamera {
            viewport_size,
            projection_plane_z,
            position: Vector4f { x: x_position, y: y_position, z: z_position, w: 0.0 },
            rotation: Matrix44f::rotation_y(angle),
        };

        buffer_canvas.clear();
        rendering::render_scene(&instances, &lights, &camera, &rendering_settings, &mut buffer_canvas);

        texture.update(None, &buffer_canvas.buffer, buffer_canvas.size * 3).unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        match if cfg!(feature = "smooth_animation") {
            event_pump.poll_event()
        } else {
            Some(event_pump.wait_event())
        } {
            Some(event) => {
                trace!("event happened");
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    },
                    Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_angle += angle_increase;
                        } else {
                            angle += delta_angle;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_angle -= angle_increase;
                        } else {
                            angle -= delta_angle;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_x += step_increase;
                        } else {
                            x_position += delta_x;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_x -= step_increase;
                        } else {
                            x_position -= delta_x;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_z += step_increase;
                        } else {
                            z_position += delta_z;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_z -= step_increase;
                        } else {
                            z_position -= delta_z;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::T), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_y += step_increase;
                        } else {
                            y_position += delta_y;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::G), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_y -= step_increase;
                        } else {
                            y_position -= delta_y;
                        };
                    },
                    Event::KeyDown { keycode: Some(Keycode::X), .. } => {
                        viewport_size += viewport_size_delta;
                    },
                    Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                        viewport_size -= viewport_size_delta;
                    },
                    Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                        projection_plane_z += projection_plane_z_delta;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                        projection_plane_z -= projection_plane_z_delta;
                    },
                    Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                        write_image(&mut buffer_canvas.buffer, buffer_canvas.size).
                            expect("Error writing image to file");
                    }
                    _ => {}
                };
            },
            None => {}
        };
//        thread::sleep(time::Duration::from_millis(10));
    }

//    let p0 = Point { x: -200, y: -250, h: 0.1 };
//    let p1 = Point { x: 200, y: 50, h: 0.0 };
//    let p2 = Point { x: 20, y: 250, h: 1.0 };
//
//    draw_filled_triangle(p0, p1, p2, green, &mut buffer_canvas);
//    draw_wireframe_triangle(p0, p1, p2, white, &mut buffer_canvas);

//    three_spheres_window(&mut buffer, size);

//    let half = 0.8;
//    let frame = Frame { x_min: -half, x_max: half, y_min: -half, y_max: half };
//    let vertices = transform(&vertices, Point3D { x: 0.0, y: 0.0, z: 45.0 });
//    render_model_to_buffer(&mut buffer, size, vertices, faces, frame);


//        point_light_position += 0.01;
//        green_sphere_position_z += 0.01;
//        blue_sphere_position_x += 0.01;

//    write_image(&mut buffer_canvas.buffer, buffer_canvas.size).expect("Error writing image to file");
//    show_buffer_in_window(&mut buffer_canvas.buffer, buffer_canvas.size);

//    rotating_cube_window(&mut buffer, size);

    //    read_ply2("resources/statue.ply2");
    //    read_ply2("resources/torus.ply2");
    //    read_ply2("resources/cube.ply2");
    //    read_ply2("resources/twirl.ply2");
    //    read_ply2("resources/octa-flower.ply2");

//    write_image(&buffer, size).expect("Error writing image to file");
//    show_buffer_in_window(&mut buffer, size);
}


#[cfg(test)]
mod tests {
    pub fn roughly_equals(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }
}