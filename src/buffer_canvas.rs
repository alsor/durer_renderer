use projective_camera::ProjectiveCamera;
use super::Color;
use super::Pixel;
use super::Point;
use super::Point2D;
use vector4f::Vector4f;

pub struct BufferCanvas {
    pub size: usize,
    pub buffer: Vec<u8>,
    pub depth_buffer: Vec<f64>
}

impl BufferCanvas {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            buffer: vec![0u8; size * size * 3],
            depth_buffer: vec![0.0; size * size]
        }
    }

    pub fn clear(&mut self) {
        self.buffer = vec![0u8; self.size * self.size * 3];
        self.depth_buffer = vec![0.0; self.size * self.size];
    }

    pub fn viewport_to_canvas(&self, vertex: Vector4f, camera: &ProjectiveCamera) -> Point {
        let point = camera.project_vertex(vertex);
        let canvas_size = self.size as f64;
        Point {
            x: (point.x * canvas_size / camera.viewport_size) as i32,
            y: (point.y * canvas_size / camera.viewport_size) as i32,
            h: 1.0,
            z: vertex.z
        }
    }

    pub fn put_pixel(&mut self, pixel: Pixel) {
        trace!("pixel.y: {}, self.size: {}, pixel.x: {}", pixel.y, self.size, pixel.x);
        let offset = pixel.y * self.size * 3 + pixel.x * 3;
//        if offset >= self.buffer.len() {
//            println!("was going to draw pixel {} {} with buffer offset {}",
//                     pixel.x, pixel.y, offset);
//            return;
//        }

        self.buffer[offset] = pixel.color.r;
        self.buffer[offset + 1] = pixel.color.g;
        self.buffer[offset + 2] = pixel.color.b;
    }

    pub fn draw_point(&mut self, x: i32, y: i32, iz: f64, color: Color) {
        let pixel = self.point_to_pixel(x, y, color);

        let depth_index = (pixel.y as usize) * self.size + (pixel.x as usize);

        if iz > self.depth_buffer[depth_index] {
            self.depth_buffer[depth_index] = iz;
            self.put_pixel(pixel);
        }
    }

    fn screen_x(&self, x_canvas: i32) -> usize {
        let canvas_width = self.size as i32;
        let result = (canvas_width / 2 + x_canvas);

        if result == canvas_width {
            return (canvas_width - 1) as usize;
        } else {
            return result as usize;
        }
    }

    fn screen_y(&self, y_canvas: i32) -> usize {
        let canvas_height = self.size as i32;
        let result = (canvas_height / 2 - y_canvas - 1);

        if result == -1 {
            return 0;
        } else {
            return result as usize;
        }
    }

    fn point_to_pixel(&self, x: i32, y: i32, color: Color) -> Pixel {
        Pixel { x: self.screen_x(x), y: self.screen_y(y), color }
    }

    pub fn draw_line(&mut self, start: Point, end: Point, color: Color) {
        trace!("drawing line [{},{}] - [{},{}]", start.x, start.y, end.x, end.y);
        let start_pixel = self.point_to_pixel(start.x, start.y, color);
        let end_pixel = self.point_to_pixel(end.x, end.y, color);
        trace!(
            "drawing line pixels [{},{}] - [{},{}]",
            start_pixel.x, start_pixel.y, end_pixel.x, end_pixel.y
        );


        self.rasterize_line(start_pixel, end_pixel);
    }

    fn rasterize_line(&mut self, start: Pixel, end: Pixel) {
        let x1 = start.x as i32;
        let y1 = start.y as i32;
        let x2 = end.x as i32;
        let y2 = end.y as i32;

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x2 >= x1 {
            1
        } else {
            -1
        };
        let sy = if y2 >= y1 {
            1
        } else {
            -1
        };

        if dy <= dx {
            let mut d = (dy << 1) - dx;
            let d1 = dy << 1;
            let d2 = (dy - dx) << 1;

            self.put_pixel(start);

            let mut x = x1 + sx;
            let mut y = y1;
            for i in 1..dx {
                if d > 0 {
                    d = d + d2;
                    y = y + sy;
                } else {
                    d = d + d1;
                }

                self.put_pixel(Pixel { x: x as usize, y: y as usize, color: start.color });

                x = x + sx;
            }
        } else {
            let mut d = (dx << 1) - dy;
            let d1 = dx << 1;
            let d2 = (dx - dy) << 1;

            self.put_pixel(start);

            let mut x = x1;
            let mut y = y1 + sy;
            for i in 1..dy {
                if d > 0 {
                    d = d + d2;
                    x = x + sx;
                } else {
                    d = d + d1;
                }

                self.put_pixel(Pixel { x: x as usize, y: y as usize, color: start.color });

                y = y + sy;
            }
        }
    }
}