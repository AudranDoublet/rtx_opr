use crate::generator::layers::{LayerData, LayerResult, LayerImpl};

pub struct LayerTooMuchOceans {}

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
}
