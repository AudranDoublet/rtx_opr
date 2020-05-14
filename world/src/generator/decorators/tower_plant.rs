use nalgebra::Vector3;
use crate::{Block, World, generator::decorators::WorldDecorator};

use crate::multi_dec;

use rand::{Rng, rngs::StdRng};

pub struct DecoratorTowerPlant {
    block_type: Block,
    accepted_support: Block,
    max_height: usize,
    allow_surrounding_blocks: bool,
}

impl DecoratorTowerPlant {
    pub fn cactus(count: usize) -> Box<dyn WorldDecorator + Sync> {
        multi_dec!(DecoratorTowerPlant {
            block_type: Block::Cactus,
            accepted_support: Block::Sand,
            max_height: 3,
            allow_surrounding_blocks: false,
        }, count)
    }

    fn is_empty_arround(&self, world: &World, position: Vector3<i64>) -> bool {
        for dx in -1..=1 {
            for dz in -1..=1 {
                if world.unsafe_block_at(Vector3::new(position.x + dx, position.y, position.z + dz)) != Block::Air {
                    return false;
                }
            }
        }

        true
    }
}

impl WorldDecorator for DecoratorTowerPlant {
    fn decorate(&self, world: &mut World, random: &mut StdRng, position: Vector3<i64>) {
        for _ in 0..10 {
            let dx = position.x + random.gen_range(0, 8) - random.gen_range(0, 8);
            let dy = position.y + random.gen_range(0, 4) - random.gen_range(0, 4);
            let dz = position.z + random.gen_range(0, 8) - random.gen_range(0, 8);

            if world.unsafe_block_at_coords(dx, dy - 1, dz) != self.accepted_support {
                continue;
            }

            if world.unsafe_block_at_coords(dx, dy, dz) != Block::Air {
                continue;
            }

            let size = 1 + random.gen_range(0, self.max_height);

            if !self.allow_surrounding_blocks && !self.is_empty_arround(world, Vector3::new(dx, dy, dz)) {
                continue;
            }

            for vy in 0..size as i64 {
                world.set_block_at_coords(dx, dy + vy, dz, self.block_type);
            }
        }
    }
}
