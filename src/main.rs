extern crate image;

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;
use std::f64;
use std::io::prelude::*;
use std::str::FromStr;

#[derive(Copy, Clone)]
struct Point3D { x: f64, y: f64, z: f64 }

#[derive(Copy, Clone)]
struct Point2D { x: f64, y: f64 }

#[derive(Copy, Clone)]
struct Frame { x_min: f64, x_max: f64, y_min: f64, y_max: f64 }

#[derive(Copy, Clone)]
struct Pixel { x: usize, y: usize }

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
    Pixel { x, y }
}

fn rasterize_line(start: Pixel, end: Pixel, buffer: &mut [u8], size: usize) {
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

        render(start, buffer, size);

        let mut x = x1 + sx;
        let mut y = y1;
        for i in 1..dx {
            if d > 0 {
                d = d + d2;
                y = y + sy;
            } else {
                d = d + d1;
            }

            render(Pixel { x: x as usize, y: y as usize }, buffer, size);

            x = x + sx;
        }
    } else {
        let mut d = (dx << 1) - dy;
        let d1 = dx << 1;
        let d2 = (dx - dy) << 1;

        render(start, buffer, size);

        let mut x = x1;
        let mut y = y1 + sy;
        for i in 1..dy {
            if d > 0 {
                d = d + d2;
                x = x + sx;
            } else {
                d = d + d1;
            }

            render(Pixel { x: x as usize, y: y as usize }, buffer, size);

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
        render(Pixel { x: x as usize, y: y as usize }, buffer, size);
    }
}

fn render(pixel: Pixel, buffer: &mut [u8], size: usize) {
    buffer[pixel.y * size + pixel.x] = 255;
}

fn write_image(buffer: &[u8], size: usize) -> Result<(), std::io::Error> {
    let output = File::create("target/result.png")?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&buffer, size as u32, size as u32, ColorType::Gray(8))?;

    Ok(())
}

fn cross_product(v: Point3D, w: Point3D) -> Point3D {
    Point3D {
        x: v.y * w.z - v.z * w.y,
        y: v.z * w.x - v.x * w.z,
        z: v.x * w.y - v.y * w.x
    }
}

fn dot_product(v: Point3D, w: Point3D) -> f64 {
    v.x * w.x + v.y * w.y + v.z * w.z
}

fn vector_difference(v1: Point3D, v2: Point3D) -> Point3D {
    Point3D {
        x: v1.x - v2.x,
        y: v1.y - v2.y,
        z: v1.z - v2.z
    }
}

fn face_visible(face: &Vec<i32>, vertices: &[Point3D]) -> bool {
    let vector1 = vector_difference(vertices[face[2] as usize], vertices[face[1] as usize]);
//    println!("vector1: {:.2} {:.2} {:.2}", vector1.x, vector1.y, vector1.z);
    let vector2 = vector_difference(vertices[face[1] as usize], vertices[face[0] as usize]);
//    println!("vector2: {:.2} {:.2} {:.2}", vector2.x, vector2.y, vector2.z);
    let face_vector = cross_product(
        vector1,
        vector2
    );
//    println!("face vector: {:.2} {:.2} {:.2}", face_vector.x, face_vector.y, face_vector.z);

    dot_product(vertices[face[0] as usize], face_vector) < 0.0
}

fn face_visible2(face: &Vec<i32>, vertices: &[Point3D]) -> bool {
    let vector1 = vector_difference(vertices[face[2] as usize], vertices[face[1] as usize]);
//    println!("vector1: {:.2} {:.2} {:.2}", vector1.x, vector1.y, vector1.z);
    let vector2 = vector_difference(vertices[face[1] as usize], vertices[face[0] as usize]);
//    println!("vector2: {:.2} {:.2} {:.2}", vector2.x, vector2.y, vector2.z);
    let face_vector = cross_product(
        vector2,
        vector1
    );
//    println!("face vector: {:.2} {:.2} {:.2}", face_vector.x, face_vector.y, face_vector.z);

    dot_product(vertices[face[0] as usize], face_vector) < 0.0
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

fn create_rotated_cube(t: f64) -> [Point3D; 8] {
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

fn read_ply2(filename: &str) -> (Vec<Point3D>, Vec<Vec<i32>>) {
    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("error reading file");

    #[derive(Copy,Clone)]
    enum Ply2Parts { NumVertices, NumFaces, Vertices, Faces }
    let ply2_structure = [
        Ply2Parts::NumVertices,
        Ply2Parts::NumFaces,
        Ply2Parts::Vertices,
        Ply2Parts::Faces,
    ];
    let mut current_section = 0;

    let mut num_vertices = 0;
    let mut num_faces = 0;
    let mut current_vertex = 0;
    let mut current_face = 0;
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for line in contents.split("\n") {
        //        let parsed = match i32::from_str(line.trim()) {
        //            Ok(num) => num,
        //            Err(e) => {
        //                println!("error: {}", e);
        //                0
        //            }
        //        };
        //        println!("parsed: {:?}", parsed);
        if current_section == 4 {
            break;
        }

        match ply2_structure[current_section] {
            Ply2Parts::NumVertices => {
                num_vertices = i32::from_str(line.trim()).unwrap();
                current_section += 1;
            }
            Ply2Parts::NumFaces => {
                num_faces = i32::from_str(line.trim()).unwrap();
                current_section += 1;
            }
            Ply2Parts::Vertices => {
                let mut coords = Vec::new();
                for float in line.trim().split(" ") {
                    coords.push(f64::from_str(float).unwrap());
                }
                vertices.push(Point3D { x: coords[0], y: coords[1], z: coords[2] });
                current_vertex += 1;
                if current_vertex == num_vertices {
                    current_section += 1;
                }
            }
            Ply2Parts::Faces => {
                let mut faces_list = Vec::new();
                let mut face = Vec::new();
                for str in line.trim().split(" ") {
                    faces_list.push(i32::from_str(str.trim()).unwrap());
                }
                let vertices_in_face = faces_list[0];
                for i in 1..(vertices_in_face + 1) {
                    face.push(faces_list[i as usize]);
                }
                faces.push(face);

                current_face += 1;
                if current_face == num_faces {
                    current_section += 1;
                }
            }
            _ => ()
        }
    }

    println!("vertices read: {}", vertices.len());
    println!("faces read: {}", faces.len());

    (vertices, faces)
}

fn cube() -> (Vec<Point3D>, Vec<Vec<i32>>) {
    let cube_vertices = create_rotated_cube(0.6).to_vec();

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

fn main() {
//    let (vertices, faces) = read_ply2("resources/statue.ply2");
//    let (vertices, faces) = read_ply2("resources/torus.ply2");
//    let (vertices, faces) = read_ply2("resources/cube.ply2");
//    let (vertices, faces) = read_ply2("resources/twirl.ply2");
    let (vertices, faces) = read_ply2("resources/octa-flower.ply2");
//    let (vertices, faces) = cube();

    let z_transform = find_z_transform(&vertices);
    println!("calculated z transform: {:.2}", z_transform);
    let z_transform = 45.0;

    let vertices = transform(&vertices, Point3D { x: 0.0, y: 0.0, z: z_transform });

    //
    // y
    // ^
    // |
    // |
    //  ------> x
    //
    // z - deeper into the screen
    //

    let frame = enclosing_frame(&vertices);
    println!("calculated frame: {:.2} {:.2} {:.2} {:.2}",
             frame.x_min, frame.x_max, frame.y_min, frame.y_max);
    let half = 0.5;
    let frame = Frame { x_min: -half, x_max: half, y_min: -half, y_max: half };


    let size = 800;
    let mut buffer = vec![0u8; size * size];

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
        if face_visible2(face, &vertices) {
            draw_face(face, &vertex_pixels, &mut buffer, size);
        };
    };

    //        simple_line(Pixel { x: 60, y: 40 }, Pixel { x: 120, y: 50 }, &mut buffer, size);
    //    simple_line(Pixel { x: 120, y: 50 }, Pixel { x: 60, y: 40 }, &mut buffer, size);
    //    rasterize_line(Pixel { x: 60, y: 50 }, Pixel { x: 120, y: 60 }, &mut buffer, size);

    write_image(&buffer, size).expect("Error writing image to file");
}

