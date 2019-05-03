use ::{Point3D, Triangle, Color};

pub struct Model {
    pub vertices: Vec<Point3D>,
    pub triangles: Vec<Triangle>,
    pub colors: Vec<Color>
}