use instance::Instance;
use buffer_canvas::BufferCanvas;
use projective_camera::ProjectiveCamera;
use super::Point2D;
use super::Point3D;
use super::Point;
use super::Color;
use matrix44f::Matrix44f;
use vector4f::Vector4f;
use super::draw_filled_triangle;
use plane::Plane;
use vectors::dot_product;
use vectors::difference;
use Triangle4f;
use vectors::sum;
use vectors::scale;

pub fn render_scene(
    scene: &Vec<Instance>,
    camera: &ProjectiveCamera,
    canvas: &mut BufferCanvas
) {
    let camera_transform = camera.camera_transform();
    let clipping_planes = camera.clipping_planes();
    for instance in scene {
        render_instance(instance, canvas, camera, camera_transform, &clipping_planes);
    }
}

fn render_instance(
    instance: &Instance,
    canvas: &mut BufferCanvas,
    camera: &ProjectiveCamera,
    camera_transform: Matrix44f,
    clipping_planes: &Vec<Plane>
) {
    debug!("rendering instance");

    let transform = match instance.transform {
        None => { camera_transform },
        Some(instance_transform) => { instance_transform.multiply(camera_transform) },
    };

    let mut transformed_vertices =
        Vec::<Vector4f>::with_capacity(instance.model.vertices.len());

    for point3d in &instance.model.vertices {
        let vertex = point3d.to_vector4f();

        let transformed_vertex = vertex.transform(transform);
        transformed_vertices.push(transformed_vertex);
    }

    for face in &instance.model.faces {
        let triangles = clip_triangles(
            convert_face_to_triangles(face, &transformed_vertices),
            clipping_planes
        );

        for triangle in triangles {
            render_wireframe_triangle(triangle, camera, canvas);
//            render_filled_triangle(triangle, camera, canvas);
        }
    }
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
                Triangle4f { a: points_inside[0], b: points_inside[1], c: points_inside[2] }
            );
            result.push(
                Triangle4f { a: points_inside[0], b: points_inside[2], c: points_inside[3] }
            );
        } else if points_inside.len() == 3 {
            result.push(
                Triangle4f { a: points_inside[0], b: points_inside[1], c: points_inside[2] }
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

// simple implementation - just assume that face IS a triangle
fn convert_face_to_triangles(face: &Vec<i32>, vertices: &Vec<Vector4f>) -> Vec<Triangle4f> {
    vec![
        Triangle4f {
            a: vertices[face[0] as usize],
            b: vertices[face[1] as usize],
            c: vertices[face[2] as usize],
        }
    ]
}

fn render_filled_triangle(
    triangle: Triangle4f,
    camera: &ProjectiveCamera,
    canvas: &mut BufferCanvas
) {
    draw_filled_triangle(
        vertex_to_canvas_point(triangle.a, camera, canvas),
        vertex_to_canvas_point(triangle.b, camera, canvas),
        vertex_to_canvas_point(triangle.c, camera, canvas),
        Color { r: 172, g: 179, b: 191 },
        canvas
    );
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

fn vertex_to_canvas_point(vertex: Vector4f, camera: &ProjectiveCamera, canvas: &BufferCanvas)
    -> Point {
    let result = canvas.viewport_to_canvas(camera.project_vertex(vertex), camera);
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
