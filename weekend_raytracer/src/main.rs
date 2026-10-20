mod color;
mod prelude;
mod ray;
mod test_utils;
mod vec3;

use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
struct ImageConfig {
    width: i32,
    height: i32,
}

impl ImageConfig {
    fn new(width: i32, aspect_ratio: f64) -> Self {
        let height = ((width as f64) / aspect_ratio).floor() as i32;
        let height = height.max(1);
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug)]
struct Viewport {
    width: f64,
    height: f64,
}

impl Viewport {
    fn new(cfg: ImageConfig, viewport_height: f64) -> Self {
        let viewport_width = viewport_height * (cfg.width as f64 / cfg.height as f64);
        Self { width: viewport_width, height: viewport_height }
    }
}

struct Camera {
    focal_length: f64,
    center: Point3,
}

impl Camera {
    fn new(focal_length: f64, center: Point3) -> Self {
        Self { focal_length, center }
    }

    fn ray_for_pixel(&self, cfg: ImageConfig, viewport: Viewport, i: i32, j: i32) -> Ray {
        let viewport_u = Vec3::new(viewport.width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport.height, 0.0);

        let pixel_delta_u = viewport_u / cfg.width as f64;
        let pixel_delta_v = viewport_v / cfg.height as f64;

        let viewport_upper_left =
            self.center - Vec3::new(0.0, 0.0, self.focal_length) - viewport_u * 0.5 - viewport_v * 0.5;

        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let pixel_center = pixel00_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
        let direction = pixel_center - self.center;

        Ray::new(self.center, direction)
    }
}

fn main() {
    let img_cfg = ImageConfig::new(400, 16.0 / 9.0);
    let viewport = Viewport::new(img_cfg, 2.0);
    let camera = Camera::new(1.0, Point3::new(0.0, 0.0, 0.0));

    println!("P3");
    println!("{} {}", img_cfg.width, img_cfg.height);
    println!("255");

    for j in 0..img_cfg.height {
        eprint!("\rScanlines remaining: {}           ", img_cfg.height - j);

        for i in 0..img_cfg.width {
            let r = camera.ray_for_pixel(img_cfg, viewport, i, j);
            let color = ray_color(&r);
            write_color(&mut std::io::stdout(), color).unwrap();
        }
    }

    eprintln!("\rDone.                                ");
}

fn ray_color(r: &Ray) -> Color {
    let unit_direction = r.direction.unit();
    let a = 0.5 * (unit_direction.y + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_config_height_not_less_than_one() {
        let cfg = ImageConfig::new(1, 100.0);
        assert_eq!(cfg.height, 1);
    }
}
