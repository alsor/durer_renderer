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

            render(Pixel { x: x as usize, y: y as usize}, buffer, size);

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

fn render(pixel: Pixel, buffer: &mut [u8], size: usize) {
    buffer[pixel.y * size + pixel.x] = 255;
}

fn write_image(buffer: &[u8], size: usize) -> Result<(), std::io::Error> {
    let output = File::create("target/result.png")?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&buffer, size as u32, size as u32, ColorType::Gray(8))?;

    Ok(())
}

fn main() {
    let vertices = [
        Point3D { x: -0.5, y: -1.5, z: 1.5 },
        Point3D { x: -0.5, y: -0.5, z: 1.5 },
        Point3D { x: 0.5, y: -0.5, z: 1.5 },
        Point3D { x: 0.5, y: -1.5, z: 1.5 },
        Point3D { x: -0.5, y: -1.5, z: 2.5 },
        Point3D { x: -0.5, y: -0.5, z: 2.5 },
        Point3D { x: 0.5, y: -0.5, z: 2.5 },
        Point3D { x: 0.5, y: -1.5, z: 2.5 },
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

    let frame = Frame { x_min: -1.2, x_max: 1.2, y_min: -1.2, y_max: 1.2 };
    let size = 500;
    let mut buffer = vec![0u8; size * size];

    let mut vertex_pixels = Vec::new();

    for point3d in &vertices {
        let point2d = project(*point3d);
        let norm_point = normalize(point2d, frame);
        vertex_pixels.push(rasterize(norm_point, size));
    }

    for edge in &edges {
        rasterize_line(vertex_pixels[edge.0], vertex_pixels[edge.1], &mut buffer, size);
    }

    write_image(&buffer, size).expect("Error writing image to file");
}

