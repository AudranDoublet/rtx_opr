use nalgebra::{Vector2, Vector3};

use crate::{Block, BiomeType, SEA_LEVEL, MAX_HEIGHT};

const WIDTH: i64 = 16;
const HEIGHT: i64 = MAX_HEIGHT;
const COUNT: i64 = WIDTH * WIDTH * HEIGHT;

pub struct Chunk {
    coords: Vector2<i64>,
    blocks: [Block; COUNT as usize],
    biomes: [BiomeType; WIDTH as usize * WIDTH as usize],
    decorated: bool,
}

impl Chunk {
    pub fn new_empty(x: i64, z: i64) -> Chunk {
        Chunk {
            coords: Vector2::new(x, z),
            blocks: [Block::Air; COUNT as usize],
            decorated: false,
            biomes: [BiomeType::Ocean; WIDTH as usize * WIDTH as usize],
        }
    }

    pub fn new_example_chunk(x: i64, z: i64) -> Chunk {
        let mut chunk = Chunk::new_empty(x, z);

        chunk.decorated = true;

        for x in 0..WIDTH {
            for z in 0..WIDTH {
                for y in 0..SEA_LEVEL {
                    chunk.set_block_at_chunk(x, y, z, Block::Grass);
                }
            }
        }

        chunk
    }

    pub fn biome_at(&self, x: i64, z: i64) -> &BiomeType {
        &self.biomes[(x + z * WIDTH) as usize]
    }

    pub fn biome_at_mut(&mut self, x: i64, z: i64) -> &mut BiomeType {
        &mut self.biomes[(x + z * WIDTH) as usize]
    }

    pub fn block_at_chunk(&self, x: i64, y: i64, z: i64) -> &Block {
        &self.blocks[
            (x + z * WIDTH + y*WIDTH*WIDTH) as usize
        ]
    }

    pub fn set_block_at_chunk(&mut self, x: i64, y: i64, z: i64, block: Block) {
        self.blocks[(x + z * WIDTH + y*WIDTH*WIDTH) as usize] = block
    }

    pub fn block_at(&self, x: i64, y: i64, z: i64) -> &Block {
        let position = self.position();

        self.block_at_chunk(x - position.x, y, z - position.y)
    }

    pub fn block_at_vec(&self, position: Vector3<i64>) -> &Block {
        self.block_at(position.x, position.y, position.z)
    }

    pub fn coords(&self) -> Vector2<i64> {
        self.coords
    }

    pub fn position(&self) -> Vector2<i64> {
        Vector2::new(
            WIDTH * self.coords.x,
            WIDTH * self.coords.y,
        )
    }

    pub fn chunk_filled_metadata(&self) -> [bool; 16] {
        let mut result = [false; 16];

        for vy in 0..16 {
            let vy = vy * 16;

            'l: for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        if *self.block_at_chunk(x, vy + y, z) != Block::Air {
                            result[vy as usize] = true;
                            break 'l;
                        }
                    }
                }
            }
        }

        result
    }

    pub fn decorate(&mut self) {
        if self.decorated {
            return;
        }

        // FIXME
    }
}

pub fn world_to_chunk(position: Vector3<i64>) -> (i64, i64) {
    (position.x / WIDTH, position.z / WIDTH)
}

pub fn worldf_to_chunk(position: Vector3<f32>) -> (i64, i64) {
    world_to_chunk(Vector3::new(position.x as i64, position.y as i64, position.z as i64))
}

pub fn ivec_to_f(p: Vector3<i64>) -> Vector3<f32> {
    Vector3::new(
        p.x as f32,
        p.y as f32,
        p.z as f32,
    )
}
