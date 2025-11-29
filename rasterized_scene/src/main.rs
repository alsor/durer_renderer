//! A rasterized scene using 'gambetta_rasterizer' lib. Scene itself configured in the main function.

mod ply2;

use common::{Color, Light, Vector3f};
use gambetta_rasterizer::model;
use gambetta_rasterizer::{
    texture, BufferCanvas, Instance, Matrix44f, ProjectiveCamera, RenderingMode, RenderingSettings,
    ShadingModel, Vector4f,
};
use image::png::PNGEncoder;
use image::ColorType;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::PixelFormat;
use std::fs::File;
use std::time::Instant;

fn main() {
    env_logger::init();

    let mut rendering_settings = RenderingSettings {
        rendering_mode: RenderingMode::Filled,
        shading_model: ShadingModel::Phong,
        show_normals: false,
        backface_culling: true,
    };
    let mut buffer_canvas = BufferCanvas::new(900);

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Durer", buffer_canvas.size as u32, buffer_canvas.size as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_static(
            PixelFormat::RGB24,
            buffer_canvas.size as u32,
            buffer_canvas.size as u32,
        )
        .unwrap();

    texture.update(None, &buffer_canvas.buffer, buffer_canvas.size * 3).unwrap();
    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    let viewport_size_delta = 0.1;
    let mut viewport_size = 1.0;
    let projection_plane_z_delta = 0.1;
    let mut projection_plane_z = 1.0;
    let mut x_position = 0.0;
    let mut y_position = 0.0;
    let mut z_position = 0.0;
    let mut angle = 0.0;

    let red = Color { r: 255, g: 0, b: 0 };
    let green = Color { r: 0, g: 255, b: 0 };
    let blue = Color { r: 0, g: 0, b: 255 };
    let white = Color { r: 255, g: 255, b: 255 };

    let wooden_crate = texture::load_from_file("resources/textures/wooden-crate.jpg");
    let bricks = texture::load_from_file("resources/textures/bricks.jpg");

    //    let cube = two_unit_cube();
    // let sphere = model::sphere(50);
    let random_cubes = model::random_cubes_scene(50, 25.0);
    println!("{random_cubes}");
    // let cube = model::cube(0.9);
    // let wooden_cube = model::textured_cube(0.9, &wooden_crate);
    // let brick_cube = model::textured_cube(1.0, &bricks);
    //    let triangle = triangle(5.0);
    // let torus = ply2::load_model("resources/torus.ply2");
    //    let twirl = ply2::load_model("resources/twirl.ply2");
    //    let octo_flower = ply2::load_model("resources/octa-flower.ply2");
    //    let statue = ply2::load_model("resources/statue.ply2");

    let mut current_instance_index: Option<usize> = None;

    let mut instances = vec![
        Instance::new(
            &random_cubes,
            Vector3f { x: 0.0, y: 0.0, z: 20.0 },
            1.0,
            Vector3f::zero_vector(),
        ),
        //    Instance::new(
        //        &triangle,
        //        Vector3f { x: 0.0, y: 0.0, z: 10.0 },
        //        1.0,
        //        Vector3f { x: 90.0, y: 0.0, z: 0.0 }
        //    ),
        // Instance::new(
        //     &wooden_cube,
        //     Vector3f { x: 1.0, y: 1.0, z: 4.0 },
        //     1.0,
        //     Vector3f { x: 0.0, y: -30.0, z: -30.0 },
        // ),
        // Instance::new(
        //     &brick_cube,
        //     Vector3f { x: -0.3, y: -0.4, z: 3.5 },
        //     1.0,
        //     Vector3f { x: 25.0, y: 20.0, z: 10.0 },
        // ),
        //    Instance::new(
        //        &torus,
        //        Vector3f { x: 0.0, y: 0.0, z: 5.0 },
        //        0.2,
        //        Vector3f { x: 90.0, y: 0.0, z: 0.0 }
        //    ),
        //    Instance::new(
        //        &cube,
        //        Vector3f { x: 0.0, y: 0.0, z: 4.0 },
        //        1.0,
        //        Vector3f::zero_vector()
        //    ),
        //        Instance::new(
        //            &cube,
        //            Vector3f { x: 2.0, y: -2.0, z: 4.5 },
        //            1.0,
        //            Vector3f { x: 0.0, y: -30.0, z: -30.0 }
        //        ),
        //    Instance::new(
        //        &torus,
        //        Vector3f { x: 0.0, y: 0.0, z: 10.0 },
        //        0.1,
        //        Vector3f { x: 90.0, y: 0.0, z: 0.0 }
        //    ),
        //
        // Instance::new(
        //     &sphere,
        //     Vector3f { x: 0.0, y: 0.0, z: 5.0 },
        //     1.3,
        //     Vector3f { x: 0.0, y: -45.0, z: 0.0 },
        // ),
        //    Instance::new(
        //        &octo_flower,
        //        Vector3f { x: 0.0, y: 0.0, z: 70.0 },
        //        1.0,
        //        Vector3f::zero_vector()
        //    ),
        //    Instance::new(
        //        &twirl,
        //        Vector3f { x: 0.0, y: 0.0, z: 30.0 },
        //        1.0,
        //        Vector3f::zero_vector()
        //    ),
        //    Instance::new(
        //        &statue,
        //        Vector3f { x: 0.0, y: 0.0, z: 10.0 },
        //        1.0,
        //        Vector3f { x: 30.0, y: 135.0, z: 0.0 }
        //    ),
    ];

    let lights = vec![
        Light::Ambient { intensity: 0.15 },
        Light::Directional {
            intensity: 0.7,
            direction: Vector3f { x: 1.0, y: 0.0, z: -0.5 },
        },
        Light::Point {
            intensity: 0.85,
            position: Vector3f { x: 0.0, y: 1.0, z: 0.0 },
        },
    ];

    //    rendering::render_scene(&scene, &camera, &mut buffer_canvas, &clipping_planes);
    //    texture.update(None, &buffer_canvas.buffer, buffer_canvas.size * 3).unwrap();
    //    canvas.clear();
    //    canvas.copy(&texture, None, None).unwrap();
    //    canvas.present();
    //

    let step_increase = 0.005;
    let angle_increase = 0.1;
    let mut delta_x;
    let mut delta_y;
    let mut delta_z;
    let mut delta_angle;
    if cfg!(feature = "smooth_animation") {
        println!("configured to smooth animation");
        delta_x = 0.0;
        delta_y = 0.0;
        delta_z = 0.0;
        delta_angle = 0.0;
    } else {
        println!("configured to by step animation");
        delta_x = 0.1;
        delta_y = 0.1;
        delta_z = 0.1;
        delta_angle = 1.0;
    };

    // instances[0].rotation_delta.x = angle_increase * 4.5;
    // instances[0].rotation_delta.y = angle_increase * 2.0;
    // instances[0].rotation_delta.z = angle_increase * 6.5;
    //    instances[0].position_delta.x = -0.005;
    //    instances[0].position_delta.z = 0.03;
    //    instances[0].scale_delta = 0.008;

    // instances[1].rotation_delta.y = angle_increase * 8.0;
    // instances[1].rotation_delta.z = angle_increase * 3.0;
    // instances[1].position_delta.z = 0.01;

    let mut now = Instant::now();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let delta = now.elapsed();
        //        println!("delta: {:?}", (delta.as_nanos() as f64) / 1000000000.0);

        log::trace!("z_position: {:.2}", z_position);

        if cfg!(feature = "smooth_animation") {
            x_position += delta_x;
            y_position += delta_y;
            z_position += delta_z;
            angle += delta_angle;

            instances[0].apply_deltas();
            // instances[1].apply_deltas();
        };

        let camera = ProjectiveCamera {
            viewport_size,
            projection_plane_z,
            position: Vector4f {
                x: x_position,
                y: y_position,
                z: z_position,
                w: 0.0,
            },
            rotation: Matrix44f::rotation_y(angle),
        };

        buffer_canvas.clear();

        gambetta_rasterizer::render_scene(
            &instances,
            &lights,
            &camera,
            &rendering_settings,
            &mut buffer_canvas,
        );

        texture.update(None, &buffer_canvas.buffer, buffer_canvas.size * 3).unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        match if cfg!(feature = "smooth_animation") {
            event_pump.poll_event()
        } else {
            Some(event_pump.wait_event())
        } {
            Some(event) => {
                log::trace!("event happened");
                match event {
                    Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    }
                    Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_angle += angle_increase;
                        } else {
                            angle += delta_angle;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_angle -= angle_increase;
                        } else {
                            angle -= delta_angle;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_x += step_increase;
                        } else {
                            x_position += delta_x;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_x -= step_increase;
                        } else {
                            x_position -= delta_x;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_z += step_increase;
                        } else {
                            z_position += delta_z;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_z -= step_increase;
                        } else {
                            z_position -= delta_z;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::T), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_y += step_increase;
                        } else {
                            y_position += delta_y;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::G), .. } => {
                        if cfg!(feature = "smooth_animation") {
                            delta_y -= step_increase;
                        } else {
                            y_position -= delta_y;
                        };
                    }
                    Event::KeyDown { keycode: Some(Keycode::X), .. } => {
                        viewport_size += viewport_size_delta;
                    }
                    Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                        viewport_size -= viewport_size_delta;
                    }
                    Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                        projection_plane_z += projection_plane_z_delta;
                    }
                    Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                        projection_plane_z -= projection_plane_z_delta;
                    }
                    Event::KeyDown { keycode: Some(Keycode::F1), .. } => {
                        rendering_settings.rendering_mode = RenderingMode::Wireframe
                    }
                    Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                        rendering_settings.rendering_mode = RenderingMode::Filled
                    }
                    Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                        rendering_settings.shading_model = ShadingModel::Flat
                    }
                    Event::KeyDown { keycode: Some(Keycode::F4), .. } => {
                        rendering_settings.shading_model = ShadingModel::Gouraud
                    }
                    Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                        rendering_settings.shading_model = ShadingModel::Phong
                    }
                    Event::KeyDown { keycode: Some(Keycode::F8), .. } => {
                        rendering_settings.backface_culling = !rendering_settings.backface_culling
                    }
                    Event::KeyDown { keycode: Some(Keycode::F9), .. } => {
                        rendering_settings.show_normals = !rendering_settings.show_normals
                    }
                    Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                        write_image(&mut buffer_canvas.buffer, buffer_canvas.size)
                            .expect("Error writing image to file");
                    }
                    Event::KeyDown { keycode, scancode, keymod, .. } => {
                        println!(
                            "Keycode: {:?}, Scancode: {:?}, Keymode: {:?}",
                            keycode, scancode, keymod
                        );
                    }
                    _ => {}
                };
            }
            None => {}
        };
        //        thread::sleep(time::Duration::from_millis(10));
        now = Instant::now();
    }

    //    let p0 = Point { x: -200, y: -250, h: 0.1 };
    //    let p1 = Point { x: 200, y: 50, h: 0.0 };
    //    let p2 = Point { x: 20, y: 250, h: 1.0 };
    //
    //    draw_filled_triangle(p0, p1, p2, green, &mut buffer_canvas);
    //    draw_wireframe_triangle(p0, p1, p2, white, &mut buffer_canvas);

    //    let half = 0.8;
    //    let frame = Frame { x_min: -half, x_max: half, y_min: -half, y_max: half };
    //    let vertices = transform(&vertices, Point3D { x: 0.0, y: 0.0, z: 45.0 });
    //    render_model_to_buffer(&mut buffer, size, vertices, faces, frame);

    //        point_light_position += 0.01;
    //        green_sphere_position_z += 0.01;
    //        blue_sphere_position_x += 0.01;

    //    write_image(&mut buffer_canvas.buffer, buffer_canvas.size).expect("Error writing image to file");
    //    show_buffer_in_window(&mut buffer_canvas.buffer, buffer_canvas.size);

    //    rotating_cube_window(&mut buffer, size);

    //    read_ply2("resources/statue.ply2");
    //    read_ply2("resources/torus.ply2");
    //    read_ply2("resources/cube.ply2");
    //    read_ply2("resources/twirl.ply2");
    //    read_ply2("resources/octa-flower.ply2");

    //    write_image(&buffer, size).expect("Error writing image to file");
    //    show_buffer_in_window(&mut buffer, size);
}

fn write_image(buffer: &[u8], size: usize) -> Result<(), std::io::Error> {
    let output = File::create("target/result.png")?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&buffer, size as u32, size as u32, ColorType::RGB(8))?;

    Ok(())
}
