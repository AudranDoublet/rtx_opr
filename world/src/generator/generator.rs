use crate::{Chunk, generator::ColumnProvider};

pub struct ChunkGenerator
{
    provider: ColumnProvider,
}

impl ChunkGenerator
{
    pub fn new() -> ChunkGenerator {
        ChunkGenerator {
            provider: ColumnProvider::new(),
        }
    }

    pub fn generate(&mut self, chunk: &mut Chunk) {
        self.provider.generate_chunk(chunk);
    }

    pub fn generate_xz(&mut self, x: i64, z: i64) -> Chunk {
        let mut result = Chunk::new_empty(x, z);

        self.generate(&mut result);
        result
    }
}
