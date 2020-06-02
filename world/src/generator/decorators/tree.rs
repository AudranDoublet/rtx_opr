use nalgebra::{Vector2, Vector3};
use crate::{Block, World, generator::decorators::WorldDecorator};

use crate::multi_dec;

use rand::{Rng, rngs::StdRng};

struct BuffBlock {
    x: i32,
    y: i32,
    z: i32,
    leaves: bool,
    block: Block,
}

impl BuffBlock {
    pub fn log(x: i32, y: i32, z: i32, tree_type: (Block, Block)) -> BuffBlock {
        BuffBlock {
            x,
            y,
            z,
            leaves: false,
            block: tree_type.0,
        }
    }

    pub fn leaves(x: i32, y: i32, z: i32, tree_type: (Block, Block)) -> BuffBlock {
        BuffBlock {
            x,
            y,
            z,
            leaves: true,
            block: tree_type.1,
        }
    }

    pub fn set(&self, world: &mut World, snow: bool) {
        let block_at = world.unsafe_block_at_coords(self.x, self.y, self.z);
        let block_at_up = world.unsafe_block_at_coords(self.x, self.y + 1, self.z);

        if snow && self.leaves && block_at_up == Block::Air {
            world.set_block_at_coords(self.x, self.y + 1, self.z, Block::Snow);
        }

        if !block_at.is_log() {
            world.set_block_at_coords(self.x, self.y, self.z, self.block);
        }
    }
}

pub enum ForestType {
    Classic,
    Normal,
    Taiga,
    Acacia,
    Jungle,
}

impl ForestType {
    pub fn tree_type(&self, random: &mut StdRng, width: i32) -> (Block, Block) {
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
                } else if rnd <= 0.95 {
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
                } else if rnd <= 0.95 {
                    (Block::OakLog, Block::OakLeaves)
                } else {
                    (Block::BigOakLog, Block::BigOakLeaves)
                }
            },
            ForestType::Acacia      => (Block::AcaciaLog, Block::AcaciaLeaves),
            ForestType::Jungle      => (Block::JungleLog, Block::JungleLeaves),
        }
    }
}

pub struct DecoratorTree {
    tree_type: ForestType,

    height: (i32, i32),
    width: (i32, i32),

    min_height_branch: i32,
    water_tree: bool,
    snow_tree: bool,

    accepted_supports: Vec<Block>,
}

impl DecoratorTree {
    pub fn jungle(count: usize) -> Box<dyn WorldDecorator + Sync> {
        multi_dec!(DecoratorTree {
            tree_type: ForestType::Jungle,
            height: (5, 17),
            width: (2, 4),
            min_height_branch: 0,
            water_tree: false,
            snow_tree: false,
            accepted_supports: vec![Block::Grass, Block::Dirt],

        }, count)
    }

    pub fn small(count: usize, tree_type: ForestType, water_tree: bool, snow_tree: bool) -> Box<dyn WorldDecorator + Sync> {
        multi_dec!(DecoratorTree {
            tree_type,
            height: (3, 6),
            width: (1, 2),
            min_height_branch: 7, // no branch
            water_tree,
            snow_tree,
            accepted_supports: vec![Block::Grass, Block::Dirt],
        }, count)
    }

    pub fn great(count: usize, tree_type: ForestType, water_tree: bool, snow_tree: bool) -> Box<dyn WorldDecorator + Sync> {
        multi_dec!(DecoratorTree {
            tree_type,
            height: (7, 13),
            width: (1, 2),
            min_height_branch: 17, // no branch
            water_tree,
            snow_tree,
            accepted_supports: vec![Block::Grass, Block::Dirt],
        }, count)
    }

    pub fn very_great(count: usize, tree_type: ForestType, water_tree: bool, snow_tree: bool) -> Box<dyn WorldDecorator + Sync> {
        multi_dec!(DecoratorTree {
            tree_type,
            height: (12, 20),
            width: (2, 4),
            min_height_branch: 17,
            water_tree,
            snow_tree,
            accepted_supports: vec![Block::Grass, Block::Dirt],
        }, count)
    }

    pub fn fat(count: usize, tree_type: ForestType, water_tree: bool, snow_tree: bool) -> Box<dyn WorldDecorator + Sync> {
        multi_dec!(DecoratorTree {
            tree_type,
            height: (5, 17),
            width: (2, 4),
            min_height_branch: 7,
            water_tree,
            snow_tree,
            accepted_supports: vec![Block::Grass, Block::Dirt],
        }, count)
    }

    fn is_support_accepted(&self, block: Block) -> bool {
        self.accepted_supports.iter().any(|b| *b == block)
    }

    fn support_tree(&self, block: Block) -> bool {
        if block == Block::Water && self.water_tree {
            return true;
        }

        !block.is_tough()
    }

    fn can_be_placed_here(&self, world: &mut World, x: i32, y: i32, z: i32) -> bool {
        for dx in -1..=1 {
            for dz in -1..=1 {
                let block = world.unsafe_block_at_coords(x + dx, y, z + dz);

                if block.is_log() {
                    return false;
                }

                if dx == 0 && dz == 0 && world.unsafe_block_at_coords(x + dx, y - 1, z + dz) == Block::Air {
                    world.set_block_at_coords(x + dx, y - 1, z + dz, self.accepted_supports[0]);
                }
            }
        }

        true
    }

    fn is_tree_block(&self, block: Block) -> bool {
        match block {
            Block::Grass | Block::Dirt | Block::Air | Block::Snow => true,
            Block::Water if self.water_tree => true,
            b if b.is_log() || b.is_leaves() => true,
            _ => false,
        }
    }

    fn generate_leaves(&self, world: &mut World, result: &mut Vec<BuffBlock>, random: &mut StdRng, x: i32, y: i32, z: i32, tree_type: (Block, Block)) {
        let mut width = 1;

        for dy in (-3..0).rev() {
            for dx in -width..=width {
                for dz in -width..=width {
                    if self.is_tree_block(world.unsafe_block_at_coords(x + dx, y + dy, z + dz)) {
                        if dx.abs() != width || dz.abs() != width || random.gen_range(0, 6) == 0 {
                            result.push(BuffBlock::leaves(x + dx, y + dy, z + dz, tree_type));
                        }
                    }
                }
            }

            width = (width + 1).min(2);
        }
    }

    fn generate_branch(&self, world: &mut World, random: &mut StdRng, tree_type: (Block, Block), mut pos: Vector3<i32>, diff: Vector2<i32>) -> Vec<BuffBlock> {
        let mut result: Vec<BuffBlock> = Vec::new();
        let mut generated = true;
        let mut count = 0;

        while random.gen_range(0, count.max(1)) <= 1 {
            pos.x += diff.x;
            pos.z += diff.x;

            if !self.is_tree_block(world.unsafe_block_at(pos)) {
                generated = false; 
                break;
            }

            result.push(BuffBlock::log(pos.x, pos.y, pos.z, tree_type));

            pos.y += 1;
            count += 1;
        }

        if !generated {
            result.clear();
        } else {
            self.generate_leaves(world, &mut result, random, pos.x, pos.y, pos.z, tree_type);
        }

        result
    }

    fn generate_tree(&self, world: &mut World, random: &mut StdRng, size: i32, height: i32, x: i32, y: i32, z: i32) -> Vec<BuffBlock> {
        let mut result: Vec<BuffBlock> = Vec::new();
        let mut branches = Vec::new();
        let mut generated = true;

        let tree_type  = self.tree_type.tree_type(random, size);

        'l: for dy in 0..=height {
            for dx in 0..size {
                for dz in 0..size {
                    let block = world.unsafe_block_at_coords(x + dx, y + dy, z + dz);

                    if (dy == 0 && !self.can_be_placed_here(world, x + dx, y, z + dz))
                        || !self.is_tree_block(block)
                        || (block == Block::Water && dy >= 1 && dy > height / 4)
                    {
                        generated = false;
                        break 'l;
                    }

                    result.push(BuffBlock::log(x + dx, y + dy, z + dz, tree_type));

                    if dy > self.min_height_branch && height > dy + 3 && random.gen_range(0, 10) == 0 {
                        let ndx = random.gen_range(0, 3) - 1;
                        let ndz = random.gen_range(0, 3) - 1;

                        if ndx != ndz && (ndx == 0 || ndz == 0) {
                            branches.push((Vector3::new(x + dx, y + dy, z + dz), Vector2::new(ndx, ndz)));
                        }
                    }
                }
            }
        }

        if !generated {
            result.clear();
        } else {
            for (pos, diff) in branches {
                result.extend(self.generate_branch(world, random, tree_type, pos, diff));
            }

            for dx in 0..=size {
                for dz in 0..=size {
                    self.generate_leaves(world, &mut result, random, x + dx, y + height + 1, z + dz, tree_type);
                }
            }
        }

        result

    }
}

impl WorldDecorator for DecoratorTree {
    fn decorate(&self, world: &mut World, random: &mut StdRng, position: Vector3<i32>) {
        for _ in 0..10 {
            let dx = position.x + random.gen_range(0, 8) - random.gen_range(0, 8);
            let dy = position.y + random.gen_range(0, 4) - random.gen_range(0, 4);
            let dz = position.z + random.gen_range(0, 8) - random.gen_range(0, 8);

            if dy < 0 {
                continue;
            }

            if self.support_tree(world.unsafe_block_at_coords(dx, dy, dz))
                && self.is_support_accepted(world.unsafe_block_at_coords(dx, dy - 1, dz))
            {
                let height = random.gen_range(self.height.0, self.height.1);
                let width = random.gen_range(self.width.0, self.width.1);

                for block in self.generate_tree(world, random, width, height, dx, dy, dz) {
                    block.set(world, self.snow_tree);
                }
            }
        }
    }
}
