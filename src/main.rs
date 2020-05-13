#[macro_use]
extern crate clap;

mod biome_generator;
mod dump;

use std::sync::RwLock;
use clap::App;
use nalgebra::Vector3;

use world::{create_main_world, World, ChunkListener};

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
        let view_distance = args.value_of("view-distance").unwrap_or("0").parse::<usize>()?;

        if seed == 0 {
            //FIXME random seed ?
        }

        let world = create_main_world(seed);
        let listener = MyChunkListener::new();

        let mut player = world.create_player(view_distance, &listener);

        // FIXME main loop
        player.update(world, Vector3::z(), Vector3::x(), Vec::new(), 0.1);
        listener.update_renderer(&world);

        loop{}
    } else if let Some(args) = matches.subcommand_matches("render_chunks") {
        let seed = args.value_of("seed").unwrap_or("0").parse::<isize>()?;
        biome_generator::generate_biome(seed)?;
    } else if let Some(args) = matches.subcommand_matches("dump") {
        dump::dump_map(args)?;
    }


    Ok(())
}
