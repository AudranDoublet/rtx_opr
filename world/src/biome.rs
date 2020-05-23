extern crate rand;

use rand::prelude::*;
use std::collections::HashMap;

use crate::generator::decorators::*;
use crate::{Block, Chunk, SEA_LEVEL};

lazy_static! {
    static ref DECORATORS: HashMap<BiomeType, Vec<Box<dyn WorldDecorator + Sync>>> = {
        let mut map = HashMap::new();

        map.insert(BiomeType::Desert, vec![DecoratorTowerPlant::cactus(30)]);
        map.insert(BiomeType::DesertHills, vec![DecoratorTowerPlant::cactus(30)]);
        map.insert(BiomeType::Plain, vec![DecoratorPlantGroup::tallgrass(3)]);
        map.insert(BiomeType::Hills, vec![DecoratorPlantGroup::tallgrass(3)]);
        map.insert(BiomeType::Savanna, vec![DecoratorPlantGroup::tallgrass(9)]);
        map.insert(BiomeType::SavannaPlateau, vec![DecoratorPlantGroup::tallgrass(9)]);

        // common decorators
        for (_, v) in map.iter_mut() {
            v.push(DecoratorPlantGroup::tallgrass(1));
        }

        map
    };
}

pub enum BiomeShapeType {
    DeepVeryLow,
    DeepLow,
    DeepMedium,
    DeepHigh,
    Flat,
    VeryLow,
    Low,
    Medium,
    HillsLow,
    HillsMedium,
    HillsHigh,
    Plateau,
}

impl BiomeShapeType {
    pub fn elevation(&self) -> f32 {
        match self {
            BiomeShapeType::DeepVeryLow => -0.2,
            BiomeShapeType::DeepLow => -0.5,
            BiomeShapeType::DeepMedium => -1.0,
            BiomeShapeType::DeepHigh => -1.8,
            BiomeShapeType::Flat => 0.0,
            BiomeShapeType::VeryLow => 0.1,
            BiomeShapeType::Low => 0.125,
            BiomeShapeType::Medium => 0.2,
            BiomeShapeType::HillsLow => 0.45,
            BiomeShapeType::HillsMedium => 1.0,
            BiomeShapeType::HillsHigh => 1.2,
            BiomeShapeType::Plateau => 1.5,
        }
    }

    pub fn depth(&self) -> f32 {
        match self {
            BiomeShapeType::DeepVeryLow => 0.1,
            BiomeShapeType::DeepLow => 0.0,
            BiomeShapeType::DeepMedium => 0.1,
            BiomeShapeType::DeepHigh => 0.1,
            BiomeShapeType::Flat => 0.025,
            BiomeShapeType::VeryLow => 0.2,
            BiomeShapeType::Low => 0.05,
            BiomeShapeType::Medium => 0.2,
            BiomeShapeType::HillsLow => 0.3,
            BiomeShapeType::HillsMedium => 0.5,
            BiomeShapeType::HillsHigh => 0.55,
            BiomeShapeType::Plateau => 0.025,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(isize)]
pub enum BiomeType {
    Ocean = 0,
    DeepOcean,

    // classifcal biomes
    Beach,
    Plain,
    Hills,
    Forest,
    ForestHills,
    Swampland,

    Jungle,
    JungleHills,
    Moutains,
    HighMoutains,

    Taiga,
    TaigaHills,

    // warms biomes
    Desert,
    DesertHills,
    Savanna,
    SavannaPlateau,

    // ice biomes
    IceBeach,
    IcePlain,
    IceHills,
    IceForest,
    IceForestHills,
    IceMoutains,
    IceHighMoutains,
    IceTaiga,
    IceTaigaHills,

    River,
}

impl BiomeType {
    pub fn from_id(i: isize) -> BiomeType {
        unsafe { std::mem::transmute(i) }
    }

    pub fn shape(&self) -> BiomeShapeType {
        match self {
            BiomeType::Ocean => BiomeShapeType::DeepMedium,
            BiomeType::DeepOcean => BiomeShapeType::DeepHigh,
            BiomeType::Plain => BiomeShapeType::VeryLow,
            BiomeType::Hills => BiomeShapeType::HillsLow,

            BiomeType::Desert => BiomeShapeType::Low,
            BiomeType::DesertHills => BiomeShapeType::HillsLow,
            BiomeType::Savanna => BiomeShapeType::Low,
            BiomeType::SavannaPlateau => BiomeShapeType::Plateau,

            BiomeType::Jungle => BiomeShapeType::VeryLow,
            BiomeType::JungleHills => BiomeShapeType::HillsLow,
            BiomeType::Moutains => BiomeShapeType::HillsMedium,
            BiomeType::HighMoutains => BiomeShapeType::HillsHigh,

            BiomeType::Taiga => BiomeShapeType::Medium,
            BiomeType::TaigaHills => BiomeShapeType::HillsLow,

            BiomeType::Beach => BiomeShapeType::Flat,
            BiomeType::Forest => BiomeShapeType::VeryLow,
            BiomeType::ForestHills => BiomeShapeType::HillsLow,
            BiomeType::Swampland => BiomeShapeType::DeepVeryLow,
            BiomeType::IceBeach => BiomeShapeType::Flat,
            BiomeType::IcePlain => BiomeShapeType::Medium,
            BiomeType::IceHills => BiomeShapeType::HillsLow,
            BiomeType::IceForest => BiomeShapeType::VeryLow,
            BiomeType::IceForestHills => BiomeShapeType::HillsLow,
            BiomeType::IceMoutains => BiomeShapeType::HillsMedium,
            BiomeType::IceHighMoutains => BiomeShapeType::HillsHigh,
            BiomeType::IceTaiga => BiomeShapeType::Medium,
            BiomeType::IceTaigaHills => BiomeShapeType::HillsLow,

            BiomeType::River => BiomeShapeType::DeepLow,
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            BiomeType::Ocean => (0, 119, 190),
            BiomeType::DeepOcean => (0, 71, 114),
            BiomeType::Plain => (119, 190, 0),

            BiomeType::Desert => (244, 164, 96),
            BiomeType::DesertHills => (164, 84, 16),
            BiomeType::Savanna => (236, 213, 64),
            BiomeType::SavannaPlateau => (136, 113, 0),

            BiomeType::Jungle => (0, 255, 0),
            BiomeType::JungleHills => (0, 203, 0),
            BiomeType::Moutains => (255, 255, 0),
            BiomeType::HighMoutains => (203, 203, 0),

            BiomeType::Taiga => (0, 255, 255),
            BiomeType::TaigaHills => (0, 203, 203),

            BiomeType::Beach => (194, 178, 128),
            BiomeType::Hills => (104, 68, 48),
            BiomeType::Forest => (138, 138, 138),
            BiomeType::ForestHills => (78, 78, 78),
            BiomeType::Swampland => (208, 108, 108),
            BiomeType::IceBeach => (224, 208, 158),
            BiomeType::IcePlain => (149, 208, 138),
            BiomeType::IceHills => (79, 138, 68),
            BiomeType::IceForest => (208, 208, 208),
            BiomeType::IceForestHills => (158, 158, 158),
            BiomeType::IceMoutains => (208, 153, 32),
            BiomeType::IceHighMoutains => (158, 103, 0),
            BiomeType::IceTaiga => (38, 143, 192),
            BiomeType::IceTaigaHills => (0, 73, 132),

            BiomeType::River => (0, 0, 255),
        }
    }

    pub fn get_hills_version(&self) -> BiomeType {
        match self {
            BiomeType::Plain => BiomeType::Hills,

            BiomeType::Desert => BiomeType::DesertHills,
            BiomeType::Savanna => BiomeType::SavannaPlateau,

            BiomeType::Jungle => BiomeType::JungleHills,
            BiomeType::Moutains => BiomeType::HighMoutains,

            BiomeType::Taiga => BiomeType::TaigaHills,

            BiomeType::Forest => BiomeType::ForestHills,
            BiomeType::IcePlain => BiomeType::IceHills,
            BiomeType::IceForest => BiomeType::IceForestHills,
            BiomeType::IceMoutains => BiomeType::IceHighMoutains,
            BiomeType::IceTaiga => BiomeType::IceTaigaHills,

            v => *v,
        }
    }

    pub fn is_ocean(&self) -> bool {
        match self {
            BiomeType::Ocean => true,
            BiomeType::DeepOcean => true,
            _ => false,
        }
    }

    pub fn elevation(&self) -> f32 {
        self.shape().elevation()
    }

    pub fn depth(&self) -> f32 {
        self.shape().depth()
    }

    pub fn top_layer(&self) -> Block {
        match self {
            BiomeType::IceTaiga
            | BiomeType::IceTaigaHills
            | BiomeType::IceHighMoutains
            | BiomeType::IceMoutains
            | BiomeType::IceForest
            | BiomeType::IceHills
            | BiomeType::IcePlain
            | BiomeType::IceBeach => Block::Snow,
            _ => Block::Air,
        }
    }

    pub fn top_block(&self) -> Block {
        match self {
            BiomeType::Desert | BiomeType::DesertHills => Block::Sand,
            BiomeType::Beach | BiomeType::IceBeach => Block::Sand,
            _ => Block::Grass,
        }
    }

    pub fn column_block(&self) -> Block {
        match self {
            BiomeType::Desert | BiomeType::DesertHills => Block::Sand,
            BiomeType::Beach | BiomeType::IceBeach => Block::Sand,
            _ => Block::Dirt,
        }
    }

    pub fn sub_column_block(&self) -> Option<Block> {
        None
    }

    pub fn decorators(&self) -> Option<&'static Vec<Box<dyn WorldDecorator + Sync>>> {
        DECORATORS.get(self)
    }

    pub fn generate_column(&self, chunk: &mut Chunk, x: i32, z: i32) {
        let top_column_height: i32 = (3. + rand::thread_rng().gen::<f32>() * 0.25) as i32;

        let mut top = -1;
        let mut first = true;

        let mut column_type = self.column_block();

        for y in (1..256).rev() {
            if chunk.block_at_chunk(x, y, z) == Block::Air {
                top = -1;
                continue;
            }

            if chunk.block_at_chunk(x, y, z) != Block::Stone {
                continue;
            }

            if top == -1 {
                if first && y > 63 {
                    first = false;
                    chunk.set_block_at_chunk(x, y + 1, z, self.top_layer());
                }

                top = top_column_height;

                let block_type = if y >= SEA_LEVEL - 1 {
                    self.top_block()
                } else if y < SEA_LEVEL - 7 - top_column_height {
                    Block::Gravel
                } else {
                    column_type
                };

                chunk.set_block_at_chunk(x, y, z, block_type);
            } else if top > 0 {
                chunk.set_block_at_chunk(x, y, z, column_type);
                top -= 1;

                if top > 0 {
                    continue;
                }

                if let Some(b) = self.sub_column_block() {
                    top = rand::thread_rng().gen_range(0, 4) + (y - SEA_LEVEL).max(0);
                    column_type = b;
                }
            }
        }
    }
}

#[repr(isize)]
pub enum BiomeGroup {
    Warm = 0,
    Temperate,
    Cold,
    Iced,
}

impl BiomeGroup {
    pub fn count() -> isize {
        4
    }

    pub fn get(i: isize) -> BiomeGroup {
        unsafe { std::mem::transmute(i) }
    }

    pub fn biomes(&self) -> Vec<BiomeType> {
        //FIXME roofed & birch forests ?
        match self {
            BiomeGroup::Warm => vec![
                BiomeType::Desert,
                BiomeType::Desert,
                BiomeType::Savanna,
                BiomeType::Plain,
            ],
            BiomeGroup::Temperate => vec![
                BiomeType::Jungle,
                BiomeType::Forest,
                BiomeType::Forest,
                BiomeType::Moutains,
                BiomeType::Plain,
                BiomeType::Forest,
                BiomeType::Swampland,
            ],
            BiomeGroup::Cold => vec![
                BiomeType::Forest,
                BiomeType::Moutains,
                BiomeType::Taiga,
                BiomeType::Plain,
            ],
            BiomeGroup::Iced => vec![
                BiomeType::IcePlain,
                BiomeType::IceForest,
                BiomeType::IceTaiga,
                BiomeType::IceMoutains,
            ],
        }
    }
}
