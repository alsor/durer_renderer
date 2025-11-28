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

pub fn textured_cube<'a>(size: f64, texture: &'a Texture) -> Model<'a> {
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
    if divs < 3 {
        panic!("Sphere division must be at least 3");
    }

    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    let num_lon = divs as usize;        // количество сегментов по долготе
    let num_lat = (divs / 2) as usize;  // в 2 раза меньше по широте → меньше геометрии у полюсов

    // 1. Добавляем полюса
    let north_pole = Vector3f { x: 0.0, y: 1.0, z: 0.0 };
    let south_pole = Vector3f { x: 0.0, y: -1.0, z: 0.0 };
    vertices.push(north_pole);
    vertices.push(south_pole);

    let d_theta = std::f64::consts::PI / (num_lat + 1) as f64;  // от полюса до полюса
    let d_phi = 2.0 * std::f64::consts::PI / num_lon as f64;

    // 2. Генерируем кольца (исключая полюса)
    for lat in 1..=num_lat {
        let theta = lat as f64 * d_theta;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let y = cos_theta;
        let r = sin_theta;

        for lon in 0..num_lon {
            let phi = lon as f64 * d_phi;
            let x = r * phi.cos();
            let z = r * phi.sin();
            vertices.push(Vector3f { x, y, z });
        }
    }

    // 3. Северный полюс → первое кольцо
    let np_idx = 0;
    for i in 0..num_lon {
        let next_i = (i + 1) % num_lon;
        let a = 2 + i;
        let b = 2 + next_i;
        triangles.push(Triangle::new_with_provided_normals(
            &vertices,
            [np_idx, a, b],
            [north_pole, vertices[a], vertices[b]],
        ));
    }

    // 4. Средние кольца
    for lat in 0..(num_lat - 1) {
        let curr = 2 + lat * num_lon;
        let next = 2 + (lat + 1) * num_lon;

        for i in 0..num_lon {
            let next_i = (i + 1) % num_lon;
            let a = curr + i;
            let b = curr + next_i;
            let c = next + next_i;
            let d = next + i;

            triangles.push(Triangle::new_with_provided_normals(&vertices, [a, b, c], [
                vertices[a], vertices[b], vertices[c],
            ]));
            triangles.push(Triangle::new_with_provided_normals(&vertices, [a, c, d], [
                vertices[a], vertices[c], vertices[d],
            ]));
        }
    }

    // 5. Южный полюс ← последнее кольцо
    let sp_idx = 1;
    let last_ring = 2 + (num_lat - 1) * num_lon;
    for i in 0..num_lon {
        let next_i = (i + 1) % num_lon;
        let a = last_ring + i;
        let b = last_ring + next_i;
        triangles.push(Triangle::new_with_provided_normals(
            &vertices,
            [sp_idx, b, a],
            [south_pole, vertices[b], vertices[a]],
        ));
    }

    let color = Color { r: 119, g: 136, b: 153 };
    let colors = vec![color; triangles.len()];

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
