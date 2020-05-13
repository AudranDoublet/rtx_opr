use crate::AABB;
use nalgebra::Vector3;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
#[repr(isize)]
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
}

impl Block {
    pub fn is_tough(&self) -> bool {
        match self {
            Block::Air | Block::Snow => false,
            Block::TallGrass => false,
            _ => true,
        }
    }

    pub fn aabb(&self, position: Vector3<f32>) -> Option<AABB> {
        let base = match self {
            Block::Air  => None,
            _           => Some(AABB::new(Vector3::zeros(), Vector3::new(1., 1., 1.))),
        };

        if let Some(base) = base {
            Some(base.translate(position))
        } else {
            None
        }
    }
}
