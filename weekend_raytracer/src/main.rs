mod color;
mod prelude;
mod vec3;

use crate::prelude::*;

fn main() {
    const IMAGE_WIDTH: i32 = 256;
    const IMAGE_HEIGHT: i32 = 256;

    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    for j in 0..IMAGE_HEIGHT {
        eprint!("\rScanlines remaining: {}           ", IMAGE_HEIGHT - j);

        for i in 0..IMAGE_WIDTH {
            let color = Color::new(
                i as f64 / (IMAGE_WIDTH - 1) as f64,
                j as f64 / (IMAGE_HEIGHT - 1) as f64,
                0.0,
            );
            write_color(&mut std::io::stdout(), color).unwrap();
        }
    }

    eprintln!("\rDone.                                ");
}
