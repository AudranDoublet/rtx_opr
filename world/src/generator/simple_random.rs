const RNG_MULT: isize = 3823478372547234723;
const RNG_ADD: isize = 1442695040888963407;

pub struct SimpleRandom {
    seed: isize,
    gen_seed: isize,
    local_seed: isize,
}

impl SimpleRandom {
    pub fn init_world(&mut self, seed: isize) {
        self.gen_seed = seed;

        for _ in 0..3 {
            self.gen_seed *= self.gen_seed * RNG_MULT + RNG_ADD;
            self.gen_seed += self.seed;
        }
    }

    pub fn init_local(&mut self, x: isize, z: isize) {
        self.local_seed = self.gen_seed;

        for _ in 0..2 {
            self.local_seed *= self.local_seed * RNG_MULT + RNG_ADD;
            self.local_seed += x;
            self.local_seed *= self.local_seed * RNG_MULT + RNG_ADD;
            self.local_seed += z;
        }
    }

    pub fn next(&mut self, max: isize) -> isize {
        let result = match (self.local_seed >> 24) % max {
            m if m < 0 => m + max,
            m => m,
        };

        self.local_seed *= self.local_seed * RNG_MULT + RNG_ADD;
        self.local_seed += self.gen_seed;

        result
    }

    pub fn cond(&mut self, diff: isize) -> bool {
        self.next(diff) == 0
    }

    pub fn next_float(&mut self, precision: isize) -> f32 {
        self.next(precision) as f32 / precision as f32
    }

    pub fn peek<'a, T>(&mut self, el: &'a [T]) -> &'a T {
        &el[self.next(el.len() as isize) as usize]
    }
}
