use nalgebra::Vector3;
use crate::{World, generator::decorators::WorldDecorator};

use rand::{Rng, rngs::StdRng};

pub struct DecoratorMulti {
    decorator: Box<dyn WorldDecorator + Sync>,
    count: usize,
}

impl DecoratorMulti {
    pub fn new(decorator: Box<dyn WorldDecorator + Sync>, count: usize) -> Box<dyn WorldDecorator + Sync> {
        Box::new(DecoratorMulti {
            decorator,
            count,
        })
    }
}

impl WorldDecorator for DecoratorMulti {
    fn decorate(&self, world: &mut World, random: &mut StdRng, position: Vector3<i32>) {
        for _ in 0..self.count {
            let dx = position.x + random.gen_range(0, 16);
            let dz = position.z + random.gen_range(0, 16);
            let dy = world.highest_y(dx, dz) * 2;

            if dy > 0 {
                let dy = random.gen_range(0, dy);
                self.decorator.decorate(world, random, Vector3::new(dx, dy, dz));
            }
        }
    }
}
