use crate::matrix44f::Matrix44f;
use crate::model::Model;
use crate::vector4f::Vector4f;
use crate::Vector3f;

pub struct Instance<'a> {
    pub model: &'a Model<'a>,
    position: Vector4f,
    scale: f64,
    rotation: Vector3f,
    pub position_delta: Vector3f,
    pub scale_delta: f64,
    pub rotation_delta: Vector3f,
}

impl<'a> Instance<'a> {
    pub fn new(model: &'a Model, position: Vector3f, scale: f64, rotation: Vector3f) -> Self {
        Self {
            model,
            position: position.into(),
            scale,
            rotation,
            position_delta: Vector3f::zero_vector(),
            scale_delta: 0.0,
            rotation_delta: Vector3f::zero_vector(),
        }
    }

    pub fn transform(&self) -> Matrix44f {
        self.rotation_transform()
            .multiply(Matrix44f::uniform_scale(self.scale))
            .multiply(Matrix44f::translation(self.position))
    }

    pub fn rotation_transform(&self) -> Matrix44f {
        Matrix44f::rotation_x(self.rotation.x)
            .multiply(Matrix44f::rotation_y(self.rotation.y).multiply(Matrix44f::rotation_z(self.rotation.z)))
    }

    pub fn apply_deltas(&mut self) {
        self.apply_rotation_deltas();
        self.apply_position_deltas();
        self.apply_scale_delta();
    }

    fn apply_rotation_deltas(&mut self) {
        self.rotation.x += self.rotation_delta.x;
        self.rotation.y += self.rotation_delta.y;
        self.rotation.z += self.rotation_delta.z;
    }

    fn apply_position_deltas(&mut self) {
        self.position.x += self.position_delta.x;
        self.position.y += self.position_delta.y;
        self.position.z += self.position_delta.z;
    }

    fn apply_scale_delta(&mut self) {
        self.scale += self.scale_delta;
    }
}

#[test]
fn test_new() {
    use super::Vector3f;
    let vertices = vec![
        Vector3f { x: 1.0, y: 0.0, z: 0.0 },
        Vector3f { x: 0.0, y: 1.0, z: 0.0 },
        Vector3f { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![crate::Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        name: "test_model",
        vertices,
        triangles,
        colors: vec![common::Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None,
    };

    let instance = Instance::new(&model, Vector3f::zero_vector(), 1.0, Vector3f::zero_vector());
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}

#[test]
fn test_new_with_position() {
    use super::Vector3f;
    let vertices = vec![
        Vector3f { x: 1.0, y: 0.0, z: 0.0 },
        Vector3f { x: 0.0, y: 1.0, z: 0.0 },
        Vector3f { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![crate::Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        name: "test_model",
        vertices,
        triangles,
        colors: vec![common::Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None,
    };

    let instance = Instance::new(
        &model,
        Vector3f { x: 1.0, y: 1.0, z: 1.0 },
        1.0,
        Vector3f::zero_vector(),
    );
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}

#[test]
fn test_new_with_scale() {
    use super::Vector3f;
    let vertices = vec![
        Vector3f { x: 1.0, y: 0.0, z: 0.0 },
        Vector3f { x: 0.0, y: 1.0, z: 0.0 },
        Vector3f { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![crate::Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        name: "test_model",
        vertices,
        triangles,
        colors: vec![common::Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None,
    };

    let instance = Instance::new(&model, Vector3f::zero_vector(), 2.0, Vector3f::zero_vector());
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}

#[test]
fn test_new_with_rotation() {
    use super::Vector3f;
    let vertices = vec![
        Vector3f { x: 1.0, y: 0.0, z: 0.0 },
        Vector3f { x: 0.0, y: 1.0, z: 0.0 },
        Vector3f { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![crate::Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        name: "test_model",
        vertices,
        triangles,
        colors: vec![common::Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None,
    };

    let instance = Instance::new(
        &model,
        Vector3f::zero_vector(),
        1.0,
        Vector3f { x: 0.0, y: 30.0, z: 0.0 },
    );
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}
