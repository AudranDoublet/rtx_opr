extern crate serde_json;

use crate::{generator::ChunkGenerator, main_world, Chunk};

use std::fs::create_dir_all;

use std::{rc::Rc, sync::mpsc};
use std::path::{Path, PathBuf};

pub struct ChunkManager {
    generator: ChunkGenerator,
    path: PathBuf,
    flat: bool,
}

impl ChunkManager {
    pub fn new(world_path: &str, seed: isize, flat: bool, channel: mpsc::Receiver<(bool, i32, i32)>) {
        let mut manager = ChunkManager {
            generator: ChunkGenerator::new(seed),
            path: Path::new(world_path).to_path_buf(),
            flat,
        };

        create_dir_all(&manager.path).unwrap();

        while let Ok((is_load, x, z)) = channel.recv() {
            if is_load {
                manager.load_or_generate_chunk_and_neighborhood(x, z).unwrap()
            } else {
                manager.unload_chunk(x, z).unwrap()
            }
        }
    }

    pub fn chunk_file(&self, x: i32, z: i32) -> PathBuf {
        self.path.join(format!("{}_{}.ck", x, z))
    }

    pub fn unload_chunk(&self, x: i32, z: i32) -> Result<(), Box<dyn std::error::Error>> {
        main_world().remove_chunk(x, z);

        Ok(())
    }

    pub fn load_or_generate_chunk(&mut self, x: i32, z: i32) -> Result<(), Box<dyn std::error::Error>> {
        if main_world().chunk_loaded(x, z) {
            return Ok(());
        }

        let path = self.chunk_file(x, z);

        let chunk = if path.exists() {
            Chunk::new_from_file(x, z, &path)?
        } else if self.flat {
            self.generator.generate_xz_flat(x, z)
        } else {
            self.generator.generate_xz(x, z)
        };

        main_world().add_chunk(chunk);
        Ok(())
    }

    pub fn load_or_generate_chunk_and_neighborhood(&mut self, x: i32, z: i32) -> Result<(), Box<dyn std::error::Error>> {
        for dz in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }

                self.load_or_generate_chunk(x + dx, z + dz)?;
            }
        }

        self.load_or_generate_chunk(x, z)?;
        unsafe { Rc::get_mut_unchecked(&mut main_world().chunk_mut(x, z).unwrap()) }.decorate();

        Ok(())
    }
}
