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
    let mut result = Vec::<Triangle4f>::new();

    for triangle in triangles {
        let mut is_inside_all_planes = true;

        'clipping: for clipping_plane in clipping_planes {
            if is_vertex_outside(clipping_plane, triangle.a) ||
                is_vertex_outside(clipping_plane, triangle.b) ||
                is_vertex_outside(clipping_plane, triangle.c) {
                is_inside_all_planes = false;
                break 'clipping
            }
        }

        if is_inside_all_planes {
            result.push(triangle);
        }
    }

    result
}

fn is_vertex_outside(plane: &Plane, vertex: Vector4f) -> bool {
    let point3d = Point3D::from_vector4f(vertex);

    dot_product(plane.normal, difference(point3d, plane.point)) < 0.0
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
    canvas.viewport_to_canvas(camera.project_vertex(vertex), camera)
}
