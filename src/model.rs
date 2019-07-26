use ::{Point3D, Triangle, Color};
use texture::Texture;
use uv::UV;

pub struct Model<'a> {
    pub vertices: Vec<Point3D>,
    pub triangles: Vec<Triangle>,
    pub colors: Vec<Color>,
    pub textures: Option<Vec<&'a Texture>>,
    pub uvs: Option<Vec<[UV; 3]>>
}