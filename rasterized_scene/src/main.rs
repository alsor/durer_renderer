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
use minifb::{Key, Window, WindowOptions};
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

    // Create window with minifb
    let mut window = Window::new(
        "Durer",
        buffer_canvas.size,
        buffer_canvas.size,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

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
    //    window.update_with_buffer(&buffer_canvas.buffer, buffer_canvas.size, buffer_canvas.size).unwrap();
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

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
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

        // Convert RGB buffer to u32 buffer for minifb
        let mut buffer_u32 = vec![0u32; buffer_canvas.size * buffer_canvas.size];
        for i in 0..buffer_canvas.size * buffer_canvas.size {
            let r = buffer_canvas.buffer[i * 3] as u32;
            let g = buffer_canvas.buffer[i * 3 + 1] as u32;
            let b = buffer_canvas.buffer[i * 3 + 2] as u32;
            buffer_u32[i] = (r << 16) | (g << 8) | b;
        }

        // Update window with new buffer
        window
            .update_with_buffer(&buffer_u32, buffer_canvas.size, buffer_canvas.size)
            .unwrap();

        // Handle keyboard input
        if window.is_key_down(Key::E) {
            if cfg!(feature = "smooth_animation") {
                delta_angle += angle_increase;
            } else {
                angle += delta_angle;
            };
        }
        if window.is_key_down(Key::Q) {
            if cfg!(feature = "smooth_animation") {
                delta_angle -= angle_increase;
            } else {
                angle -= delta_angle;
            };
        }
        if window.is_key_down(Key::D) {
            if cfg!(feature = "smooth_animation") {
                delta_x += step_increase;
            } else {
                x_position += delta_x;
            };
        }
        if window.is_key_down(Key::A) {
            if cfg!(feature = "smooth_animation") {
                delta_x -= step_increase;
            } else {
                x_position -= delta_x;
            };
        }
        if window.is_key_down(Key::W) {
            if cfg!(feature = "smooth_animation") {
                delta_z += step_increase;
            } else {
                z_position += delta_z;
            };
        }
        if window.is_key_down(Key::S) {
            if cfg!(feature = "smooth_animation") {
                delta_z -= step_increase;
            } else {
                z_position -= delta_z;
            };
        }
        if window.is_key_down(Key::T) {
            if cfg!(feature = "smooth_animation") {
                delta_y += step_increase;
            } else {
                y_position += delta_y;
            };
        }
        if window.is_key_down(Key::G) {
            if cfg!(feature = "smooth_animation") {
                delta_y -= step_increase;
            } else {
                y_position -= delta_y;
            };
        }
        if window.is_key_down(Key::X) {
            viewport_size += viewport_size_delta;
        }
        if window.is_key_down(Key::Z) {
            viewport_size -= viewport_size_delta;
        }
        if window.is_key_down(Key::R) {
            projection_plane_z += projection_plane_z_delta;
        }
        if window.is_key_down(Key::F) {
            projection_plane_z -= projection_plane_z_delta;
        }
        if window.is_key_down(Key::F1) {
            rendering_settings.rendering_mode = RenderingMode::Wireframe;
        }
        if window.is_key_down(Key::F2) {
            rendering_settings.rendering_mode = RenderingMode::Filled;
        }
        if window.is_key_down(Key::F3) {
            rendering_settings.shading_model = ShadingModel::Flat;
        }
        if window.is_key_down(Key::F4) {
            rendering_settings.shading_model = ShadingModel::Gouraud;
        }
        if window.is_key_down(Key::F5) {
            rendering_settings.shading_model = ShadingModel::Phong;
        }
        if window.is_key_down(Key::F8) {
            rendering_settings.backface_culling = !rendering_settings.backface_culling;
        }
        if window.is_key_down(Key::F9) {
            rendering_settings.show_normals = !rendering_settings.show_normals;
        }
        if window.is_key_down(Key::F12) {
            write_image(&mut buffer_canvas.buffer, buffer_canvas.size)
                .expect("Error writing image to file");
        }

        // Reset key states for next frame
        window.update();

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
