use crate::buffer_canvas::BufferCanvas;
use std::time::Duration;
use rand::Rng;
use crate::{Pixel, Color};

pub struct Starfield {
    num_stars: usize,
    spread: f64,
    speed: f64,
    star_x: Vec<f64>,
    star_y: Vec<f64>,
    star_z: Vec<f64>,
}

impl Starfield {
    pub fn new(num_stars: usize, spread: f64, speed: f64) -> Self {
        let mut star_x = Vec::with_capacity(num_stars);
        let mut star_y = Vec::with_capacity(num_stars);
        let mut star_z = Vec::with_capacity(num_stars);

        for i in 0..num_stars {
            let (x, y, z) = init_star(spread);
            star_x.push(x);
            star_y.push(y);
            star_z.push(z);
        }

        Self { num_stars, spread, speed, star_x, star_y, star_z }
    }

    pub fn update_and_render(&mut self, delta: &Duration, canvas: &mut BufferCanvas) {
        let half_size = (canvas.size as f64) / 2.0;

        for i in 0..self.num_stars {
            let new_z = self.star_z[i] - ((delta.as_nanos() as f64)/ 1000000000.0) * self.speed;

            if new_z <= 0.0 {
                self.new_star(i);
            } else {
                self.star_z[i] = new_z;
            }

            let screen_x = ((self.star_x[i] / self.star_z[i]) * half_size + half_size) as usize;
            let screen_y = ((self.star_y[i] / self.star_z[i]) * half_size + half_size) as usize;

            if (screen_x < 0 || screen_x >= canvas.size) || (screen_y < 0 || screen_y >= canvas.size) {
                self.new_star(i);
            } else {
                canvas.put_pixel(Pixel { x: screen_x, y: screen_y, color: Color { r: 255, g: 255, b: 255} })
            }
        }
    }

    fn new_star(&mut self, i: usize) {
        let (x, y, z) = init_star(self.spread);
        self.star_x[i] = x;
        self.star_y[i] = y;
        self.star_z[i] = z;
    }
}

fn init_star(spread: f64) -> (f64, f64, f64) {
    let mut rng = rand::thread_rng();

    (
        2.0 * (rng.gen::<f64>() - 0.5) * spread,
        2.0 * (rng.gen::<f64>() - 0.5) * spread,
        (rng.gen::<f64>() + 0.0001) * spread
    )
}

#[test]
fn test_init_star() {
    let spread = 10.0;

    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
    println!("star: {:?}", init_star(spread));
}
