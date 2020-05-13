use nalgebra::{Vector2, Vector3};

use std::thread;
use std::sync::mpsc;

use dashmap::DashMap;

use crate::{Player, ChunkListener, Chunk, Block, world_to_chunk, ChunkManager};

pub static mut WORLD: Option<Box<World>> = None;

pub fn create_main_world(seed: isize) -> &'static mut Box<World> {
    let (tx, rx) = mpsc::channel();

    unsafe {
        WORLD = Some(Box::new(World::new(tx)));
    }

    thread::spawn(move || {
        ChunkManager::new(seed, rx)
    });

    main_world()
}

pub fn main_world() -> &'static mut Box<World> {
    unsafe {
        WORLD.as_mut().unwrap()
    }
}

pub struct World {
    chunks: DashMap<Vector2<i64>, Box<Chunk>>,
    sender: mpsc::Sender<(bool, i64, i64)>,
}

impl World {
    pub fn new(sender: mpsc::Sender<(bool, i64, i64)>) -> World {
        World {
            chunks: DashMap::new(),
            sender,
        }
    }

    pub fn chunk_loaded(&self, x: i64, z: i64) -> bool {
        self.chunks.contains_key(&Vector2::new(x, z))
    }

    pub fn chunk(&self, x: i64, z: i64) -> Option<dashmap::mapref::one::Ref<Vector2<i64>, Box<Chunk>>> {
        self.chunks.get(&Vector2::new(x, z))
    }

    pub fn chunk_mut(&self, x: i64, z: i64) -> Option<dashmap::mapref::one::RefMut<Vector2<i64>, Box<Chunk>>> {
        self.chunks.get_mut(&Vector2::new(x, z))
    }

    pub fn chunk_at(&self, position: Vector3<i64>) -> Option<dashmap::mapref::one::Ref<Vector2<i64>, Box<Chunk>>> {
        let (x, z) = world_to_chunk(position);

        self.chunk(x, z)
    }

    pub fn add_chunk(&mut self, chunk: Box<Chunk>) {
        self.chunks.insert(chunk.coords(), chunk);
    }

    pub fn remove_chunk(&mut self, x: i64, z: i64) {
        self.chunks.remove(&Vector2::new(x, z));
    }

    pub fn generate_chunk(&self, x: i64, z: i64) {
        self.sender.send((true, x, z)).expect("error while sending load chunk request");
    }

    pub fn unload_chunk(&self, x: i64, z: i64) {
        self.sender.send((false, x, z)).expect("error while sending load chunk request");
    }

    pub fn block_at(&self, position: Vector3<i64>) -> Option<Block> {
        if let Some(chunk) = self.chunk_at(position) {
            Some(chunk.block_at_vec(position))
        } else {
            None
        }
    }

    pub fn create_player<'a>(&self, view_distance: usize, listener: &'a dyn ChunkListener) -> Player<'a> {
        let mut player = Player::new(view_distance, listener);
        player.set_position(self, Vector3::new(0.0, 100.0, 0.0));

        player
    }
}
