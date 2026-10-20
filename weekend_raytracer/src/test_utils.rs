use crate::prelude::*;

pub fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

pub fn vec3_approx_eq(a: Vec3, b: Vec3) -> bool {
    approx_eq(a.x, b.x) && approx_eq(a.y, b.y) && approx_eq(a.z, b.z)
}
