use super::Point2D;
use super::Point3D;
use vector4f::Vector4f;
use matrix44f::Matrix44f;
use plane::Plane;
use plane::PlaneType::*;

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

    pub fn clipping_planes(&self) -> Vec<Plane> {
        let s2 = (2.0 as f64).sqrt();

        vec![
            Plane {
                plane_type: Near,
                normal: Point3D { x: 0.0, y: 0.0, z: 1.0 },
                point: Point3D { x: 0.0, y: 0.0, z: self.projection_plane_z }
            },
            Plane {
                plane_type: Left,
                normal: Point3D { x: s2, y: 0.0, z: s2 },
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Right,
                normal: Point3D { x: -s2, y: 0.0, z: s2 },
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Top,
                normal: Point3D { x: 0.0, y: -s2, z: s2 },
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Bottom,
                normal: Point3D { x: 0.0, y: s2, z: s2 },
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            }
        ]
    }
}