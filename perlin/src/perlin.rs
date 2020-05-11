use nalgebra::Vector3;
use rand::Rng;

const GRAD_1: [f32; 16] = [1., -1., 1., -1., 1., -1., 1., -1., 0., 0., 0., 0., 1., 0., -1., 0.];
const GRAD_2: [f32; 16] = [1., 1., -1., -1., 0., 0., 0., 0., 1., -1., 1., -1., 1., -1., 1., -1.];
const GRAD_3: [f32; 16] = [0., 0., 0., 0., 1., 1., -1., -1., 1., 1., -1., -1., 0., 1., 0., -1.];
const GRAD_4: [f32; 16] = [1., -1., 1., -1., 1., -1., 1., -1., 0., 0., 0., 0., 1., 0., -1., 0.];
const GRAD_5: [f32; 16] = [0., 0., 0., 0., 1., 1., -1., -1., 1., 1., -1., -1., 0., 1., 0., -1.];

pub struct PerlinNoise
{
    random: Vector3<f32>,
    perlin_values: [usize; 512],
}

pub struct PerlinOctaves
{
    octaves: Vec<PerlinNoise>,
}

#[inline]
fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

#[inline]
fn floor(v: f32) -> i32 {
    (v.floor() as i32) & 255
}

#[inline]
fn delta(v: f32) -> f32 {
    v.powi(3) * (v * (v * 6. - 15.) + 10.)
}

impl PerlinNoise
{
    pub fn new() -> PerlinNoise {
        let mut rng = rand::thread_rng();
        let mut values = [0; 512];

        for i in 0..256 {
            values[i] = i;
        }

        for i in 0..256 {
            let j = rng.gen_range(0, 256 - i) + i;

            values.swap(i, j);
            values[i + 256] = values[i];
        }

        PerlinNoise {
            random: Vector3::new(
                        rng.gen_range(0., 256.),
                        rng.gen_range(0., 256.),
                        rng.gen_range(0., 256.),
            ),
            perlin_values: values,
        }
    }

    fn gradient2d(&self, pos: usize, x: f32, y: f32) -> f32 {
        let pos = pos % 16;
        GRAD_4[pos] * x + GRAD_5[pos] * y
    }

    fn gradient(&self, pos: usize, x: f32, y: f32, z: f32) -> f32 {
        let pos = pos % 16;
        GRAD_1[pos] * x + GRAD_2[pos] * y + GRAD_3[pos] * z
    }

    pub fn noise2d(&self, result: &mut Vec<f32>, position: Vector3<f32>,
                          size: Vector3<usize>, freq: f32, amplitude: Vector3<f32>)
    {
        let freq = 1. / freq;
        let mut pos = 0;

        for dx in 0..size.x {
            let x = position.x + dx as f32 * amplitude.x + self.random.x;

            let norm_x = floor(x) as usize;

            let x = x.fract();

            for dz in 0..size.z {
                let z = position.z + dz as f32 * amplitude.z + self.random.z;
                let norm_z = floor(z) as usize;
                let z = z.fract();

                let p1 = self.perlin_values[self.perlin_values[norm_x]] + norm_z;
                let p2 = self.perlin_values[self.perlin_values[norm_x+ 1]] + norm_z;

                result[pos] += lerp(
                    delta(z),
                    lerp(
                        delta(x),
                        self.gradient2d(self.perlin_values[p1], x, z),
                        self.gradient(self.perlin_values[p2], x - 1.0, 0.0, z)
                    ),
                    lerp(
                        delta(x),
                        self.gradient(self.perlin_values[p1 + 1], x, 0.0, z - 1.0), 
                        self.gradient(self.perlin_values[p2 + 1], x - 1.0, 0.0, z - 1.0)
                    )
                ) * freq;

                pos += 1;
            }
        }
    }

    pub fn noise(&self, result: &mut Vec<f32>, position: Vector3<f32>,
                          size: Vector3<usize>, freq: f32, amplitude: Vector3<f32>)
    {
        if size.y == 1 {
            return self.noise2d(result, position, size, freq, amplitude);
        }

        let mut pos = 0;
        let freq = 1. / freq;

        let mut last_y = 1000; // norm_y can't reach this vlaue (% 256)
        let mut cache = [0.; 4];

        for dx in 0..size.x {
            let x = position.x + dx as f32 * amplitude.x + self.random.x;
            let norm_x = floor(x) as usize;
            let x = x.fract();

            for dz in 0..size.z {
                let z = position.z + dz as f32 * amplitude.z + self.random.z;

                let norm_z = floor(z) as usize;
                let z = z.fract();

                for dy in 0..size.y {
                    let y = position.y + dy as f32 * amplitude.y + self.random.y;

                    let norm_y = floor(y) as usize;
                    let y = y.fract();

                    if dy == 0 || norm_y != last_y {
                        last_y = norm_y;
                        let a = self.perlin_values[norm_x] + norm_y;
                        let b = self.perlin_values[a] + norm_z;
                        let c = self.perlin_values[a + 1] + norm_z;
                        let d = self.perlin_values[norm_x + 1] + norm_y;

                        let e = self.perlin_values[d] + norm_z;
                        let f = self.perlin_values[d + 1] + norm_z;

                        cache[0] = lerp(delta(x), self.gradient(self.perlin_values[b], x, y, z), self.gradient(self.perlin_values[e], x - 1., y, z));
                        cache[1] = lerp(delta(x), self.gradient(self.perlin_values[c], x, y - 1., z), self.gradient(self.perlin_values[f], x - 1., y - 1., z));
                        cache[2] = lerp(delta(x), self.gradient(self.perlin_values[b + 1], x, y, z - 1.), self.gradient(self.perlin_values[e + 1], x - 1., y, z - 1.));
                        cache[3] = lerp(delta(x), self.gradient(self.perlin_values[c + 1], x, y - 1., z - 1.), self.gradient(self.perlin_values[f + 1], x - 1., y - 1., z - 1.));
                    }

                    result[pos] += lerp(
                        delta(z),
                        lerp(delta(y), cache[0], cache[1]),
                        lerp(delta(y), cache[2], cache[3])
                    ) * freq;

                    pos += 1;
                }
            }
        }

    }
}

impl PerlinOctaves
{
    pub fn new(count: usize) -> PerlinOctaves {
        PerlinOctaves {
            octaves: (0..count).map(|_| PerlinNoise::new()).collect()
        }
    }

    pub fn noise(&self, position: Vector3<f32>, size: Vector3<usize>, amplitude: Vector3<f32>) -> Vec<f32> {
        let mut result = vec![0.0; size.x * size.y * size.z];
        let mut freq = 1.0;

        for octave in &self.octaves {
            let amp = amplitude * freq;
            let pos = position.component_mul(&amp);

            octave.noise(&mut result, pos, size, freq, amp);
            freq /= 2.0;
        }

        result
    }

    pub fn noise2d(&self, mut position: Vector3<f32>, mut size: Vector3<usize>, mut amplitude: Vector3<f32>) -> Vec<f32> {
        size.y = 1;
        position.y = 10.;
        amplitude.y = 1.;

        self.noise(position, size, amplitude)
    }
}
