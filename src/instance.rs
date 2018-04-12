use model::Model;
use matrix44f::Matrix44f;
use vector4f::Vector4f;

pub struct Instance<'a> {
    pub model: &'a Model,
    pub vertices: Vec<Vector4f>
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

        match transform {
            None => {
                let mut vertices = Vec::<Vector4f>::with_capacity(model.vertices.len());
                for point in &model.vertices {
                    vertices.push(point.to_vector4f());
                }
                Instance { model, vertices }
            },
            Some(transform_matrix) => { Self::transform_model(model, transform_matrix) }
        }
    }

    pub fn transform_model(model: &'a Model, transform: Matrix44f) -> Self {
        let mut vertices = Vec::<Vector4f>::with_capacity(model.vertices.len());

        for point in &model.vertices {
            let vertex = point.to_vector4f();
            vertices.push(vertex.transform(&transform))
        }

        Self { model, vertices }
    }
}

#[test]
fn test_new() {
    use super::Point3D;
    let model = Model {
        vertices: vec![
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 1.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 1.0 },
        ],
        faces: vec![vec![0, 1, 2]]
    };

    let instance = Instance::new(
        &model,
        None,
        None,
        None
    );
    for vertex in &instance.vertices {
        println!("{:.2} {:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z, vertex.w);
    }
}

#[test]
fn test_new_with_position() {
    use super::Point3D;
    let model = Model {
        vertices: vec![
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 1.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 1.0 },
        ],
        faces: vec![vec![0, 1, 2]]
    };

    let instance = Instance::new(
        &model,
        Some(Vector4f { x: 1.0, y: 1.0, z: 1.0, w: 0.0 }),
        None,
        None
    );
    for vertex in &instance.vertices {
        println!("{:.2} {:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z, vertex.w);
    }
}

#[test]
fn test_new_with_scale() {
    use super::Point3D;
    let model = Model {
        vertices: vec![
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 1.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 1.0 },
        ],
        faces: vec![vec![0, 1, 2]]
    };

    let instance = Instance::new(
        &model,
        None,
        Some(2.0),
        None
    );
    for vertex in &instance.vertices {
        println!("{:.2} {:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z, vertex.w);
    }
}

#[test]
fn test_new_with_rotation() {
    use super::Point3D;
    let model = Model {
        vertices: vec![
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 1.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 1.0 },
        ],
        faces: vec![vec![0, 1, 2]]
    };

    let instance = Instance::new(
        &model,
        None,
        None,
        Some(Matrix44f::rotation_y(30.0))
    );
    for vertex in &instance.vertices {
        println!("{:.2} {:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z, vertex.w);
    }
}

#[test]
fn test_transform_model() {
    use super::Point3D;
    let model = Model {
        vertices: vec![
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 1.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 1.0 },
        ],
        faces: vec![vec![0, 1, 2]]
    };

    let transform = Matrix44f::translation(Vector4f { x: 1.0, y: 1.0, z: 1.0, w: 0.0 });
    let instance = Instance::transform_model(&model, transform);
    for vertex in &instance.vertices {
        println!("{:.2} {:.2} {:.2} {:.2}", vertex.x, vertex.y, vertex.z, vertex.w);
    }
}
