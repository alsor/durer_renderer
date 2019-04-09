extern crate rand;

use super::Pixel;
use super::Color;
use super::Point3D;
use super::screen_x;
use super::screen_y;
use super::put_pixel;
use super::multiply_color;
use ::{std, Light};
use std::f64;
use vectors;
use self::rand::Rng;

#[derive(Copy, Clone)]
pub struct Sphere {
    pub center: Point3D,
    pub radius: f64,
    pub color: Color,
    pub specular: i32,
    pub reflective: f64
}

pub fn render_scene_to_buffer(
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
    buffer: &mut [u8],
    size: usize,
    origin: Point3D,
    rotation: [[f64; 3]; 3]
) {
    let canvas_width = size as i32;
    let canvas_height = size as i32;
    let recursion_depth = 4;

    for x in -canvas_width/2..canvas_width/2 {
        for y in -canvas_height/2..canvas_height/2 {
//            let direction = canvas_to_viewport(x, y, canvas_width, canvas_height);
            let direction = Point3D::from_vec(
                vectors::multiply_vec_and_mat(
                    canvas_to_viewport(x, y, canvas_width, canvas_height).to_vec(),
                    rotation
                )
            );
            let color = trace_ray(
                spheres,
                lights,
                origin,
                direction,
                1.0,
                std::f64::INFINITY,
                recursion_depth
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

fn compute_lighting(
    point: Point3D,
    normal: Point3D,
    view: Point3D,
    lights: &Vec<Light>,
    shininess: i32,
    spheres: &Vec<Sphere>
) -> f64 {
    let mut result = 0.0;
    for light in lights {
        result += match *light {
            Light::Ambient { intensity } => intensity,
            Light::Point { intensity, position } => {
                compute_light_from_direction(
                    point,
                    normal,
                    view,
                    shininess,
                    spheres,
                    intensity,
                    ::vectors::difference(position, point),
                    1.0
                )
            }
            Light::Directional { intensity, direction } => {
                compute_light_from_direction(
                    point,
                    normal,
                    view,
                    shininess,
                    spheres,
                    intensity,
                    direction,
                    std::f64::INFINITY
                )
            }
        }
    }
    result
}

fn compute_light_from_direction(
    point: Point3D,
    normal: Point3D,
    view: Point3D,
    shininess: i32,
    spheres: &Vec<Sphere>,
    light_intensity: f64,
    light_direction: Point3D,
    max_t: f64
) -> f64 {
    let mut result = 0.0;

    // shadow check
    let (closest_sphere, _) = closest_intersection(
        point,
        light_direction,
        0.001,
        max_t,
        spheres
    );
    match closest_sphere {
        Some(_) => (),
        None => {
            // diffuse
            let dot = vectors::dot_product(normal, light_direction);
            if dot > 0.0 {
                // assuming that normal is a unit vector (has length 1)
                result += light_intensity * dot / vectors::length(light_direction);
            }

            // specular
            // TODO add color of the light to this component
            if shininess > 0 {
                let reflection_direction = vectors::reflect(light_direction, normal);
                let reflection_dot_view = vectors::dot_product(
                    reflection_direction,
                    view
                );
                if reflection_dot_view > 0.0 {
                    result += light_intensity *
                        (
                            reflection_dot_view /
                            (vectors::length(reflection_direction) * vectors::length(view))
                        ).powi(shininess)
                }
            }
        }
    }

    result
}

fn closest_intersection(
    origin: Point3D,
    direction: Point3D,
    min_t: f64,
    max_t: f64,
    spheres: &Vec<Sphere>
) -> (Option<Sphere>, f64) {
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

    (closest_sphere, closest_t)
}

fn trace_ray(
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
    origin: Point3D,
    direction: Point3D,
    min_t: f64,
    max_t: f64,
    recursion_depth: i32
) -> Color {
    let mut rng = rand::thread_rng();

    let (closest_sphere, closest_t) = closest_intersection(
        origin,
        direction,
        min_t,
        max_t,
        spheres
    );
    match closest_sphere {
        Some(sphere) => {
            let point = vectors::sum(origin, vectors::scale(closest_t, direction));

            // randomize normal vectors to create "bumpiness"
//            let point_normal = vectors::difference(point, sphere.center);
//            let normal = vectors::normalize(
//                Point3D {
//                    x: point_normal.x + rng.gen_range(-0.05, 0.05),
//                    y: point_normal.y + rng.gen_range(-0.05, 0.05),
//                    z: point_normal.z + rng.gen_range(-0.05, 0.05),
//                }
//            );



            let normal = vectors::normalize(
                vectors::difference(point, sphere.center)
            );
            let view = vectors::negate(direction);
            let intensity = compute_lighting(
                point,
                normal,
                view,
                lights,
                sphere.specular,
                spheres
            );
            let local_color = multiply_color(intensity, sphere.color);
            let reflective = sphere.reflective;

            if reflective > 0.0 && recursion_depth > 0 {
                let reflected_color = trace_ray(
                    spheres,
                    lights,
                    point,
                    vectors::reflect(view, normal),
                    0.0001,
                    std::f64::INFINITY,
                    recursion_depth - 1
                );
                add_colors(
                    multiply_color(1.0 - reflective, local_color),
                    multiply_color(reflective, reflected_color)
                )
            } else {
                local_color
            }

        }
        None => Color { r: 0, g: 0, b: 0 }
    }
}

fn add_colors(color1: Color, color2: Color) -> Color {
    Color {
        r: color1.r + color2.r,
        g: color1.g + color2.g,
        b: color1.b + color2.b
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
