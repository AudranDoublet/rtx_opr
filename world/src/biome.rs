extern crate rand;

use nalgebra::Vector3;

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
        map.insert(BiomeType::Savanna, vec![DecoratorPlantGroup::tallgrass(9), DecoratorTree::fat(20, ForestType::Acacia, false, false)]);
        map.insert(BiomeType::SavannaPlateau, vec![DecoratorPlantGroup::tallgrass(9), DecoratorTree::fat(20, ForestType::Acacia, false, false)]);

        map.insert(BiomeType::Forest, vec![DecoratorTree::small(10, ForestType::Normal, false, false), DecoratorTree::great(80, ForestType::Normal, false, false), DecoratorTree::fat(20, ForestType::Normal, false, false)]);
        map.insert(BiomeType::ForestHills, vec![DecoratorTree::small(10, ForestType::Normal, false, false), DecoratorTree::great(80, ForestType::Normal, false, false), DecoratorTree::fat(20, ForestType::Normal, false, false)]);
        map.insert(BiomeType::Taiga, vec![DecoratorTree::great(10, ForestType::Taiga, false, true), DecoratorTree::very_great(80, ForestType::Taiga, false, true), DecoratorTree::fat(20, ForestType::Taiga, false, true)]);
        map.insert(BiomeType::TaigaHills, vec![DecoratorTree::great(10, ForestType::Taiga, false, false), DecoratorTree::very_great(80, ForestType::Taiga, false, false), DecoratorTree::fat(20, ForestType::Taiga, false, false)]);

        map.insert(BiomeType::IceForest, vec![DecoratorTree::small(10, ForestType::Normal, false, true), DecoratorTree::great(80, ForestType::Normal, false, true), DecoratorTree::fat(20, ForestType::Normal, false, true)]);
        map.insert(BiomeType::IceForestHills, vec![DecoratorTree::small(10, ForestType::Normal, false, true), DecoratorTree::great(80, ForestType::Normal, false, true), DecoratorTree::fat(20, ForestType::Normal, false, true)]);
        map.insert(BiomeType::IceTaiga, vec![DecoratorTree::great(10, ForestType::Taiga, false, true), DecoratorTree::very_great(80, ForestType::Taiga, false, true), DecoratorTree::fat(20, ForestType::Taiga, false, true)]);
        map.insert(BiomeType::IceTaigaHills, vec![DecoratorTree::great(10, ForestType::Taiga, false, true), DecoratorTree::very_great(80, ForestType::Taiga, false, true), DecoratorTree::fat(20, ForestType::Taiga, false, true)]);

        map.insert(BiomeType::Jungle, vec![DecoratorTree::jungle(150), DecoratorTree::small(10, ForestType::Normal, false, false)]);
        map.insert(BiomeType::Forest, vec![DecoratorTree::small(20, ForestType::Normal, true, false), DecoratorTree::great(10, ForestType::Normal, true, false), DecoratorTree::fat(5, ForestType::Normal, true, false)]);

        // common decorators
        for (_, v) in map.iter_mut() {
            v.push(DecoratorPlantGroup::tallgrass(1));
            v.push(DecoratorPlantGroup::flowers(1));
            v.push(DecoratorTree::small(1, ForestType::Classic, false, false));
        }

        map
    };

    static ref GRASS_COLORS: [Vector3<f32>; 3] = [
        Vector3::new(191.0, 183.0,  85.0),
        Vector3::new(128.0, 180.0, 151.0),
        Vector3::new(71.0 , 205.0,  51.0),
    ];
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

    pub fn temperature(&self) -> f32 {
        match self {
            BiomeType::Ocean            => 0.5,
            BiomeType::DeepOcean        => 0.5,
            BiomeType::Plain            => 0.8,
            BiomeType::Hills            => 0.8,

            BiomeType::Desert           => 2.0,
            BiomeType::DesertHills      => 2.0,
            BiomeType::Savanna          => 1.2,
            BiomeType::SavannaPlateau   => 1.0,

            BiomeType::Jungle           => 0.95,
            BiomeType::JungleHills      => 0.95,
            BiomeType::Moutains         => 0.2,
            BiomeType::HighMoutains     => 0.1,

            BiomeType::Taiga            => 0.25,
            BiomeType::TaigaHills       => 0.25,

            BiomeType::Beach            => 0.8,
            BiomeType::Forest           => 0.7,
            BiomeType::ForestHills      => 0.7,
            BiomeType::Swampland        => 0.8,
            BiomeType::IceBeach         => 0.05,
            BiomeType::IcePlain         => 0.05,
            BiomeType::IceHills         => 0.05,
            BiomeType::IceForest        => 0.05,
            BiomeType::IceForestHills   => 0.05,
            BiomeType::IceMoutains      => -0.5,
            BiomeType::IceHighMoutains  => -0.6,
            BiomeType::IceTaiga         => -0.5,
            BiomeType::IceTaigaHills    => -0.5,

            BiomeType::River            => 0.5,
        }
    }

    pub fn rainfall(&self) -> f32 {
        match self {
            BiomeType::Ocean            => 0.5,
            BiomeType::DeepOcean        => 0.5,
            BiomeType::Plain            => 0.4,
            BiomeType::Hills            => 0.4,

            BiomeType::Desert           => 0.0,
            BiomeType::DesertHills      => 0.0,
            BiomeType::Savanna          => 0.0,
            BiomeType::SavannaPlateau   => 0.0,

            BiomeType::Jungle           => 0.9,
            BiomeType::JungleHills      => 0.9,
            BiomeType::Moutains         => 0.3,
            BiomeType::HighMoutains     => 0.3,

            BiomeType::Taiga            => 0.8,
            BiomeType::TaigaHills       => 0.8,

            BiomeType::Beach            => 0.4,
            BiomeType::Forest           => 0.8,
            BiomeType::ForestHills      => 0.8,
            BiomeType::Swampland        => 0.9,
            BiomeType::IceBeach         => 0.3,
            BiomeType::IcePlain         => 0.3,
            BiomeType::IceHills         => 0.3,
            BiomeType::IceForest        => 0.3,
            BiomeType::IceForestHills   => 0.3,
            BiomeType::IceMoutains      => 0.5,
            BiomeType::IceHighMoutains  => 0.5,
            BiomeType::IceTaiga         => -0.5,
            BiomeType::IceTaigaHills    => -0.5,

            BiomeType::River            => 0.5,
        }
    }

    pub fn grass_color(&self) -> Vector3<f32> {
        match self {
            BiomeType::Swampland => Vector3::new(106.0, 112.0, 57.0),
            biome => {
                let temperature = biome.temperature().clamp(0.0, 1.0);
                let rainfall = biome.rainfall().clamp(0.0, 1.0) * temperature;

                let color = (temperature - rainfall) * GRASS_COLORS[0]
                                + (1.0 - temperature) * GRASS_COLORS[1]
                                + (rainfall) * GRASS_COLORS[2];

                Vector3::new(
                    color.x.clamp(0., 255.),
                    color.y.clamp(0., 255.),
                    color.z.clamp(0., 255.),
                )
            }
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
