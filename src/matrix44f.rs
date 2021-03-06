use crate::vector4f::Vector4f;

#[derive(Copy, Clone)]
pub struct Matrix44f {
    pub elements: [[f64; 4]; 4]
}

impl Matrix44f {
    pub fn translation(vector: Vector4f) -> Matrix44f {
        Matrix44f {
            elements: [
                [     1.0,      0.0,      0.0, 0.0],
                [     0.0,      1.0,      0.0, 0.0],
                [     0.0,      0.0,      1.0, 0.0],
                [vector.x, vector.y, vector.z, 1.0],
            ]
        }
    }

    pub fn uniform_scale(scale: f64) -> Matrix44f {
        Matrix44f {
            elements: [
                [scale,   0.0,   0.0, 0.0],
                [  0.0, scale,   0.0, 0.0],
                [  0.0,   0.0, scale, 0.0],
                [  0.0,   0.0,   0.0, 1.0],
            ]
        }
    }

    pub fn rotation_x(degrees: f64) -> Matrix44f {
        let radians = degrees.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();

        Matrix44f {
            elements: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, sin, 0.0],
                [0.0, -sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    pub fn rotation_y(degrees: f64) -> Matrix44f {
        let radians = degrees.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();

        Matrix44f {
            elements: [
                [cos, 0.0, -sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    pub fn rotation_z(degrees: f64) -> Matrix44f {
        let radians = degrees.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();

        Matrix44f {
            elements: [
                [cos, sin, 0.0, 0.0],
                [-sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
        }
    }

    pub fn multiply(&self, other: Matrix44f) -> Matrix44f {
        let mut elements = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                elements[i][j] =
                    self.elements[i][0] * other.elements[0][j] +
                    self.elements[i][1] * other.elements[1][j] +
                    self.elements[i][2] * other.elements[2][j] +
                    self.elements[i][3] * other.elements[3][j];
            }
        }

        Matrix44f { elements }
    }

    pub fn transpose(&self) -> Self {
        let mut elements = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                elements[i][j] = self.elements[j][i];
            }
        }

        Matrix44f { elements }
    }
}

#[test]
fn test_transpose() {
    let matrix = Matrix44f {
        elements: [
            [ 1.0,  2.0,  3.0,  4.0],
            [ 5.0,  6.0,  7.0,  8.0],
            [ 9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]
    };
    let result = matrix.transpose();
    for i in 0..4 {
        println!(
            "{:.2} {:.2} {:.2} {:.2}",
            result.elements[i][0],
            result.elements[i][1],
            result.elements[i][2],
            result.elements[i][3]
        );
    }
}

