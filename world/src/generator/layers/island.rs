use crate::generator::layers::{LayerData, LayerResult, LayerImpl};

#[derive(Clone)]
pub struct LayerIsland {}

impl LayerIsland {
    pub fn new() -> LayerIsland {
        LayerIsland {}
    }
}

impl LayerImpl for LayerIsland {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        result.iter().for_each(|((dx, dy), v)| {
            data.rand.init_local(x + dx as isize, y + dy as isize);
            *v = data.rand.cond(10) as isize;
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct LayerAddIsland {}

impl LayerAddIsland {
    pub fn new() -> LayerAddIsland {
        LayerAddIsland {}
    }
}

impl LayerImpl for LayerAddIsland {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let neighbours = [
                *parent.at(dx + 0, dy + 0),
                *parent.at(dx + 2, dy + 0),
                *parent.at(dx + 0, dy + 2),
                *parent.at(dx + 2, dy + 2),
            ];

            let s = *parent.at(dx + 1, dy + 1);

            data.rand.init_local(x + dx as isize, y + dy as isize);

            *v = if s == 0 && neighbours.iter().any(|v| *v != 0) {
                let mut diff = 1;
                let mut res = 1;

                for &n in &neighbours {
                    if n != 0 && data.rand.cond(diff) {
                        diff += 1;
                        res = n;
                    }
                }

                if data.rand.cond(3) || res == 4 {
                    res
                } else {
                    0
                }
            } else if s > 0 && neighbours.iter().any(|v| *v == 0) && data.rand.cond(5) {
                if s == 4 {
                    4
                } else {
                    0
                }
            } else {
                s
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}
