use crate::matrix44f::Matrix44f;

#[derive(Copy, Clone)]
pub struct Vector4f {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64
}

impl Vector4f {
    pub fn zero_vector() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }

    pub fn transform(&self, matrix: Matrix44f) -> Self {
        Self {
            x: self.x * matrix.elements[0][0] +
                self.y * matrix.elements[1][0] +
                self.z * matrix.elements[2][0] +
                self.w * matrix.elements[3][0],
            y: self.x * matrix.elements[0][1] +
                self.y * matrix.elements[1][1] +
                self.z * matrix.elements[2][1] +
                self.w * matrix.elements[3][1],
            z: self.x * matrix.elements[0][2] +
                self.y * matrix.elements[1][2] +
                self.z * matrix.elements[2][2] +
                self.w * matrix.elements[3][2],
            w: self.x * matrix.elements[0][3] +
                self.y * matrix.elements[1][3] +
                self.z * matrix.elements[2][3] +
                self.w * matrix.elements[3][3]
        }
    }

    pub fn negate(&self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z, w: -self.w }
    }
}

#[test]
fn test_transform() {
    let vertex = Vector4f { x: 1.0, y: 2.0, z: 3.0, w: 4.0 };
    let matrix = Matrix44f {
        elements: [
            [ 5.0,  6.0,  7.0,  8.0],
            [ 9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
            [17.0, 18.0, 19.0, 20.0],
        ]
    };
    let transformed = vertex.transform(matrix);
    assert_eq!(transformed.x, 130.0);
    assert_eq!(transformed.y, 140.0);
    assert_eq!(transformed.z, 150.0);
    assert_eq!(transformed.w, 160.0);
}