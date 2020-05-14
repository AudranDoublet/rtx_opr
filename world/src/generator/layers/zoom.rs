use crate::generator::layers::{LayerData, LayerImpl, LayerResult};

#[derive(Clone)]
pub struct LayerZoom {
    fuzzy: bool,
}

impl LayerZoom {
    pub fn new() -> LayerZoom {
        LayerZoom { fuzzy: false }
    }

    pub fn new_fuzzy() -> LayerZoom {
        LayerZoom { fuzzy: true }
    }

    fn select(&self, data: &mut LayerData, a: isize, b: isize, c: isize, d: isize) -> isize {
        let mut random = || *data.rand.peek(&[a, b, c, d]);

        if self.fuzzy {
            random()
        } else if a == b {
            if c != d || a == c {
                a
            } else {
                random()
            }
        } else if a == c && b != d {
            a
        } else if a == d && b != c {
            a
        } else if (b == c && a != d) || b == d {
            b
        } else {
            random()
        }
    }
}

impl LayerImpl for LayerZoom {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult) {
        let px = x >> 1;
        let py = y >> 1;
        let mut parent = data.parent.as_mut().unwrap().generate(px, py, (result.width >> 1) + 2, (result.height >> 1) + 2);

        let vx = x & 1;
        let vy = y & 1;

        for line in 0..parent.height as isize - 1 {
            let mut a = *parent.at(0, line);
            let mut b = *parent.at(0, line + 1);

            let y = line * 2 - vy;

            for column in 0..parent.width as isize - 1 {
                let x = column * 2 - vx;

                data.rand.init_local((column + px) << 1, (line + py) << 1);

                let c = *parent.at(column + 1, line);
                let d = *parent.at(column + 1, line + 1);

                result.safe_set(x + 0, y + 0, a);
                result.safe_set(x + 0, y + 1, *data.rand.peek(&[a, b]));
                result.safe_set(x + 1, y + 0, *data.rand.peek(&[a, c]));
                result.safe_set(x + 1, y + 1, self.select(data, a, b, c, d));

                a = c;
                b = d;
            }
        }
    }

    fn clone_layer(&self) -> Box<dyn LayerImpl> {
        Box::new(self.clone())
    }
}
