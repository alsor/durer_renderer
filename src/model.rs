use crate::{Color, Triangle, Vector3f};
use crate::texture::Texture;
use crate::uv::UV;

pub struct Model<'a> {
    pub name: &'a str,
    pub vertices: Vec<Vector3f>,
    pub triangles: Vec<Triangle>,
    pub colors: Vec<Color>,
    pub textures: Option<Vec<&'a Texture>>,
    pub uvs: Option<Vec<[UV; 3]>>
}