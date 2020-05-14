mod island;
mod zoom;
mod zoom_voronoi;
mod remove_too_much_oceans;
mod biome;
mod smooth;
mod cleaner;
mod river;

pub use island::*;
pub use zoom::*;
pub use zoom_voronoi::*;
pub use remove_too_much_oceans::*;
pub use biome::*;
pub use smooth::*;
pub use cleaner::*;
pub use river::*;

use crate::generator::SimpleRandom;
use crate::BiomeType;

#[derive(Clone)]
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
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            *self.at(x, y) = value;
        }
    }

    pub fn biome(&self, x: isize, z: isize) -> BiomeType {
        BiomeType::from_id(self.data[x as usize + z as usize * self.width])
    }

    pub fn iter(&mut self) -> impl Iterator<Item=((usize, usize), &mut isize)>{
        let width = self.width;
        self.data.iter_mut().enumerate().map(move |(i, v)| ((i % width, i / width), v))
    }
}

pub trait LayerImpl {
    fn generate(&self, data: &mut LayerData, x: isize, y: isize, result: &mut LayerResult);

    fn clone_layer(&self) -> Box<dyn LayerImpl>;
}

impl Clone for Box<dyn LayerImpl> {
    fn clone(&self) -> Box<dyn LayerImpl> {
        self.as_ref().clone_layer()
    }
}

#[derive(Clone)]
pub struct Layer {
    pub data: LayerData,
    pub layer: Box<dyn LayerImpl>,
}

impl Layer {
    pub fn create_generator(world_seed: isize) -> (Box<Layer>, Box<Layer>) {
        let generator = |seed| SimpleRandom::new(seed, world_seed);
        let layer = |parent, seed, layer: Box<dyn LayerImpl>| Some(Box::new(Layer {
            data: LayerData {
                parent,
                rand: generator(seed),
            },
            layer,
        }));

        let mut lay = layer(None, 101, Box::new(LayerIsland::new()));

        lay = layer(lay, 38127, Box::new(LayerZoom::new_fuzzy()));
        lay = layer(lay, 3919, Box::new(LayerAddIsland::new()));
        lay = layer(lay, 38127, Box::new(LayerZoom::new()));

        lay = layer(lay, 39319, Box::new(LayerAddIsland::new()));
        lay = layer(lay, 399, Box::new(LayerAddIsland::new()));
        lay = layer(lay, 63119, Box::new(LayerAddIsland::new()));

        lay = layer(lay, 3821, Box::new(LayerTooMuchOceans::new()));
        
        lay = layer(lay, 381, Box::new(LayerBiomeGroup::new()));
        lay = layer(lay, 38127, Box::new(LayerZoom::new()));

        let mut alter = layer(lay.clone(), 9272, Box::new(LayerCleaner::new()));
        alter = layer(alter, 812, Box::new(LayerZoom::new()));
        alter = layer(alter, 898, Box::new(LayerZoom::new()));

        lay = layer(lay, 38138, Box::new(LayerBiomeType::new()));
        lay = layer(lay, 382, Box::new(LayerDeepOcean::new()));


        lay = layer(lay, 38131, Box::new(LayerZoom::new()));

        lay = layer(lay, 38131, Box::new(LayerZoom::new()));
        lay = layer(lay, 9833, Box::new(LayerAddIsland::new()));

        lay = layer(lay, 3881, Box::new(LayerSmooth::new()));

        let mut river = None;

        for i in 0..4
        {
            lay = layer(lay, i * 37, Box::new(LayerZoom::new()));

            if i == 0
            {
                lay = layer(lay, i * 3913, Box::new(LayerAddIsland::new()));
                lay = layer(lay, 38133, Box::new(LayerBiomeHills::new(alter.clone().unwrap())));
            }

            if i == 1
            {
                river = layer(lay.clone(), 9282, Box::new(LayerRiver::new()));
                river = layer(river, 3882, Box::new(LayerSmooth::new()));

                lay = layer(lay, 28138, Box::new(LayerBiomeEdge::new()));
            }
        }

        lay = layer(lay, 3981, Box::new(LayerSmooth::new()));
        lay = layer(lay, 29282, Box::new(LayerRiverApply::new(river.unwrap())));

        let mut zoomed = layer(lay.clone(), 9128, Box::new(LayerZoomVoronoi::new()));

        for _ in 0..5
        {
            zoomed = layer(zoomed, 3981 + 37, Box::new(LayerSmooth::new()));
            zoomed = layer(zoomed, 3981 + 37 * 2, Box::new(LayerSmooth::new()));
        }

        (lay.unwrap(), zoomed.unwrap())
    }

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
