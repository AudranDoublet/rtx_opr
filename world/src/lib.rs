#![feature(get_mut_unchecked)]
#[macro_use]
extern crate lazy_static;

mod aabb;
mod biome;
mod block;
mod chunk;
mod chunk_manager;
mod player;
mod world;

pub mod generator;

pub use aabb::*;
pub use biome::*;
pub use block::*;
pub use chunk::*;
pub use chunk_manager::*;
pub use player::*;
pub use world::*;

pub const SEA_LEVEL: i32 = 63;
pub const MAX_HEIGHT: i32 = 256;
