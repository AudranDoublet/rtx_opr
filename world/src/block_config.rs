use std::collections::{HashSet, HashMap};
use crate::*;
use serde_derive::*;

use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct ClassicBlockConfig {
    side: String,
    side_overlay: Option<String>,
    side_material: Option<u32>,

    top: Option<String>,
    top_overlay: Option<String>,
    top_material: Option<u32>,

    bottom: Option<String>,
    bottom_overlay: Option<String>,
    bottom_material: Option<u32>,

    width: Option<i32>,
    height: Option<i32>,

    continuum: Option<bool>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct FlowerConfig {
    texture: String,
    material: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BlockConfig {
    texture_path: String,
    texture_extension: String,
    texture_normal_extension: String,
    texture_mer_extension: String,
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
    fn add_path(&mut self, path: String) {
        let png = format!("{}.png", path);

        if Path::new(&png).exists() {
            self.paths.push(png);
        } else {
            self.paths.push(format!("{}.tga", path));
        }
    }

    pub fn texture(&mut self, config: &BlockConfig, texture: &str, overlay: Option<&String>) -> usize {
        if !self.textures.contains_key(texture) {
            self.textures.insert(texture.to_string(), self.paths.len());
            self.add_path(format!(
                "{}/{}{}",
                config.texture_path,
                texture,
                config.texture_extension,
            ));

            self.add_path(format!(
                "{}/{}{}",
                config.texture_path,
                texture,
                config.texture_normal_extension,
            ));

            self.add_path(format!(
                "{}/{}{}",
                config.texture_path,
                texture,
                config.texture_mer_extension,
            ));

            if let Some(overlay) = overlay {
                self.texture(config, overlay, None);
            }
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
            let side = texture.texture(&self, &block.side, block.side_overlay.as_ref());
            let side_material = block.side_material.unwrap_or(0);

            let top = if let Some(top) = block.top.as_ref() {
                texture.texture(&self, top, block.top_overlay.as_ref())
            } else {
                side
            };
            let top_material = block.top_material.unwrap_or(side_material);

            let bottom = if let Some(bottom) = block.bottom.as_ref() {
                texture.texture(&self, bottom, block.bottom_overlay.as_ref())
            } else {
                top
            };
            let bottom_material = block.bottom_material.unwrap_or(top_material);

            let width = block.width.unwrap_or(10);
            let height = block.height.unwrap_or(10);

            assert!(width >= 1 && width <= 10);
            assert!(height >= 1 && height <= 10);

            let side = FaceProperties::new(side as u32, side_material);
            let top = FaceProperties::new(top as u32, top_material);
            let bottom = FaceProperties::new(bottom as u32, bottom_material);

            BlockRenderer::ClassicBlock {
                faces: [top, bottom, side, side, side, side],
                width,
                height,
                continuum: block.continuum.unwrap_or(false),
            }
        } else if let Some(block) = self.flower_blocks.get(&block_name) {
            let texture = texture.texture(&self, &block.texture, None);
            let material = block.material.unwrap_or(0);

            BlockRenderer::FlowerBlock {
                face: FaceProperties::new(texture as u32, material),
            }
        } else {
            panic!("block {} not in configuration", block_name);
        }
    }
}
