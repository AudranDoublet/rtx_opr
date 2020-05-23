use crate::{generator::ColumnProvider, Chunk};
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
}
