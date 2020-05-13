use nalgebra::{Vector2, Vector3};

use std::collections::HashMap;
use crate::{Player, Chunk, Block, world_to_chunk, generator::ChunkGenerator};

pub trait ChunkListener {
    /**
     * Called when a chunk is loaded or modified
     */
    fn chunk_load(&self, x: i64, y: i64);

    /**
     * Called when a chunk is unloaded
     */
    fn chunk_unload(&self, x: i64, y: i64);
}

pub struct World<'a> {
    generator: ChunkGenerator,
    chunks: HashMap<Vector2<i64>, Chunk>,
    chunk_listener: &'a dyn ChunkListener,
}

impl<'a> World<'a> {
    pub fn new(seed: isize, listener: &'a dyn ChunkListener) -> World<'a> {
        World {
            generator: ChunkGenerator::new(seed),
            chunks: HashMap::new(),
            chunk_listener: listener,
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

        self.chunk_listener.chunk_load(x, z);
    }

    pub fn unload_chunk(&mut self, x: i64, z: i64) {
        self.chunks.remove(&Vector2::new(x, z));
        self.chunk_listener.chunk_unload(x, z);
    }

    pub fn block_at(&self, position: Vector3<i64>) -> Option<Block> {
        if let Some(chunk) = self.chunk_at(position) {
            Some(chunk.block_at_vec(position))
        } else {
            None
        }
    }

    pub fn create_player(&mut self) -> Player {
        let mut player = Player::new();
        player.set_position(self, Vector3::new(0.0, 100.0, 0.0));

        player
    }
}
