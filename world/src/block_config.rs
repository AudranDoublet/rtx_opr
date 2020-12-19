use std::collections::{HashSet, HashMap};
use crate::*;
use serde_derive::*;

#[derive(Debug, Deserialize)]
pub struct ClassicBlockConfig {
    side: String,
    top: Option<String>,
    bottom: Option<String>,

    width: Option<i32>,
    height: Option<i32>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct FlowerConfig {
    texture: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockConfig {
    texture_path: String,
    texture_extension: String,
    texture_normal_extension: String,
    texture_dimension: (usize, usize),

    empty_blocks: HashSet<String>,
    classic_blocks: HashMap<String, ClassicBlockConfig>,
    flower_blocks: HashMap<String, FlowerConfig>,
}

pub struct TextureList {
    paths: Vec<String>,
    textures: HashMap<String, usize>,
    dimension: (usize, usize),
}

impl TextureList {
    pub fn texture(&mut self, config: &BlockConfig, texture: &str) -> usize {
        if !self.textures.contains_key(texture) {
            self.textures.insert(texture.to_string(), self.paths.len());
            self.paths.push(format!(
                "{}/{}{}",
                config.texture_path,
                texture,
                config.texture_extension,
            ));

            self.paths.push(format!(
                "{}/{}{}",
                config.texture_path,
                texture,
                config.texture_normal_extension,
            ));
        }

        self.textures[texture]
    }

    pub fn dimensions(&self) -> (usize, usize) {
        self.dimension
    }

    pub fn paths(&self) -> &Vec<String> {
        &self.paths
    }
}

impl BlockConfig {
    pub fn init_texture_list(&self) -> TextureList {
        TextureList {
            paths: Vec::new(),
            textures: HashMap::new(),
            dimension: self.texture_dimension,
        }
    }

    pub fn build_block_renderer(
        &self,
        block_name: String,
        texture: &mut TextureList,
    ) -> BlockRenderer {
        if self.empty_blocks.contains(&block_name) {
            BlockRenderer::Empty
        } else if let Some(block) = self.classic_blocks.get(&block_name) {
            let side = texture.texture(&self, &block.side);

            let top = if let Some(top) = block.top.as_ref() {
                texture.texture(&self, top)
            } else {
                side
            };

            let bottom = if let Some(bottom) = block.bottom.as_ref() {
                texture.texture(&self, bottom)
            } else {
                top
            };

            let width = block.width.unwrap_or(10);
            let height = block.height.unwrap_or(10);

            assert!(width >= 1 && width <= 10);
            assert!(height >= 1 && height <= 10);

            let side = FaceProperties::new(side as u32, 0);
            let top = FaceProperties::new(top as u32, 0);
            let bottom = FaceProperties::new(bottom as u32, 0);

            BlockRenderer::ClassicBlock {
                faces: [top, bottom, side, side, side, side],
                width,
                height,
            }
        } else if let Some(_block) = self.flower_blocks.get(&block_name) {
            //FIXME render flower blocks
            BlockRenderer::Empty
        } else {
            panic!("block {} not in configuration", block_name);
        }
    }
}
