//! A raytracer implementation based on the Part I of the online
//! book [Computer Graphics from Scratch](https://gabrielgambetta.com/computer-graphics-from-scratch/)
//! by Gabriel Gambetta. It can only render spheres, can't work with polygonal models.

use common::vectors;
use common::{Color, Light, Pixel, Vector3f};

#[derive(Copy, Clone)]
pub struct Sphere {
    pub center: Vector3f,
    pub radius: f64,
    pub color: Color,
    pub specular: i32,
    pub reflective: f64,
}

#[derive(Clone)]
pub enum Shape {
    Sphere(Sphere),
    CSG {
        op: CSGOperation,
        left: Box<Shape>,
        right: Box<Shape>,
    },
}

#[derive(Clone)]
pub enum CSGOperation {
    Union,
    Intersection,
    Difference,
}

#[derive(Clone)]
pub struct Hit {
    pub t: f64,
    pub point: Vector3f,
    pub normal: Vector3f,
    pub color: Color,
    pub specular: i32,
    pub reflective: f64,
}

pub fn render_scene_to_buffer(
    scene: &Vec<Shape>,
    lights: &Vec<Light>,
    buffer: &mut [u8],
    size: usize,
    origin: Vector3f,
    rotation: [[f64; 3]; 3],
) {
    let canvas_width = size as i32;
    let canvas_height = size as i32;
    let recursion_depth = 4;

    for x in -canvas_width / 2..canvas_width / 2 {
        for y in -canvas_height / 2..canvas_height / 2 {
            // let direction = canvas_to_viewport(x, y, canvas_width, canvas_height);
            let direction = Vector3f::from_vec(crate::vectors::multiply_vec_and_mat(
                canvas_to_viewport(x, y, canvas_width, canvas_height).to_vec(),
                rotation,
            ));
            let color = trace_ray(
                scene,
                lights,
                origin,
                direction,
                1.0,
                std::f64::INFINITY,
                recursion_depth,
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

pub fn screen_x(x_canvas: i32, canvas_width: i32) -> usize {
    (canvas_width / 2 + x_canvas) as usize
}

pub fn screen_y(y_canvas: i32, canvas_height: i32) -> usize {
    (canvas_height / 2 - y_canvas - 1) as usize
}

fn put_pixel(pixel: Pixel, buffer: &mut [u8], size: usize) {
    let offset = pixel.y * size * 3 + pixel.x * 3;
    buffer[offset] = pixel.color.r;
    buffer[offset + 1] = pixel.color.g;
    buffer[offset + 2] = pixel.color.b;
}

fn canvas_to_viewport(x: i32, y: i32, canvas_width: i32, canvas_height: i32) -> Vector3f {
    let d = 1.0;
    let viewport_width = 1.0;
    let viewport_height = 1.0;

    Vector3f {
        x: x as f64 * viewport_width / canvas_width as f64,
        y: y as f64 * viewport_height / canvas_height as f64,
        z: d,
    }
}

fn trace_ray(
    scene: &Vec<Shape>,
    lights: &Vec<Light>,
    origin: Vector3f,
    direction: Vector3f,
    min_t: f64,
    max_t: f64,
    recursion_depth: i32,
) -> Color {
    // let mut rng = rand::thread_rng();

    let (closest_hit, closest_t) = closest_intersection(origin, direction, min_t, max_t, scene);
    match closest_hit {
        Some(hit) => {
            let point = vectors::sum(origin, vectors::scale(closest_t, direction));

            // just for fun: randomize normal vectors to create "bumpiness"
            // let point_normal = vectors::difference(point, sphere.center);
            // let normal = vectors::normalize(Point3D {
            //     x: point_normal.x + rng.gen_range(-0.05, 0.05),
            //     y: point_normal.y + rng.gen_range(-0.05, 0.05),
            //     z: point_normal.z + rng.gen_range(-0.05, 0.05),
            // });

            let normal = hit.normal;
            let view = vectors::negate(direction);
            let intensity = compute_lighting(point, normal, view, lights, hit.specular, scene);
            let local_color = common::multiply_color(intensity, hit.color);
            let reflective = hit.reflective;

            if reflective > 0.0 && recursion_depth > 0 {
                let reflected_color = trace_ray(
                    scene,
                    lights,
                    point,
                    vectors::reflect(view, normal),
                    0.0001,
                    std::f64::INFINITY,
                    recursion_depth - 1,
                );
                add_colors(
                    common::multiply_color(1.0 - reflective, local_color),
                    common::multiply_color(reflective, reflected_color),
                )
            } else {
                local_color
            }
        }
        None => Color { r: 0, g: 0, b: 0 },
    }
}

fn compute_lighting(
    point: Vector3f,
    normal: Vector3f,
    view: Vector3f,
    lights: &Vec<Light>,
    shininess: i32,
    scene: &Vec<Shape>,
) -> f64 {
    let mut result = 0.0;
    for light in lights {
        result += match *light {
            Light::Ambient { intensity } => intensity,
            Light::Point { intensity, position } => compute_light_from_direction(
                point,
                normal,
                view,
                shininess,
                scene,
                intensity,
                crate::vectors::difference(position, point),
                1.0,
            ),
            Light::Directional { intensity, direction } => compute_light_from_direction(
                point,
                normal,
                view,
                shininess,
                scene,
                intensity,
                direction,
                std::f64::INFINITY,
            ),
        }
    }
    result
}

fn compute_light_from_direction(
    point: Vector3f,
    normal: Vector3f,
    view: Vector3f,
    shininess: i32,
    scene: &Vec<Shape>,
    light_intensity: f64,
    light_direction: Vector3f,
    max_t: f64,
) -> f64 {
    let mut result = 0.0;

    // shadow check
    let (closest_hit, _) = closest_intersection(point, light_direction, 0.001, max_t, scene);
    if closest_hit.is_none() {
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
            let reflection_dot_view = vectors::dot_product(reflection_direction, view);
            if reflection_dot_view > 0.0 {
                result += light_intensity
                    * (reflection_dot_view / (vectors::length(reflection_direction) * vectors::length(view)))
                        .powi(shininess)
            }
        }
    }

    result
}

fn add_colors(color1: Color, color2: Color) -> Color {
    Color {
        r: color1.r + color2.r,
        g: color1.g + color2.g,
        b: color1.b + color2.b,
    }
}

fn closest_intersection(
    origin: Vector3f,
    direction: Vector3f,
    min_t: f64,
    max_t: f64,
    scene: &Vec<Shape>,
) -> (Option<Hit>, f64) {
    let mut closest_t = std::f64::INFINITY;
    let mut closest_hit: Option<Hit> = None;

    for shape in scene {
        let hits = intersect_ray_with_shape(origin, direction, shape);

        for hit in hits {
            if hit.t >= min_t && hit.t < max_t && hit.t < closest_t {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }
    }

    (closest_hit, closest_t)
}

fn intersect_ray_with_sphere(origin: Vector3f, direction: Vector3f, sphere: Sphere) -> (f64, f64) {
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

fn intersect_ray_with_shape(origin: Vector3f, direction: Vector3f, shape: &Shape) -> Vec<Hit> {
    match shape {
        Shape::Sphere(sphere) => {
            let (t1, t2) = intersect_ray_with_sphere(origin, direction, *sphere);
            let mut hits = Vec::new();
            for t in [t1, t2] {
                if (t > 0.001) && (t < std::f64::INFINITY) {
                    let point = vectors::sum(origin, vectors::scale(t, direction));
                    let mut normal = vectors::normalize(vectors::difference(point, sphere.center));
                    hits.push(Hit {
                        t,
                        point,
                        normal,
                        color: sphere.color,
                        specular: sphere.specular,
                        reflective: sphere.reflective,
                    });
                }
            }
            hits
        }
        Shape::CSG { op, left, right } => {
            let left_hits = intersect_ray_with_shape(origin, direction, left);
            let right_hits = intersect_ray_with_shape(origin, direction, right);
            merge_csg_hits(left_hits, right_hits, op)
        }
    }
}

fn merge_csg_hits(mut left_hits: Vec<Hit>, mut right_hits: Vec<Hit>, op: &CSGOperation) -> Vec<Hit> {
    left_hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    right_hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
    
    // Сортируем по t
    let mut all_events = Vec::new();
    all_events.extend(left_hits.iter().map(|h| (h.t, true)));
    all_events.extend(right_hits.iter().map(|h| (h.t, false)));
    all_events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Алгоритм: отслеживаем, внутри ли мы левого и правого объекта
    let mut in_left = false;
    let mut in_right = false;
    let mut result = Vec::new();

    for (t, is_left) in all_events {
        let prev_inside = is_inside_csg(in_left, in_right, op);

        if is_left {
            in_left = !in_left;
        } else {
            in_right = !in_right;
        }

        let now_inside = is_inside_csg(in_left, in_right, op);

        // Переход между состояниями
        match (prev_inside, now_inside) {
            (false, true) => {
                // Вход в составной объект — добавляем hit
                if is_left {
                    if let Some(hit) = left_hits.iter().find(|h| (h.t - t).abs() < 1e-6) {
                        result.push(hit.clone());
                    }
                } else {
                    if let Some(hit) = right_hits.iter().find(|h| (h.t - t).abs() < 1e-6) {
                        let mut hit = hit.clone();
                        hit.normal = vectors::negate(hit.normal); // нормаль внутрь -> наружу
                        result.push(hit);
                    }
                }
            }
            (true, false) => {
                // Выход — нормаль разворачивается
                if is_left {
                    if let Some(hit) = left_hits.iter().find(|h| (h.t - t).abs() < 1e-6) {
                        let mut hit = hit.clone();
                        hit.normal = vectors::negate(hit.normal);
                        result.push(hit);
                    }
                } else {
                    if let Some(hit) = right_hits.iter().find(|h| (h.t - t).abs() < 1e-6) {
                        result.push(hit.clone());
                    }
                }
            }
            _ => {}
        }
    }

    result
}

fn is_inside_csg(in_left: bool, in_right: bool, op: &CSGOperation) -> bool {
    match op {
        CSGOperation::Union => in_left || in_right,
        CSGOperation::Intersection => in_left && in_right,
        CSGOperation::Difference => in_left && !in_right,
    }
}

fn contains(range: (f64, f64), n: f64) -> bool {
    (range.0 <= n) && (n < range.1)
}

#[test]
fn test_canvas_to_viewport() {
    let point = canvas_to_viewport(500, 500, 1000, 1000);
    assert!(test_utils::roughly_equals(point.x, 0.5));
    assert!(test_utils::roughly_equals(point.y, 0.5));
    assert!(test_utils::roughly_equals(point.z, 1.0));
}
