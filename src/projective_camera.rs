use super::Point2D;
use super::Point3D;

#[derive(Copy, Clone)]
pub struct ProjectiveCamera {
    pub viewport_size: f64,
    pub projection_plane_z: f64
}

impl ProjectiveCamera {
    pub fn project(&self, point: Point3D) -> Point2D {
        Point2D {
            x: point.x * self.projection_plane_z / point.z,
            y: point.y * self.projection_plane_z / point.z
        }
    }
}