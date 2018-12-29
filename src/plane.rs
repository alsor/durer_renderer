use super::Point3D;

#[derive(Copy, Clone, Debug)]
pub enum PlaneType { Near, Left, Right, Top, Bottom }

pub struct Plane {
    pub plane_type: PlaneType,
    pub normal: Point3D,
    pub point: Point3D
}