use std::sync::mpsc;
use crate::{generator::ChunkGenerator, main_world};

pub struct ChunkManager {
    generator: ChunkGenerator,
}

impl ChunkManager {
    pub fn new(seed: isize, channel: mpsc::Receiver<(bool, i64, i64)>) {
        let mut manager = ChunkManager {
            generator: ChunkGenerator::new(seed),
        };

        while let Ok((is_load, x, z)) = channel.recv() {
            if is_load {
                manager.load_or_generate_chunk_and_neighborhood(x, z)
            } else {
                manager.unload_chunk(x, z)
            }
        }
    }

    pub fn unload_chunk(&self, x: i64, z: i64) {
        //FIXME save

        main_world().remove_chunk(x, z)
    }

    pub fn load_or_generate_chunk(&mut self, x: i64, z: i64) {
        //FIXME load if exists

        if main_world().chunk_loaded(x, z) {
            return;
        }

        let chunk = self.generator.generate_xz(x, z);

        main_world().add_chunk(Box::new(chunk));
    }

    pub fn load_or_generate_chunk_and_neighborhood(&mut self, x: i64, z: i64) {
        for dz in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }

                self.load_or_generate_chunk(x + dx, z + dz);
            }
        }

        self.load_or_generate_chunk(x, z);
        main_world().chunk_mut(x, z).unwrap().decorate();
    }
}
