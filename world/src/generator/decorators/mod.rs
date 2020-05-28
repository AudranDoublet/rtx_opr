use nalgebra::{Vector2, Vector3};
use crate::World;

use rand::{SeedableRng, rngs::StdRng};

mod multi;
mod tower_plant;
mod plant_group;
mod tree;

pub use multi::*;

#[macro_export]
macro_rules! multi_dec {
    ($code:expr, $count:expr) => ({
        use crate::generator::decorators::DecoratorMulti;
        DecoratorMulti::new(Box::new($code), $count)
    })
}

pub use tower_plant::*;
pub use plant_group::*;
pub use tree::*;

pub trait WorldDecorator {
    fn decorate(&self, world: &mut World, random: &mut StdRng, position: Vector3<i32>);
}

pub fn decorator_random(world: &World, position: Vector2<i32>) -> StdRng {
    let seed = (world.seed() as i32 | position.x | position.y) + position.x + position.y;

    SeedableRng::seed_from_u64(seed as u64)
}
