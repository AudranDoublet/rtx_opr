use std::collections::HashMap;
use crate::{World, Chunk, FaceProperties, main_world};
use nalgebra::{Vector2, Vector3};

use std::sync::mpsc;
use std::thread;

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

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TriangleData {
    pub material: u32,
    pub normal: [f32; 4],
}

pub struct ChunkMesh {
    // only for build
    vertices_map: HashMap<Vector3<i32>, u32>,
    textures_map: HashMap<Vector3<i32>, u32>,

    pub vertices: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub texture_indices: Vec<u32>,

    pub texture_vertices: Vec<[f32; 4]>,
    pub triangle_data: Vec<TriangleData>,
}

impl ChunkMesh {
    pub fn new() -> ChunkMesh {
        ChunkMesh {
            vertices_map: HashMap::new(),
            textures_map: HashMap::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            texture_indices: Vec::new(),
            texture_vertices: Vec::new(),
            triangle_data: Vec::new(),
        }
    }

    pub fn from_chunk(world: &World, chunk: &Chunk) -> ChunkMesh {
        let (cx, cz) = {
            let cpos = chunk.position();
            (cpos.x, cpos.y)
        };

        let mut mesh = ChunkMesh::new();
        for y in 0..256 {
            for z in 0..16 {
                for x in 0..16 {
                    world.renderers[chunk.block_at_chunk(x, y, z) as usize]
                         .render(world, Vector3::new(x + cx, y, z + cz), &mut mesh);
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
        for (v, t) in &[(v1, t1), (v2, t2), (v3, t3)] {
            let t = Vector3::new(texture_id, t.x, t.y);

            self.indices.push(
                add_vertice(*v, &mut self.vertices, &mut self.vertices_map),
            );
            self.texture_indices.push(
                add_vertice(t, &mut self.texture_vertices, &mut self.textures_map),
            );
        }

        // add triangle data
        self.triangle_data.push(TriangleData {
            normal: [normal.x as f32 / normal_sum, normal.y as f32 / normal_sum, normal.z as f32 / normal_sum, 0.0],
            material: face_properties.material_id,
        });
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

pub struct ChunkMesher {
    request: mpsc::Receiver<(i32, i32)>,
    callback: mpsc::Sender<(i32, i32, ChunkMesh)>,
}

impl ChunkMesher {
    pub fn run(&self) {
        while let Ok((x, z)) = self.request.recv() {
            let world = main_world();
            if let Some(chunk) = world.chunk(x, z) {
                let mesh = ChunkMesh::from_chunk(world, chunk);
                self.callback.send((x, z, mesh)).expect("can't send meshing response");
            }
        }
    }
}

pub struct ChunkMesherClient {
    request: mpsc::Sender<(i32, i32)>,
    receiver: mpsc::Receiver<(i32, i32, ChunkMesh)>,
}

impl ChunkMesherClient {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();

        thread::spawn(move || {
            ChunkMesher {
                request: request_receiver,
                callback: response_sender,
            }.run()
        });

        ChunkMesherClient {
            request: request_sender,
            receiver: response_receiver,
        }
    }

    pub fn pull(&self) -> Option<(i32, i32, ChunkMesh)> {
        match self.receiver.try_recv() {
            Ok(v) => Some(v),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(e) => Err(e).expect("can't pull chunk mesh"),
        }
    }

    pub fn request(&self, x: i32, y: i32) {
        self.request.send((x, y)).expect("can't send meshing request")
    }
}
