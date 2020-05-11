use crate::generator::layers::{LayerData, LayerResult, LayerImpl, Layer};
use crate::{BiomeGroup, BiomeType};

use std::sync::RwLock;

#[derive(Clone)]
pub struct LayerBiomeGroup {}

impl LayerBiomeGroup {
    pub fn new() -> LayerBiomeGroup {
        LayerBiomeGroup {}
    }
}

impl LayerImpl for LayerBiomeGroup {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x , y, result.width, result.height);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let val = *parent.at(dx, dy);

            *v = if val == 0 {
                0
            } else {
                data.rand.init_local(x + dx, y + dy);
                data.rand.next(BiomeGroup::count()) + 1
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct LayerBiomeType {}

impl LayerBiomeType {
    pub fn new() -> LayerBiomeType {
        LayerBiomeType {}
    }
}

impl LayerImpl for LayerBiomeType {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x , y, result.width, result.height);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let val = *parent.at(dx, dy);

            *v = if val == 0 {
                0
            } else {
                data.rand.init_local(x + dx, y + dy);
                *data.rand.peek(&BiomeGroup::get(val - 1).biomes()) as isize
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct LayerBiomeEdge {}

impl LayerBiomeEdge {
    pub fn new() -> LayerBiomeEdge {
        LayerBiomeEdge {}
    }
}

fn is_ocean(i: isize) -> bool {
    BiomeType::from_id(i).is_ocean()
}

impl LayerImpl for LayerBiomeEdge {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let neighbours = [
                *parent.at(dx + 1, dy + 0),
                *parent.at(dx + 2, dy + 1),
                *parent.at(dx + 0, dy + 1),
                *parent.at(dx + 1, dy + 2),
            ];

            let s = *parent.at(dx + 1, dy + 1);

            *v = if neighbours.iter().any(|v| is_ocean(*v)) && !is_ocean(s) {
                BiomeType::Beach as isize
            } else  {
                s
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}

pub struct LayerBiomeHills {
    scd_parent: RwLock<Box<Layer>>,
}

impl LayerBiomeHills {
    pub fn new(other: Box<Layer>) -> LayerBiomeHills {
        LayerBiomeHills {
            scd_parent: RwLock::new(other),
        }
    }
}

impl LayerImpl for LayerBiomeHills {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);
        let mut sentry = self.scd_parent.write().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            data.rand.init_local(dx + x, dy + y);

            let a = *parent.at(dx + 1, dy + 1);
            let b = *sentry.at(dx + 1, dy + 1);

            *v = if !is_ocean(a) && (b - 2) % 7 == 0 {
                BiomeType::from_id(a).get_hills_version() as isize
            } else {
                a
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(LayerBiomeHills::new(self.scd_parent.read().unwrap().clone()))
    }
}
