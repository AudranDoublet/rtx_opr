use crate::{generator::ColumnProvider, Chunk, Block};
use std::rc::Rc;

pub struct ChunkGenerator {
    provider: ColumnProvider,
}

impl ChunkGenerator {
    pub fn new(seed: isize) -> ChunkGenerator {
        ChunkGenerator {
            provider: ColumnProvider::new(seed),
        }
    }

    pub fn generate(&mut self, chunk: &mut Rc<Chunk>) {
        self.provider.generate_chunk(chunk);
    }

    pub fn generate_xz(&mut self, x: i32, z: i32) -> Rc<Chunk> {
        let mut result = Chunk::new_empty(x, z);

        self.generate(&mut result);
        result
    }

    pub fn generate_xz_flat(&mut self, x: i32, z: i32) -> Rc<Chunk> {
        let mut result = Chunk::new_empty(x, z);
        let chunk = unsafe { Rc::get_mut_unchecked(&mut result) };

        for x in 0..16 {
            for z in 0..16 {
                for y in 0..4 {
                    let t = if y == 3 {
                        Block::Grass
                    } else {
                        Block::Stone
                    };

                    chunk.set_block_at_chunk(x, y, z, t)
                }
            }
        }

        result
    }
}
