use super::Point2D;
use super::Point3D;
use vector4f::Vector4f;
use matrix44f::Matrix44f;

#[derive(Copy, Clone)]
pub struct ProjectiveCamera {
    pub viewport_size: f64,
    pub projection_plane_z: f64,
    pub position: Vector4f,
    pub rotation: Matrix44f
}

impl ProjectiveCamera {
    pub fn project(&self, point: Point3D) -> Point2D {
        Point2D {
            x: point.x * self.projection_plane_z / point.z,
            y: point.y * self.projection_plane_z / point.z
        }
    }

    pub fn project_vertex(&self, vertex: Vector4f) -> Point2D {
        Point2D {
            x: vertex.x * self.projection_plane_z / vertex.z,
            y: vertex.y * self.projection_plane_z / vertex.z
        }
    }

    pub fn camera_transform(&self) -> Matrix44f {
        Matrix44f::translation(self.position.negate()).multiply(self.rotation.transpose())
    }
}