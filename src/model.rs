use super::Point3D;

pub struct Model {
    pub vertices: Vec<Point3D>,
    pub faces: Vec<Vec<i32>>
}