use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.origin + t * self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_at_t_zero() {
        let origin = Point3::new(1.0, 2.0, 3.0);
        let direction = Vec3::new(4.0, 5.0, 6.0);
        let r = Ray::new(origin, direction);

        let p = r.at(0.0);
        assert!(vec3_approx_eq(p, origin));
    }

    #[test]
    fn test_at_linearity() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(1.0, 0.0, 0.0);
        let r = Ray::new(origin, direction);

        let p1 = r.at(1.0);
        let p2 = r.at(2.5);

        assert!(vec3_approx_eq(p1, Point3::new(1.0, 0.0, 0.0)));
        assert!(vec3_approx_eq(p2, Point3::new(2.5, 0.0, 0.0)));
    }
}
