extern crate image;
extern crate sdl2;
extern crate mpeg_encoder;

mod ray_tracing;
mod vectors;
mod projective_camera;
mod buffer_canvas;
mod model;
mod instance;
mod ply2;
mod matrix44f;
mod vector4f;

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;
use std::f64;
use std::io::prelude::*;
use std::str::FromStr;

use ray_tracing::Sphere;
use ray_tracing::Light;
use buffer_canvas::BufferCanvas;
use projective_camera::ProjectiveCamera;
use model::Model;
use vector4f::Vector4f;

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
struct Point { x: i32, y: i32, h: f64 }

#[derive(Copy, Clone)]
struct Frame { x_min: f64, x_max: f64, y_min: f64, y_max: f64 }

#[derive(Copy, Clone)]
pub struct Color {
    r: u8, g: u8, b: u8
}

#[derive(Copy, Clone)]
pub struct Pixel { pub x: usize, pub y: usize, pub color: Color }

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
//    println!("vector1: {:.2} {:.2} {:.2}", vector1.x, vector1.y, vector1.z);
    let vector2 = vectors::difference(vertices[face[1] as usize], vertices[face[0] as usize]);
//    println!("vector2: {:.2} {:.2} {:.2}", vector2.x, vector2.y, vector2.z);
    let face_vector = vectors::cross_product(
        vector1,
        vector2
    );
//    println!("face vector: {:.2} {:.2} {:.2}", face_vector.x, face_vector.y, face_vector.z);

    vectors::dot_product(vertices[face[0] as usize], face_vector) < 0.0
}

fn face_visible2(face: &Vec<i32>, vertices: &[Point3D]) -> bool {
    let vector1 = vectors::difference(vertices[face[2] as usize], vertices[face[1] as usize]);
//    println!("vector1: {:.2} {:.2} {:.2}", vector1.x, vector1.y, vector1.z);
    let vector2 = vectors::difference(vertices[face[1] as usize], vertices[face[0] as usize]);
//    println!("vector2: {:.2} {:.2} {:.2}", vector2.x, vector2.y, vector2.z);
    let face_vector = vectors::cross_product(
        vector2,
        vector1
    );
//    println!("face vector: {:.2} {:.2} {:.2}", face_vector.x, face_vector.y, face_vector.z);

    vectors::dot_product(vertices[face[0] as usize], face_vector) < 0.0
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

fn two_unit_cube() -> Model {
    let vertices = vec![
        Point3D { x: 1.0, y: 1.0, z: 1.0 },
        Point3D { x: -1.0, y: 1.0, z: 1.0 },
        Point3D { x: -1.0, y: -1.0, z: 1.0 },
        Point3D { x: 1.0, y: -1.0, z: 1.0 },
        Point3D { x: 1.0, y: 1.0, z: -1.0 },
        Point3D { x: -1.0, y: 1.0, z: -1.0 },
        Point3D { x: -1.0, y: -1.0, z: -1.0 },
        Point3D { x: 1.0, y: -1.0, z: -1.0 },
    ];

    let faces = vec![
        vec![0, 1, 2],
        vec![0, 2, 3],
        vec![4, 0, 3],
        vec![4, 3, 7],
        vec![5, 4, 7],
        vec![5, 7, 6],
        vec![1, 5, 6],
        vec![1, 6, 2],
        vec![4, 5, 1],
        vec![4, 1, 0],
        vec![2, 6, 7],
        vec![2, 7, 3],
    ];

    Model { vertices, faces }
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

fn interpolate_int(i0: i32, d0: i32, i1: i32, d1: i32) -> Vec<i32> {
    if i0 == i1 {
        return vec![d0];
    }

    let mut results = Vec::<i32>::new();
    let a = (d1 - d0) as f64 / (i1 - i0) as f64;
    let mut d = d0 as f64;
    for i in i0..(i1 + 1) {
        results.push(d.round() as i32);
        d += a;
    }

    results
}

fn interpolate_float(i0: i32, d0: f64, i1: i32, d1: f64) -> Vec<f64> {
    if i0 == i1 {
        return vec![d0];
    }

    let mut results = Vec::<f64>::new();
    let a = (d1 - d0) / ((i1 - i0) as f64);
    let mut d = d0;
    for i in i0..(i1 + 1) {
        results.push(d);
        d += a;
    }

    results
}

#[test]
fn test_interpolate_int() {
    let results = interpolate_int(0, 0, 10, 6);
    for result in results {
        println!("result: {}", result);
    }
}

#[test]
fn test_interpolate_float() {
    let results = interpolate_float(0, 0.0, 10, 10.0);
    for result in results {
        println!("result float: {:.2}", result);
    }
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

fn draw_filled_triangle(
    mut p0: Point,
    mut p1: Point,
    mut p2: Point,
    color: Color,
    canvas: &mut BufferCanvas
) {
    // sort points from bottom to top
    if p1.y < p0.y {
        let swap = p0;
        p0 = p1;
        p1 = swap;
    }
    if p2.y < p0.y {
        let swap = p0;
        p0 = p2;
        p2 = swap;
    }
    if p2.y < p1.y {
        let swap = p1;
        p1 = p2;
        p2 = swap;
    }

    // x coordinates of the edges
    let mut x01 = interpolate_int(p0.y, p0.x, p1.y, p1.x);
    let mut h01 = interpolate_float(p0.y, p0.h, p1.y, p1.h);
    let mut x12 = interpolate_int(p1.y, p1.x, p2.y, p2.x);
    let mut h12 = interpolate_float(p1.y, p1.h, p2.y, p2.h);
    let mut x02 = interpolate_int(p0.y, p0.x, p2.y, p2.x);
    let mut h02 = interpolate_float(p0.y, p0.h, p2.y, p2.h);

    x01.pop();
    let mut x012 = Vec::<i32>::new();
    x012.append(&mut x01);
    x012.append(&mut x12);

    h01.pop();
    let mut h012 = Vec::<f64>::new();
    h012.append(&mut h01);
    h012.append(&mut h12);

    let mut x_left;
    let mut x_right;
    let mut h_left;
    let mut h_right;

    let m = x02.len() / 2;
    if x02[m] < x012[m] {
        x_left = x02;
        x_right = x012;

        h_left = h02;
        h_right = h012;
    } else {
        x_left = x012;
        x_right = x02;

        h_left = h012;
        h_right = h02;
    };

    for y in p0.y..(p2.y + 1) {
        let y_cur = (y - p0.y) as usize;
        let x_l = x_left[y_cur];
        let x_r = x_right[y_cur];
        let h_segment = interpolate_float(x_l, h_left[y_cur], x_r, h_right[y_cur]);
        for x in x_l..(x_r + 1) {
            let shaded_color = multiply_color(h_segment[(x - x_l) as usize], color);
            canvas.draw_point(Point { x, y, h: 0.0 }, shaded_color);
        };
    }
}

pub fn multiply_color(k: f64, color: Color) -> Color {
    Color {
        r: multiply_channel(k, color.r),
        g: multiply_channel(k, color.g),
        b: multiply_channel(k, color.b)
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

fn main() {
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

    let mut x_position = 0.0;
    let mut z_position = 0.0;

    let mut angle = 0.0;


    let mut camera = ProjectiveCamera {
        viewport_size: 1.0,
        projection_plane_z: 1.0,
        position: Vector4f { x: x_position, y: 0.0, z: z_position, w: 0.0 },
        rotation: Matrix44f::rotation_y(angle)
    };

    let red = Color { r: 255, g: 0, b: 0 };
    let green = Color { r: 0, g: 255, b: 0 };
    let blue = Color { r: 0, g: 0, b: 255 };
    let white = Color { r: 255, g: 255, b: 255 };

    let cube = two_unit_cube();
//    let twirl = ply2::load_model("resources/twirl.ply2");
//    let octo_flawer = ply2::load_model("resources/octa-flower.ply2");
    let torus = ply2::load_model("resources/torus.ply2");

    let scene = vec![
        Instance::new(
            &cube,
            Some(Vector4f { x: -1.0, y: -1.5, z: 10.0, w: 0.0 }),
            Some(1.5),
            Some(Matrix44f::rotation_z(-20.0).multiply(Matrix44f::rotation_x(-15.0)))
        ),
        Instance::new(
            &cube,
            Some(Vector4f { x: 1.25, y: 2.5, z: 10.5, w: 0.0 }),
            None,
            Some(Matrix44f::rotation_y(30.0).multiply(Matrix44f::rotation_z(10.0)))
        ),
        Instance::new(
            &torus,
            Some(Vector4f { x: -20.0, y: 10.0, z: 80.0, w: 0.0 }),
            None,
            Some(Matrix44f::rotation_y(-30.0).multiply(Matrix44f::rotation_x(-90.0)))
        ),
        Instance::new(
            &torus,
            Some(Vector4f { x: 20.0, y: 0.0, z: 70.0, w: 0.0 }),
            None,
            Some(Matrix44f::rotation_x(90.0).multiply(Matrix44f::rotation_y(50.0)))
        ),
    ];

    rendering::render_scene(&scene, &camera, &mut buffer_canvas);

    let delta_x = 1.0;
    let delta_z = 1.0;
    let delta_angle = 5.0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                    angle += delta_angle;
                },
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    angle -= delta_angle;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    x_position += delta_x;
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    x_position -= delta_x;
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    z_position += delta_z;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    z_position -= delta_z;
                },
                _ => {}
            }

//            let origin = Point3D { x: x_position, y: 0.0, z: z_position };
//            let rotation = vectors::rotation_around_y(angle);

            camera = ProjectiveCamera {
                viewport_size: 1.0,
                projection_plane_z: 1.0,
                position: Vector4f { x: x_position, y: 0.0, z: z_position, w: 0.0 },
                rotation: Matrix44f::rotation_y(angle)
            };

            buffer_canvas.clear();
            rendering::render_scene(&scene, &camera, &mut buffer_canvas);

            texture.update(None, &buffer_canvas.buffer, buffer_canvas.size * 3).unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
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