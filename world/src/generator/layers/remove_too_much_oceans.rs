use crate::BiomeType;
use crate::generator::layers::{LayerData, LayerResult, LayerImpl};

#[derive(Clone)]
pub struct LayerTooMuchOceans {}

impl LayerTooMuchOceans {
    pub fn new() -> LayerTooMuchOceans {
        LayerTooMuchOceans {}
    }
}

impl LayerImpl for LayerTooMuchOceans {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let val = *parent.at(dx + 1, dy + 1);

            *v = if val == 0 {
                data.rand.init_local(x + dx, y + dy);

                let cond = *parent.at(dx + 1, dy + 0) == 0;
                let cond = cond && *parent.at(dx + 2, dy + 1) == 0;
                let cond = cond && *parent.at(dx + 0, dy + 1) == 0;
                let cond = cond && *parent.at(dx + 1, dy + 2) == 0;

                (cond && data.rand.cond(2)) as isize
            } else {
                val
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct LayerDeepOcean {}

impl LayerDeepOcean {
    pub fn new() -> LayerDeepOcean {
        LayerDeepOcean {}
    }
}

impl LayerImpl for LayerDeepOcean {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let s = *parent.at(dx + 1, dy + 1);

            *v = if s != 0 {
                s
            } else {
                let neighbours = [
                    *parent.at(dx + 1, dy + 0),
                    *parent.at(dx + 2, dy + 1),
                    *parent.at(dx + 0, dy + 1),
                    *parent.at(dx + 1, dy + 2),
                ];


                if neighbours.iter().filter(|v| **v == 0).count() > 3 {
                    BiomeType::DeepOcean as isize
                } else {
                    0
                }
            }
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}
