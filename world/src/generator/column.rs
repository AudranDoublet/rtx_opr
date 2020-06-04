use nalgebra::Vector3;
use perlin::PerlinOctaves;

use std::rc::Rc;

use crate::generator::layers::{Layer, LayerResult};
use crate::{Block, Chunk, SEA_LEVEL};

use rand::SeedableRng;

pub struct ColumnProvider {
    perlins: [PerlinOctaves; 4],
    column_weights: [f32; 825],
    grass_colors: [Vector3<f32>; 25],
    biome_provider: Box<Layer>,
    unzoomed_biome_provider: Box<Layer>,
}

const BASE_SIZE: f32 = 8.5;
const Y_STRETCH: f32 = 12.;

#[inline]
fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn lerp3(t: f32, a: Vector3<f32>, b: Vector3<f32>) -> Vector3<f32> {
    a + (b - a) * t
}

#[inline]
fn clamp_lerp(t: f32, a: f32, b: f32) -> f32 {
    lerp(t.min(1.0).max(0.0), a, b)
}

#[inline]
fn noise_amplitude() -> Vector3<f32> {
    Vector3::new(684.412, 684.412, 684.412)
}

#[inline]
fn depth_noise_amplitude() -> Vector3<f32> {
    Vector3::new(200., 0.0, 200.)
}

#[inline]
fn main_noise_scale() -> Vector3<f32> {
    Vector3::new(80., 160., 80.)
}

fn nearby_column_mult(x: isize, y: isize) -> f32 {
    10. / ((x * x + y * y) as f32 + 0.2).sqrt()
}

impl ColumnProvider {
    pub fn new(seed: isize) -> ColumnProvider {
        let (b, z) = Layer::create_generator(seed);

        let mut rng = SeedableRng::seed_from_u64(seed as u64);

        ColumnProvider {
            perlins: [
                PerlinOctaves::new(8, &mut rng),
                PerlinOctaves::new(16, &mut rng),
                PerlinOctaves::new(16, &mut rng),
                PerlinOctaves::new(16, &mut rng),
            ],
            column_weights: [0.0; 825],
            grass_colors: [Vector3::zeros(); 25],
            biome_provider: z,
            unzoomed_biome_provider: b,
        }
    }

    pub fn generate_chunk(&mut self, chunk: &mut Rc<Chunk>) {
        let chunk = unsafe { Rc::get_mut_unchecked(chunk) };

        let cx = chunk.coords().x as isize;
        let cy = chunk.coords().y as isize;

        self.set_blocks(cx, cy, chunk);

        let biomes = self.biome_provider.generate(cx * 16, cy * 16, 16, 16);

        for x in 0..16 {
            for z in 0..16 {
                biomes
                    .biome(x, z)
                    .generate_column(chunk, x as i32, z as i32);
                *chunk.biome_at_mut(x as i32, z as i32) = biomes.biome(x, z);
            }
        }
    }

    /**
     * Create chunk general shape, with only stone and water
     */
    fn set_blocks(&mut self, cx: isize, cy: isize, chunk: &mut Chunk) {
        let biomes = self
            .unzoomed_biome_provider
            .generate(cx * 4 - 2, cy * 4 - 2, 10, 10);

        self.generate_weights(&biomes, cx * 4, cy * 4);

        for x in 0..4 {
            for z in 0..4 {
                for y in 0..32 {
                    self.interpolate_weights(chunk, x, y, z);
                }
            }
        }
    }

    #[inline]
    fn interpolate_weights(&mut self, chunk: &mut Chunk, x: usize, y: usize, z: usize) {
        // for performance reasons, weights are generated for only a few blocks (each x/z: 4, y: 8)
        // this function make a trilinear interpolation to generate the missing weights

        let mut v1 = self.column_weight_at(x + 0, y, z + 0);
        let mut v2 = self.column_weight_at(x + 0, y, z + 1);
        let mut v3 = self.column_weight_at(x + 1, y, z + 0);
        let mut v4 = self.column_weight_at(x + 1, y, z + 1);

        let v1_step = (self.column_weight_at(x + 0, y + 1, z + 0) - v1) / 8.;
        let v2_step = (self.column_weight_at(x + 0, y + 1, z + 1) - v2) / 8.;
        let v3_step = (self.column_weight_at(x + 1, y + 1, z + 0) - v3) / 8.;
        let v4_step = (self.column_weight_at(x + 1, y + 1, z + 1) - v4) / 8.;

        let gc1 = *self.grass_color_at(x + 0, z + 0);
        let gc2 = *self.grass_color_at(x + 0, z + 1);
        let gc3 = *self.grass_color_at(x + 1, z + 0);
        let gc4 = *self.grass_color_at(x + 1, z + 1);

        for dx in 0..4 {
            for dz in 0..4 {
                let color = lerp3(
                    dz as f32 / 4.,
                    lerp3(dx as f32 / 4., gc1, gc3),
                    lerp3(dx as f32 / 4., gc2, gc4),
                );

                let x = (x * 4 + dx) as i32;
                let z = (z * 4 + dz) as i32;

                chunk.set_grass_color(x, z, color);
            }
        }

        for dy in 0..8 {
            let mut v5 = v1;
            let mut v6 = v2;
            let v5_step = (v3 - v1) / 4.;
            let v6_step = (v4 - v2) / 4.;

            for dx in 0..4 {
                let v7_step = (v6 - v5) / 4.;
                let mut v7 = v5 - v7_step;

                for dz in 0..4 {
                    v7 += v7_step;
                    let weight = v7;

                    let x = (x * 4 + dx) as i32;
                    let y = (y * 8 + dy) as i32;
                    let z = (z * 4 + dz) as i32;

                    let block_type = match weight {
                        w if w > 0.0 => Block::Stone,
                        _ if y < SEA_LEVEL => Block::Water,
                        _ => Block::Air,
                    };

                    chunk.set_block_at_chunk(x, y, z, block_type);
                }

                v5 += v5_step;
                v6 += v6_step;
            }

            v1 += v1_step;
            v2 += v2_step;
            v3 += v3_step;
            v4 += v4_step;
        }
    }

    fn column_weight_at(&self, x: usize, y: usize, z: usize) -> f32 {
        self.column_weights[(x * 5 + z) * 33 + y]
    }

    fn grass_color_at(&mut self, x: usize, z: usize) -> &mut Vector3<f32> {
        &mut self.grass_colors[x * 5 + z]
    }

    fn generate_weights(&mut self, biomes: &LayerResult, x: isize, z: isize) {
        let position = Vector3::new(x as f32, 0., z as f32);
        let noise_size = Vector3::new(5, 33, 5);
        let amplitude = noise_amplitude();

        let noise1 = self.perlins[0].noise(
            position,
            noise_size,
            amplitude.component_div(&main_noise_scale()),
        );
        let noise2 = self.perlins[1].noise(position, noise_size, amplitude);
        let noise3 = self.perlins[2].noise(position, noise_size, amplitude);

        let depth_noises = self.perlins[2].noise2d(position, noise_size, depth_noise_amplitude());

        let mut id_2d = 0;
        let mut id_3d = 0;

        for x in 0..5 {
            for z in 0..5 {
                let mut scale = 0.0;
                let mut depth = 0.0;
                let mut force = 0.0;

                // compute depth and scale considering neighbouring biomes
                let self_biome = biomes.biome(z + 2, x + 2);
                let mut grass_color = Vector3::zeros();

                for dz in -2..=2 {
                    for dx in -2..=2 {
                        let biome = biomes.biome(x + dx + 2, z + dz + 2);

                        let mut c_force = nearby_column_mult(dx, dz) / (2. + biome.elevation());

                        if biome.elevation() > self_biome.elevation() {
                            c_force /= 2.;
                        }

                        scale += biome.depth() * c_force;
                        depth += biome.elevation() * c_force;
                        force += c_force;

                        grass_color += biome.grass_color();
                    }
                }

                *self.grass_color_at(x as usize, z as usize) = grass_color / 25.0;

                scale = (scale / force) * 0.9 + 0.1;
                depth = ((depth / force) * 4. - 1.0) / 8.;

                // compute random depth noise from 2d noise
                let mut depth_noise = depth_noises[id_2d] / 8000.;

                if depth_noise < 0. {
                    depth_noise = -depth_noise * 0.3;
                }

                depth_noise = depth_noise * 3. - 2.;

                if depth_noise < 0.0 {
                    depth_noise /= 2.;
                    depth_noise = depth_noise.max(-1.);
                    depth_noise /= 2.8;
                } else {
                    depth_noise = depth_noise.min(1.) / 8.;
                }

                let final_depth = (depth + 0.2 * depth_noise) * BASE_SIZE / 2. + BASE_SIZE;

                // compute final values
                for y in 0..33 {
                    let threshold = match (y as f32 - final_depth) * Y_STRETCH * 128. / 256. / scale
                    {
                        t if t < 0.0 => 4. * t,
                        t => t,
                    };

                    let a = noise2[id_3d] / 512.;
                    let b = noise3[id_3d] / 512.;
                    let t = (noise1[id_3d] / 10. + 1.) / 2.;

                    let mut res = clamp_lerp(t, a, b) - threshold;

                    // reduce block probability at high altitude
                    if y > 29 {
                        let scale = (y as f32 - 29.) / 3.;
                        res = res * (1. - scale) - 10. * scale;
                    }

                    self.column_weights[id_3d] = res;
                    id_3d += 1;
                }

                id_2d += 1;
            }
        }
    }
}
