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
    let mut canvas_points =
        Vec::<Point>::with_capacity(instance.model.vertices.len());

    for point3d in &instance.model.vertices {
        let vertex = point3d.to_vector4f();

        let transformed_vertex = vertex.transform(transform);
        transformed_vertices.push(transformed_vertex);

        let viewport_point = camera.project_vertex(transformed_vertex);
        let canvas_point = canvas.viewport_to_canvas(viewport_point, camera);
        trace!("model {:.2} {:.2} {:.2} =>transformed: {:.2} {:.2} {:.2} =>viewport point: {:.2} {:.2} =>canvas point: {:.2} {:.2}",
             point3d.x, point3d.y, point3d.z,
             transformed_vertex.x, transformed_vertex.y, transformed_vertex.z,
             viewport_point.x, viewport_point.y,
             canvas_point.x, canvas_point.y
        );

        canvas_points.push(canvas_point);
    }

    for face in &instance.model.faces {
        let mut all_vertices_in = true;
//        let mut vertices_in = Vec::<Point3D>::with_capacity(face.len());
//        let mut vertices_out = Vec::<Point3D>::with_capacity(face.len());



        // face : [index, index, index, index]

//        for vertex_in_face_index in 0..face.len() {
//            let vertex = transformed_vertices[face[vertex_in_face_index]];
//        }

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
            debug!("rendering face");

            let face_points = vec![
                canvas_points[face[0] as usize],
                canvas_points[face[1] as usize],
                canvas_points[face[2] as usize],
            ];

            render_face_wireframe(&face_points, canvas);
//        } else {
//
        }
    }

//    for face in &instance.model.faces {
//        render_face_wireframe(face, &canvas_points, canvas);
//        render_face_filled(face, &canvas_points, canvas);
//    }
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

fn render_face_wireframe(face_points: &Vec<Point>, canvas: &mut BufferCanvas) {
    for vertex_index in 0..face_points.len() {
        let start_vertex;
        let end_vertex;
        if vertex_index + 1 < face_points.len() {
            start_vertex = vertex_index;
            end_vertex = vertex_index + 1;
        } else {
            start_vertex = vertex_index;
            end_vertex = 0;
        }
        canvas.draw_line(
            face_points[start_vertex],
            face_points[end_vertex],
            Color { r: 255, g: 255, b: 255 }
        );
    }
}
