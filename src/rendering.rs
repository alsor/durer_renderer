use instance::Instance;
use buffer_canvas::BufferCanvas;
use projective_camera::ProjectiveCamera;
use super::Point2D;
use super::Point3D;
use super::Point;
use super::Color;
use matrix44f::Matrix44f;
use vector4f::Vector4f;
use plane::Plane;
use vectors::dot_product;
use vectors::difference;
use ::{Triangle, Triangle4f, face_visible};
use vectors::sum;
use vectors::scale;
use ::{Light, vectors};
use ::{face_visible_4f, transform};
use sdl2::video::WindowPos::Positioned;
use ::{ShadingModel, RenderingSettings};

pub fn render_scene(
    instances: &Vec<Instance>,
    lights: &Vec<Light>,
    camera: &ProjectiveCamera,
    rendering_settings: &RenderingSettings,
    canvas: &mut BufferCanvas
) {
    let camera_transform = camera.camera_transform();
    let camera_rotation_transform = camera.rotation.transpose();
    let clipping_planes = camera.clipping_planes();
    for instance in instances {
        render_instance(
            instance,
            canvas,
            camera,
            lights,
            rendering_settings,
            camera_transform,
            camera_rotation_transform,
            &clipping_planes
        );
    }
}

fn render_instance(
    instance: &Instance,
    canvas: &mut BufferCanvas,
    camera: &ProjectiveCamera,
    lights: &Vec<Light>,
    rendering_settings: &RenderingSettings,
    camera_transform: Matrix44f,
    camera_rotation_transform: Matrix44f,
    clipping_planes: &Vec<Plane>
) {
    debug!("rendering instance");

    let mut transformed_lights = Vec::<Light>::with_capacity(lights.len());
    for light in lights {
        let transformed_light = match *light {
            Light::Ambient { intensity } => {
                Light::Ambient { intensity }
            },
            Light::Point { intensity, position } => {
                let transformed_position =
                    position.to_vector4f().transform(camera_transform);
                Light::Point { intensity, position: Point3D::from_vector4f(transformed_position) }
            }
            Light::Directional { intensity, direction } => {
                let transformed_direction =
                    direction.to_vector4f().transform(camera_rotation_transform);
                Light::Directional { intensity, direction: Point3D::from_vector4f(transformed_direction) }
            }
        };
        transformed_lights.push(transformed_light);
    }

    let instance_transform = match instance.transform {
        None => { camera_transform },
        Some(instance_transform) => { instance_transform.multiply(camera_transform) },
    };
    let combined_rotation_transform = match instance.rotation_transform {
        None => { camera_rotation_transform },
        Some(instance_rotation_transform) => {
            instance_rotation_transform.multiply(camera_rotation_transform)
        }
    };

    let mut transformed_vertices =
        Vec::<Vector4f>::with_capacity(instance.model.vertices.len());

    for point3d in &instance.model.vertices {
        let vertex = point3d.to_vector4f();

        let transformed_vertex = vertex.transform(instance_transform);
        transformed_vertices.push(transformed_vertex);
    }

    let mut i = 0;
    for triangle in &instance.model.triangles {
        let transformed_triangle_normal =
            triangle.calculated_normal.to_vector4f().transform(combined_rotation_transform);

        let is_face_visible = if rendering_settings.backface_culling {
            face_visible_4f(
                Point3D::from_vector4f(transformed_vertices[triangle.indexes[0]]),
                Point3D::from_vector4f(transformed_triangle_normal),
            )
        } else {
            true
        };

        if is_face_visible {
            let triangles = clip_triangles(
                convert_face_to_triangles(
                    triangle,
                    &transformed_vertices,
                    combined_rotation_transform,
                    instance.model.colors[i]
                ),
                clipping_planes
            );

            for triangle in triangles {
                render_filled_triangle(
                    triangle,
                    camera,
                    canvas,
                    &transformed_lights,
                    rendering_settings
                );

//                render_wireframe_triangle(triangle, camera, canvas);
            }
        }
        i += 1;
    }
}

fn face_normal_direction_in_right(face: &Vec<i32>, vertices: &[Vector4f]) -> Point3D {
    let vector1 = vectors::difference(
        Point3D::from_vector4f(vertices[face[2] as usize]),
        Point3D::from_vector4f(vertices[face[1] as usize])
    );
    let vector2 = vectors::difference(
        Point3D::from_vector4f(vertices[face[1] as usize]),
        Point3D::from_vector4f(vertices[face[0] as usize])
    );
    vectors::cross_product(vector1, vector2)
}

fn face_normal_direction_in_left(triangle: &Triangle, vertices: &[Vector4f]) -> Point3D {
    let vector1 = vectors::difference(
        Point3D::from_vector4f(vertices[triangle.indexes[2]]),
        Point3D::from_vector4f(vertices[triangle.indexes[1]])
    );
    let vector2 = vectors::difference(
        Point3D::from_vector4f(vertices[triangle.indexes[1]]),
        Point3D::from_vector4f(vertices[triangle.indexes[0]])
    );
    vectors::cross_product(vector2, vector1)
}

fn clip_triangles(triangles: Vec<Triangle4f>, clipping_planes: &Vec<Plane>) -> Vec<Triangle4f> {
    let mut clipped_triangles = triangles.clone();

    'clipping: for clipping_plane in clipping_planes {
        if clipped_triangles.is_empty() {
            break 'clipping;
        }

        clipped_triangles = clip_triangles_against_plane(clipped_triangles, clipping_plane);
    }

    clipped_triangles
}

fn clip_triangles_against_plane(
    triangles: Vec<Triangle4f>,
    clipping_plane: &Plane
) -> Vec<Triangle4f> {
    let mut result = Vec::<Triangle4f>::new();

    for triangle in triangles {
        for new_triangle in clip_triangle_against_plane(triangle, clipping_plane) {
            result.push(new_triangle);
        }
    }

    result
}

fn clip_triangle_against_plane(triangle: Triangle4f, clipping_plane: &Plane) -> Vec<Triangle4f> {
    let color = triangle.color;
    let mut result = Vec::<Triangle4f>::new();

    let point_a = Point3D::from_vector4f(triangle.a);
    let point_b = Point3D::from_vector4f(triangle.b);
    let point_c = Point3D::from_vector4f(triangle.c);

    let vector_p_a = difference(point_a, clipping_plane.point);
    let vector_p_b = difference(point_b, clipping_plane.point);
    let vector_p_c = difference(point_c, clipping_plane.point);

    let dot_product_n_with_p_a = dot_product(clipping_plane.normal, vector_p_a);
    let dot_product_n_with_p_b = dot_product(clipping_plane.normal, vector_p_b);
    let dot_product_n_with_p_c = dot_product(clipping_plane.normal, vector_p_c);

    let is_a_inside = dot_product_n_with_p_a > 0.0;
    let is_b_inside = dot_product_n_with_p_b > 0.0;
    let is_c_inside = dot_product_n_with_p_c > 0.0;

    // all vertices inside
    if is_a_inside && is_b_inside && is_c_inside {
        result.push(triangle);
    // at least one vertex inside
    } else if is_a_inside || is_b_inside || is_c_inside {
        let mut points_inside = Vec::<Vector4f>::new();

        if is_a_inside {
            points_inside.push(triangle.a);
        }

        // requires split
        if is_a_inside != is_b_inside {
            points_inside.push(
                find_split_vertex(point_a, dot_product_n_with_p_a, point_b, dot_product_n_with_p_b)
            );
        }

        if is_b_inside {
            points_inside.push(triangle.b);
        }

        // requires split
        if is_b_inside != is_c_inside {
            points_inside.push(
                find_split_vertex(point_b, dot_product_n_with_p_b, point_c, dot_product_n_with_p_c)
            );
        }

        if is_c_inside {
            points_inside.push(triangle.c)
        }

        // requires split
        if is_c_inside != is_a_inside {
            points_inside.push(
                find_split_vertex(point_c, dot_product_n_with_p_c, point_a, dot_product_n_with_p_a)
            );
        }

        if points_inside.len() == 4 {
            // split to two triangles
            result.push(
                Triangle4f { a: points_inside[0], b: points_inside[1], c: points_inside[2], color, normals: triangle.normals }
            );
            result.push(
                Triangle4f { a: points_inside[0], b: points_inside[2], c: points_inside[3], color, normals: triangle.normals }
            );
        } else if points_inside.len() == 3 {
            result.push(
                Triangle4f { a: points_inside[0], b: points_inside[1], c: points_inside[2], color, normals: triangle.normals }
            );
        } else {
            panic!("unexpected number of points inside: {}", points_inside.len());
        }
    }

    result
}

fn find_split_vertex(
    point1: Point3D,
    dot_product1: f64,
    point2: Point3D,
    dot_product2: f64
) -> Vector4f {
    let t = dot_product1 / (dot_product1 - dot_product2);
    let vector = difference(point2, point1);
    let result = sum(point1, scale(t, vector)).to_vector4f();
    trace!(
        "found split point between [{:.2} {:.2} {:.2}] and [{:.2} {:.2} {:.2}] is [{:.2} {:.2} {:.2}]",
        point1.x,
        point1.y,
        point1.z,
        point2.x,
        point2.y,
        point2.z,
        result.x,
        result.y,
        result.z,
    );
    result
}

fn is_vertex_outside(plane: &Plane, vertex: Vector4f) -> bool {
    let point3d = Point3D::from_vector4f(vertex);

    dot_product(plane.normal, difference(point3d, plane.point)) < 0.0
}

fn is_vertex_inside(plane: &Plane, vertex: Vector4f) -> bool {
    !is_vertex_outside(plane, vertex)
}

fn convert_face_to_triangles(
    triangle: &Triangle,
    vertices: &Vec<Vector4f>,
    combined_rotation_transform: Matrix44f,
    color: Color
) -> Vec<Triangle4f> {
    let transformed_normals = [
        Point3D::from_vector4f(triangle.normals[0].to_vector4f().transform(combined_rotation_transform)),
        Point3D::from_vector4f(triangle.normals[1].to_vector4f().transform(combined_rotation_transform)),
        Point3D::from_vector4f(triangle.normals[2].to_vector4f().transform(combined_rotation_transform)),
    ];

    vec![
        Triangle4f {
            a: vertices[triangle.indexes[0]],
            b: vertices[triangle.indexes[1]],
            c: vertices[triangle.indexes[2]],
            color,
            normals: transformed_normals
        }
    ]
}

fn render_filled_triangle(
    triangle: Triangle4f,
    camera: &ProjectiveCamera,
    canvas: &mut BufferCanvas,
    lights: &Vec<Light>,
    rendering_settings: &RenderingSettings
) {
    match rendering_settings.shading_model {
        ShadingModel::Flat => flat_shaded_triangle(triangle, camera, lights, canvas),
        ShadingModel::Gouraud => gouraud_shaded_triangle(triangle, camera, lights, canvas),
    }

    if rendering_settings.show_normals {
        draw_normal_to_vertex(triangle.a, triangle.normals[0], camera, canvas);
        draw_normal_to_vertex(triangle.b, triangle.normals[1], camera, canvas);
        draw_normal_to_vertex(triangle.c, triangle.normals[2], camera, canvas);
    }
}

fn draw_normal_to_vertex(
    vertex: Vector4f,
    normal: Point3D,
    camera: &ProjectiveCamera,
    canvas: &mut BufferCanvas
) {
    let start = vertex_to_canvas_point(vertex, camera, canvas);
    let end = vertex_to_canvas_point(
        vectors::sum(Point3D::from_vector4f(vertex), normal).to_vector4f(),
        camera,
        canvas
    );

    if is_point_in_canvas(start, canvas) && is_point_in_canvas(end, canvas) {
        canvas.draw_line(start, end, Color { r: 0, g: 0, b: 255 });
    }

}

fn is_point_in_canvas(point: Point, canvas: &BufferCanvas) -> bool {
    let canvas_half_size = (canvas.size / 2) as i32;
    let min_x = -canvas_half_size;
    let max_x = canvas_half_size;
    let min_y = -canvas_half_size;
    let max_y = canvas_half_size;

    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}

fn compute_illumination(
    vertex: Point3D,
    normal_direction: Point3D,
    camera: &ProjectiveCamera,
    lights: &Vec<Light>
) -> f64 {
    let mut result = 0.0;
    let normal = vectors::normalize(normal_direction);

    for light in lights {
        result += match *light {
            Light::Ambient { intensity } => intensity,
            Light::Point { intensity, position } => {
                let direction = vectors::difference(position, vertex);
                light_from_direction(vertex, normal, direction, intensity)
            }
            Light::Directional { intensity, direction } => {
                light_from_direction(vertex, normal, direction, intensity)
            }
        }
    }
    result
}

fn light_from_direction(
    vertex: Point3D,
    normal: Point3D,
    light_direction: Point3D,
    light_intensity: f64
) -> f64 {
    let mut result = 0.0;
    let shininess= 50;

    // diffuse
    let dot = vectors::dot_product(normal, light_direction);
    if dot > 0.0 {
        // assuming that normal is a unit vector (has length 1)
        result += light_intensity * dot / vectors::length(light_direction);
    }

    // specular
    // TODO add color of the light to this component
    if shininess > 0 {
        let view = vectors::negate(vertex);
        let reflection_direction = vectors::reflect (light_direction, normal);
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

    result
}

fn render_wireframe_triangle(
    triangle: Triangle4f,
    camera: &ProjectiveCamera,
    canvas: &mut BufferCanvas
) {
    let a = vertex_to_canvas_point(triangle.a, camera, canvas);
    let b = vertex_to_canvas_point(triangle.b, camera, canvas);
    let c = vertex_to_canvas_point(triangle.c, camera, canvas);

    canvas.draw_line(a, b, Color { r: 255, g: 255, b: 255 });
    canvas.draw_line(b, c, Color { r: 255, g: 255, b: 255 });
    canvas.draw_line(c, a, Color { r: 255, g: 255, b: 255 });
}

fn flat_shaded_triangle(
    triangle: Triangle4f,
    camera: &ProjectiveCamera,
    lights: &Vec<Light>,
    canvas: &mut BufferCanvas,
) {
    let mut p0 = vertex_to_canvas_point(triangle.a, camera, canvas);
    let mut p1 = vertex_to_canvas_point(triangle.b, camera, canvas);
    let mut p2 = vertex_to_canvas_point(triangle.c, camera, canvas);

    let center = Point3D {
        x: (triangle.a.x + triangle.b.x + triangle.c.x) / 3.0,
        y: (triangle.a.y + triangle.b.y + triangle.c.y) / 3.0,
        z: (triangle.a.z + triangle.b.z + triangle.c.z) / 3.0
    };
    let intensity = compute_illumination(
        center,
        triangle.normals[0],
        camera,
        lights
    );

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
    let mut iz01 = interpolate_float(p0.y, 1.0 / p0.z, p1.y, 1.0 / p1.z);

    let mut x12 = interpolate_int(p1.y, p1.x, p2.y, p2.x);
    let mut h12 = interpolate_float(p1.y, p1.h, p2.y, p2.h);
    let mut iz12 = interpolate_float(p1.y, 1.0 / p1.z, p2.y, 1.0 / p2.z);

    let mut x02 = interpolate_int(p0.y, p0.x, p2.y, p2.x);
    let mut h02 = interpolate_float(p0.y, p0.h, p2.y, p2.h);
    let mut iz02 = interpolate_float(p0.y, 1.0 / p0.z, p2.y, 1.0 / p2.z);

    x01.pop();
    let mut x012 = Vec::<i32>::new();
    x012.append(&mut x01);
    x012.append(&mut x12);

    h01.pop();
    let mut h012 = Vec::<f64>::new();
    h012.append(&mut h01);
    h012.append(&mut h12);

    iz01.pop();
    let mut iz012 = Vec::<f64>::new();
    iz012.append(&mut iz01);
    iz012.append(&mut iz12);

    let mut x_left;
    let mut x_right;
    let mut h_left;
    let mut h_right;
    let mut iz_left;
    let mut iz_right;

    let m = x02.len() / 2;
    if x02[m] < x012[m] {
        x_left = x02;
        x_right = x012;

        h_left = h02;
        h_right = h012;

        iz_left = iz02;
        iz_right = iz012;
    } else {
        x_left = x012;
        x_right = x02;

        h_left = h012;
        h_right = h02;

        iz_left = iz012;
        iz_right = iz02;
    };

    for y in p0.y..(p2.y + 1) {
        let y_index = (y - p0.y) as usize;
        let x_l = x_left[y_index];
        let x_r = x_right[y_index];
        let h_segment = interpolate_float(x_l, h_left[y_index], x_r, h_right[y_index]);
        let iz_segment = interpolate_float(x_l, iz_left[y_index], x_r, iz_right[y_index]);
        for x in x_l..(x_r + 1) {
            let x_index = (x - x_l) as usize;
            let shaded_color = multiply_color(intensity, triangle.color);
            canvas.draw_point(x, y, iz_segment[x_index], shaded_color);
        };
    }
}

fn gouraud_shaded_triangle(
    triangle: Triangle4f,
    camera: &ProjectiveCamera,
    lights: &Vec<Light>,
    canvas: &mut BufferCanvas,
) {
    let mut v0 = Point3D::from_vector4f(triangle.a);
    let mut v1 = Point3D::from_vector4f(triangle.b);
    let mut v2 = Point3D::from_vector4f(triangle.c);

    let mut normal0 = triangle.normals[0];
    let mut normal1 = triangle.normals[1];
    let mut normal2 = triangle.normals[2];

    let mut p0 = vertex_to_canvas_point(triangle.a, camera, canvas);
    let mut p1 = vertex_to_canvas_point(triangle.b, camera, canvas);
    let mut p2 = vertex_to_canvas_point(triangle.c, camera, canvas);

    // sort points from bottom to top
    if p1.y < p0.y {
        let swap = p0;
        p0 = p1;
        p1 = swap;

        let swap = v0;
        v0 = v1;
        v1 = swap;

        let swap = normal0;
        normal0 = normal1;
        normal1 = swap;
    }
    if p2.y < p0.y {
        let swap = p0;
        p0 = p2;
        p2 = swap;

        let swap = v0;
        v0 = v2;
        v2 = swap;

        let swap = normal0;
        normal0 = normal2;
        normal2 = swap;
    }
    if p2.y < p1.y {
        let swap = p1;
        p1 = p2;
        p2 = swap;

        let swap = v1;
        v1 = v2;
        v2 = swap;

        let swap = normal1;
        normal1 = normal2;
        normal2 = swap;
    }

    let i0 = compute_illumination(v0, normal0, camera, lights);
    let i1 = compute_illumination(v1, normal1, camera, lights);
    let i2 = compute_illumination(v2, normal2, camera, lights);

    //interpolating attributes along edges
    let mut x01 = interpolate_int(p0.y, p0.x, p1.y, p1.x);
    let mut h01 = interpolate_float(p0.y, p0.h, p1.y, p1.h);
    let mut iz01 = interpolate_float(p0.y, 1.0 / p0.z, p1.y, 1.0 / p1.z);
    let mut i01 = interpolate_float(p0.y, i0, p1.y, i1);

    let mut x12 = interpolate_int(p1.y, p1.x, p2.y, p2.x);
    let mut h12 = interpolate_float(p1.y, p1.h, p2.y, p2.h);
    let mut iz12 = interpolate_float(p1.y, 1.0 / p1.z, p2.y, 1.0 / p2.z);
    let mut i12 = interpolate_float(p1.y, i1, p2.y, i2);

    let mut x02 = interpolate_int(p0.y, p0.x, p2.y, p2.x);
    let mut h02 = interpolate_float(p0.y, p0.h, p2.y, p2.h);
    let mut iz02 = interpolate_float(p0.y, 1.0 / p0.z, p2.y, 1.0 / p2.z);
    let mut i02 = interpolate_float(p0.y, i0, p2.y, i2);

    // combining 3 edges to left and right boundaries
    x01.pop();
    let mut x012 = Vec::<i32>::new();
    x012.append(&mut x01);
    x012.append(&mut x12);

    h01.pop();
    let mut h012 = Vec::<f64>::new();
    h012.append(&mut h01);
    h012.append(&mut h12);

    iz01.pop();
    let mut iz012 = Vec::<f64>::new();
    iz012.append(&mut iz01);
    iz012.append(&mut iz12);

    i01.pop();
    let mut i012 = Vec::<f64>::new();
    i012.append(&mut i01);
    i012.append(&mut i12);

    let mut x_left;
    let mut x_right;
    let mut h_left;
    let mut h_right;
    let mut iz_left;
    let mut iz_right;
    let mut i_left;
    let mut i_right;

    let m = x02.len() / 2;
    if x02[m] < x012[m] {
        x_left = x02;
        x_right = x012;

        h_left = h02;
        h_right = h012;

        iz_left = iz02;
        iz_right = iz012;

        i_left = i02;
        i_right = i012;
    } else {
        x_left = x012;
        x_right = x02;

        h_left = h012;
        h_right = h02;

        iz_left = iz012;
        iz_right = iz02;

        i_left = i012;
        i_right = i02;
    };

    for y in p0.y..(p2.y + 1) {
        let y_index = (y - p0.y) as usize;
        let x_l = x_left[y_index];
        let x_r = x_right[y_index];
        let h_segment = interpolate_float(x_l, h_left[y_index], x_r, h_right[y_index]);
        let iz_segment = interpolate_float(x_l, iz_left[y_index], x_r, iz_right[y_index]);
        let i_segment = interpolate_float(x_l, i_left[y_index], x_r, i_right[y_index]);
        for x in x_l..(x_r + 1) {
            let x_index = (x - x_l) as usize;
            let shaded_color = multiply_color(i_segment[x_index], triangle.color);
            canvas.draw_point(x, y, iz_segment[x_index], shaded_color);
        };
    }
}

fn vertex_to_canvas_point(vertex: Vector4f, camera: &ProjectiveCamera, canvas: &BufferCanvas)
    -> Point {
    let result = canvas.viewport_to_canvas(vertex, camera);
    trace!(
        "vertex [{:.2} {:.2} {:.2}] converted to canvas point [{} {}]",
        vertex.x,
        vertex.y,
        vertex.z,
        result.x,
        result.y,
    );

    result
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
