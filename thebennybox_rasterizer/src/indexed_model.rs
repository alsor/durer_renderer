use crate::Vector4f;

pub struct IndexedModel {
    pub positions: Vec<Vector4f>,
    pub tex_coords: Vec<Vector4f>,
    pub normals: Vec<Vector4f>,
    pub tangents: Vec<Vector4f>,
    pub indices: Vec<usize>,
}

impl IndexedModel {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            tex_coords: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Вычисляет нормали для каждой вершины на основе граней (треугольников)
    pub fn calc_normals(&mut self) {
        // Инициализируем нормали нулевыми векторами
        self.normals = vec![Vector4f::new(0.0, 0.0, 0.0, 0.0); self.positions.len()];

        // Обрабатываем треугольники по 3 индекса
        for chunk in self.indices.chunks(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            let pos0 = self.positions[i0];
            let pos1 = self.positions[i1];
            let pos2 = self.positions[i2];

            // Векторы вдоль рёбер
            let edge1 = pos1.sub(pos0);
            let edge2 = pos2.sub(pos0);

            // Векторное произведение даёт нормаль к грани (w = 0)
            let face_normal = edge1.cross(edge2).normalized();

            // Накапливаем нормали для каждой вершины
            self.normals[i0] = self.normals[i0].add(face_normal);
            self.normals[i1] = self.normals[i1].add(face_normal);
            self.normals[i2] = self.normals[i2].add(face_normal);
        }

        // Нормализуем все накопленные нормали
        for normal in self.normals.iter_mut() {
            *normal = normal.normalized();
        }
    }

    /// Вычисляет касательные (tangents) для каждой вершины
    pub fn calc_tangents(&mut self) {
        // Инициализируем касательные нулями
        self.tangents = vec![Vector4f::new(0.0, 0.0, 0.0, 0.0); self.positions.len()];

        for chunk in self.indices.chunks(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            let pos0 = self.positions[i0];
            let pos1 = self.positions[i1];
            let pos2 = self.positions[i2];

            let uv0 = self.tex_coords[i0];
            let uv1 = self.tex_coords[i1];
            let uv2 = self.tex_coords[i2];

            // Рёбра в пространстве вершин
            let edge1 = pos1.sub(pos0);
            let edge2 = pos2.sub(pos0);

            // Δ координат UV
            let delta_u1 = uv1.x() - uv0.x();
            let delta_v1 = uv1.y() - uv0.y();
            let delta_u2 = uv2.x() - uv0.x();
            let delta_v2 = uv2.y() - uv0.y();

            // Вычисляем делитель (результат якобиана отображения)
            let dividend = delta_u1 * delta_v2 - delta_u2 * delta_v1;
            let f = if dividend == 0.0 { 0.0 } else { 1.0 / dividend };

            // Касательный вектор (в 3D, w = 0)
            let tangent = Vector4f::new(
                f * (delta_v2 * edge1.x - delta_v1 * edge2.x),
                f * (delta_v2 * edge1.y - delta_v1 * edge2.y),
                f * (delta_v2 * edge1.z - delta_v1 * edge2.z),
                0.0,
            );

            // Накапливаем касательные
            self.tangents[i0] = self.tangents[i0].add(tangent);
            self.tangents[i1] = self.tangents[i1].add(tangent);
            self.tangents[i2] = self.tangents[i2].add(tangent);
        }

        // Нормализуем все касательные
        for tangent in self.tangents.iter_mut() {
            *tangent = tangent.normalized();
        }
    }

    // Методы для добавления данных (необязательно, но удобно)
    pub fn add_position(&mut self, pos: Vector4f) {
        self.positions.push(pos);
        self.normals.push(Vector4f::new(0.0, 0.0, 0.0, 0.0));
        self.tangents.push(Vector4f::new(0.0, 0.0, 0.0, 0.0));
    }

    pub fn add_tex_coord(&mut self, tex: Vector4f) {
        self.tex_coords.push(tex);
    }

    pub fn add_index(&mut self, idx: u32) {
        self.indices.push(idx as usize);
    }

    pub fn add_normal(&mut self, normal: Vector4f) {
        self.normals.push(normal);
    }

    pub fn add_tangent(&mut self, tangent: Vector4f) {
        self.tangents.push(tangent);
    }
}
