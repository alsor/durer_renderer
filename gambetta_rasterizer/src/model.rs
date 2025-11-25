use crate::texture::Texture;
use common::vectors;
use common::{Color, Vector3f};

pub struct Model<'a> {
    pub name: &'a str,
    pub vertices: Vec<Vector3f>,
    pub triangles: Vec<Triangle>,
    pub colors: Vec<Color>,
    pub textures: Option<Vec<&'a Texture>>,
    pub uvs: Option<Vec<[UV; 3]>>,
}

#[derive(Copy, Clone)]
pub struct Triangle {
    pub indexes: [usize; 3],
    pub normals: [Vector3f; 3],
    pub calculated_normal: Vector3f,
}

impl Triangle {
    pub fn new_with_calculated_normals(vertices: &Vec<Vector3f>, indexes: [usize; 3]) -> Self {
        let calculated_normal = vectors::normalize(Self::calculate_normal_in_left(indexes, vertices));

        Self {
            indexes,
            normals: [calculated_normal, calculated_normal, calculated_normal],
            calculated_normal,
        }
    }

    pub fn new_with_provided_normals(
        vertices: &Vec<Vector3f>,
        indexes: [usize; 3],
        normal_directions: [Vector3f; 3],
    ) -> Self {
        let calculated_normal = vectors::normalize(Self::calculate_normal_in_left(indexes, vertices));

        let normals = [
            vectors::normalize(normal_directions[0]),
            vectors::normalize(normal_directions[1]),
            vectors::normalize(normal_directions[2]),
        ];

        Self { indexes, normals, calculated_normal }
    }

    fn calculate_normal_in_left(indexes: [usize; 3], vertices: &Vec<Vector3f>) -> Vector3f {
        let vector1 = vectors::difference(vertices[indexes[2]], vertices[indexes[1]]);
        let vector2 = vectors::difference(vertices[indexes[1]], vertices[indexes[0]]);
        vectors::cross_product(vector2, vector1)
    }
}

#[test]
fn test_new_with_calculated_normal() {
    let triangle = Triangle::new_with_calculated_normals(
        &vec![
            Vector3f { x: 0.0, y: 0.0, z: 0.0 },
            Vector3f { x: 5.0, y: 0.0, z: 0.0 },
            Vector3f { x: 0.0, y: 0.0, z: 5.0 },
        ],
        [2, 1, 0],
    );
    assert!(test_utils::roughly_equals(triangle.normals[0].x, 0.0));
    assert!(test_utils::roughly_equals(triangle.normals[0].y, 1.0));
    assert!(test_utils::roughly_equals(triangle.normals[0].z, 0.0));

    println!(
        "normal: {:.2} {:.2} {:.2}",
        triangle.normals[0].x, triangle.normals[0].y, triangle.normals[0].z
    );
}

#[derive(Clone, Copy)]
pub struct UV {
    pub u: f64,
    pub v: f64,
}

pub fn cube<'a>(size: f64) -> Model<'a> {
    let half_size = size / 2.0;

    let vertices = vec![
        Vector3f { x: half_size, y: half_size, z: half_size },
        Vector3f { x: -half_size, y: half_size, z: half_size },
        Vector3f { x: -half_size, y: -half_size, z: half_size },
        Vector3f { x: half_size, y: -half_size, z: half_size },
        Vector3f { x: half_size, y: half_size, z: -half_size },
        Vector3f { x: -half_size, y: half_size, z: -half_size },
        Vector3f { x: -half_size, y: -half_size, z: -half_size },
        Vector3f { x: half_size, y: -half_size, z: -half_size },
    ];

    let triangles = vec![
        Triangle::new_with_calculated_normals(&vertices, [0, 1, 2]),
        Triangle::new_with_calculated_normals(&vertices, [0, 2, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 0, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 3, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 4, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 7, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 5, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 6, 2]),
        Triangle::new_with_calculated_normals(&vertices, [4, 5, 1]),
        Triangle::new_with_calculated_normals(&vertices, [4, 1, 0]),
        Triangle::new_with_calculated_normals(&vertices, [2, 6, 7]),
        Triangle::new_with_calculated_normals(&vertices, [2, 7, 3]),
    ];

    // let mut rng = rand::thread_rng();
    // let colors = vec![
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    //     Color { r: rng.gen(), g: rng.gen(), b: rng.gen() },
    // ];

    let colors = vec![
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
    ];

    Model {
        name: "cube",
        vertices,
        triangles,
        colors,
        textures: None,
        uvs: None,
    }
}

pub fn textured_cube(size: f64, texture: &Texture) -> Model {
    let half_size = size / 2.0;

    let vertices = vec![
        Vector3f { x: half_size, y: half_size, z: half_size },
        Vector3f { x: -half_size, y: half_size, z: half_size },
        Vector3f { x: -half_size, y: -half_size, z: half_size },
        Vector3f { x: half_size, y: -half_size, z: half_size },
        Vector3f { x: half_size, y: half_size, z: -half_size },
        Vector3f { x: -half_size, y: half_size, z: -half_size },
        Vector3f { x: -half_size, y: -half_size, z: -half_size },
        Vector3f { x: half_size, y: -half_size, z: -half_size },
    ];

    let triangles = vec![
        Triangle::new_with_calculated_normals(&vertices, [0, 1, 2]),
        Triangle::new_with_calculated_normals(&vertices, [0, 2, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 0, 3]),
        Triangle::new_with_calculated_normals(&vertices, [4, 3, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 4, 7]),
        Triangle::new_with_calculated_normals(&vertices, [5, 7, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 5, 6]),
        Triangle::new_with_calculated_normals(&vertices, [1, 6, 2]),
        Triangle::new_with_calculated_normals(&vertices, [1, 0, 5]),
        Triangle::new_with_calculated_normals(&vertices, [5, 0, 4]),
        Triangle::new_with_calculated_normals(&vertices, [2, 6, 7]),
        Triangle::new_with_calculated_normals(&vertices, [2, 7, 3]),
    ];

    let colors = vec![
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
        Color { r: 119, g: 136, b: 153 },
    ];

    let textures = vec![
        texture, texture, texture, texture, texture, texture, texture, texture, texture, texture, texture,
        texture,
    ];

    let uvs = vec![
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 0.0, v: 1.0 }],
        [UV { u: 0.0, v: 1.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }],
        [UV { u: 0.0, v: 0.0 }, UV { u: 1.0, v: 1.0 }, UV { u: 0.0, v: 1.0 }],
    ];

    Model {
        name: "cube",
        vertices,
        triangles,
        colors,
        textures: Some(textures),
        uvs: Some(uvs),
    }
}

pub fn sphere<'a>(divs: i32) -> Model<'a> {
    let mut vertices = Vec::<Vector3f>::new();
    let mut triangles = Vec::<Triangle>::new();
    let mut colors = Vec::<Color>::new();

    let delta_angle = 2.0 * std::f64::consts::PI / (divs as f64);

    // generate vertices
    for d in 0..(divs + 1) {
        let y = (2.0 / (divs as f64)) * ((d as f64) - (divs as f64) / 2.0);
        let radius = (1.0 - y * y).sqrt();

        for i in 0..divs {
            let x = radius * ((i as f64) * delta_angle).cos();
            let z = radius * ((i as f64) * delta_angle).sin();
            vertices.push(Vector3f { x, y, z });
            // println!("generated vertex: {:.2}, {:.2}, {:.2}", x, y, z)
        }
    }

    // generate triangles
    for d in 0..divs {
        for i in 0..(divs - 1) {
            let i0 = d * divs + i;

            triangles.push(Triangle::new_with_provided_normals(
                &vertices,
                [i0 as usize, (i0 + divs + 1) as usize, (i0 + 1) as usize],
                [
                    vertices[i0 as usize],
                    vertices[(i0 + divs + 1) as usize],
                    vertices[(i0 + 1) as usize],
                ],
            ));
            colors.push(Color { r: 119, g: 136, b: 153 });

            triangles.push(Triangle::new_with_provided_normals(
                &vertices,
                [i0 as usize, (i0 + divs) as usize, (i0 + divs + 1) as usize],
                [
                    vertices[i0 as usize],
                    vertices[(i0 + divs) as usize],
                    vertices[(i0 + divs + 1) as usize],
                ],
            ));
            colors.push(Color { r: 119, g: 136, b: 153 });
        }
    }

    Model {
        name: "sphere",
        vertices,
        triangles,
        colors,
        textures: None,
        uvs: None,
    }
}

pub fn two_unit_cube<'a>() -> Model<'a> {
    cube(2.0)
}
