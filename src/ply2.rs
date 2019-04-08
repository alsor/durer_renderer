extern crate rand;

use std::fs::File;
use super::Point3D;
use model::Model;
use std::str::FromStr;
use std::io::prelude::*;
use Color;
use self::rand::Rng;

pub fn load_model(filename: &str) -> Model {
    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("error reading file");

    #[derive(Copy,Clone)]
    enum Ply2Parts { NumVertices, NumFaces, Vertices, Faces }
    let ply2_structure = [
        Ply2Parts::NumVertices,
        Ply2Parts::NumFaces,
        Ply2Parts::Vertices,
        Ply2Parts::Faces,
    ];
    let mut current_section = 0;

    let mut num_vertices = 0;
    let mut num_faces = 0;
    let mut current_vertex = 0;
    let mut current_face = 0;
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    let mut colors = Vec::new();
    let mut rng = rand::thread_rng();

    for line in contents.split("\n") {
        //        let parsed = match i32::from_str(line.trim()) {
        //            Ok(num) => num,
        //            Err(e) => {
        //                println!("error: {}", e);
        //                0
        //            }
        //        };
        //        println!("parsed: {:?}", parsed);
        if current_section == 4 {
            break;
        }

        match ply2_structure[current_section] {
            Ply2Parts::NumVertices => {
                num_vertices = i32::from_str(line.trim()).unwrap();
                current_section += 1;
            }
            Ply2Parts::NumFaces => {
                num_faces = i32::from_str(line.trim()).unwrap();
                current_section += 1;
            }
            Ply2Parts::Vertices => {
                let mut coords = Vec::new();
                for float in line.trim().split(" ") {
                    coords.push(f64::from_str(float).unwrap());
                }
                vertices.push(Point3D { x: coords[0], y: coords[1], z: coords[2] });
                current_vertex += 1;
                if current_vertex == num_vertices {
                    current_section += 1;
                }
            }
            Ply2Parts::Faces => {
                let mut faces_list = Vec::new();
                let mut face = Vec::new();
                for str in line.trim().split(" ") {
                    faces_list.push(i32::from_str(str.trim()).unwrap());
                }
                let vertices_in_face = faces_list[0];
                for i in 1..(vertices_in_face + 1) {
                    face.push(faces_list[i as usize]);
                }
                faces.push(face);
//                colors.push(Color { r: rng.gen(), g: rng.gen(), b: rng.gen() });
                colors.push(Color { r: 119, g: 136, b: 153 });

                current_face += 1;
                if current_face == num_faces {
                    current_section += 1;
                }
            }
            _ => ()
        }
    }

    println!("vertices read: {}", vertices.len());
    println!("faces read: {}", faces.len());

    Model { vertices, faces, colors }
}
