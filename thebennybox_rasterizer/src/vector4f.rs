#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector4f {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vector4f {
    // Конструктор
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    // Длина вектора
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    // Максимальная компонента
    pub fn max(self) -> f64 {
        self.x.max(self.y).max(self.z).max(self.w)
    }

    // Скалярное произведение
    pub fn dot(self, r: Vector4f) -> f64 {
        self.x * r.x + self.y * r.y + self.z * r.z + self.w * r.w
    }

    // Векторное произведение (аналог 3D)
    pub fn cross(self, r: Vector4f) -> Vector4f {
        let x_ = self.y * r.z - self.z * r.y;
        let y_ = self.z * r.x - self.x * r.z;
        let z_ = self.x * r.y - self.y * r.x;
        Vector4f::new(x_, y_, z_, 0.0)
    }

    // Нормализация
    pub fn normalized(self) -> Vector4f {
        let length = self.length();
        if length == 0.0 {
            return Vector4f::new(0.0, 0.0, 0.0, 0.0);
        }
        Vector4f::new(self.x / length, self.y / length, self.z / length, self.w / length)
    }

    // Поворот вокруг оси
    pub fn rotate(self, axis: Vector4f, angle: f64) -> Vector4f {
        let sin_angle = (-angle).sin();
        let cos_angle = (-angle).cos();

        let axis_scaled = axis.mul_scalar(sin_angle);
        let cross_part = self.cross(axis_scaled);
        let cos_part = self.mul_scalar(cos_angle);
        let dot_part = axis.mul_scalar(self.dot(axis.mul_scalar(1.0 - cos_angle)));

        cross_part.add(cos_part).add(dot_part)
    }

    // Линейная интерполяция
    pub fn lerp(self, dest: Vector4f, lerp_factor: f64) -> Vector4f {
        dest.sub(self).mul_scalar(lerp_factor).add(self)
    }

    // Сложение вектора
    pub fn add(self, r: Vector4f) -> Vector4f {
        Vector4f::new(self.x + r.x, self.y + r.y, self.z + r.z, self.w + r.w)
    }

    // Сложение скаляра
    pub fn add_scalar(self, r: f64) -> Vector4f {
        Vector4f::new(self.x + r, self.y + r, self.z + r, self.w + r)
    }

    // Вычитание вектора
    pub fn sub(self, r: Vector4f) -> Vector4f {
        Vector4f::new(self.x - r.x, self.y - r.y, self.z - r.z, self.w - r.w)
    }

    // Вычитание скаляра
    pub fn sub_scalar(self, r: f64) -> Vector4f {
        Vector4f::new(self.x - r, self.y - r, self.z - r, self.w - r)
    }

    // Поэлементное умножение
    pub fn mul(self, r: Vector4f) -> Vector4f {
        Vector4f::new(self.x * r.x, self.y * r.y, self.z * r.z, self.w * r.w)
    }

    // Умножение на скаляр
    pub fn mul_scalar(self, r: f64) -> Vector4f {
        Vector4f::new(self.x * r, self.y * r, self.z * r, self.w * r)
    }

    // Поэлементное деление
    pub fn div(self, r: Vector4f) -> Vector4f {
        Vector4f::new(self.x / r.x, self.y / r.y, self.z / r.z, self.w / r.w)
    }

    // Деление на скаляр
    pub fn div_scalar(self, r: f64) -> Vector4f {
        Vector4f::new(self.x / r, self.y / r, self.z / r, self.w / r)
    }

    // Абсолютные значения
    pub fn abs(self) -> Vector4f {
        Vector4f::new(self.x.abs(), self.y.abs(), self.z.abs(), self.w.abs())
    }

    // Геттеры
    pub fn x(self) -> f64 {
        self.x
    }
    pub fn y(self) -> f64 {
        self.y
    }
    pub fn z(self) -> f64 {
        self.z
    }
    pub fn w(self) -> f64 {
        self.w
    }
}

// Операторы

use std::ops::*;

impl Add<Vector4f> for Vector4f {
    type Output = Vector4f;
    fn add(self, rhs: Vector4f) -> Vector4f {
        self.add(rhs)
    }
}

impl Sub<Vector4f> for Vector4f {
    type Output = Vector4f;
    fn sub(self, rhs: Vector4f) -> Vector4f {
        self.sub(rhs)
    }
}

impl Mul<f64> for Vector4f {
    type Output = Vector4f;
    fn mul(self, rhs: f64) -> Vector4f {
        self.mul_scalar(rhs)
    }
}

impl Div<f64> for Vector4f {
    type Output = Vector4f;
    fn div(self, rhs: f64) -> Vector4f {
        self.div_scalar(rhs)
    }
}

// Вывод на экран
impl std::fmt::Display for Vector4f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

// Тесты
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length() {
        let v = Vector4f::new(1.0, 2.0, 3.0, 4.0);
        assert!((v.length() - 5.477225575051661).abs() < 1e-10);
    }

    #[test]
    fn test_dot() {
        let a = Vector4f::new(1.0, 0.0, 0.0, 0.0);
        let b = Vector4f::new(0.0, 1.0, 0.0, 0.0);
        assert_eq!(a.dot(b), 0.0);
    }

    #[test]
    fn test_normalized() {
        let v = Vector4f::new(2.0, 0.0, 0.0, 0.0);
        let n = v.normalized();
        assert!((n.length() - 1.0).abs() < 1e-10);
        assert_eq!(n.x, 1.0);
    }

    #[test]
    fn test_lerp() {
        let a = Vector4f::new(0.0, 0.0, 0.0, 0.0);
        let b = Vector4f::new(10.0, 0.0, 0.0, 0.0);
        let c = a.lerp(b, 0.5);
        assert_eq!(c.x, 5.0);
    }

    #[test]
    fn test_rotate() {
        let v = Vector4f::new(1.0, 0.0, 0.0, 0.0);
        let axis = Vector4f::new(0.0, 0.0, 1.0, 0.0); // ось Z
        let angle = std::f64::consts::PI / 2.0; // 90 градусов
        let rotated = v.rotate(axis, angle);
        assert!((rotated.x.abs() - 0.0) < 1e-10);
        assert!((rotated.y - 1.0).abs() < 1e-10);
    }
}
