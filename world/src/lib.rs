#[macro_use]
extern crate lazy_static;

mod aabb;
mod chunk;
mod block;
mod world;
mod player;
mod biome;
mod chunk_manager;

pub mod generator;

pub use aabb::*;
pub use chunk::*;
pub use block::*;
pub use world::*;
pub use player::*;
pub use biome::*;
pub use chunk_manager::*;

pub const SEA_LEVEL: i32 = 63;
pub const MAX_HEIGHT: i32 = 256;
