use super::Pixel;
use super::Color;
use super::Point3D;
use super::put_pixel;
use super::vector_difference;
use super::dot_product;
use std;
use std::f64;

#[derive(Copy, Clone)]
struct Sphere {
    center: Point3D,
    radius: f64,
    color: Color
}

pub fn render_scene_to_buffer(buffer: &mut [u8], size: usize) {
    let spheres = vec![
        Sphere {
            center: Point3D { x: 0.0, y: -1.0, z: 3.0 },
            radius: 1.0,
            color: Color { r: 255, g: 0, b: 0 }
        },
        Sphere {
            center: Point3D { x: 2.0, y: 0.0, z: 6.0 },
            radius: 1.0,
            color: Color { r: 0, g: 0, b: 255 }
        },
        Sphere {
            center: Point3D { x: -1.5, y: 0.0, z: 4.0 },
            radius: 1.0,
            color: Color { r: 0, g: 255, b: 0 }
        },
    ];

    let origin = Point3D { x: 0.0, y: 0.0, z: 0.0 };

    let canvas_width = size as i32;
    let canvas_height = size as i32;

    for x in -canvas_width/2..canvas_width/2 {
        for y in -canvas_height/2..canvas_height/2 {
            let direction = canvas_to_viewport(x, y, canvas_width, canvas_height);
            let color = trace_ray(&spheres, origin, direction, 1.0, std::f64::INFINITY);

            let screen_x = screen_x(x, canvas_width);
            let screen_y = screen_y(y, canvas_height);
            if screen_x >= size {
                println!("x is outside: {} -> {}", x, screen_x);
            } else if screen_y >= size {
                println!("y is outside: {} -> {}", y, screen_y);
            } else {
                put_pixel(Pixel { x: screen_x, y: screen_y, color }, buffer, size);
            }

        }
    }
}

fn screen_x(x_canvas: i32, canvas_width: i32) -> usize {
    (canvas_width / 2 + x_canvas) as usize
}

fn screen_y(y_canvas: i32, canvas_height: i32) -> usize {
    (canvas_height / 2 - y_canvas - 1) as usize
}

fn canvas_to_viewport(x: i32, y: i32, canvas_width: i32, canvas_height: i32) -> Point3D {
    let d = 1.0;
    let viewport_width = 1.0;
    let viewport_height = 1.0;

    Point3D {
        x: x as f64 * viewport_width / canvas_width as f64,
        y: y as f64 * viewport_height / canvas_height as f64,
        z: d
    }
}

fn trace_ray(spheres: &Vec<Sphere>, origin: Point3D, direction: Point3D, min_t: f64, max_t: f64)
-> Color {
    let mut closest_t = std::f64::INFINITY;
    let mut closest_sphere: Option<Sphere> = None;

    for sphere in spheres {
        let (t1, t2) = intersect_ray_with_sphere(origin, direction, *sphere);
        if contains((min_t, max_t), t1) && t1 < closest_t {
            closest_t = t1;
            closest_sphere = Some(*sphere);
        }
        if contains((min_t, max_t), t2) && t2 < closest_t {
            closest_t = t2;
            closest_sphere = Some(*sphere);
        }
    }
    match closest_sphere {
        Some(sphere) => sphere.color,
        None => Color { r: 0, g: 0, b: 0 }
    }
}

fn contains(range: (f64, f64), n: f64) -> bool {
    (range.0 <= n) && (n < range.1)
}


fn intersect_ray_with_sphere(origin: Point3D, direction: Point3D, sphere: Sphere) -> (f64, f64) {
    let oc = vector_difference(origin, sphere.center);
    let k1 = dot_product(direction, direction);
    let k2 = 2.0 * dot_product(oc, direction);
    let k3 = dot_product(oc, oc) - sphere.radius * sphere.radius;

    let discriminant = k2 * k2 - 4.0 * k1 * k3;
    if discriminant < 0.0 {
        return (std::f64::INFINITY, std::f64::INFINITY);
    }

    let t1 = (-k2 + discriminant.sqrt()) / (2.0 * k1);
    let t2 = (-k2 - discriminant.sqrt()) / (2.0 * k1);
    (t1, t2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roughly_equals(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn test_canvas_to_viewport() {
        let point = canvas_to_viewport(500, 500, 1000, 1000);
        assert!(roughly_equals(point.x, 0.5));
        assert!(roughly_equals(point.y, 0.5));
        assert!(roughly_equals(point.z, 1.0));
    }
}