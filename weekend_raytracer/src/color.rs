use std::io::{self, Write};

use crate::prelude::*;

pub type Color = Vec3;

impl Color {
    pub fn to_rgb32(self) -> (u32, u32, u32) {
        let r = self.x;
        let g = self.y;
        let b = self.z;

        ((255.999 * r) as u32, (255.999 * g) as u32, (255.999 * b) as u32)
    }
}

pub fn write_color<W: Write>(out: &mut W, c: Color) -> io::Result<()> {
    let (r, g, b) = c.to_rgb32();
    writeln!(out, "{} {} {}", r, g, b)?;
    Ok(())
}

#[test]
fn test_to_rgb32() {
    assert_eq!(Color::new(0.0, 0.0, 0.0).to_rgb32(), (0, 0, 0));
    assert_eq!(Color::new(1.0, 1.0, 1.0).to_rgb32(), (255, 255, 255));
}
