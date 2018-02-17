extern crate image;

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

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
    buffer[pixel.y * size + pixel.x] = 0;
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

fn face_visible(face: &(i32, i32, i32, i32), vertices: &[Point3D]) -> bool {
    let vector1 = vector_difference(vertices[face.2 as usize], vertices[face.1 as usize]);
//    println!("vector1: {:.2} {:.2} {:.2}", vector1.x, vector1.y, vector1.z);
    let vector2 = vector_difference(vertices[face.1 as usize], vertices[face.0 as usize]);
//    println!("vector2: {:.2} {:.2} {:.2}", vector2.x, vector2.y, vector2.z);
    let face_vector = cross_product(
        vector1,
        vector2
    );
//    println!("face vector: {:.2} {:.2} {:.2}", face_vector.x, face_vector.y, face_vector.z);

    dot_product(vertices[face.0 as usize], face_vector) < 0.0
}

fn draw_face(face: &(i32, i32, i32, i32),
             vertex_pixels: &Vec<Pixel>,
             buffer: &mut [u8],
             size: usize) {
    rasterize_line(vertex_pixels[face.0 as usize], vertex_pixels[face.1 as usize], buffer, size);
    rasterize_line(vertex_pixels[face.1 as usize], vertex_pixels[face.2 as usize], buffer, size);
    rasterize_line(vertex_pixels[face.2 as usize], vertex_pixels[face.3 as usize], buffer, size);
    rasterize_line(vertex_pixels[face.3 as usize], vertex_pixels[face.0 as usize], buffer, size);
}

fn main() {
    //
    // y
    // ^
    // |
    // |
    //  ------> x
    //
    // z - deeper into the screen
    //
    let vertices = [
        // 0
        Point3D { x: -0.5, y: -1.2, z: 1.5 },
        // 1
        Point3D { x: -0.5, y: -0.2, z: 1.5 },
        // 2
        Point3D { x: 0.5, y: -0.2, z: 1.5 },
        // 3
        Point3D { x: 0.5, y: -1.2, z: 1.5 },
        // 4
        Point3D { x: -0.5, y: -1.2, z: 2.5 },
        // 5
        Point3D { x: -0.5, y: -0.2, z: 2.5 },
        // 6
        Point3D { x: 0.5, y: -0.2, z: 2.5 },
        // 7
        Point3D { x: 0.5, y: -1.2, z: 2.5 },
    ];

    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
    ];

    let faces = [
        (0, 3, 2, 1),
        (3, 7, 6, 2),
        (7, 4, 5, 6),
        (4, 0, 1, 5),
        (0, 4, 7, 3),
        (1, 2, 6, 5),
    ];

    let frame = Frame { x_min: -1.2, x_max: 1.2, y_min: -1.2, y_max: 1.2 };
    let size = 500;
    let mut buffer = vec![255u8; size * size];

    let mut vertex_pixels = Vec::new();

    for point3d in &vertices {
        let point2d = project(*point3d);
        let norm_point = normalize(point2d, frame);
        //        println!("normalized Point2D: {:.2} {:.2}", norm_point.x, norm_point.y);
        let pixel = rasterize(norm_point, size);
        //        println!("Pixel: {:?} {:?}", pixel.x, pixel.y);
        vertex_pixels.push(pixel);
    }

    // line from eye to vertex
//    rasterize_line(
//        rasterize(normalize(project(Point3D { x: 0.0, y: 0.0, z: 1.1 }), frame), size),
//        rasterize(normalize(project(vertices[3]), frame), size),
//        &mut buffer,
//        size
//    );

    //    for edge in &edges {
    //        rasterize_line(vertex_pixels[edge.0], vertex_pixels[edge.1], &mut buffer, size);
    //    }

    // draw faces
    for face in &faces {
        if face_visible(face, &vertices) {
            draw_face(face, &vertex_pixels, &mut buffer, size);
        };
    };

    //        simple_line(Pixel { x: 60, y: 40 }, Pixel { x: 120, y: 50 }, &mut buffer, size);
    //    simple_line(Pixel { x: 120, y: 50 }, Pixel { x: 60, y: 40 }, &mut buffer, size);
    //    rasterize_line(Pixel { x: 60, y: 50 }, Pixel { x: 120, y: 60 }, &mut buffer, size);

    write_image(&buffer, size).expect("Error writing image to file");
}

