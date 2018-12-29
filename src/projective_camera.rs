use super::Point2D;
use super::Point3D;
use vector4f::Vector4f;
use matrix44f::Matrix44f;
use plane::Plane;
use plane::PlaneType::*;
use vectors::cross_product;

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

    // we are in left handed coordinate-system
    pub fn clipping_planes(&self) -> Vec<Plane> {
        let half_viewport_size = self.viewport_size / 2.0;

        vec![
            Plane {
                plane_type: Near,
                normal: Point3D { x: 0.0, y: 0.0, z: 1.0 },
                point: Point3D { x: 0.0, y: 0.0, z: self.projection_plane_z }
            },
            Plane {
                plane_type: Left,
                normal: self.left_plane_normal(half_viewport_size),
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Right,
                normal: self.right_plane_normal(half_viewport_size),
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Top,
                normal: self.top_plane_normal(half_viewport_size),
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            },
            Plane {
                plane_type: Bottom,
                normal: self.bottom_plane_normal(half_viewport_size),
                point: Point3D { x: 0.0, y: 0.0, z: 0.0 }
            }
        ]
    }

    fn right_plane_normal(&self, half_viewport_size: f64) -> Point3D {
        let v1 = Point3D {
            x: half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Point3D {
            x: half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }

    fn top_plane_normal(&self, half_viewport_size: f64) -> Point3D {
        let v1 = Point3D {
            x: half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Point3D {
            x: -half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }

    fn left_plane_normal(&self, half_viewport_size: f64) -> Point3D {
        let v1 = Point3D {
            x: -half_viewport_size,
            y: half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Point3D {
            x: -half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }

    fn bottom_plane_normal(&self, half_viewport_size: f64) -> Point3D {
        let v1 = Point3D {
            x: -half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z,
        };
        let v2 = Point3D {
            x: half_viewport_size,
            y: -half_viewport_size,
            z: self.projection_plane_z
        };

        cross_product(v1, v2)
    }
}