use super::Vector3f;

#[derive(Copy, Clone, Debug)]
pub enum PlaneType { Near, Left, Right, Top, Bottom }

pub struct Plane {
    pub plane_type: PlaneType,
    pub normal: Vector3f,
    pub point: Vector3f
}