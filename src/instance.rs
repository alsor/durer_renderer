use model::Model;
use matrix44f::Matrix44f;
use vector4f::Vector4f;
use ::{Color, Triangle};

pub struct Instance<'a> {
    pub model: &'a Model<'a>,
    pub transform: Option<Matrix44f>,
    pub rotation_transform: Option<Matrix44f>
}

impl<'a> Instance<'a> {
    pub fn new(
        model: &'a Model,
        position: Option<Vector4f>,
        scale: Option<f64>,
        rotation: Option<Matrix44f>
    ) -> Self {
        let mut transform = rotation;

        match scale {
            None => (),
            Some(scale_factor) => {
                let scale_matrix = Matrix44f::uniform_scale(scale_factor);
                transform = match transform {
                    None => { Some(scale_matrix) },
                    Some(existing_transform) => {
                        Some(existing_transform.multiply(scale_matrix))
                    },
                };
            },
        };

        match position {
            None => (),
            Some(position_vector) => {
                let translate_matrix = Matrix44f::translation(position_vector);
                transform = match transform {
                    None => { Some(translate_matrix) },
                    Some(existing_transform) => {
                        Some(existing_transform.multiply(translate_matrix))
                    },
                };
            },
        };

        Self { model, transform, rotation_transform: rotation }
    }
}

#[test]
fn test_new() {
    use super::Point3D;
    let vertices = vec![
        Point3D { x: 1.0, y: 0.0, z: 0.0 },
        Point3D { x: 0.0, y: 1.0, z: 0.0 },
        Point3D { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        vertices,
        triangles,
        colors: vec![Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None
    };

    let instance = Instance::new(
        &model,
        None,
        None,
        None
    );
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}

#[test]
fn test_new_with_position() {
    use super::Point3D;
    let vertices = vec![
        Point3D { x: 1.0, y: 0.0, z: 0.0 },
        Point3D { x: 0.0, y: 1.0, z: 0.0 },
        Point3D { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        vertices,
        triangles,
        colors: vec![Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None
    };

    let instance = Instance::new(
        &model,
        Some(Vector4f { x: 1.0, y: 1.0, z: 1.0, w: 0.0 }),
        None,
        None
    );
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}

#[test]
fn test_new_with_scale() {
    use super::Point3D;
    let vertices = vec![
        Point3D { x: 1.0, y: 0.0, z: 0.0 },
        Point3D { x: 0.0, y: 1.0, z: 0.0 },
        Point3D { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        vertices,
        triangles,
        colors: vec![Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None
    };

    let instance = Instance::new(
        &model,
        None,
        Some(2.0),
        None
    );
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}

#[test]
fn test_new_with_rotation() {
    use super::Point3D;
    let vertices = vec![
        Point3D { x: 1.0, y: 0.0, z: 0.0 },
        Point3D { x: 0.0, y: 1.0, z: 0.0 },
        Point3D { x: 0.0, y: 0.0, z: 1.0 },
    ];
    let triangles = vec![Triangle::new_with_calculated_normals(&vertices, [0, 1, 2])];
    let model = Model {
        vertices,
        triangles,
        colors: vec![Color { r: 0, g: 0, b: 0 }],
        textures: None,
        uvs: None
    };

    let instance = Instance::new(
        &model,
        None,
        None,
        Some(Matrix44f::rotation_y(30.0))
    );
    for vertex in &instance.model.vertices {
        println!("{:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z);
    }
}
