use super::Pixel;
use super::Color;
use super::Point3D;
use super::put_pixel;
use std;
use std::f64;
use vectors;

#[derive(Copy, Clone)]
struct Sphere {
    center: Point3D,
    radius: f64,
    color: Color
}

#[derive(Copy, Clone)]
enum LightType { Ambient, Point, Directional }

#[derive(Copy, Clone)]
struct Light {
    kind: LightType,
    intensity: f64,
    vector: Option<Point3D> // position for point light, direction for directional light
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

    let lights = vec![
        Light {
            kind: LightType::Ambient,
            intensity: 0.15,
            vector: None
        },
//        Light {
//            kind: LightType::Point,
//            intensity: 0.6,
//            vector: Some(Point3D { x: 2.0, y: 1.0, z: 0.0 })
//        },
//        Light {
//            kind: LightType::Point,
//            intensity: 0.7,
//            vector: Some(Point3D { x: 1.0, y: 3.0, z: -0.5 })
//        },
        Light {
            kind: LightType::Directional,
            intensity: 0.7,
            vector: Some(Point3D { x: 1.0, y: 3.0, z: -0.5 })
        }
    ];

    let origin = Point3D { x: 0.0, y: 0.0, z: 0.0 };

    let canvas_width = size as i32;
    let canvas_height = size as i32;

    for x in -canvas_width/2..canvas_width/2 {
        for y in -canvas_height/2..canvas_height/2 {
            let direction = canvas_to_viewport(x, y, canvas_width, canvas_height);
            let color = trace_ray(
                &spheres,
                &lights,
                origin,
                direction,
                1.0,
                std::f64::INFINITY
            );

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

fn compute_lighting(point: Point3D, normal: Point3D, lights: &Vec<Light>) -> f64 {
    let mut result = 0.0;
    for light in lights {
        match light.kind {
            LightType::Ambient => result += light.intensity,
            LightType::Point | LightType::Directional => {
                let light_direction = match light.kind {
                    LightType::Point => ::vectors::difference(light.vector.unwrap(), point),
                    LightType::Directional => light.vector.unwrap(),
                    _ => panic!()
                };
                let dot = vectors::dot_product(normal, light_direction);
                if dot > 0.0 {
                    result += light.intensity * dot / vectors::length(light_direction);
                }
            }
        }
    }
    result
}

fn trace_ray(
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
    origin: Point3D,
    direction: Point3D,
    min_t: f64,
    max_t: f64
) -> Color {
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
        Some(sphere) => {
            let point = vectors::sum(origin, vectors::scale(closest_t, direction));
            let normal = vectors::normalize(
                vectors::difference(point, sphere.center)
            );
            let intensity = compute_lighting(point, normal, lights);
            multiply_color(intensity, sphere.color)
        }
        None => Color { r: 0, g: 0, b: 0 }
    }
}

fn multiply_color(k: f64, color: Color) -> Color {
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

fn contains(range: (f64, f64), n: f64) -> bool {
    (range.0 <= n) && (n < range.1)
}


fn intersect_ray_with_sphere(origin: Point3D, direction: Point3D, sphere: Sphere) -> (f64, f64) {
    let oc = vectors::difference(origin, sphere.center);
    let k1 = vectors::dot_product(direction, direction);
    let k2 = 2.0 * vectors::dot_product(oc, direction);
    let k3 = vectors::dot_product(oc, oc) - sphere.radius * sphere.radius;

    let discriminant = k2 * k2 - 4.0 * k1 * k3;
    if discriminant < 0.0 {
        return (std::f64::INFINITY, std::f64::INFINITY);
    }

    let t1 = (-k2 + discriminant.sqrt()) / (2.0 * k1);
    let t2 = (-k2 - discriminant.sqrt()) / (2.0 * k1);
    (t1, t2)
}

#[test]
fn test_canvas_to_viewport() {
    let point = canvas_to_viewport(500, 500, 1000, 1000);
    assert!(::tests::roughly_equals(point.x, 0.5));
    assert!(::tests::roughly_equals(point.y, 0.5));
    assert!(::tests::roughly_equals(point.z, 1.0));
}
