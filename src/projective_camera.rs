use crate::matrix44f::Matrix44f;
use crate::plane::Plane;
use crate::plane::PlaneType;
use crate::plane::PlaneType::*;
use crate::Point2D;
use crate::Vector3f;
use crate::vector4f::Vector4f;
use crate::vectors::cross_product;

#[derive(Copy, Clone)]
pub struct ProjectiveCamera {
    pub viewport_size: f64,
    pub projection_plane_z: f64,
    pub position: Vector4f,
    pub rotation: Matrix44f
}

impl ProjectiveCamera {
    pub fn project(&self, point: Vector3f) -> Point2D {
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

    // we are in left handed coordinate-system
    pub fn clipping_planes(&self) -> Vec<Plane> {
        let half_viewport_size = self.viewport_size / 2.0;

        vec![
            Plane {
                plane_type: Near,
                normal: Vector3f { x: 0.0, y: 0.0, z: 1.0 },
                point: Vector3f { x: 0.0, y: 0.0, z: self.projection_plane_z }
            },
            Plane {
                plane_type: Left,
                normal: self.left_plane_normal(half_viewport_size),
                point: Vector3f { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Right,
                normal: self.right_plane_normal(half_viewport_size),
                point: Vector3f { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Top,
                normal: self.top_plane_normal(half_viewport_size),
                point: Vector3f { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Bottom,
                normal: self.bottom_plane_normal(half_viewport_size),
                point: Vector3f { x: 0.0, y: 0.0, z: 0.0 }
            }
        ]
    }

    fn right_plane_normal(&self, half_viewport_size: f64) -> Vector3f {
        let v1 = Vector3f {
            x: half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Vector3f {
            x: half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }

    fn top_plane_normal(&self, half_viewport_size: f64) -> Vector3f {
        let v1 = Vector3f {
            x: half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Vector3f {
            x: -half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }

    fn left_plane_normal(&self, half_viewport_size: f64) -> Vector3f {
        let v1 = Vector3f {
            x: -half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Vector3f {
            x: -half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }

    fn bottom_plane_normal(&self, half_viewport_size: f64) -> Vector3f {
        let v1 = Vector3f {
            x: -half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Vector3f {
            x: half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }
}