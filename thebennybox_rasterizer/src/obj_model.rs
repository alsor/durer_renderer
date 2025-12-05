use crate::indexed_model::IndexedModel;
use crate::Vector4f;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OBJIndex {
    vertex_index: i32,
    tex_coord_index: i32,
    normal_index: i32,
}

impl OBJIndex {
    fn new(vertex_index: i32, tex_coord_index: i32, normal_index: i32) -> Self {
        Self { vertex_index, tex_coord_index, normal_index }
    }
}

pub struct OBJModel {
    positions: Vec<Vector4f>,
    tex_coords: Vec<Vector4f>,
    normals: Vec<Vector4f>,
    indices: Vec<OBJIndex>,
    has_tex_coords: bool,
    has_normals: bool,
}

impl OBJModel {
    fn remove_empty_strings(tokens: &[String]) -> Vec<String> {
        tokens.iter().filter(|s| !s.is_empty()).cloned().collect()
    }

    pub fn new<P: AsRef<Path>>(file_name: P) -> Result<Self, std::io::Error> {
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);

        let mut positions = Vec::new();
        let mut tex_coords = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut has_tex_coords = false;
        let mut has_normals = false;

        for line_result in reader.lines() {
            let line = line_result?;
            let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
            let tokens = Self::remove_empty_strings(&tokens);

            if tokens.is_empty() || tokens[0] == "#" {
                continue;
            }

            match tokens[0].as_str() {
                "v" => {
                    if tokens.len() >= 4 {
                        let x = tokens[1].parse::<f64>().unwrap_or(0.0);
                        let y = tokens[2].parse::<f64>().unwrap_or(0.0);
                        let z = tokens[3].parse::<f64>().unwrap_or(0.0);
                        positions.push(Vector4f::new(x, y, z, 1.0));
                    }
                }
                "vt" => {
                    if tokens.len() >= 3 {
                        let u = tokens[1].parse::<f64>().unwrap_or(0.0);
                        let v = 1.0 - tokens[2].parse::<f64>().unwrap_or(0.0);
                        tex_coords.push(Vector4f::new(u, v, 0.0, 0.0));
                        has_tex_coords = true;
                    }
                }
                "vn" => {
                    if tokens.len() >= 4 {
                        let x = tokens[1].parse::<f64>().unwrap_or(0.0);
                        let y = tokens[2].parse::<f64>().unwrap_or(0.0);
                        let z = tokens[3].parse::<f64>().unwrap_or(0.0);
                        normals.push(Vector4f::new(x, y, z, 0.0));
                        has_normals = true;
                    }
                }
                "f" => {
                    let mut face_indices = Vec::new();
                    for token in &tokens[1..] {
                        face_indices.push(parse_obj_index(token, &mut has_tex_coords, &mut has_normals));
                    }

                    // Триангуляция полигона: fan triangulation
                    for i in 1..(face_indices.len() - 1) {
                        indices.push(face_indices[0]);
                        indices.push(face_indices[i]);
                        indices.push(face_indices[i + 1]);
                    }
                }
                _ => {}
            }
        }

        Ok(OBJModel {
            positions,
            tex_coords,
            normals,
            indices,
            has_tex_coords,
            has_normals,
        })
    }

    pub fn to_indexed_model(self) -> IndexedModel {
        let mut result = IndexedModel::new();
        let mut normal_model = IndexedModel::new();

        let mut result_index_map = HashMap::new();
        let mut normal_index_map = HashMap::new();
        let mut index_map = HashMap::new(); // modelVertexIndex → normalModelIndex

        for obj_index in &self.indices {
            let pos = self.positions[obj_index.vertex_index as usize];
            let tex = if self.has_tex_coords && obj_index.tex_coord_index >= 0 {
                self.tex_coords[obj_index.tex_coord_index as usize]
            } else {
                Vector4f::new(0.0, 0.0, 0.0, 0.0)
            };
            let norm = if self.has_normals && obj_index.normal_index >= 0 {
                self.normals[obj_index.normal_index as usize]
            } else {
                Vector4f::new(0.0, 0.0, 0.0, 0.0)
            };

            let model_vertex_index = *result_index_map.entry(*obj_index).or_insert_with(|| {
                result.positions.push(pos);
                result.tex_coords.push(tex);
                if self.has_normals {
                    result.normals.push(norm);
                }
                (result.positions.len() - 1) as u32
            });

            let normal_model_index = *normal_index_map.entry(obj_index.vertex_index).or_insert_with(|| {
                normal_model.positions.push(pos);
                normal_model.tex_coords.push(tex);
                normal_model.normals.push(norm);
                normal_model.tangents.push(Vector4f::new(0.0, 0.0, 0.0, 0.0));
                (normal_model.positions.len() - 1) as u32
            });

            result.indices.push(model_vertex_index as usize);
            normal_model.indices.push(normal_model_index as usize);
            index_map.insert(model_vertex_index, normal_model_index);
        }

        // Если нормалей не было — вычислить из геометрии
        if !self.has_normals {
            normal_model.calc_normals();
            // Скопировать нормали в result
            for i in 0..result.positions.len() {
                let normal_idx = index_map[&(i as u32)];
                result.normals.push(normal_model.normals[normal_idx as usize]);
            }
        }

        // Вычислить касательные (даже если нормалей не было — они нужны для нормальных карт)
        normal_model.calc_tangents();
        for i in 0..result.positions.len() {
            let tangent_idx = index_map[&(i as u32)];
            result.tangents.push(normal_model.tangents[tangent_idx as usize]);
        }

        result
    }
}

fn parse_obj_index(token: &str, has_tex_coords: &mut bool, has_normals: &mut bool) -> OBJIndex {
    let parts: Vec<&str> = token.split('/').collect();

    let vertex_index = parts[0].parse::<i32>().unwrap_or(0) - 1;

    let mut tex_coord_index = -1;
    let mut normal_index = -1;

    if parts.len() > 1 {
        if !parts[1].is_empty() {
            *has_tex_coords = true;
            tex_coord_index = parts[1].parse::<i32>().unwrap_or(0) - 1;
        }

        if parts.len() > 2 && !parts[2].is_empty() {
            *has_normals = true;
            normal_index = parts[2].parse::<i32>().unwrap_or(0) - 1;
        }
    }

    OBJIndex::new(vertex_index, tex_coord_index, normal_index)
}
