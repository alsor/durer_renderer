use instance::Instance;
use buffer_canvas::BufferCanvas;
use projective_camera::ProjectiveCamera;
use super::Point2D;
use super::Point3D;
use super::Point;
use super::Color;
use matrix44f::Matrix44f;

pub fn render_scene(scene: &Vec<Instance>, camera: &ProjectiveCamera, canvas: &mut BufferCanvas) {
    let camera_transform = camera.camera_transform();
    for instance in scene {
        render_instance(instance, canvas, camera, camera_transform);
    }
}

fn render_instance(
    instance: &Instance,
    canvas: &mut BufferCanvas,
    camera: &ProjectiveCamera,
    camera_transform: Matrix44f
) {
    let transform = match instance.transform {
        None => { camera_transform },
        Some(instance_transform) => { instance_transform.multiply(camera_transform) },
    };
    let mut canvas_points =
        Vec::<Point>::with_capacity(instance.model.vertices.len());

    for point3d in &instance.model.vertices {
        let vertex = point3d.to_vector4f();
        let point2d = camera.project_vertex(vertex.transform(transform));
        canvas_points.push(canvas.viewport_to_canvas(point2d, camera));
    }

    for face in &instance.model.faces {
        render_face(face, &canvas_points, canvas)
    }
}

fn render_face(face: &Vec<i32>, canvas_points: &Vec<Point>, canvas: &mut BufferCanvas) {
    for vertex_index in 0..face.len() {
        let start_vertex;
        let end_vertex;
        if vertex_index + 1 < face.len() {
            start_vertex = vertex_index;
            end_vertex = vertex_index + 1;
        } else {
            start_vertex = vertex_index;
            end_vertex = 0;
        }
        canvas.draw_line(
            canvas_points[face[start_vertex] as usize],
            canvas_points[face[end_vertex] as usize],
            Color { r: 255, g: 255, b: 255 }
        );
    }
}
