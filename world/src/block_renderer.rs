use crate::{World, ChunkMesh, BlockFace};
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
        height: i32,
        width: i32,
    },
    FlowerBlock {
        id: usize,
        normal_id: Option<usize>,
    },
}

#[macro_export]
macro_rules! topdown_renderer {
    // default behaviour
    ($side_face:expr, $top_face:expr, $down_face:expr) => {{
        topdown_renderer!($side_face, $top_face, $down_face, width=10, height=10)
    }};

    // When something is passed
    ($side_face:expr, $top_face:expr, $down_face:expr, width=$width:expr, height=$height:expr) => {{
        BlockRenderer::ClassicBlock {
            faces: [$top_face, $down_face, $side_face, $side_face, $side_face, $side_face],
            height: $height,
            width: $width,
        }
    }}
}

#[macro_export]
macro_rules! classic_renderer {
    // default behaviour
    ($face:expr) => {{
        classic_renderer!($face, width=10, height=10)
    }};

    // When something is passed
    ($face:expr, width=$width:expr, height=$height:expr) => {{
        BlockRenderer::ClassicBlock {
            faces: [$face; 6],
            height: $height,
            width: $width,
        }
    }}
}

pub const BLOCK_RENDERERS: [BlockRenderer; 1] = [
    BlockRenderer::Empty,
];

impl BlockRenderer {
    pub fn classic(prop: FaceProperties) -> BlockRenderer {
        BlockRenderer::ClassicBlock {
            faces: [prop; 6],
            height: 10,
            width: 10,
        }
    }

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

    pub fn render(&self, world: &World, position: Vector3<i32>, mesh: &mut ChunkMesh) {
        match self {
            BlockRenderer::Empty => (),
            BlockRenderer::ClassicBlock{faces, height, width} => {
                for (i, face) in BlockFace::faces().enumerate() {
                    let rel = face.relative();

                    // skip face if the neighbouring block is opaque (the face won't be seen)
                    if let Some(block) = world.block_at(position + rel) {
                        if block.is_opaque() {
                            continue;
                        }
                    }

                    let width_offset = (10 - width) / 2;

                    // up/down faces
                    let (height, height_offset, up, right, position) = if rel.y != 0 {
                        let z = -rel.cross(&Vector3::x());

                        // compute starting corner of the face
                        let position = position * 10
                                            // up face: change y
                                        + Vector3::new(0, rel.y.max(0), 0) * *height
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

                        (*height, 0, Vector3::y(), right, position * 10 + dpos)
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
            _ => (),
        }
    }
}
