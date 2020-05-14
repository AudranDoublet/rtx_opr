use crate::{Chunk, generator::ColumnProvider};

pub struct ChunkGenerator
{
    provider: ColumnProvider,
}

impl ChunkGenerator
{
    pub fn new(seed: isize) -> ChunkGenerator {
        ChunkGenerator {
            provider: ColumnProvider::new(seed),
        }
    }

    pub fn generate(&mut self, chunk: &mut Box<Chunk>) {
        self.provider.generate_chunk(chunk);
    }

    pub fn generate_xz(&mut self, x: i64, z: i64) -> Box<Chunk> {
        let mut result = Chunk::new_empty(x, z);

        self.generate(&mut result);
        result
    }
}
