use instance::Instance;
use buffer_canvas::BufferCanvas;
use projective_camera::ProjectiveCamera;
use super::Point2D;
use super::Point3D;
use super::Point;
use super::Color;

pub fn render_scene(scene: &Vec<Instance>, camera: &ProjectiveCamera, canvas: &mut BufferCanvas) {
    for instance in scene {
        render_instance(instance, canvas, camera);
    }
}

fn render_instance(instance: &Instance, canvas: &mut BufferCanvas, camera: &ProjectiveCamera) {
    let mut canvas_points =
        Vec::<Point>::with_capacity(instance.vertices.len());

    for vertex in &instance.vertices {
        let point2d = camera.project_vertex(*vertex);
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
