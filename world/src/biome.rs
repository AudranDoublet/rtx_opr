extern crate rand;

use rand::prelude::*;

use crate::{Chunk, Block, SEA_LEVEL};

pub enum BiomeShapeType {
    Low,
}

impl BiomeShapeType {
    pub fn elevation(&self) -> f32 {
        match self {
            BiomeShapeType::Low => 0.0,
        }
    }

    pub fn depth(&self) -> f32 {
        match self {
            BiomeShapeType::Low => 0.025,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(isize)]
pub enum BiomeType {
    Plain,
}

impl BiomeType {
    pub fn shape(&self) -> BiomeShapeType {
        match self {
            BiomeType::Plain => BiomeShapeType::Low,
        }
    }

    pub fn elevation(&self) -> f32 {
        self.shape().elevation()
    }

    pub fn depth(&self) -> f32 {
        self.shape().depth()
    }

    pub fn top_layer(&self) -> Block {
        Block::Air
    }

    pub fn top_block(&self) -> Block {
        Block::Grass
    }

    pub fn column_block(&self) -> Block {
        Block::Dirt
    }

    pub fn sub_column_block(&self) -> Option<Block> {
        None
    }

    pub fn generate_column(&self, chunk: &mut Chunk, x: i64, z: i64) {
        let top_column_height: i64 = (3. + rand::thread_rng().gen::<f32>() * 0.25) as i64;

        let mut top = -1;
        let mut first = true;

        let mut column_type = self.column_block();

        for y in (1..256).rev() {
            if *chunk.block_at_chunk(x, y, z) == Block::Air {
                top = -1;
                continue;
            }

            if *chunk.block_at_chunk(x, y, z) == Block::Stone{
                continue;
            }

            if top == -1 {
                if first && y > 63 {
                    first = false;
                    chunk.set_block_at_chunk(x, y + 1, z, self.top_layer());
                }

                top = top_column_height;

                let block_type = if y >= SEA_LEVEL - 1 {
                    self.top_block()
                } else if y < SEA_LEVEL - 7 - top_column_height {
                    Block::Gravel
                } else {
                    column_type
                };

                chunk.set_block_at_chunk(x, y, z, block_type);
            } else if top > 0 {
                chunk.set_block_at_chunk(x, y, z, column_type);
                top -= 1;

                if top > 0 {
                    continue;
                }

                if let Some(b) = self.sub_column_block() {
                    top = rand::thread_rng().gen_range(0, 4) + (y - SEA_LEVEL).max(0);
                    column_type = b;
                }
            }
        }
    }
}

pub enum BiomeGroup {
    Classic,
}

impl BiomeGroup {
    pub fn count() -> isize {
        1
    }

    pub fn get(i: isize) -> BiomeGroup {
        match i {
            1 => BiomeGroup::Classic,
            _ => BiomeGroup::Classic,
        }
    }

    pub fn biomes(&self) -> Vec<BiomeType> {
        match self {
            BiomeGroup::Classic => vec![ BiomeType::Plain ],
        }
    }
}
