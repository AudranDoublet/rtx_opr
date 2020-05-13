#[macro_use]
extern crate clap;

use clap::App;

use std::sync::RwLock;

mod biome_generator;

use nalgebra::Vector3;

use world::{World, ChunkListener};

pub struct MyChunkListener {
    pub loaded_chunks: RwLock<Vec<(i64, i64)>>,
    pub unloaded_chunks: RwLock<Vec<(i64, i64)>>,
}

impl MyChunkListener {
    pub fn new() -> MyChunkListener {
        MyChunkListener {
            loaded_chunks: RwLock::new(Vec::new()),
            unloaded_chunks: RwLock::new(Vec::new()),
        }
    }

    pub fn update_renderer(&self, _: &World) {
       // for (x, z) in &self.loaded_chunks {
       //     world.chunk(*x, *z).unwrap().dump_chunk_raw(&std::path::Path::new("./"));
       // }

        //FIXME
        self.clear();
    }

    pub fn clear(&self) {
        self.loaded_chunks.write().unwrap().clear();
        self.unloaded_chunks.write().unwrap().clear();
    }
}

impl ChunkListener for MyChunkListener {
    fn chunk_load(&self, x: i64, y: i64) {
        self.loaded_chunks.write().unwrap().push((x, y));
    }

    fn chunk_unload(&self, x: i64, y: i64) {
        self.unloaded_chunks.write().unwrap().push((x, y));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conf = load_yaml!("cli.yaml");

    let matches = App::from_yaml(conf).get_matches();

    if let Some(args) = matches.subcommand_matches("play") {
        let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;

        if seed == 0 {
            //FIXME random seed ?
        }

        let listener = MyChunkListener::new();

        let mut world = World::new(seed, &listener);
        let mut player = world.create_player();

        // FIXME main loop
        player.update(&mut world, Vector3::z(), Vector3::x(), Vec::new(), 0.1);
        listener.update_renderer(&world);
    } else if let Some(args) = matches.subcommand_matches("render_chunks") {
        let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;
        biome_generator::generate_biome(seed)?;
    }

    Ok(())
}
