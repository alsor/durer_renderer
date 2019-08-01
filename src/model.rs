use ::{Vector3f, Triangle, Color};
use texture::Texture;
use uv::UV;

pub struct Model<'a> {
    pub name: &'a str,
    pub vertices: Vec<Vector3f>,
    pub triangles: Vec<Triangle>,
    pub colors: Vec<Color>,
    pub textures: Option<Vec<&'a Texture>>,
    pub uvs: Option<Vec<[UV; 3]>>
}