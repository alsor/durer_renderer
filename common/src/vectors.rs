use crate::Vector3f;

pub fn rotate_x(angle: f64) -> [[f64; 3]; 3] {
    let cos = angle.cos();
    let sin = angle.sin();
    [[1.0, 0.0, 0.0], [0.0, cos, sin], [0.0, -sin, cos]]
}

pub fn rotate_y(angle: f64) -> [[f64; 3]; 3] {
    let cos = angle.cos();
    let sin = angle.sin();
    [[cos, 0.0, sin], [0.0, 1.0, 0.0], [-sin, 0.0, cos]]
}

pub fn rotate_z(angle: f64) -> [[f64; 3]; 3] {
    let cos = angle.cos();
    let sin = angle.sin();
    [[cos, sin, 0.0], [-sin, cos, 0.0], [0.0, 0.0, 1.0]]
}

pub fn rotate_x_deg(angle: f64) -> [[f64; 3]; 3] {
    rotate_x(angle.to_radians())
}

pub fn rotate_y_deg(angle: f64) -> [[f64; 3]; 3] {
    rotate_y(angle.to_radians())
}

pub fn rotate_z_deg(angle: f64) -> [[f64; 3]; 3] {
    rotate_z(angle.to_radians())
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

/// Транспонирует матрицу 3x3.
pub fn transpose_3x3(matrix: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [matrix[0][0], matrix[1][0], matrix[2][0]],
        [matrix[0][1], matrix[1][1], matrix[2][1]],
        [matrix[0][2], matrix[1][2], matrix[2][2]],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpose_3x3() {
        let matrix = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];

        let expected = [[1.0, 4.0, 7.0], [2.0, 5.0, 8.0], [3.0, 6.0, 9.0]];

        let result = transpose_3x3(matrix);

        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (result[i][j] - expected[i][j]).abs() < f64::EPSILON,
                    "Mismatch at [{},{}]: {} != {}",
                    i,
                    j,
                    result[i][j],
                    expected[i][j]
                );
            }
        }
    }

    #[test]
    fn test_transpose_identity() {
        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

        let result = transpose_3x3(identity);

        // Транспонирование единичной матрицы — это она сама
        for i in 0..3 {
            for j in 0..3 {
                assert!((result[i][j] - identity[i][j]).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_transpose_symmetric() {
        let symmetric = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];

        let result = transpose_3x3(symmetric);

        // Симметричная матрица не меняется при транспонировании
        for i in 0..3 {
            for j in 0..3 {
                assert!((result[i][j] - symmetric[i][j]).abs() < 1e-10);
            }
        }
    }
}

/// Умножает две матрицы 3x3: result = mat1 * mat2
pub fn multiply_mat_3x3(mat1: [[f64; 3]; 3], mat2: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut result = [[0.0; 3]; 3];

    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result[i][j] += mat1[i][k] * mat2[k][j];
            }
        }
    }

    result
}

#[test]
fn test_multiply_mat_3x3() {
    let mat1 = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let mat2 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]; // единичная

    let result = multiply_mat_3x3(mat1, mat2);

    for i in 0..3 {
        for j in 0..3 {
            assert!((result[i][j] - mat1[i][j]).abs() < f64::EPSILON);
        }
    }
}

#[test]
pub fn test_multiply_vec_and_mat() {
    let vec = [1.0, 2.0, 3.0];
    let mat = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    let result = multiply_vec_and_mat(vec, mat);

    assert!(test_utils::roughly_equals(result[0], 1.0));
    assert!(test_utils::roughly_equals(result[1], 2.0));
    assert!(test_utils::roughly_equals(result[2], 3.0));
}

pub fn scale(scalar: f64, vector: Vector3f) -> Vector3f {
    Vector3f {
        x: vector.x * scalar,
        y: vector.y * scalar,
        z: vector.z * scalar,
    }
}

pub fn sum(v1: Vector3f, v2: Vector3f) -> Vector3f {
    Vector3f { x: v1.x + v2.x, y: v1.y + v2.y, z: v1.z + v2.z }
}

pub fn difference(v1: Vector3f, v2: Vector3f) -> Vector3f {
    Vector3f { x: v1.x - v2.x, y: v1.y - v2.y, z: v1.z - v2.z }
}

pub fn negate(vector: Vector3f) -> Vector3f {
    Vector3f { x: -vector.x, y: -vector.y, z: -vector.z }
}

pub fn reflect(v1: Vector3f, v2: Vector3f) -> Vector3f {
    difference(scale(2.0 * dot_product(v1, v2), v2), v1)
}

pub fn dot_product(v1: Vector3f, v2: Vector3f) -> f64 {
    v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
}

pub fn normalize(vector: Vector3f) -> Vector3f {
    let length = length(vector);
    Vector3f {
        x: vector.x / length,
        y: vector.y / length,
        z: vector.z / length,
    }
}

#[test]
fn test_normalize() {
    let v = normalize(Vector3f { x: 6.0, y: 2.0, z: 0.0 });
    assert!(test_utils::roughly_equals(v.x, 0.948683));
    assert!(test_utils::roughly_equals(v.y, 0.316227));
    assert!(test_utils::roughly_equals(v.z, 0.0));
}

pub fn length(vector: Vector3f) -> f64 {
    (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt()
}

#[test]
fn test_length() {
    assert!(test_utils::roughly_equals(
        length(Vector3f { x: 6.0, y: 2.0, z: 0.0 }),
        6.324555
    ));
}

pub fn cross_product(v: Vector3f, w: Vector3f) -> Vector3f {
    Vector3f {
        x: v.y * w.z - v.z * w.y,
        y: v.z * w.x - v.x * w.z,
        z: v.x * w.y - v.y * w.x,
    }
}
