use std::num::Wrapping;

const RNG_MULT: Wrapping<isize> = Wrapping(3823478372547234723);
const RNG_ADD: Wrapping<isize> = Wrapping(1442695040888963407);

#[derive(Clone)]
pub struct SimpleRandom {
    // wrapping are used to prevent integer overflow checks

    seed: Wrapping<isize>,
    gen_seed: Wrapping<isize>,
    local_seed: Wrapping<isize>,
}

impl SimpleRandom {
    pub fn new(seed: isize, world_seed: isize) -> SimpleRandom {
        let mut res = SimpleRandom {
            seed: Wrapping(seed),
            gen_seed: Wrapping(0),
            local_seed: Wrapping(0),
        };

        res.init_world(world_seed);
        res
    }

    pub fn init_world(&mut self, seed: isize) {
        self.gen_seed = Wrapping(seed);

        for _ in 0..3 {
            self.gen_seed *= self.gen_seed * RNG_MULT + RNG_ADD;
            self.gen_seed += self.seed;
        }
    }

    pub fn init_local(&mut self, x: isize, z: isize) {
        self.local_seed = self.gen_seed;

        let x = Wrapping(x);
        let z = Wrapping(z);

        for _ in 0..2 {
            self.local_seed *= self.local_seed * RNG_MULT + RNG_ADD;
            self.local_seed += x;
            self.local_seed *= self.local_seed * RNG_MULT + RNG_ADD;
            self.local_seed += z;
        }
    }

    pub fn next(&mut self, max: isize) -> isize {
        let result = match (self.local_seed >> 24).0 % max {
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
