use crate::{World, ChunkMesh, BlockFace, Block};
use nalgebra::{Vector2, Vector3};

#[derive(Clone, Copy)]
pub struct FaceProperties {
    pub texture_id: u32,
    pub material_id: u32,
}

impl FaceProperties {
    pub fn new(texture_id: u32, material_id: u32) -> FaceProperties {
        FaceProperties {
            texture_id,
            material_id,
        }
    }
}

pub enum BlockRenderer {
    Empty,
    ClassicBlock {
        faces: [FaceProperties; 6],
        continuum: bool,
        height: i32,
        width: i32,
    },
    FlowerBlock {
        face: FaceProperties,
    },
}

pub const BLOCK_RENDERERS: [BlockRenderer; 1] = [
    BlockRenderer::Empty,
];

impl BlockRenderer {
    fn generate_face(
        &self,
        mesh: &mut ChunkMesh,

        face_properties: &FaceProperties,

        width_offset: i32,
        height: i32,
        height_offset: i32,

        position: Vector3<i32>,
        up: Vector3<i32>,
        right: Vector3<i32>,
    ) {
        mesh.add_triangle(
            face_properties,
            // positions
            position,
            position + up,
            position + right,
            // textures
            Vector2::new(width_offset     , height_offset),
            Vector2::new(width_offset     , height - height_offset),
            Vector2::new(10 - width_offset, height_offset),
            // normal
            up.cross(&right),
        );

        mesh.add_triangle(
            face_properties,
            // positions
            position + up + right,
            position + up,
            position + right,
            // textures
            Vector2::new(10 - width_offset, height - height_offset),
            Vector2::new(width_offset     , height - height_offset),
            Vector2::new(10 - width_offset, height_offset),
            // normal
            up.cross(&right),
        );
    }

    pub fn render(&self, world: &World, self_type: Block, position: Vector3<i32>, mesh: &mut ChunkMesh) {
        match self {
            BlockRenderer::Empty => (),
            BlockRenderer::ClassicBlock{faces, height, width, continuum} => {
                let mut height = *height;

                if *continuum {
                    if let Some(block) = world.block_at(position + BlockFace::Up.relative()) {
                        if block == self_type {
                            height = 10;
                        }
                    }
                }

                for (i, face) in BlockFace::faces().enumerate() {
                    let rel = face.relative();

                    // skip face if the neighbouring block is opaque (the face won't be seen)
                    if let Some(block) = world.block_at(position + rel) {
                        if block.is_opaque() && height == 10 {
                            continue;
                        }

                        if *continuum && block == self_type {
                            continue;
                        }
                    }

                    if (position + rel).y < 0 {
                        continue;
                    }

                    let width_offset = (10 - width) / 2;

                    // up/down faces
                    let (height, height_offset, up, right, position) = if rel.y != 0 {
                        let z = -rel.cross(&Vector3::x());

                        // compute starting corner of the face
                        let position = position * 10
                                            // up face: change y
                                        + Vector3::new(0, rel.y.max(0), 0) * height
                                            // width offsets
                                        + z * width_offset
                                        + Vector3::x() * width_offset
                                        + Vector3::new(0, 0, -z.z.min(0)) * 10;

                        (*width, width_offset, z, Vector3::x(), position)
                    } else { // other faces
                        // direction of `right` vector (up vector is always (0, 1, 0))
                        let right = rel.cross(&Vector3::y()); 

                        // compute starting corner of the face
                        let dpos = right - rel;
                        let dpos = Vector3::new(-dpos.x.min(0), 0, -dpos.z.min(0)) * 10;

                        // add width offsets
                        let dpos = dpos - rel * width_offset + right * width_offset;

                        (height, 0, Vector3::y(), right, position * 10 + dpos)
                    };

                    self.generate_face(
                        mesh,
                        &faces[i],
                        width_offset,
                        height,
                        height_offset,
                        position,
                        up * height,
                        right * *width,
                    );
                }
            }
            BlockRenderer::FlowerBlock { face} => {
                self.generate_face(
                    mesh,
                    face,
                    0,
                    10,
                    0,
                    position * 10,
                    Vector3::y() * 10,
                    Vector3::new(10, 0, 10),
                );

                self.generate_face(
                    mesh,
                    face,
                    0,
                    10,
                    0,
                    position * 10 + Vector3::new(10, 0, 0),
                    Vector3::y() * 10,
                    Vector3::new(-10, 0, 10),
                );
            }
        }
    }
}
