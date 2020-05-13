use nalgebra::{Vector2, Vector3};

use std::collections::HashMap;
use crate::{Player, ChunkListener, Chunk, Block, world_to_chunk, generator::ChunkGenerator};

pub struct World {
    generator: ChunkGenerator,
    chunks: HashMap<Vector2<i64>, Chunk>,
}

impl World {
    pub fn new(seed: isize) -> World {
        World {
            generator: ChunkGenerator::new(seed),
            chunks: HashMap::new(),
        }
    }

    pub fn chunk(&self, x: i64, z: i64) -> Option<&Chunk> {
        self.chunks.get(&Vector2::new(x, z))
    }

    pub fn chunk_at(&self, position: Vector3<i64>) -> Option<&Chunk> {
        let (x, z) = world_to_chunk(position);

        self.chunk(x, z)
    }

    pub fn generate_chunk(&mut self, x: i64, z: i64) {
        let generator = &mut self.generator;

        self.chunks.entry(Vector2::new(x, z))
                   .or_insert_with(|| generator.generate_xz(x, z))
                   .decorate();
    }

    pub fn unload_chunk(&mut self, x: i64, z: i64) {
        self.chunks.remove(&Vector2::new(x, z));
    }

    pub fn block_at(&self, position: Vector3<i64>) -> Option<Block> {
        if let Some(chunk) = self.chunk_at(position) {
            Some(chunk.block_at_vec(position))
        } else {
            None
        }
    }

    pub fn create_player(&mut self, listener: &mut dyn ChunkListener) -> Player {
        let mut player = Player::new();
        player.set_position(self, listener, Vector3::new(0.0, 100.0, 0.0));

        player
    }
}
