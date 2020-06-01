use nalgebra::{Vector2, Vector3};

use std::{collections::HashMap, rc::Rc, sync::mpsc, thread};

use crate::{world_to_chunk, Block, Chunk, ChunkListener, ChunkManager, Player};

pub static mut WORLD: Option<Box<World>> = None;

pub fn create_main_world(seed: isize) -> &'static mut Box<World> {
    let (tx, rx) = mpsc::channel();

    unsafe {
        WORLD = Some(Box::new(World::new(tx, seed)));
    }

    thread::spawn(move || ChunkManager::new(seed, rx));

    main_world()
}

pub fn main_world() -> &'static mut Box<World> {
    unsafe { WORLD.as_mut().unwrap() }
}

pub struct World {
    chunks: HashMap<Vector2<i32>, Rc<Chunk>>,
    sender: mpsc::Sender<(bool, i32, i32)>,
    seed: isize,
}

impl World {
    pub fn new(sender: mpsc::Sender<(bool, i32, i32)>, seed: isize) -> World {
        World {
            chunks: HashMap::new(),
            seed,
            sender,
        }
    }

    pub fn seed(&self) -> isize {
        self.seed
    }

    pub fn get_ref_chunks(&self) -> &HashMap<Vector2<i32>, Rc<Chunk>> {
        &self.chunks
    }

    pub fn chunk_loaded(&self, x: i32, z: i32) -> bool {
        self.chunks.contains_key(&Vector2::new(x, z))
    }

    pub fn chunk(&self, x: i32, z: i32) -> Option<&Rc<Chunk>> {
        self.chunks.get(&Vector2::new(x, z))
    }

    pub fn chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut Rc<Chunk>> {
        self.chunks.get_mut(&Vector2::new(x, z))
    }

    pub fn chunk_at(&self, position: Vector3<i32>) -> Option<&Rc<Chunk>> {
        let (x, z) = world_to_chunk(position);

        self.chunk(x, z)
    }

    pub fn chunk_mut_at(&mut self, position: Vector3<i32>) -> Option<&mut Rc<Chunk>> {
        let (x, z) = world_to_chunk(position);

        self.chunk_mut(x, z)
    }

    pub fn add_chunk(&mut self, chunk: Rc<Chunk>) {
        self.chunks.insert(chunk.coords(), chunk);
    }

    pub fn remove_chunk(&mut self, x: i32, z: i32) {
        self.chunks.remove(&Vector2::new(x, z));
    }

    pub fn generate_chunk(&self, x: i32, z: i32) {
        if let Some(c) = self.chunk(x, z) {
            if c.decorated() {
                return;
            }
        }

        self.sender
            .send((true, x, z))
            .expect("error while sending load chunk request");
    }

    pub fn unload_chunk(&self, x: i32, z: i32) {
        self.sender
            .send((false, x, z))
            .expect("error while sending load chunk request");
    }

    pub fn highest_y(&self, x: i32, z: i32) -> i32 {
        if let Some(chunk) = self.chunk_at(Vector3::new(x, 0, z)) {
            chunk.highest_y(x, z)
        } else {
            0
        }
    }

    pub fn block_at(&self, position: Vector3<i32>) -> Option<Block> {
        if let Some(chunk) = self.chunk_at(position) {
            Some(chunk.block_at_vec(position))
        } else {
            None
        }
    }

    pub fn set_block_at(&mut self, position: Vector3<i32>, block: Block) {
        if let Some(chunk) = self.chunk_mut_at(position) {
            let chunk = unsafe { Rc::get_mut_unchecked(chunk) };
            chunk.set_block(position.x, position.y, position.z, block)
        }
    }

    pub fn set_block_at_coords(&mut self, x: i32, y: i32, z: i32, block: Block) {
        self.set_block_at(Vector3::new(x, y, z), block)
    }

    pub fn unsafe_block_at(&self, position: Vector3<i32>) -> Block {
        self.block_at(position).unwrap_or(Block::Air)
    }

    pub fn unsafe_block_at_coords(&self, x: i32, y: i32, z: i32) -> Block {
        self.unsafe_block_at(Vector3::new(x, y, z))
    }

    pub fn create_player(
        &mut self,
        listener: &mut dyn ChunkListener,
        view_distance: usize,
    ) -> Player {
        let mut player = Player::new(view_distance);
        player.set_position(self, listener, Vector3::new(0.0, 100.0, 0.0));

        player
    }
}
