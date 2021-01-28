use serde::{Deserialize, Serialize};

use crate::AABB;
use nalgebra::Vector3;

#[derive(Debug)]
#[repr(u32)]
pub enum BlockFace {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

impl BlockFace {
    pub fn faces() -> Box<dyn Iterator<Item=BlockFace>> {
        Box::new(
            (0..6).into_iter()
              .map(|i| unsafe { std::mem::transmute(i) })
        )
    }

    pub fn opposite(&self) -> BlockFace {
        match self {
            BlockFace::Up => BlockFace::Down,
            BlockFace::Down => BlockFace::Up,
            BlockFace::North => BlockFace::South,
            BlockFace::South => BlockFace::North,
            BlockFace::East => BlockFace::West,
            BlockFace::West => BlockFace::East,
        }
    }

    pub fn relative(&self) -> Vector3<i32> {
        match self {
            BlockFace::Up => Vector3::new(0, 1, 0),
            BlockFace::Down => Vector3::new(0, -1, 0),
            BlockFace::North => Vector3::new(0, 0, 1),
            BlockFace::South => Vector3::new(0, 0, -1),
            BlockFace::East => Vector3::new(1, 0, 0),
            BlockFace::West => Vector3::new(-1, 0, 0),
        }
    }

    pub fn coord(c: usize) -> BlockFace {
        match c {
            0 => BlockFace::East,
            1 => BlockFace::Up,
            _ => BlockFace::North,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum Block {
    Air,
    Water,
    Stone,
    Grass,
    Dirt,
    Gravel,

    Sand,
    Cactus,
    Snow,
    TallGrass,

    OakLog,
    AcaciaLog,
    BigOakLog,
    BirchLog,
    JungleLog,
    SpruceLog,
    OakLeaves,
    AcaciaLeaves,
    BigOakLeaves,
    BirchLeaves,
    JungleLeaves,
    SpruceLeaves,

    OrangeTulipe,
    PinkTulip,
    RedTulip,
    WhiteTulip,
    Dandelion,
    AzureBluet,
    OxeyeDaisy,
    BlueOrchid,
    Allium,
    Poppy,

    LightWhite,
    LightRed,
    LightGreen,
    LightBlue,
    LightYellow,
    LightCyan,

    OakPlanks,
    AcaciaPlanks,
    BigOakPlanks,
    BirchPlanks,
    JunglePlanks,
    SprucePlanks,

    Brick,
    StoneBricks,

    LightMagenta,
    Glass,
    GlassBlack,
    GlassBlue,
    GlassBrown,
    GlassCyan,
    GlassGray,
    GlassGreen,
    GlassLightBlue,
    GlassLime,
    GlassMagenta,
    GlassOrange,
    GlassPink,
    GlassPurple,
    GlassRed,
    GlassSilver,
    GlassWhite,
    GlassYellow,

    Mirror,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Block {
    pub fn from_id(i: u32) -> Block {
        unsafe { std::mem::transmute(i) }
    }

    pub fn get_light(t: u32) -> (bool, Block) {
        let v = match t {
            0 => Block::LightWhite,
            1 => Block::LightRed,
            2 => Block::LightGreen,
            3 => Block::LightBlue,
            4 => Block::LightYellow,
            5 => Block::LightCyan,
            _ => Block::LightMagenta,
        };

        if t >= 6 {
            (true, v)
        } else {
            (false, v)
        }
    }

    pub fn get_colored_glass(t: u32) -> (bool, Block) {
        let v = match t {
            0  => Block::GlassBlack,
            1  => Block::GlassBlue,
            2  => Block::GlassBrown,
            3  => Block::GlassCyan,
            4  => Block::GlassGray,
            5  => Block::GlassGreen,
            6  => Block::GlassLightBlue,
            7  => Block::GlassLime,
            8  => Block::GlassMagenta,
            9  => Block::GlassOrange,
            10 => Block::GlassPink,
            11 => Block::GlassPurple,
            12 => Block::GlassRed,
            13 => Block::GlassSilver,
            14 => Block::GlassWhite,
            _  => Block::GlassYellow,
        };

        if t >= 15 {
            (true, v)
        } else {
            (false, v)
        }
    }

    pub fn is_glass(&self) -> bool {
        match self {
            Block::Glass
           | Block::GlassBlack
           | Block::GlassBlue
           | Block::GlassBrown
           | Block::GlassCyan
           | Block::GlassGray
           | Block::GlassGreen
           | Block::GlassLightBlue
           | Block::GlassLime
           | Block::GlassMagenta
           | Block::GlassOrange
           | Block::GlassPink
           | Block::GlassPurple
           | Block::GlassRed
           | Block::GlassSilver
           | Block::GlassWhite
           | Block::GlassYellow => true,
           _ => false,
        }
    }

    pub fn is_tough(&self) -> bool {
        match self {
            Block::Air | Block::Snow => false,
            Block::TallGrass => false,
            _ => true,
        }
    }

    pub fn is_flower(&self) -> bool {
        match self {
            Block::OrangeTulipe
            | Block::PinkTulip
            | Block::RedTulip
            | Block::WhiteTulip
            | Block::Dandelion
            | Block::AzureBluet
            | Block::OxeyeDaisy
            | Block::BlueOrchid
            | Block::Allium
            | Block::Poppy => true,
            _ => false,
        }
    }

    pub fn is_log(&self) -> bool {
        match self {
            Block::OakLog
            | Block::AcaciaLog
            | Block::BigOakLog
            | Block::BirchLog
            | Block::JungleLog
            | Block::SpruceLog => true,
            _ => false,
        }
    }

    pub fn is_leaves(&self) -> bool {
        match self {
            Block::OakLeaves
            | Block::AcaciaLeaves
            | Block::BigOakLeaves
            | Block::BirchLeaves
            | Block::JungleLeaves
            | Block::SpruceLeaves => true,
            _ => false,
        }
    }

    pub fn is_liquid(&self) -> bool {
        match self {
            Block::Water => true,
            _ => false,
        }
    }

    pub fn is_opaque(&self) -> bool {
        match self {
            Block::Air | Block::Water | Block::Snow => false,
            Block::TallGrass | Block::Cactus => false,
            b if b.is_leaves() => false,
            b if b.is_flower() => false,
            b if b.is_glass() => false,
            _ => true,
        }
    }

    pub fn aabb(&self, position: Vector3<f32>) -> Option<AABB> {
        let base = match self {
            Block::Air | Block::Water | Block::TallGrass => None,
            _ if self.is_flower() => None,
            Block::Snow => Some(AABB::new(Vector3::zeros(), Vector3::new(1., 0.1, 1.))),
            _ => Some(AABB::new(Vector3::zeros(), Vector3::new(1., 1., 1.))),
        };

        if let Some(base) = base {
            Some(base.translate(position))
        } else {
            None
        }
    }
}
