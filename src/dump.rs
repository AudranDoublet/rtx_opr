use clap::ArgMatches;

use nalgebra::Vector3;
use std::sync::RwLock;

use std::collections::HashSet;
use std::path::Path;

use world::{create_main_world, ChunkListener, World};

pub struct DumpChunkListener {
    pub loaded_chunks: RwLock<Vec<(i64, i64)>>,
    pub unloaded_chunks: RwLock<Vec<(i64, i64)>>,
}

impl DumpChunkListener {
    pub fn new() -> DumpChunkListener {
        DumpChunkListener {
            loaded_chunks: RwLock::new(Vec::new()),
            unloaded_chunks: RwLock::new(Vec::new()),
        }
    }

    pub fn update_renderer(
        &self,
        world: &World,
        path: &Path,
        known: &mut HashSet<(i64, i64)>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        for (x, z) in &*self.loaded_chunks.read().unwrap() {
            known.insert((*x, *z));
            world.chunk(*x, *z).unwrap().dump_chunk_raw(path)?;
        }

        self.clear();
        Ok(known.len())
    }

    pub fn clear(&self) {
        self.loaded_chunks.write().unwrap().clear();
        self.unloaded_chunks.write().unwrap().clear();
    }
}

impl ChunkListener for DumpChunkListener {
    fn chunk_load(&self, x: i64, y: i64) {
        self.loaded_chunks.write().unwrap().push((x, y));
    }

    fn chunk_unload(&self, x: i64, y: i64) {
        self.unloaded_chunks.write().unwrap().push((x, y));
    }
}

pub fn dump_map(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;
    let folder = Path::new(args.value_of("folder").unwrap_or("./map_dump"));
    let view_distance = args
        .value_of("view-distance")
        .unwrap_or("0")
        .parse::<usize>()?;

    std::fs::create_dir_all(&folder)?;

    let world = create_main_world(seed);
    let listener = DumpChunkListener::new();

    let mut player = world.create_player(&listener, view_distance);
    let mut known = HashSet::new();

    let max = (2 * view_distance).pow(2);

    while listener.update_renderer(&world, &folder, &mut known)? < max {
        player.update(
            world,
            &listener,
            Vector3::z(),
            Vector3::x(),
            Vec::new(),
            0.1,
        );
    }

    Ok(())
}
