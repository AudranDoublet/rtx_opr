use crate::generator::layers::{LayerData, LayerResult, LayerImpl};

#[derive(Clone)]
pub struct LayerCleaner {
}

impl LayerCleaner {
    pub fn new() -> LayerCleaner {
        LayerCleaner {
        }
    }
}

impl LayerImpl for LayerCleaner {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x, y, result.width, result.height);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            data.rand.init_local(dx + x, dy + y);
            let s = *parent.at(dx, dy);

            *v = if s > 0 {
                2 + data.rand.next(299999)
            } else {
                0
            };
        })
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}
