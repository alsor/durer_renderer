use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub type Point3 = Vec3;

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn unit(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            return Self::zero();
        }
        self / len
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_neg() {
        let v = Vec3::new(1.0, -2.0, 3.5);
        let r = -v;
        assert!(vec3_approx_eq(r, Vec3::new(-1.0, 2.0, -3.5)));
    }

    #[test]
    fn test_div() {
        let v = Vec3::new(6.0, 8.0, 10.0);
        let r = v / 2.0;
        assert!(vec3_approx_eq(r, Vec3::new(3.0, 4.0, 5.0)));
    }

    #[test]
    fn test_add() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        let r = a + b;
        assert!(vec3_approx_eq(r, Vec3::new(5.0, 7.0, 9.0)));
    }

    #[test]
    fn test_sub() {
        let a = Vec3::new(10.0, 10.0, 10.0);
        let b = Vec3::new(3.0, 4.0, 5.0);
        let r = a - b;
        assert!(vec3_approx_eq(r, Vec3::new(7.0, 6.0, 5.0)));
    }

    #[test]
    fn test_mul_vec3_scalar() {
        let v = Vec3::new(2.0, 3.0, 4.0);
        let s = 2.5;
        let r = v * s;
        assert!(vec3_approx_eq(r, Vec3::new(5.0, 7.5, 10.0)));
    }

    #[test]
    fn test_mul_scalar_vec3() {
        let v = Vec3::new(2.0, 3.0, 4.0);
        let s = 2.5;
        let r = s * v;
        assert!(vec3_approx_eq(r, Vec3::new(5.0, 7.5, 10.0)));
    }

    #[test]
    fn test_dot() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert!(approx_eq(a.dot(b), 0.0));

        let c = Vec3::new(2.0, 3.0, 4.0);
        let d = Vec3::new(5.0, 6.0, 7.0);
        assert!(approx_eq(c.dot(d), 56.0));
    }

    #[test]
    fn test_length_and_unit() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!(approx_eq(v.length_squared(), 25.0));
        assert!(approx_eq(v.length(), 5.0));

        let u = v.unit();
        assert!(approx_eq(u.length(), 1.0));
        assert!(vec3_approx_eq(u, Vec3::new(0.6, 0.8, 0.0)));

        assert!(vec3_approx_eq(Vec3::zero().unit(), Vec3::zero()));
    }

    #[test]
    fn test_cross_standard_basis() {
        let i = Vec3::new(1.0, 0.0, 0.0);
        let j = Vec3::new(0.0, 1.0, 0.0);
        let k = Vec3::new(0.0, 0.0, 1.0);

        assert!(vec3_approx_eq(i.cross(j), k));
        assert!(vec3_approx_eq(j.cross(i), Vec3::new(0.0, 0.0, -1.0)));
    }

    #[test]
    fn test_cross_orthogonality() {
        let a = Vec3::new(2.0, 3.0, 4.0);
        let b = Vec3::new(5.0, 6.0, 7.0);
        let c = a.cross(b);

        assert!(approx_eq(a.dot(c), 0.0));
        assert!(approx_eq(b.dot(c), 0.0));
    }

    #[test]
    fn test_cross_anticommutative() {
        let a = Vec3::new(1.5, -2.0, 3.5);
        let b = Vec3::new(-4.0, 5.5, 0.2);

        let ab = a.cross(b);
        let ba = b.cross(a);

        assert!(vec3_approx_eq(ab, -ba));
    }

    #[test]
    fn test_cross_with_zero() {
        let zero = Vec3::zero();
        let v = Vec3::new(3.0, -4.0, 5.0);

        assert!(vec3_approx_eq(v.cross(zero), zero));
        assert!(vec3_approx_eq(zero.cross(v), zero));
    }
}
