use crate::{Chunk, Block};

const LIGHTNING_DIRECT: f32 = 0.2;

pub struct ChunkLightning<'a>
{
    current_chunks: [&'a mut Chunk; 4],
}

impl<'a> ChunkLightning<'a> {
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        Block::Air // FIXME
    }

    fn add_face_lightning(&self, x: i32, y: i32, z: i32, strength: f32) {

    }

    pub fn propagate_lightning(&self) {

    }

    pub fn compute_lightning(&self) {
        for x in 0..16 {
            for z in 0..16 {
                let mut energy: f32 = 1.0;

                for y in (0..256).rev() {
                    self.add_face_lightning(x, y + 1, z, LIGHTNING_DIRECT);

                    energy = energy * self.block_at(x, y, z).transparency();

                    if energy < 1e-3 {
                        break;
                    }

                    self.add_face_lightning(x, y, z, LIGHTNING_DIRECT);
                }
            }
        }
    }
}
