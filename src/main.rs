use nalgebra::Vector3;

use world::{World, ChunkListener};

pub struct MyChunkListener {
    pub loaded_chunks: Vec<(i64, i64)>,
    pub unloaded_chunks: Vec<(i64, i64)>,
}

impl MyChunkListener {
    pub fn new() -> MyChunkListener {
        MyChunkListener {
            loaded_chunks: Vec::new(),
            unloaded_chunks: Vec::new(),
        }
    }

    pub fn update_renderer(&mut self) {
        //FIXME
        self.clear();
    }

    pub fn clear(&mut self) {
        self.loaded_chunks.clear();
        self.unloaded_chunks.clear();
    }
}

impl ChunkListener for MyChunkListener {
    fn chunk_load(&mut self, x: i64, y: i64) {
        self.loaded_chunks.push((x, y));
    }

    fn chunk_unload(&mut self, x: i64, y: i64) {
        self.unloaded_chunks.push((x, y));
    }
}

fn main() {
    let mut listener = MyChunkListener::new();

    let mut world = World::new();
    let mut player = world.create_player(&mut listener);

    // FIXME main loop
    player.update(&mut world, &mut listener, Vector3::z(), Vector3::x(), Vec::new(), 0.1);
    listener.update_renderer();
}
