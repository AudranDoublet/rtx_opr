use crate::generator::layers::{LayerData, LayerResult, LayerImpl};

#[derive(Clone)]
pub struct LayerSmooth {}

impl LayerSmooth {
    pub fn new() -> LayerSmooth {
        LayerSmooth {}
    }
}

impl LayerImpl for LayerSmooth {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x - 1, y - 1, result.width + 2, result.height + 2);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let neighbours = [
                *parent.at(dx + 0, dy + 1),
                *parent.at(dx + 2, dy + 1),
                *parent.at(dx + 1, dy + 0),
                *parent.at(dx + 1, dy + 2),
            ];

            *v = if neighbours[0] == neighbours[1] && neighbours[2] == neighbours[3] {
                data.rand.init_local(x + dx, y + dy);

                if data.rand.cond(2) {
                    neighbours[0]
                } else {
                    neighbours[2]
                }
            } else if neighbours[0] == neighbours[1] {
                neighbours[0]
            } else if neighbours[2] == neighbours[3] {
                neighbours[2]
            } else {
                *parent.at(dx + 1, dy + 1)
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}
