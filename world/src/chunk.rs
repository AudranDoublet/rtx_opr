use std::{cmp::Ordering, fs::File, io::prelude::*, path::Path, rc::Rc};

use nalgebra::{Vector2, Vector3};

use crate::generator::decorators::decorator_random;
use crate::{main_world, BiomeType, Block, MAX_HEIGHT, SEA_LEVEL};

const WIDTH: i32 = 16;
const HEIGHT: i32 = MAX_HEIGHT;
const COUNT: i32 = WIDTH * WIDTH * HEIGHT;

pub struct Chunk {
    coords: Vector2<i32>,
    pub blocks: [Block; COUNT as usize],
    pub lightning: [f32; COUNT as usize],
    pub grass_color: [f32; (WIDTH * WIDTH * 3) as usize],
    biomes: [BiomeType; WIDTH as usize * WIDTH as usize],

    decorated: bool,
    modified: bool,
}

impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coords.data.cmp(&other.coords.data)
    }
}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.coords == other.coords
    }
}

impl Eq for Chunk {}

impl Chunk {
    pub fn new_empty(x: i32, z: i32) -> Rc<Chunk> {
        Rc::new(Chunk {
            coords: Vector2::new(x, z),
            blocks: [Block::Air; COUNT as usize],
            lightning: [0.0; COUNT as usize],
            decorated: false,
            grass_color: [0.0; (WIDTH * WIDTH * 3) as usize],
            biomes: [BiomeType::Ocean; WIDTH as usize * WIDTH as usize],
            modified: true,
        })
    }

    pub fn new_example_chunk(x: i32, z: i32) -> Rc<Chunk> {
        let mut chunk = Chunk::new_empty(x, z);
        let mut_chunk = unsafe { Rc::get_mut_unchecked(&mut chunk) };

        mut_chunk.decorated = true;

        for x in 0..WIDTH {
            for z in 0..WIDTH {
                for y in 0..SEA_LEVEL {
                    mut_chunk.set_block_at_chunk(x, y, z, Block::Grass);
                }
            }
        }

        chunk
    }

    pub fn biome_at(&self, x: i32, z: i32) -> &BiomeType {
        &self.biomes[(x + z * WIDTH) as usize]
    }

    pub fn biome_at_mut(&mut self, x: i32, z: i32) -> &mut BiomeType {
        &mut self.biomes[(x + z * WIDTH) as usize]
    }

    pub fn check_modified(&mut self) -> bool {
        if self.modified {
            self.modified = false;
            true
        } else {
            false
        }
    }

    pub fn highest_y(&self, x: i32, z: i32) -> i32 {
        let pos = self.position();
        let x = x - pos.x;
        let z = z - pos.y;

        for y in (0..256).rev() {
            if self.block_at_chunk(x, y, z) != Block::Air {
                return y;
            }
        }

        0
    }

    pub fn set_grass_color(&mut self, x: i32, z: i32, color: Vector3<f32>) {
        let pos = (x + z * WIDTH) as usize * 3;

        self.grass_color[pos + 0] = color.x / 255.;
        self.grass_color[pos + 1] = color.y / 255.;
        self.grass_color[pos + 2] = color.z / 255.;
    }

    pub fn block_at_chunk(&self, x: i32, y: i32, z: i32) -> Block {
        if y < 0 || y >= MAX_HEIGHT {
            Block::Air
        } else {
            self.blocks[(x + z * WIDTH + y * WIDTH * WIDTH) as usize]
        }
    }

    pub fn set_block_at_chunk(&mut self, x: i32, y: i32, z: i32, block: Block) {
        if y < 0 || y > MAX_HEIGHT {
            return;
        }

        self.modified = true;
        self.blocks[(x + z * WIDTH + y * WIDTH * WIDTH) as usize] = block
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) {
        let position = self.position();

        self.set_block_at_chunk(x - position.x, y, z - position.y, block)
    }

    pub fn block_at(&self, x: i32, y: i32, z: i32) -> Block {
        let position = self.position();

        self.block_at_chunk(x - position.x, y, z - position.y)
    }

    pub fn block_at_vec(&self, position: Vector3<i32>) -> Block {
        self.block_at(position.x, position.y, position.z)
    }

    pub fn coords(&self) -> Vector2<i32> {
        self.coords
    }

    pub fn position(&self) -> Vector2<i32> {
        Vector2::new(WIDTH * self.coords.x, WIDTH * self.coords.y)
    }

    pub fn chunk_filled_metadata(&self) -> [bool; 16] {
        let mut result = [false; 16];

        for vy in 0..16 {
            let base_y = vy * 16;

            'l: for y in 0..16 {
                for z in 0..16 {
                    for x in 0..16 {
                        if self.block_at_chunk(x, base_y + y, z) != Block::Air {
                            result[vy as usize] = true;
                            break 'l;
                        }
                    }
                }
            }
        }

        result
    }

    pub fn decorated(&self) -> bool {
        self.decorated
    }

    pub fn decorate(&mut self) {
        if self.decorated {
            return;
        }

        self.decorated = true;

        let biome = self.biome_at(0, 0);
        let world = main_world();
        let mut random = decorator_random(world, self.coords());

        for decorator in biome.decorators().unwrap_or(&vec![]) {
            let p = self.position();
            decorator.decorate(world, &mut random, Vector3::new(p.x, 0, p.y));
        }
    }

    pub fn dump_chunk_raw(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path.join(&Path::new(&format!(
            "{}_{}.ck",
            self.coords.x, self.coords.y
        ))))?;
        let mut blocks: [u8; COUNT as usize] = [0; COUNT as usize];

        for i in 0..COUNT as usize {
            blocks[i] = (self.blocks[i] as isize) as u8;
        }

        file.write_all(&blocks)?;

        Ok(())
    }
}

pub fn world_to_chunk(position: Vector3<i32>) -> (i32, i32) {
    (position.x >> 4, position.z >> 4)
}

pub fn worldf_to_chunk(position: Vector3<f32>) -> (i32, i32) {
    world_to_chunk(Vector3::new(
        position.x as i32,
        position.y as i32,
        position.z as i32,
    ))
}

pub fn ivec_to_f(p: Vector3<i32>) -> Vector3<f32> {
    Vector3::new(p.x as f32, p.y as f32, p.z as f32)
}
