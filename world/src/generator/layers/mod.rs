pub mod island;
pub mod zoom;
pub mod zoom_voronoi;
pub mod remove_too_much_oceans;
pub mod biome;

use crate::generator::SimpleRandom;

pub struct LayerData {
    parent: Option<Box<Layer>>,
    rand: SimpleRandom,
}

pub struct LayerResult {
    pub width: usize,
    pub height: usize,
    pub data: Vec<isize>,
}

impl LayerResult {
    pub fn new(width: usize, height: usize) -> LayerResult {
        LayerResult {
            width,
            height,
            data: vec![0; width * height],
        }
    }

    pub fn at(&mut self, x: isize, y: isize) -> &mut isize {
        &mut self.data[x as usize + y as usize * self.width]
    }

    pub fn safe_set(&mut self, x: isize, y: isize, value: isize) {
        if x >= 0 && x <= self.width as isize && y >= 0 && y <= self.height as isize {
            *self.at(x, y) = value;
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item=((usize, usize), &mut isize)>{
        let width = self.width;
        self.data.iter_mut().enumerate().map(move |(i, v)| ((i % width, i / width), v))
    }
}

pub trait LayerImpl {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult);
}

pub struct Layer {
    pub data: LayerData,
    pub layer: Box<dyn LayerImpl>,
}

impl Layer {
    pub fn init_seed(&mut self, seed: isize) {
        if let Some(parent) = &mut self.data.parent {
            parent.init_seed(seed);
        }

        self.data.rand.init_world(seed);
    }

    pub fn generate(&mut self, x: isize, y: isize, width: usize, height: usize) -> LayerResult {
        let mut layer = LayerResult::new(width, height);
        self.layer.generate(&mut self.data, x, y, &mut layer);

        layer
    }
}
