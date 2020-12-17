use std::collections::HashMap;
use crate::{World, Chunk, FaceProperties, BlockRenderer};
use nalgebra::{Vector2, Vector3};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 4],
    texture: [f32; 4],
}

fn add_vertice(v: Vector3<i32>, vertices: &mut Vec<[f32; 4]>, map: &mut HashMap<Vector3<i32>, u32>) -> u32 {
    let len = map.len() as u32;

     *map.entry(v)
         .or_insert_with(|| {
             vertices.push([
                 v.x as f32 / 10.,
                 v.y as f32 / 10.,
                 v.z as f32 / 10.,
                 0.,
             ]);

             len
         })
}

pub struct ChunkMesh {
    vertices_map: HashMap<Vector3<i32>, u32>,
    textures_map: HashMap<Vector3<i32>, u32>,

    vertices: Vec<[f32; 4]>,
    indices: Vec<u32>,

    texture_vertices: Vec<[f32; 4]>,
    texture_indices: Vec<u32>,

    triangle_materials: Vec<u32>,
    triangle_normals: Vec<[f32; 4]>,
}

impl ChunkMesh {
    pub fn new() -> ChunkMesh {
        ChunkMesh {
            vertices_map: HashMap::new(),
            textures_map: HashMap::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            texture_vertices: Vec::new(),
            texture_indices: Vec::new(),
            triangle_materials: Vec::new(),
            triangle_normals: Vec::new(),
        }
    }

    pub fn from_chunk(world: &World, chunk: &Chunk) -> ChunkMesh {
        let renderer = BlockRenderer::classic(FaceProperties {
            texture_id: 0,
            material_id: 0,
        });

        let (cx, cz) = {
            let cpos = chunk.position();
            (cpos.x, cpos.y)
        };

        let mut mesh = ChunkMesh::new();
        for y in 0..256 {
            for z in 0..16 {
                for x in 0..16 {
                    if chunk.block_at_chunk(x, y, z) == crate::Block::Air {
                        continue;
                    }
                    renderer.render(world, Vector3::new(x + cx, y, z + cz), &mut mesh);
                }
            }
        }

        mesh
    }

    pub fn add_triangle(
        &mut self,
        face_properties: &FaceProperties,
        v1: Vector3<i32>, v2: Vector3<i32>, v3: Vector3<i32>,
        t1: Vector2<i32>, t2: Vector2<i32>, t3: Vector2<i32>,
        normal: Vector3<i32>,
    ) {
        let normal_sum = normal.sum() as f32;
        let texture_id = face_properties.texture_id as i32;

        // add triangle vertices
        for v in &[v1, v2, v3] {
            self.indices.push(
                add_vertice(*v, &mut self.vertices, &mut self.vertices_map)
            );
        }

        // add triangle texture vertices
        for v in &[t1, t2, t3] {
            let v = Vector3::new(texture_id, v.x, v.y);

            self.texture_indices.push(
                add_vertice(v, &mut self.texture_vertices, &mut self.textures_map)
            );
        }

        // add triangle normal
        self.triangle_normals.push(
            [normal.x as f32 / normal_sum, normal.y as f32 / normal_sum, normal.z as f32 / normal_sum, 0.0],
        );

        // add triangle mateirla
        self.triangle_materials.push(
            face_properties.material_id,
        );
    }

    pub fn dump(&self) {
        for v in &self.vertices {
            println!("v {} {} {}", v[0], v[1], v[2]);
        }

        for i in (0..self.indices.len()).step_by(3) {
            println!("f {} {} {}", self.indices[i] + 1, self.indices[i + 1] + 1, self.indices[i + 2] + 1);
        }
    }
}
