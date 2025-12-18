use common::vectors;
use common::{Color, Light, Vector3f};
use gambetta_raytracer::{CSGOperation, Shape, Sphere, Transform};
use image::RgbImage;
use sdl3::{event::Event, keyboard::Keycode, pixels::PixelFormat};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Проверка на наличие флага --animate-to
    if let Some(dir_index) = args.iter().position(|a| a == "--animate-to") {
        if dir_index + 3 < args.len()
            && args.get(dir_index + 2) == Some(&"--frames-limit".to_string())
            && args.get(dir_index + 4) == Some(&"--delta".to_string())
        {
            let output_dir = &args[dir_index + 1];
            let frames_limit: usize = args[dir_index + 3].parse().expect("Frames limit must be a number");
            let delta: f64 = args[dir_index + 5].parse().expect("Delta must be a float");

            println!(
                "Запуск анимации: сохранение {} кадров в '{}', шаг поворота = {}",
                frames_limit, output_dir, delta
            );

            // Создаём директорию
            fs::create_dir_all(output_dir).expect("Не удалось создать директорию");

            let size = 900;
            let mut buffer = vec![0u8; size as usize * size as usize * 3];

            let complex_shape = create_complex_shape();
            let ground_sphere = Shape::Sphere(Sphere {
                center: Vector3f { x: 0.0, y: -5001.5, z: 0.0 },
                radius: 5000.0,
                color: Color { r: 100, g: 100, b: 0 },
                specular: 50,
                reflective: 0.4,
            });

            let rotation = vectors::rotate_y_deg(0.0);
            // Анимация вращения
            for frame in 0..frames_limit {
                let angle = frame as f64 * delta; // Меняем угол
                let x_position = 0.0;
                let y_position = 0.5;
                let z_position = -7.0;

                let origin = Vector3f { x: x_position, y: y_position, z: z_position };

                // === АНИМАЦИЯ СВЕТА: движение по оси X от -1 до +1 ===
                let light_x = (frame as f64 * delta * 0.02).sin(); // Медленное колебание
                let lights = vec![
                    Light::Ambient { intensity: 0.25 },
                    Light::Point {
                        intensity: 0.85,
                        position: Vector3f { x: light_x, y: 2.0, z: 0.0 },
                    },
                ];

                let complex_shape_with_transform = Shape::Transformed {
                    shape: Box::new(complex_shape.clone()),
                    transform: Transform {
                        translation: Vector3f::new(0.0, 0.0, 0.0),
                        rotation: vectors::multiply_mat_3x3(
                            vectors::rotate_y_deg(angle),
                            vectors::rotate_x_deg(angle / 2.0),
                        ),
                    },
                };

                let scene = vec![complex_shape_with_transform, ground_sphere.clone()];

                // Рендерим кадр
                gambetta_raytracer::render_scene_to_buffer(
                    &scene,
                    &lights,
                    &mut buffer,
                    size,
                    origin,
                    rotation,
                );

                // Конвертируем буфер в изображение
                let img = RgbImage::from_raw(size as u32, size as u32, buffer.clone())
                    .expect("Не удалось создать изображение");

                // Имя файла: frame_000001.png, frame_000002.png и т.д.
                let filename = format!("{}/frame_{:06}.png", output_dir, frame + 1);
                img.save(&filename).unwrap_or_else(|e| {
                    eprintln!("Ошибка при сохранении {}: {}", filename, e);
                });

                println!("Сохранён кадр {}/{}: {}", frame + 1, frames_limit, filename);
            }

            println!("Анимация завершена. Кадры сохранены в '{}'.", output_dir);
            return;
        } else {
            eprintln!("Использование: --animate-to <dir> --frames-limit <число> --delta <значение>");
            std::process::exit(1);
        }
    }

    // === Основной интерактивный режим (GUI) ===

    let size = 900;
    let mut buffer = vec![0u8; size as usize * size as usize * 3];

    let mut x_position = 0.0;
    let mut y_position = 0.5;
    let mut z_position = -7.0;

    let mut angle = 0.0;

    let origin = Vector3f { x: x_position, y: y_position, z: z_position };
    let rotation = vectors::rotate_y_deg(angle);

    let ground_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 0.0, y: -5001.5, z: 0.0 },
        radius: 5000.0,
        color: Color { r: 100, g: 100, b: 0 },
        specular: 50,
        reflective: 0.4,
    });

    let complex_shape = create_complex_shape();
    let complex_shape_with_transform = Shape::Transformed {
        shape: Box::new(complex_shape),
        transform: Transform {
            translation: Vector3f::new(0.0, 0.0, 0.0),
            rotation: vectors::multiply_mat_3x3(vectors::rotate_y_deg(45.0), vectors::rotate_z_deg(45.0)),
        },
    };
    // let rotated_complex_shape = complex_shape.rotate_y_all_deg(45.0, Vector3f::new(0.0, 0.0, 0.0));

    let scene = vec![complex_shape_with_transform, ground_sphere];

    // let scene = vec![
    //     Shape::Sphere(Sphere {
    //         center: Vector3f { x: 0.0, y: 0.0, z: 3.0 },
    //         radius: 1.0,
    //         color: Color { r: 255, g: 0, b: 0 },
    //         specular: 200,
    //         reflective: 0.0,
    //     }),
    //     Shape::Sphere(Sphere {
    //         center: Vector3f { x: 0.6, y: 0.6, z: 2.6 },
    //         radius: 1.2,
    //         color: Color { r: 0, g: 0, b: 255 },
    //         specular: 200,
    //         reflective: 0.0,
    //     }),
    // ];

    // let scene = vec![
    //     Shape::Sphere(Sphere {
    //         center: Vector3f { x: 0.0, y: -1.0, z: 3.0 },
    //         radius: 1.0,
    //         color: Color { r: 255, g: 0, b: 0 },
    //         specular: 200,
    //         reflective: 0.0,
    //     }),
    //     Shape::Sphere(Sphere {
    //         center: Vector3f { x: -2.0, y: 0.5, z: 4.0 },
    //         radius: 1.0,
    //         color: Color { r: 150, g: 150, b: 150 },
    //         specular: 200,
    //         reflective: 0.5,
    //     }),
    //     Shape::Sphere(Sphere {
    //         center: Vector3f { x: 2.0, y: 1.0, z: 3.0 },
    //         radius: 1.0,
    //         color: Color { r: 0, g: 0, b: 255 },
    //         specular: 200,
    //         reflective: 0.3,
    //     }),
    //     Shape::Sphere(Sphere {
    //         center: Vector3f { x: 0.0, y: -5001.0, z: 0.0 },
    //         radius: 5000.0,
    //         color: Color { r: 100, g: 100, b: 0 },
    //         specular: 0,
    //         reflective: 0.0,
    //     }),
    // ];

    let lights = vec![
        Light::Ambient { intensity: 0.25 },
        Light::Point {
            intensity: 0.85,
            position: Vector3f { x: 0.0, y: 2.0, z: 0.0 },
        }, // Light::Directional {
           //     intensity: 0.8,
           //     direction: Vector3f { x: -0.5, y: -0.2, z: 0.0 },
           // },
    ];

    gambetta_raytracer::render_scene_to_buffer(&scene, &lights, &mut buffer, size, origin, rotation);

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
                Event::KeyDown { keycode: Some(Keycode::F12), .. } => {
                    // Сохраняем скриншот
                    let img = RgbImage::from_raw(size as u32, size as u32, buffer.clone())
                        .expect("Невозможно создать изображение из буфера");

                    if let Err(e) = img.save("screenshot.png") {
                        eprintln!("Ошибка при сохранении скриншота: {}", e);
                    } else {
                        println!("Скриншот сохранён как screenshot.png");
                    }
                }
                _ => {}
            }

            let mut buffer = vec![0u8; size as usize * size as usize * 3];
            let origin = Vector3f { x: x_position, y: y_position, z: z_position };
            let rotation = vectors::rotate_y_deg(angle);

            // let lights = vec![
            //     Light::Ambient { intensity: 0.1 },
            //     Light::Point {
            //         intensity: 0.8,
            //         position: Vector3f { x: x_position, y: y_position, z: z_position }
            //     },
            // ];

            gambetta_raytracer::render_scene_to_buffer(&scene, &lights, &mut buffer, size, origin, rotation);

            texture.update(None, &buffer, size * 3).unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
    }
}

fn create_complex_shape() -> Shape {
    // // Внутренняя сфера — неоново-бирюзовая
    // let blue_inside_sphere = Shape::Sphere(Sphere {
    //     center: Vector3f { x: 0.0, y: 0.0, z: 0.0 },
    //     radius: 0.9,
    //     color: Color { r: 0, g: 255, b: 200 }, // ← Cyber Cyan
    //     specular: 200,
    //     reflective: 0.2,
    // });

    // // Или внешняя сфера — металлический фиолетовый
    // let red_sphere = Shape::Sphere(Sphere {
    //     center: Vector3f { x: 0.0, y: 0.0, z: 0.0 },
    //     radius: 1.0,
    //     color: Color { r: 80, g: 0, b: 150 }, // ← Metallic Violet
    //     specular: 300,                        // Увеличим блики для "металлического" эффекта
    //     reflective: 0.4,                      // Добавим отражений
    // });

    let red_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 0.0, y: 0.0, z: 0.0 },
        radius: 1.0,
        color: Color { r: 255, g: 0, b: 0 },
        specular: 200,
        reflective: 0.0,
    });
    let blue_inside_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 0.0, y: 0.0, z: 0.0 },
        radius: 0.9,
        color: Color { r: 0, g: 0, b: 255 },
        specular: 200,
        reflective: 0.0,
    });
    let thin_sphere = Shape::CSG {
        op: CSGOperation::Difference,
        left: Box::new(red_sphere),
        right: Box::new(blue_inside_sphere),
    };
    let right_cutoff_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 1.0, y: 0.0, z: 0.0 },
        radius: 0.6,
        color: Color { r: 0, g: 255, b: 0 },
        specular: 200,
        reflective: 0.0,
    });
    let cutoff_from_right = Shape::CSG {
        op: CSGOperation::Difference,
        left: Box::new(thin_sphere),
        right: Box::new(right_cutoff_sphere),
    };
    let left_cuttoff_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: -1.0, y: 0.0, z: 0.0 },
        radius: 0.6,
        color: Color { r: 0, g: 255, b: 0 },
        specular: 200,
        reflective: 0.0,
    });
    let cutoff_from_left = Shape::CSG {
        op: CSGOperation::Difference,
        left: Box::new(cutoff_from_right),
        right: Box::new(left_cuttoff_sphere),
    };
    let top_cuttoff_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 0.0, y: 1.0, z: 0.0 },
        radius: 0.6,
        color: Color { r: 0, g: 255, b: 0 },
        specular: 200,
        reflective: 0.0,
    });
    let cutoff_from_top = Shape::CSG {
        op: CSGOperation::Difference,
        left: Box::new(cutoff_from_left),
        right: Box::new(top_cuttoff_sphere),
    };
    let bottom_cuttoff_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 0.0, y: -1.0, z: 0.0 },
        radius: 0.6,
        color: Color { r: 0, g: 255, b: 0 },
        specular: 200,
        reflective: 0.0,
    });
    let cutoff_from_bottom = Shape::CSG {
        op: CSGOperation::Difference,
        left: Box::new(cutoff_from_top),
        right: Box::new(bottom_cuttoff_sphere),
    };
    let front_cuttoff_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 0.0, y: 0.0, z: -1.0 },
        radius: 0.6,
        color: Color { r: 0, g: 255, b: 0 },
        specular: 200,
        reflective: 0.0,
    });
    let cutoff_from_front = Shape::CSG {
        op: CSGOperation::Difference,
        left: Box::new(cutoff_from_bottom),
        right: Box::new(front_cuttoff_sphere),
    };
    let back_cuttoff_sphere = Shape::Sphere(Sphere {
        center: Vector3f { x: 0.0, y: 0.0, z: 1.0 },
        radius: 0.6,
        color: Color { r: 0, g: 255, b: 0 },
        specular: 200,
        reflective: 0.0,
    });
    Shape::CSG {
        op: CSGOperation::Difference,
        left: Box::new(cutoff_from_front),
        right: Box::new(back_cuttoff_sphere),
    }
}
