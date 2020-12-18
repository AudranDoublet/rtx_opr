use serde::{Deserialize, Serialize};

use crate::AABB;
use crate::{BlockRenderer, FaceProperties, classic_renderer, topdown_renderer};
use nalgebra::Vector3;

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
}

impl Block {
    pub fn from_id(i: u32) -> Block {
        unsafe { std::mem::transmute(i) }
    }

    pub fn get_nb_lights() -> u32 {
        6
    }

    pub fn get_light(t: u32) -> Block {
        match t {
            0 => Block::LightWhite,
            1 => Block::LightRed,
            2 => Block::LightGreen,
            3 => Block::LightBlue,
            4 => Block::LightYellow,
            _ => Block::LightCyan,
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
            b if b.is_leaves() => false,
            b if b.is_flower() => false,
            _ => true,
        }
    }

    pub fn block_renderer(&self) -> BlockRenderer {
        match self {
            Block::Air              => BlockRenderer::Empty,
            Block::Water            => BlockRenderer::Empty, //FIXME
            Block::Stone            => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Grass            => topdown_renderer!(
                FaceProperties::new(1, 0),
                FaceProperties::new(2, 0),
                FaceProperties::new(3, 0)
            ),
            Block::Dirt             => classic_renderer!(FaceProperties::new(3, 0)),
            Block::Gravel           => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Sand             => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Cactus           => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Snow             => classic_renderer!(FaceProperties::new(0, 0), width=10, height=1),
            Block::TallGrass        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::OakLog           => classic_renderer!(FaceProperties::new(0, 0)),
            Block::AcaciaLog        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::BigOakLog        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::BirchLog         => classic_renderer!(FaceProperties::new(0, 0)),
            Block::JungleLog        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::SpruceLog        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::OakLeaves        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::AcaciaLeaves     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::BigOakLeaves     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::BirchLeaves      => classic_renderer!(FaceProperties::new(0, 0)),
            Block::JungleLeaves     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::SpruceLeaves     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::OrangeTulipe     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::PinkTulip        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::RedTulip         => classic_renderer!(FaceProperties::new(0, 0)),
            Block::WhiteTulip       => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Dandelion        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::AzureBluet       => classic_renderer!(FaceProperties::new(0, 0)),
            Block::OxeyeDaisy       => classic_renderer!(FaceProperties::new(0, 0)),
            Block::BlueOrchid       => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Allium           => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Poppy            => classic_renderer!(FaceProperties::new(0, 0)),
            Block::LightWhite       => classic_renderer!(FaceProperties::new(0, 0)),
            Block::LightRed         => classic_renderer!(FaceProperties::new(0, 0)),
            Block::LightGreen       => classic_renderer!(FaceProperties::new(0, 0)),
            Block::LightBlue        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::LightYellow      => classic_renderer!(FaceProperties::new(0, 0)),
            Block::LightCyan        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::OakPlanks        => classic_renderer!(FaceProperties::new(0, 0)),
            Block::AcaciaPlanks     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::BigOakPlanks     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::BirchPlanks      => classic_renderer!(FaceProperties::new(0, 0)),
            Block::JunglePlanks     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::SprucePlanks     => classic_renderer!(FaceProperties::new(0, 0)),
            Block::Brick            => classic_renderer!(FaceProperties::new(0, 0)),
            Block::StoneBricks      => classic_renderer!(FaceProperties::new(0, 0)),
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
