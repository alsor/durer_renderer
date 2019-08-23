use crate::Vector3f;

pub fn dot_product(v1: Vector3f, v2: Vector3f) -> f64 {
    v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
}

pub fn cross_product(v: Vector3f, w: Vector3f) -> Vector3f {
    Vector3f {
        x: v.y * w.z - v.z * w.y,
        y: v.z * w.x - v.x * w.z,
        z: v.x * w.y - v.y * w.x
    }
}

pub fn multiply_vec_and_mat(vec: [f64; 3], mat: [[f64; 3]; 3]) -> [f64; 3] {
    let mut result = [0.0; 3];

    for i in 0..3 {
        for j in 0..3 {
            result[i] += vec[j] * mat[i][j];
        }
    }

    result
}

pub fn sum(v1: Vector3f, v2: Vector3f) -> Vector3f {
    Vector3f {
        x: v1.x + v2.x,
        y: v1.y + v2.y,
        z: v1.z + v2.z
    }
}

pub fn difference(v1: Vector3f, v2: Vector3f) -> Vector3f {
    Vector3f {
        x: v1.x - v2.x,
        y: v1.y - v2.y,
        z: v1.z - v2.z
    }
}

pub fn negate(vector: Vector3f) -> Vector3f {
    Vector3f { x: -vector.x, y: -vector.y, z: -vector.z }
}

pub fn reflect(v1: Vector3f, v2: Vector3f) -> Vector3f {
    difference(
        scale(2.0 * dot_product(v1, v2), v2),
        v1
    )
}

pub fn scale(scalar: f64, vector: Vector3f) -> Vector3f {
    Vector3f {
        x: vector.x * scalar,
        y: vector.y * scalar,
        z: vector.z * scalar
    }
}

pub fn length(vector: Vector3f) -> f64 {
    (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt()
}

pub fn normalize(vector: Vector3f) -> Vector3f {
    let length = length(vector);
    Vector3f { x: vector.x / length, y: vector.y / length, z: vector.z / length }
}

pub fn rotation_around_y(d: f64) -> [[f64; 3]; 3] {
    let a = d / 57.2958;
    [
        [a.cos() , 0.0, a.sin()],
        [0.0     , 1.0,     0.0],
        [-a.sin(), 0.0, a.cos()],
    ]
}

#[test]
fn test_length() {
    assert!(crate::tests::roughly_equals(length(Vector3f { x: 6.0, y: 2.0, z: 0.0 }), 6.324555));
}

#[test]
fn test_normalize() {
    let v = normalize(Vector3f { x: 6.0, y: 2.0, z: 0.0 });
    assert!(crate::tests::roughly_equals(v.x, 0.948683));
    assert!(crate::tests::roughly_equals(v.y, 0.316227));
    assert!(crate::tests::roughly_equals(v.z, 0.0));
}

#[test]
pub fn test_multiply_vec_and_mat() {
    let vec = [1.0, 2.0, 3.0];
    let mat = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];

    let result = multiply_vec_and_mat(vec, mat);

    assert!(crate::tests::roughly_equals(result[0], 1.0));
    assert!(crate::tests::roughly_equals(result[1], 2.0));
    assert!(crate::tests::roughly_equals(result[2], 3.0));
}