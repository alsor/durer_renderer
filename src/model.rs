use super::Point3D;
use Color;

pub struct Model {
    pub vertices: Vec<Point3D>,
    pub faces: Vec<Vec<i32>>,
    pub colors: Vec<Color>
}