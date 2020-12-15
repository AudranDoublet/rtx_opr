use ash::vk;
use std::path::Path;
use std::sync::Arc;

use crate::context::Context;

use crate::datatypes::BufferVariable;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 4],
    normal: [f32; 4],
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Mesh {
        let (models, _) = tobj::load_obj(path.as_ref()).expect("Failed to load obj model");

        // Compute model bounds
        let mut min_x = std::f32::MAX;
        let mut min_y = std::f32::MAX;
        let mut min_z = std::f32::MAX;

        let mut max_x = std::f32::MIN;
        let mut max_y = std::f32::MIN;
        let mut max_z = std::f32::MIN;

        for model in &models {
            let mesh = &model.mesh;

            for index in 0..mesh.positions.len() / 3 {
                let x = mesh.positions[index * 3];
                let y = mesh.positions[index * 3 + 1];
                let z = mesh.positions[index * 3 + 2];

                min_x = min_x.min(x);
                min_y = min_y.min(y);
                min_z = min_z.min(z);

                max_x = max_x.max(x);
                max_y = max_y.max(y);
                max_z = max_z.max(z);
            }
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let depth = max_z - min_z;

        let x_offset = min_x + (width * 0.5);
        let y_offset = min_y + (height * 0.5);
        let z_offset = min_z + (depth * 0.5);

        let biggest_dimension = width.max(height).max(depth);

        // Build model
        let mut vertices = Vec::new();
        let mut indices = Vec::<u32>::new();
        for model in &models {
            let index_offset = vertices.len() as u32;
            let mesh = &model.mesh;

            for index in 0..mesh.positions.len() / 3 {
                let x = (mesh.positions[index * 3] - x_offset) / biggest_dimension;
                let y = (mesh.positions[index * 3 + 1] - y_offset) / biggest_dimension + 60.0;
                let z = (mesh.positions[index * 3 + 2] - z_offset) / biggest_dimension;

                let nx = mesh.normals[index * 3];
                let ny = mesh.normals[index * 3 + 1];
                let nz = mesh.normals[index * 3 + 2];

                let v = Vertex {
                    position: [x, y, z, 0.0],
                    normal: [nx, ny, nz, 0.0],
                };

                vertices.push(v);
            }

            for index in &mesh.indices {
                indices.push(*index + index_offset);
            }
        }

        Mesh { vertices, indices }
    }

    pub fn vertices_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn indices_count(&self) -> usize {
        self.indices.len()
    }

    pub fn device_vertices(&self, context: &Arc<Context>) -> BufferVariable {
        BufferVariable::device_buffer(
            context,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
            &self.vertices,
        )
        .0
    }

    pub fn device_indices(&self, context: &Arc<Context>) -> BufferVariable {
        BufferVariable::device_buffer(
            context,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
            &self.indices,
        )
        .0
    }
}
