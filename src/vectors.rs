use super::Point3D;

pub fn dot_product(v1: Point3D, v2: Point3D) -> f64 {
    v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
}

pub fn cross_product(v: Point3D, w: Point3D) -> Point3D {
    Point3D {
        x: v.y * w.z - v.z * w.y,
        y: v.z * w.x - v.x * w.z,
        z: v.x * w.y - v.y * w.x
    }
}

pub fn sum(v1: Point3D, v2: Point3D) -> Point3D {
    Point3D {
        x: v1.x + v2.x,
        y: v1.y + v2.y,
        z: v1.z + v2.z
    }
}

pub fn difference(v1: Point3D, v2: Point3D) -> Point3D {
    Point3D {
        x: v1.x - v2.x,
        y: v1.y - v2.y,
        z: v1.z - v2.z
    }
}

pub fn negate(vector: Point3D) -> Point3D {
    Point3D { x: -vector.x, y: -vector.y, z: -vector.z }
}

pub fn scale(scalar: f64, vector: Point3D) -> Point3D {
    Point3D {
        x: vector.x * scalar,
        y: vector.y * scalar,
        z: vector.z * scalar
    }
}

pub fn length(vector: Point3D) -> f64 {
    (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt()
}

pub fn normalize(vector: Point3D) -> Point3D {
    let length = length(vector);
    Point3D { x: vector.x / length, y: vector.y / length, z: vector.z / length }
}

#[test]
fn test_length() {
    assert!(::tests::roughly_equals(length(Point3D { x: 6.0, y: 2.0, z: 0.0 }), 6.324555));
}

#[test]
fn test_normalize() {
    let v = normalize(Point3D { x: 6.0, y: 2.0, z: 0.0 });
    assert!(::tests::roughly_equals(v.x, 0.948683));
    assert!(::tests::roughly_equals(v.y, 0.316227));
    assert!(::tests::roughly_equals(v.z, 0.0));
}