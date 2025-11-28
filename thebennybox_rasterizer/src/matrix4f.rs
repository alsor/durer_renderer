use crate::vector4f::Vector4f;
use std::ops::Mul;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4f {
    pub m: [[f64; 4]; 4],
}

impl Matrix4f {
    #[must_use]
    pub const fn new() -> Self {
        Self { m: [[0.0; 4]; 4] }
    }

    #[must_use]
    pub fn init_identity() -> Self {
        let mut m = Self::new();
        m.m[0][0] = 1.0;
        m.m[1][1] = 1.0;
        m.m[2][2] = 1.0;
        m.m[3][3] = 1.0;
        m
    }

    #[must_use]
    pub fn init_screen_space_transform(half_width: f64, half_height: f64) -> Self {
        let mut m = Self::new();
        m.m[0][0] = half_width;
        m.m[0][3] = half_width;
        m.m[1][1] = -half_height;
        m.m[1][3] = half_height;
        m.m[2][2] = 1.0;
        m.m[3][3] = 1.0;
        m
    }

    #[must_use]
    pub fn init_translation(x: f64, y: f64, z: f64) -> Self {
        let mut m = Self::init_identity();
        m.m[0][3] = x;
        m.m[1][3] = y;
        m.m[2][3] = z;
        m
    }

    #[must_use]
    pub fn init_rotation_axis(x: f64, y: f64, z: f64, angle: f64) -> Self {
        let sin = angle.sin();
        let cos = angle.cos();
        let axis_len_sq = x * x + y * y + z * z;

        if axis_len_sq == 0.0 {
            return Self::init_identity();
        }

        let inv_len = (axis_len_sq).sqrt().recip();
        let x = x * inv_len;
        let y = y * inv_len;
        let z = z * inv_len;

        let omc = 1.0 - cos;

        let mut m = Self::new();
        m.m[0][0] = cos + x * x * omc;
        m.m[0][1] = x * y * omc - z * sin;
        m.m[0][2] = x * z * omc + y * sin;
        m.m[1][0] = y * x * omc + z * sin;
        m.m[1][1] = cos + y * y * omc;
        m.m[1][2] = y * z * omc - x * sin;
        m.m[2][0] = z * x * omc - y * sin;
        m.m[2][1] = z * y * omc + x * sin;
        m.m[2][2] = cos + z * z * omc;
        m.m[3][3] = 1.0;
        m
    }

    #[must_use]
    pub fn init_rotation_euler(pitch: f64, yaw: f64, roll: f64) -> Self {
        let cx = pitch.cos();
        let sx = pitch.sin();
        let cy = yaw.cos();
        let sy = yaw.sin();
        let cz = roll.cos();
        let sz = roll.sin();

        let rx = Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cx, -sx, 0.0],
                [0.0, sx, cx, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        let ry = Self {
            m: [
                [cy, 0.0, -sy, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [sy, 0.0, cy, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        let rz = Self {
            m: [
                [cz, -sz, 0.0, 0.0],
                [sz, cz, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        rz * ry * rx
    }

    #[must_use]
    pub fn init_scale(x: f64, y: f64, z: f64) -> Self {
        let mut m = Self::init_identity();
        m.m[0][0] = x;
        m.m[1][1] = y;
        m.m[2][2] = z;
        m
    }

    #[must_use]
    pub fn init_perspective(fov: f64, aspect_ratio: f64, z_near: f64, z_far: f64) -> Self {
        let tan_half_fov = (fov / 2.0).tan();
        let z_range = z_near - z_far;

        let mut m = Self::new();
        m.m[0][0] = 1.0 / (tan_half_fov * aspect_ratio);
        m.m[1][1] = 1.0 / tan_half_fov;
        m.m[2][2] = (-z_near - z_far) / z_range;
        m.m[2][3] = 2.0 * z_far * z_near / z_range;
        m.m[3][2] = 1.0;
        m.m[3][3] = 0.0;
        m
    }

    #[must_use]
    pub fn init_orthographic(left: f64, right: f64, bottom: f64, top: f64, z_near: f64, z_far: f64) -> Self {
        let width = right - left;
        let height = top - bottom;
        let depth = z_far - z_near;

        let mut m = Self::new();
        m.m[0][0] = 2.0 / width;
        m.m[0][3] = -(right + left) / width;
        m.m[1][1] = 2.0 / height;
        m.m[1][3] = -(top + bottom) / height;
        m.m[2][2] = -2.0 / depth;
        m.m[2][3] = -(z_far + z_near) / depth;
        m.m[3][3] = 1.0;
        m
    }

    #[must_use]
    pub fn init_rotation_look_at(forward: Vector4f, up: Vector4f) -> Self {
        let f = forward.normalized();
        let r = up.normalized().cross(f).normalized();
        let u = f.cross(r);
        Self::init_rotation_basis(r, u, f)
    }

    #[must_use]
    pub fn init_rotation_basis(right: Vector4f, up: Vector4f, forward: Vector4f) -> Self {
        let mut m = Self::init_identity();
        m.m[0][0] = right.x;
        m.m[0][1] = right.y;
        m.m[0][2] = right.z;
        m.m[1][0] = up.x;
        m.m[1][1] = up.y;
        m.m[1][2] = up.z;
        m.m[2][0] = forward.x;
        m.m[2][1] = forward.y;
        m.m[2][2] = forward.z;
        m
    }

    #[must_use]
    pub fn transform(&self, r: Vector4f) -> Vector4f {
        Vector4f::new(
            self.m[0][0] * r.x + self.m[0][1] * r.y + self.m[0][2] * r.z + self.m[0][3] * r.w,
            self.m[1][0] * r.x + self.m[1][1] * r.y + self.m[1][2] * r.z + self.m[1][3] * r.w,
            self.m[2][0] * r.x + self.m[2][1] * r.y + self.m[2][2] * r.z + self.m[2][3] * r.w,
            self.m[3][0] * r.x + self.m[3][1] * r.y + self.m[3][2] * r.z + self.m[3][3] * r.w,
        )
    }

    #[must_use]
    pub fn mul(self, rhs: Self) -> Self {
        let mut res = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                res.m[i][j] = self.m[i][0] * rhs.m[0][j]
                    + self.m[i][1] * rhs.m[1][j]
                    + self.m[i][2] * rhs.m[2][j]
                    + self.m[i][3] * rhs.m[3][j];
            }
        }
        res
    }

    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.m[row][col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.m[row][col] = value;
    }

    #[must_use]
    pub fn get_m(&self) -> [[f64; 4]; 4] {
        self.m
    }

    pub fn set_m(&mut self, matrix: [[f64; 4]; 4]) {
        self.m = matrix;
    }
}

// Операторы: теперь по значению
impl Mul for Matrix4f {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl std::ops::Index<usize> for Matrix4f {
    type Output = [f64; 4];

    fn index(&self, index: usize) -> &Self::Output {
        &self.m[index]
    }
}

impl std::ops::IndexMut<usize> for Matrix4f {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.m[index]
    }
}

impl std::fmt::Display for Matrix4f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Matrix4f:")?;
        for row in &self.m {
            writeln!(
                f,
                "[{:>10.4}, {:>10.4}, {:>10.4}, {:>10.4}]",
                row[0], row[1], row[2], row[3]
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector4f_basic() {
        let v = Vector4f::new(1.0, 0.0, 0.0, 0.0);
        assert_eq!(v.length(), 1.0);
        assert_eq!(v.x, 1.0);
    }

    #[test]
    fn test_vector4f_dot() {
        let a = Vector4f::new(1.0, 0.0, 0.0, 0.0);
        let b = Vector4f::new(0.0, 1.0, 0.0, 0.0);
        assert_eq!(a.dot(b), 0.0);
    }

    #[test]
    fn test_vector4f_cross() {
        let i = Vector4f::new(1.0, 0.0, 0.0, 0.0);
        let j = Vector4f::new(0.0, 1.0, 0.0, 0.0);
        let k = i.cross(j);
        assert!((k.z - 1.0).abs() < 1e-10);
        assert_eq!(k.w, 0.0);
    }

    #[test]
    fn test_matrix_identity() {
        let m = Matrix4f::init_identity();
        let v = Vector4f::new(1.0, 2.0, 3.0, 1.0);
        let res = m.transform(v);
        assert_eq!(res, v);
    }

    #[test]
    fn test_matrix_translation() {
        let m = Matrix4f::init_translation(1.0, 2.0, 3.0);
        let v = Vector4f::new(0.0, 0.0, 0.0, 1.0);
        let res = m.transform(v);
        assert!((res.x - 1.0).abs() < 1e-10);
        assert!((res.y - 2.0).abs() < 1e-10);
        assert!((res.z - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_mul() {
        let a = Matrix4f::init_translation(1.0, 0.0, 0.0);
        let b = Matrix4f::init_translation(0.0, 1.0, 0.0);
        let m = a * b;
        let v = Vector4f::new(0.0, 0.0, 0.0, 1.0);
        let res = m.transform(v);
        assert!((res.x - 1.0).abs() < 1e-10);
        assert!((res.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_perspective_projection() {
        let proj = Matrix4f::init_perspective(std::f64::consts::PI / 4.0, 1.0, 0.1, 100.0);
        let v = Vector4f::new(0.0, 0.0, -2.0, 1.0);
        let res = proj.transform(v);
        assert!(res.w != 0.0); // Чтобы убедиться, что проекция работает
    }

    #[test]
    fn test_rotation_euler() {
        let rot = Matrix4f::init_rotation_euler(0.0, 0.0, 0.0);
        let v = Vector4f::new(1.0, 0.0, 0.0, 1.0);
        let res = rot.transform(v);
        assert!((res.x - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_look_at() {
        let forward = Vector4f::new(0.0, 0.0, 1.0, 0.0);
        let up = Vector4f::new(0.0, 1.0, 0.0, 0.0);
        let rot = Matrix4f::init_rotation_look_at(forward, up);
        let v = Vector4f::new(1.0, 0.0, 0.0, 1.0);
        let res = rot.transform(v);
        // После поворота "вперёд" — должен быть вдоль X?
        assert!(res.x.abs() > 0.1); // Грубая проверка
    }
}
