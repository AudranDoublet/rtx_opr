use crate::generator::layers::{LayerData, LayerResult, LayerImpl};

pub struct LayerIsland {}

impl LayerImpl for LayerIsland {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        result.iter().for_each(|((dx, dy), v)| {
            data.rand.init_local(x + dx as isize, y + dy as isize);
            *v = data.rand.cond(10) as isize;
        })
    }
}
