//! A raytracer implementation based on the Part I of the online
//! book [Computer Graphics from Scratch](https://gabrielgambetta.com/computer-graphics-from-scratch/)
//! by Gabriel Gambetta. It can only render spheres, can't work with polygonal models.

use common::vectors;
use common::{Color, Light, Pixel, Vector3f};
use smallvec::SmallVec;

type HitList = SmallVec<[Hit; 4]>;

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
    Transformed {
        shape: Box<Shape>,
        transform: Transform,
    },
}

impl Shape {
    /// Рекурсивно транслирует всё дерево
    pub fn translate_all(self, dx: f64, dy: f64, dz: f64) -> Self {
        let translation = Vector3f::new(dx, dy, dz);
        match self {
            Shape::Sphere(mut sphere) => {
                sphere.center = vectors::sum(sphere.center, translation);
                Shape::Sphere(sphere)
            }
            Shape::CSG { op, left, right } => Shape::CSG {
                op,
                left: Box::new(left.translate_all(dx, dy, dz)),
                right: Box::new(right.translate_all(dx, dy, dz)),
            },
            Shape::Transformed { shape, transform } => Shape::Transformed {
                shape: Box::new(shape.translate_all(dx, dy, dz)),
                transform,
            },
        }
    }

    /// Рекурсивно поворачивает все примитивы вокруг точки по оси X
    pub fn rotate_x_all(self, angle: f64, point: Vector3f) -> Self {
        let rotation_matrix = vectors::rotate_x(angle);
    
        match self {
            Shape::Sphere(mut sphere) => {
                let local_center = vectors::difference(sphere.center, point);
                let rotated_vec = vectors::multiply_vec_and_mat(local_center.to_vec(), rotation_matrix);
                let world_center = vectors::sum(Vector3f::from_vec(rotated_vec), point);
                sphere.center = world_center;
                Shape::Sphere(sphere)
            }
            Shape::CSG { op, left, right } => Shape::CSG {
                op,
                left: Box::new(left.rotate_x_all(angle, point)),
                right: Box::new(right.rotate_x_all(angle, point)),
            },
            Shape::Transformed { shape, transform } => Shape::Transformed {
                shape: Box::new(shape.rotate_x_all(angle, point)),
                transform,
            },
        }
    }

    /// Рекурсивно поворачивает все примитивы вокруг точки по оси Y
    pub fn rotate_y_all(self, angle: f64, point: Vector3f) -> Self {
        let rotation_matrix = vectors::rotate_y(angle);

        match self {
            Shape::Sphere(mut sphere) => {
                // Сдвигаем центр в локальные координаты, поворачиваем, возвращаем
                let local_center = vectors::difference(sphere.center, point);
                let rotated_vec = vectors::multiply_vec_and_mat(local_center.to_vec(), rotation_matrix);
                let world_center = vectors::sum(Vector3f::from_vec(rotated_vec), point);
                sphere.center = world_center;
                Shape::Sphere(sphere)
            }
            Shape::CSG { op, left, right } => Shape::CSG {
                op,
                left: Box::new(left.rotate_y_all(angle, point)),
                right: Box::new(right.rotate_y_all(angle, point)),
            },
            Shape::Transformed { shape, transform } => {
                // Применяем к внутреннему объекту, трансформ не трогаем
                Shape::Transformed {
                    shape: Box::new(shape.rotate_y_all(angle, point)),
                    transform,
                }
            }
        }
    }

    /// Рекурсивно поворачивает все примитивы вокруг точки по оси Z
    pub fn rotate_z_all(self, angle: f64, point: Vector3f) -> Self {
        let rotation_matrix = vectors::rotate_z(angle);

        match self {
            Shape::Sphere(mut sphere) => {
                let local_center = vectors::difference(sphere.center, point);
                let rotated_vec = vectors::multiply_vec_and_mat(local_center.to_vec(), rotation_matrix);
                let world_center = vectors::sum(Vector3f::from_vec(rotated_vec), point);
                sphere.center = world_center;
                Shape::Sphere(sphere)
            }
            Shape::CSG { op, left, right } => Shape::CSG {
                op,
                left: Box::new(left.rotate_z_all(angle, point)),
                right: Box::new(right.rotate_z_all(angle, point)),
            },
            Shape::Transformed { shape, transform } => Shape::Transformed {
                shape: Box::new(shape.rotate_z_all(angle, point)),
                transform,
            },
        }
    }

    /// Поворачивает вокруг оси X на угол в градусах
    pub fn rotate_x_all_deg(self, angle_deg: f64, point: Vector3f) -> Self {
        let angle_rad = angle_deg.to_radians();
        self.rotate_x_all(angle_rad, point)
    }

    /// Поворачивает вокруг оси Y на угол в градусах
    pub fn rotate_y_all_deg(self, angle_deg: f64, point: Vector3f) -> Self {
        let angle_rad = angle_deg.to_radians();
        self.rotate_y_all(angle_rad, point)
    }

    /// Поворачивает вокруг оси Z на угол в градусах
    pub fn rotate_z_all_deg(self, angle_deg: f64, point: Vector3f) -> Self {
        let angle_rad = angle_deg.to_radians();
        self.rotate_z_all(angle_rad, point)
    }
}

#[derive(Clone)]
pub enum CSGOperation {
    Union,
    Intersection,
    Difference,
}

#[derive(Clone)]
pub struct Transform {
    pub translation: Vector3f,
    pub rotation: [[f64; 3]; 3], // матрица поворота 3x3
}

impl Transform {
    fn inverse(&self) -> Self {
        // Обратная трансформация: сначала обратный поворот (транспонированная матрица), потом обратный перевод
        let inv_rotation = crate::vectors::transpose_3x3(self.rotation);
        let inv_translation = crate::vectors::multiply_vec_and_mat(
            crate::vectors::negate(self.translation).to_vec(),
            inv_rotation,
        );
        Transform {
            translation: Vector3f::from_vec(inv_translation),
            rotation: inv_rotation,
        }
    }

    fn transform_point(&self, point: Vector3f) -> Vector3f {
        let rotated = Vector3f::from_vec(crate::vectors::multiply_vec_and_mat(
            point.to_vec(),
            self.rotation,
        ));
        vectors::sum(rotated, self.translation)
    }

    fn transform_direction(&self, direction: Vector3f) -> Vector3f {
        Vector3f::from_vec(crate::vectors::multiply_vec_and_mat(
            direction.to_vec(),
            self.rotation,
        ))
    }
}

#[derive(Copy, Clone)]
pub struct Hit {
    pub t: f64,
    pub point: Vector3f,
    pub normal: Vector3f,
    pub color: Color,
    pub specular: i32,
    pub reflective: f64,
}

impl Hit {
    fn negate_normal(&self) -> Hit {
        Hit {
            t: self.t,
            point: self.point,
            normal: vectors::negate(self.normal),
            color: self.color,
            specular: self.specular,
            reflective: self.reflective,
        }
    }
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

    let closest_hit = closest_intersection(origin, direction, min_t, max_t, scene);
    match closest_hit {
        Some(hit) => {
            // just for fun: randomize normal vectors to create "bumpiness"
            // let point_normal = vectors::difference(point, sphere.center);
            // let normal = vectors::normalize(Point3D {
            //     x: point_normal.x + rng.gen_range(-0.05, 0.05),
            //     y: point_normal.y + rng.gen_range(-0.05, 0.05),
            //     z: point_normal.z + rng.gen_range(-0.05, 0.05),
            // });

            let normal = hit.normal;
            let view = vectors::negate(direction);
            let intensity = compute_lighting(hit.point, normal, view, lights, hit.specular, scene);
            let local_color = common::multiply_color(intensity, hit.color);
            let reflective = hit.reflective;

            if reflective > 0.0 && recursion_depth > 0 {
                let reflected_color = trace_ray(
                    scene,
                    lights,
                    hit.point,
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
    let closest_hit = closest_intersection(point, light_direction, 0.001, max_t, scene);
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
) -> Option<Hit> {
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

    closest_hit
}

fn intersect_ray_with_sphere(origin: Vector3f, direction: Vector3f, sphere: Sphere) -> Option<(f64, f64)> {
    let oc = vectors::difference(origin, sphere.center);
    let k1 = vectors::dot_product(direction, direction);
    let k2 = 2.0 * vectors::dot_product(oc, direction);
    let k3 = vectors::dot_product(oc, oc) - sphere.radius * sphere.radius;

    let discriminant = k2 * k2 - 4.0 * k1 * k3;
    if discriminant < 0.0 {
        return None;
    }

    let t1 = (-k2 + discriminant.sqrt()) / (2.0 * k1);
    let t2 = (-k2 - discriminant.sqrt()) / (2.0 * k1);
    Some((t1, t2))
}

fn intersect_ray_with_shape(origin: Vector3f, direction: Vector3f, shape: &Shape) -> HitList {
    match shape {
        Shape::Sphere(sphere) => {
            let mut hits = HitList::new();
            if let Some((t1, t2)) = intersect_ray_with_sphere(origin, direction, *sphere) {
                for t in [t1, t2] {
                    let point = vectors::sum(origin, vectors::scale(t, direction));
                    let normal = vectors::normalize(vectors::difference(point, sphere.center));
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
        Shape::Transformed { shape, transform } => {
            let inv_transform = transform.inverse();

            // Трансформируем луч в локальные координаты
            let local_origin = vectors::difference(origin, inv_transform.translation);
            let local_origin = Vector3f::from_vec(crate::vectors::multiply_vec_and_mat(
                local_origin.to_vec(),
                inv_transform.rotation,
            ));
            let local_direction = Vector3f::from_vec(crate::vectors::multiply_vec_and_mat(
                direction.to_vec(),
                inv_transform.rotation,
            ));

            // Пересекаем с исходной формой в локальных координатах
            let local_hits = intersect_ray_with_shape(local_origin, local_direction, shape);

            // Преобразуем точки и нормали обратно в мировые координаты
            let mut world_hits = HitList::new();
            for mut hit in local_hits {
                hit.point = transform.transform_point(hit.point);
                hit.normal = vectors::normalize(transform.transform_direction(hit.normal));
                world_hits.push(hit);
            }
            world_hits
        }
    }
}

fn merge_csg_hits(left_hits: HitList, right_hits: HitList, op: &CSGOperation) -> HitList {
    let mut all_events = Vec::new();
    all_events.extend(left_hits.iter().map(|h| (h, true)));
    all_events.extend(right_hits.iter().map(|h| (h, false)));

    // Сортируем по t
    all_events.sort_by(|a, b| a.0.t.partial_cmp(&b.0.t).unwrap());

    // Алгоритм: отслеживаем, внутри ли мы левого и правого объекта
    let mut in_left = false;
    let mut in_right = false;
    let mut result = HitList::new();

    for (hit, is_left) in all_events {
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
                    result.push(*hit);
                } else {
                    result.push(hit.negate_normal());
                }
            }
            (true, false) => {
                // Выход — нормаль разворачивается
                if is_left {
                    result.push(hit.negate_normal());
                } else {
                    result.push(*hit);
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

#[test]
fn test_canvas_to_viewport() {
    let point = canvas_to_viewport(500, 500, 1000, 1000);
    assert!(test_utils::roughly_equals(point.x, 0.5));
    assert!(test_utils::roughly_equals(point.y, 0.5));
    assert!(test_utils::roughly_equals(point.z, 1.0));
}
