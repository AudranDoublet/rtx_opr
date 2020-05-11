use crate::generator::layers::{LayerData, LayerResult, LayerImpl};
use crate::BiomeGroup;

pub struct LayerBiomeGroup {}

impl LayerImpl for LayerBiomeGroup {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x , y, result.width, result.height);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let val = *parent.at(dx + 1, dy + 1);

            *v = if val == 0 {
                0
            } else {
                data.rand.init_local(x + dx, y + dy);
                data.rand.next(BiomeGroup::count()) + 1
            };
        })
    }
}

pub struct LayerBiomeType {}

impl LayerImpl for LayerBiomeType {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let mut parent = data.parent.as_mut().unwrap().generate(x , y, result.width, result.height);

        result.iter().for_each(|((dx, dy), v)| {
            let dx = dx as isize;
            let dy = dy as isize;

            let val = *parent.at(dx + 1, dy + 1);

            *v = if val == 0 {
                0
            } else {
                data.rand.init_local(x + dx, y + dy);
                *data.rand.peek(&BiomeGroup::get(val - 1).biomes()) as isize
            };
        })
    }
}
