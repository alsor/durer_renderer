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
        let mut triangles = vec![
            Triangle4f {
                a: transformed_vertices[face[0] as usize],
                b: transformed_vertices[face[1] as usize],
                c: transformed_vertices[face[2] as usize],
            }
        ];
//
//        'triangle_check: for clipping_plan in clipping_planes {
//            for triangle in triangles
//            if triangle_outside?() {
//                break 'triangle_check;
//            }
//        }
//


        let mut all_vertices_in = true;

        'vertex_check: for vertex_index in face {
            let vertex = Point3D::from_vector4f(
                transformed_vertices[*vertex_index as usize]
            );
            for clipping_plane in clipping_planes {
                let dot_product = dot_product(
                    clipping_plane.normal,
                    difference(vertex, clipping_plane.point)
                );
                if dot_product < 0.0 {
                    trace!(
                        "vertex: [{:.2} {:.2} {:.2}] outside: [{:.2} {:.2} {:.2}] ({:?})",
                        vertex.x,
                        vertex.y,
                        vertex.z,
                        clipping_plane.normal.x,
                        clipping_plane.normal.y,
                        clipping_plane.normal.z,
                        clipping_plane.plane_type
                    );
                    all_vertices_in = false;
                    break 'vertex_check;
                }
            }
        }

        if all_vertices_in {
            for triangle in triangles {
                render_triangle_wireframe(triangle, camera, canvas);
            }
        }
    }
}

fn render_face_filled(face_points: &Vec<Point>, canvas: &mut BufferCanvas) {
    draw_filled_triangle(
        face_points[0],
        face_points[1],
        face_points[2],
        Color { r: 172, g: 179, b: 191 },
        canvas
    );
}

fn render_triangle_wireframe(
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
