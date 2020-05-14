use crate::generator::layers::{LayerData, LayerImpl, LayerResult};

#[derive(Clone)]
pub struct LayerZoomVoronoi {
}

#[inline]
fn rand(data: &mut LayerData, ma: f32, mb: f32) -> (f32, f32) {
    let a = (data.rand.next_float(1024) - 0.5) * 3.6;
    let b = (data.rand.next_float(1024) - 0.5) * 3.6;

    (a + ma, b + mb)
}

#[inline]
fn sq_dist(a: (isize, isize), b: (f32, f32)) -> f32 {
    (a.0 as f32 - b.0).powi(2) + (a.1 as f32 - b.1).powi(2)
}

#[inline]
fn smallest(p: (isize, isize), values: [isize; 4], keys: [(f32, f32); 4]) -> isize {
    let mut smallest = sq_dist(p, keys[0]);
    let mut id = 0;

    for i in 1..4 {
        let v = sq_dist(p, keys[i]);

        if v < smallest {
            smallest = v;
            id = i;
        }
    }

    values[id]
}

impl LayerZoomVoronoi {
    pub fn new() -> LayerZoomVoronoi {
        LayerZoomVoronoi {}
    }
}

impl LayerImpl for LayerZoomVoronoi {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let x = x - 2;
        let y = y - 2;

        let px = x >> 2;
        let py = y >> 2;
        let mut parent = data.parent.as_mut().unwrap().generate(px, py, (result.width >> 2) + 2, (result.height >> 2) + 2);

        let vx = x & 3;
        let vy = y & 3;

        let mut parent_values = [0; 4];
        let mut rand_values = [(0.0, 0.0); 4];

        for line in 0..parent.height as isize - 1 {
            parent_values[0] = *parent.at(0, line);
            parent_values[1] = *parent.at(0, line + 1);

            let y = line * 4 - vy;

            for column in 0..parent.width as isize - 1 {
                let x = column * 4 - vx;

                data.rand.init_local((column + px) << 2, (line + py) << 2);
                rand_values[0] = rand(data, 0.0, 0.0);

                data.rand.init_local((column + px) << 2, (line + py + 1) << 2);
                rand_values[1] = rand(data, 0.0, 4.0);

                data.rand.init_local((column + px + 1) << 2, (line + py) << 2);
                rand_values[2] = rand(data, 4.0, 0.0);

                data.rand.init_local((column + px + 1) << 2, (line + py + 1) << 2);
                rand_values[3] = rand(data, 4.0, 4.0);

                parent_values[2] = *parent.at(column + 1, line);
                parent_values[3] = *parent.at(column + 1, line + 1);

                for dy in 0..4 {
                    for dx in 0..4 {
                        result.safe_set(x + dx, y + dy, smallest((dx, dy), parent_values, rand_values));
                    }
                }

                parent_values[0] = parent_values[2];
                parent_values[1] = parent_values[3];
            }
        }
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}
