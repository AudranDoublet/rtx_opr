use nalgebra::Vector3;
use crate::{Block, World, generator::decorators::WorldDecorator};

use crate::multi_dec;

use rand::{Rng, rngs::StdRng};

pub enum ForestType {
    Classic,
    Normal,
    Taiga,
    Acacia,
    Jungle,
}

impl ForestType {
    pub fn tree_type(&self, random: &mut StdRng, with: usize, height: usize) -> (Block, Block) {
        match self {
            ForestType::Classic     => (Block::OakLog, Block::OakLeaves),
            ForestType::Normal      => {
                let rnd = random.gen::<f32>();

                if rnd <= 0.9 {
                    (Block::OakLog, Block::OakLeaves)
                } else if width >= 2 {
                    if rnd <= 0.82 {
                        (Block::BirchLog, Block::BirchLeaves)
                    } else {
                        (Block::BigOakLog, Block::BigOakLeaves)
                    }
                } else if v <= 0.95 {
                    (Block::BirchLog, Block::BirchLeaves)
                } else {
                    (Block::BigOakLog, Block::BigOakLeaves)
                }
            },
            ForestType::Taiga       => {
                let rnd = random.gen::<f32>();

                if rnd <= 0.9 {
                    (Block::SpruceLog, Block::SpruceLeaves)
                } else if width >= 2 {
                    if rnd <= 0.92 {
                        (Block::OakLog, Block::OakLeaves)
                    } else {
                        (Block::BigOakLog, Block::BigOakLeaves)
                    }
                } else if v <= 0.95 {
                    (Block::OakLog, Block::OakLeaves)
                } else {
                    (Block::BigOakLog, Block::BigOakLeaves)
                }
            },
            ForestType::Aciacia     => (Block::AcaciaLog, Block::AcaciaLeaves),
            ForestType::Jungle      => (Block::JungleLog, Block::JungleLeaves),
        }
    }
}

pub struct DecoratorTree {
    block_type: Vec<Vec<Block>>,
    count: usize,
    accepted_supports: Vec<Block>,
}

impl DecoratorTree {
    pub fn tallgrass(count: usize) -> Box<dyn WorldDecorator + Sync> {
        multi_dec!(DecoratorTree {
            block_type: vec![vec![Block::TallGrass]],
            accepted_supports: vec![Block::Grass, Block::Dirt],
            count: 128,
        }, count)
    }

    fn is_support_accepted(&self, block: Block) -> bool {
        self.accepted_supports.iter().any(|b| *b == block)
    }
}

impl WorldDecorator for DecoratorTree {
    fn decorate(&self, world: &mut World, random: &mut StdRng, position: Vector3<i64>) {
        let mut y = position.y;

        while y > 0 && world.unsafe_block_at_coords(position.x, y, position.z) == Block::Air {
            y -= 1;
        }

        let types = &self.block_type[random.gen_range(0, self.block_type.len())];

        for _ in 0..self.count {
            let dx = position.x + random.gen_range(0, 8) - random.gen_range(0, 8);
            let dy = y          + random.gen_range(0, 4) - random.gen_range(0, 4);
            let dz = position.z + random.gen_range(0, 8) - random.gen_range(0, 8);

            if !world.unsafe_block_at_coords(dx, dy, dz).is_tough() && self.is_support_accepted(world.unsafe_block_at_coords(dx, dy - 1, dz)) {
                world.set_block_at_coords(dx, dy, dz, types[random.gen_range(0, types.len())]);
            }
        }
    }
}
