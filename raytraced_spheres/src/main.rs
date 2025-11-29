//! A scene with few spheres. Scene itself configured in the main function.

use common::vectors;
use common::{Color, Light, Vector3f};
use gambetta_raytracer::Sphere;
use sdl3::{event::Event, keyboard::Keycode, pixels::PixelFormat};

fn main() {
    let size = 900;
    let mut buffer = vec![0u8; size as usize * size as usize * 3];

    let mut x_position = 0.0;
    let mut y_position = 1.0;
    let mut z_position = -3.0;

    let mut angle = 0.0;

    let origin = Vector3f { x: x_position, y: y_position, z: z_position };
    let rotation = vectors::rotation_around_y(angle);

    let spheres = vec![
        Sphere {
            center: Vector3f { x: 0.0, y: -1.0, z: 3.0 },
            radius: 1.0,
            color: Color { r: 255, g: 0, b: 0 },
            specular: 200,
            reflective: 0.0,
        },
        Sphere {
            center: Vector3f { x: -2.0, y: 0.5, z: 4.0 },
            radius: 1.0,
            color: Color { r: 150, g: 150, b: 150 },
            specular: 200,
            reflective: 0.5,
        },
        Sphere {
            center: Vector3f { x: 2.0, y: 1.0, z: 3.0 },
            radius: 1.0,
            color: Color { r: 0, g: 0, b: 255 },
            specular: 200,
            reflective: 0.3,
        },
        Sphere {
            center: Vector3f { x: 0.0, y: -5001.0, z: 0.0 },
            radius: 5000.0,
            color: Color { r: 100, g: 100, b: 0 },
            specular: 0,
            reflective: 0.0,
        },
    ];

    let lights = vec![
        Light::Ambient { intensity: 0.3 },
        // Light::Point { intensity: 0.8, position: Vector3f { x: x_position, y: y_position, z: z_position } }
        Light::Directional {
            intensity: 0.8,
            direction: Vector3f { x: 1.0, y: 4.0, z: 4.0 },
        },
    ];

    gambetta_raytracer::render_scene_to_buffer(&spheres, &lights, &mut buffer, size, origin, rotation);

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Durer", size as u32, size as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();

    //    let mut texture = texture_creator.create_texture_streaming(
    //        PixelFormatEnum::RGB24, 256, 256
    //    ).unwrap();

    //    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
    //        for y in 0..256 {
    //            for x in 0..256 {
    //                let offset = y * pitch + x * 3;
    //                buffer[offset] = x as u8;
    //                buffer[offset + 1] = y as u8;
    //                buffer[offset + 2] = 0;
    //            }
    //        }
    //    }).unwrap();

    let mut texture = texture_creator
        .create_texture_static(PixelFormat::RGB24, size as u32, size as u32)
        .unwrap();

    texture.update(None, &buffer, size * 3).unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    //    canvas.copy_ex(&texture, None, Some(Rect::new(450, 100, 256, 256)), 30.0, None, false, false).unwrap();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.wait_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                    angle += 10.0;
                }
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    angle -= 10.5;
                }
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    x_position += 0.5;
                }
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    x_position -= 0.5;
                }
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    z_position += 0.5;
                }
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    z_position -= 0.5;
                }
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    y_position += 0.5;
                }
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    y_position -= 0.5;
                }
                _ => {}
            }

            let mut buffer = vec![0u8; size as usize * size as usize * 3];
            let origin = Vector3f { x: x_position, y: y_position, z: z_position };
            let rotation = vectors::rotation_around_y(angle);

            // let lights = vec![
            //     Light::Ambient { intensity: 0.1 },
            //     Light::Point {
            //         intensity: 0.8,
            //         position: Vector3f { x: x_position, y: y_position, z: z_position }
            //     },
            // ];

            gambetta_raytracer::render_scene_to_buffer(
                &spheres,
                &lights,
                &mut buffer,
                size,
                origin,
                rotation,
            );

            texture.update(None, &buffer, size * 3).unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
    }
}
