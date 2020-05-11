use crate::generator::layers::{LayerData, LayerResult, LayerImpl, Layer};
use crate::BiomeType;

use std::sync::RwLock;

#[derive(Clone)]
pub struct LayerRiver {
}

impl LayerRiver {
    pub fn new() -> LayerRiver {
        LayerRiver { }
    }
}

#[inline]
fn scale(i: isize) -> isize {
    if i >= 2 {
        2 + (i & 1)
    } else {
        i
    }
}

impl LayerImpl for LayerRiver {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let a = scale(*parent.at(dx + 0, dy + 1));
            let b = scale(*parent.at(dx + 2, dy + 1));
            let c = scale(*parent.at(dx + 1, dy + 0));
            let d = scale(*parent.at(dx + 1, dy + 2));

            let s = scale(*parent.at(dx + 1, dy + 1));

            *v = if s == a && s == b && s == c && s == d {
               -1 
            } else {
                BiomeType::River as isize
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}

pub struct LayerRiverApply {
    scd_parent: RwLock<Box<Layer>>,
}

impl LayerRiverApply {
    pub fn new(other: Box<Layer>) -> LayerRiverApply {
        LayerRiverApply {
            scd_parent: RwLock::new(other),
        }
    }
}

fn is_ocean(i: isize) -> bool {
    BiomeType::from_id(i).is_ocean()
}

impl LayerImpl for LayerRiverApply {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x, y, result.width, result.height);
        let mut sentry = self.scd_parent.write().unwrap().generate(x, y, result.width, result.height);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            data.rand.init_local(dx + x, dy + y);

            let a = *parent.at(dx, dy);
            let b = *sentry.at(dx , dy);

            *v = if !is_ocean(a) && b == BiomeType::River as isize {
                b
            } else {
                a
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(LayerRiverApply::new(self.scd_parent.read().unwrap().clone()))
    }
}
